use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};
use std::path::{Path, PathBuf};
use std::str::FromStr;

const PROJECT_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
  version TEXT PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS kv_state (
  namespace TEXT NOT NULL,
  key TEXT NOT NULL,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(namespace, key)
);

CREATE TABLE IF NOT EXISTS migration_audit (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  scope TEXT NOT NULL,
  source_path TEXT NOT NULL,
  target_path TEXT NOT NULL,
  action TEXT NOT NULL,
  created_at TEXT NOT NULL
);
"#;

const GLOBAL_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
  version TEXT PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS global_state (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
"#;

const PROJECT_MIGRATIONS: &[(&str, &str)] = &[("0001_project_schema", PROJECT_SCHEMA_V1)];
const GLOBAL_MIGRATIONS: &[(&str, &str)] = &[("0001_global_schema", GLOBAL_SCHEMA_V1)];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DatabaseMigrationReport {
    pub db_path: PathBuf,
    pub created: bool,
    pub applied_migrations: usize,
}

/// Returns `~/.ship/state/<project-slug>/ship.db` for the given ship_dir.
/// The slug is derived from the canonical project root path, making it stable
/// across sessions and safe to store alongside the global DB.
pub fn project_db_path(ship_dir: &Path) -> Result<PathBuf> {
    let slug = project_slug(ship_dir)?;
    Ok(ship_global_dir()?.join("state").join(slug).join("ship.db"))
}

pub fn ensure_project_database(ship_dir: &Path) -> Result<DatabaseMigrationReport> {
    let db_path = project_db_path(ship_dir)?;
    ensure_database(&db_path, PROJECT_MIGRATIONS)
}

pub fn ensure_global_database(global_dir: &Path) -> Result<DatabaseMigrationReport> {
    let db_path = global_dir.join("ship.db");
    ensure_database(&db_path, GLOBAL_MIGRATIONS)
}

// ─── Path helpers ─────────────────────────────────────────────────────────────

fn ship_global_dir() -> Result<PathBuf> {
    home::home_dir()
        .map(|h| h.join(".ship"))
        .ok_or_else(|| anyhow!("Could not determine home directory"))
}

/// Derives a filesystem-safe slug from the project root path.
/// `/home/alice/dev/my-app` → `home-alice-dev-my-app`
fn project_slug(ship_dir: &Path) -> Result<String> {
    let project_root = ship_dir
        .parent()
        .ok_or_else(|| anyhow!("Cannot determine project root from {}", ship_dir.display()))?;

    // Canonicalize if possible (resolves symlinks), fall back to raw path.
    let canonical = std::fs::canonicalize(project_root)
        .unwrap_or_else(|_| project_root.to_path_buf());

    let raw = canonical.to_string_lossy();
    // Strip leading slash, map non-alphanumeric/hyphen/underscore to hyphens,
    // then collapse consecutive hyphens so the slug stays readable.
    let slug: String = raw
        .trim_start_matches('/')
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '-' })
        .collect();
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if slug.is_empty() {
        return Err(anyhow!("Could not derive a project slug from path: {}", canonical.display()));
    }
    Ok(slug)
}

// ─── Core ─────────────────────────────────────────────────────────────────────

fn ensure_database(db_path: &Path, migrations: &[(&str, &str)]) -> Result<DatabaseMigrationReport> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let created = !db_path.exists();

    let db_url = sqlite_url(db_path);
    let options = SqliteConnectOptions::from_str(&db_url)
        .with_context(|| format!("Invalid sqlite url {}", db_url))?
        .create_if_missing(true);
    let mut connection = block_on(async { SqliteConnection::connect_with(&options).await })?;

    // Keep write behavior stable across CLI/UI/MCP call paths.
    block_on(async {
        sqlx::query("PRAGMA journal_mode = WAL;")
            .execute(&mut connection)
            .await
    })?;
    block_on(async {
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&mut connection)
            .await
    })?;

    let mut applied = 0usize;
    for (version, ddl) in migrations {
        let has_row = block_on(async {
            sqlx::query("SELECT version FROM schema_migrations WHERE version = ?")
                .bind(*version)
                .fetch_optional(&mut connection)
                .await
        })
        .ok()
        .flatten()
        .is_some();
        if has_row {
            continue;
        }

        block_on(async { sqlx::query(ddl).execute(&mut connection).await }).with_context(|| {
            format!(
                "Failed applying schema migration {} for {}",
                version,
                db_path.display()
            )
        })?;
        block_on(async {
            sqlx::query("INSERT INTO schema_migrations (version, applied_at) VALUES (?, ?)")
                .bind(*version)
                .bind(Utc::now().to_rfc3339())
                .execute(&mut connection)
                .await
        })
        .with_context(|| {
            format!(
                "Failed recording schema migration {} for {}",
                version,
                db_path.display()
            )
        })?;
        applied += 1;
    }

    Ok(DatabaseMigrationReport {
        db_path: db_path.to_path_buf(),
        created,
        applied_migrations: applied,
    })
}

fn sqlite_url(path: &Path) -> String {
    let mut raw = path.to_string_lossy().replace('\\', "/");
    if !raw.starts_with('/') {
        raw = format!("/{}", raw);
    }
    format!("sqlite://{}", raw)
}

fn block_on<F, T>(future: F) -> Result<T>
where
    F: std::future::Future<Output = std::result::Result<T, sqlx::Error>>,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|err| anyhow!("Failed to create SQLite runtime: {}", err))?;
    runtime
        .block_on(future)
        .map_err(|err| anyhow!("SQLite operation failed: {}", err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn ensure_project_database_is_idempotent() -> Result<()> {
        let tmp = tempdir()?;
        // ship_dir must have a parent (project root) to derive the slug
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;

        let report_a = ensure_project_database(&ship_dir)?;
        let report_b = ensure_project_database(&ship_dir)?;

        assert!(report_a.created);
        assert!(report_a.applied_migrations >= 1);
        assert!(!report_b.created);
        assert_eq!(report_b.applied_migrations, 0);
        // DB lives outside the project dir
        assert!(!report_a.db_path.starts_with(tmp.path()));
        assert!(report_a.db_path.to_string_lossy().contains("ship.db"));

        // Clean up the DB we just created in ~/.ship/state/
        std::fs::remove_file(&report_a.db_path).ok();
        Ok(())
    }

    #[test]
    fn project_slug_strips_leading_slash_and_collapses_separators() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        let slug = project_slug(&ship_dir)?;
        assert!(!slug.starts_with('-'), "slug should not start with a dash");
        assert!(!slug.contains("--"), "slug should not contain consecutive dashes");
        assert!(!slug.is_empty());
        Ok(())
    }
}
