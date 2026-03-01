use anyhow::{Context, Result, anyhow};
use serde_json;
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

const PROJECT_SCHEMA_OPERATIONAL: &str = r#"
CREATE TABLE IF NOT EXISTS managed_mcp_state (
  provider TEXT PRIMARY KEY,
  server_ids_json TEXT NOT NULL DEFAULT '[]',
  last_mode TEXT,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS branch_context (
  branch TEXT PRIMARY KEY,
  doc_type TEXT NOT NULL,
  doc_id TEXT NOT NULL,
  last_synced TEXT NOT NULL
);
"#;

const PROJECT_SCHEMA_WORKSPACE: &str = r#"
CREATE TABLE IF NOT EXISTS workspace (
  branch         TEXT PRIMARY KEY,
  feature_id     TEXT,
  spec_id        TEXT,
  active_mode    TEXT,
  providers_json TEXT NOT NULL DEFAULT '[]',
  resolved_at    TEXT NOT NULL,
  is_worktree    INTEGER NOT NULL DEFAULT 0,
  worktree_path  TEXT
);
"#;

const PROJECT_MIGRATIONS: &[(&str, &str)] = &[
    ("0001_project_schema", PROJECT_SCHEMA_V1),
    ("0002_operational_state", PROJECT_SCHEMA_OPERATIONAL),
    ("0003_workspace", PROJECT_SCHEMA_WORKSPACE),
];
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

// ─── CRUD helpers ─────────────────────────────────────────────────────────────

fn open_project_db(ship_dir: &Path) -> Result<SqliteConnection> {
    ensure_project_database(ship_dir)?;
    let db_path = project_db_path(ship_dir)?;
    let db_url = sqlite_url(&db_path);
    let options = SqliteConnectOptions::from_str(&db_url)
        .with_context(|| format!("Invalid sqlite url {}", db_url))?;
    block_on(async { SqliteConnection::connect_with(&options).await })
}

/// Returns `(server_ids, last_mode)` for the given provider, or empty defaults.
pub fn get_managed_state_db(
    ship_dir: &Path,
    provider: &str,
) -> Result<(Vec<String>, Option<String>)> {
    let mut conn = open_project_db(ship_dir)?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT server_ids_json, last_mode FROM managed_mcp_state WHERE provider = ?",
        )
        .bind(provider)
        .fetch_optional(&mut conn)
        .await
    })?;
    if let Some(row) = row_opt {
        use sqlx::Row;
        let ids_json: String = row.get(0);
        let last_mode: Option<String> = row.get(1);
        let ids: Vec<String> = serde_json::from_str(&ids_json).unwrap_or_default();
        Ok((ids, last_mode))
    } else {
        Ok((Vec::new(), None))
    }
}

/// Persist the managed server ids and last mode for the given provider.
pub fn set_managed_state_db(
    ship_dir: &Path,
    provider: &str,
    ids: &[String],
    last_mode: Option<&str>,
) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    let ids_json = serde_json::to_string(ids)
        .with_context(|| format!("Failed to serialize server ids for provider {}", provider))?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO managed_mcp_state (provider, server_ids_json, last_mode, updated_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(provider) DO UPDATE SET
               server_ids_json = excluded.server_ids_json,
               last_mode = excluded.last_mode,
               updated_at = excluded.updated_at",
        )
        .bind(provider)
        .bind(&ids_json)
        .bind(last_mode)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

/// Look up which document is associated with `branch`. Returns `(doc_type, doc_uuid)` or `None`.
pub fn get_branch_doc(ship_dir: &Path, branch: &str) -> Result<Option<(String, String)>> {
    let mut conn = open_project_db(ship_dir)?;
    let row_opt = block_on(async {
        sqlx::query("SELECT doc_type, doc_id FROM branch_context WHERE branch = ?")
            .bind(branch)
            .fetch_optional(&mut conn)
            .await
    })?;
    if let Some(row) = row_opt {
        use sqlx::Row;
        Ok(Some((row.get(0), row.get(1))))
    } else {
        Ok(None)
    }
}

/// Record that `branch` is associated with `doc_type` and the document's UUID.
pub fn set_branch_doc(
    ship_dir: &Path,
    branch: &str,
    doc_type: &str,
    doc_uuid: &str,
) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO branch_context (branch, doc_type, doc_id, last_synced)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(branch) DO UPDATE SET
               doc_type = excluded.doc_type,
               doc_id = excluded.doc_id,
               last_synced = excluded.last_synced",
        )
        .bind(branch)
        .bind(doc_type)
        .bind(doc_uuid)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
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

// ─── Workspace ────────────────────────────────────────────────────────────────

/// Retrieve the workspace record for the given branch, or None if none exists.
pub fn get_workspace_db(
    ship_dir: &Path,
    branch: &str,
) -> Result<Option<(Option<String>, Option<String>, Option<String>, Vec<String>, String, bool, Option<String>)>> {
    let mut conn = open_project_db(ship_dir)?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT feature_id, spec_id, active_mode, providers_json, resolved_at, is_worktree, worktree_path
             FROM workspace WHERE branch = ?",
        )
        .bind(branch)
        .fetch_optional(&mut conn)
        .await
    })?;
    if let Some(row) = row_opt {
        use sqlx::Row;
        let feature_id: Option<String> = row.get(0);
        let spec_id: Option<String> = row.get(1);
        let active_mode: Option<String> = row.get(2);
        let providers_json: String = row.get(3);
        let resolved_at: String = row.get(4);
        let is_worktree: i64 = row.get(5);
        let worktree_path: Option<String> = row.get(6);
        let providers: Vec<String> = serde_json::from_str(&providers_json).unwrap_or_default();
        Ok(Some((feature_id, spec_id, active_mode, providers, resolved_at, is_worktree != 0, worktree_path)))
    } else {
        Ok(None)
    }
}

/// Upsert the workspace record for the given branch.
pub fn upsert_workspace_db(
    ship_dir: &Path,
    branch: &str,
    feature_id: Option<&str>,
    spec_id: Option<&str>,
    active_mode: Option<&str>,
    providers: &[String],
    resolved_at: &str,
    is_worktree: bool,
    worktree_path: Option<&str>,
) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    let providers_json = serde_json::to_string(providers)
        .with_context(|| "Failed to serialize workspace providers")?;
    block_on(async {
        sqlx::query(
            "INSERT INTO workspace (branch, feature_id, spec_id, active_mode, providers_json, resolved_at, is_worktree, worktree_path)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(branch) DO UPDATE SET
               feature_id    = excluded.feature_id,
               spec_id       = excluded.spec_id,
               active_mode   = excluded.active_mode,
               providers_json = excluded.providers_json,
               resolved_at   = excluded.resolved_at,
               is_worktree   = excluded.is_worktree,
               worktree_path = excluded.worktree_path",
        )
        .bind(branch)
        .bind(feature_id)
        .bind(spec_id)
        .bind(active_mode)
        .bind(&providers_json)
        .bind(resolved_at)
        .bind(if is_worktree { 1i64 } else { 0i64 })
        .bind(worktree_path)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
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
    // When called from within an existing tokio runtime (e.g. MCP async handlers),
    // we can't start a second runtime on the same thread. Return an error so callers
    // can fall back to non-DB paths.
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err(anyhow!("SQLite block_on called from within async context"));
    }
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
