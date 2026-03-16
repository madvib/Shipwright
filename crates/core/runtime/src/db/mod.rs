//! Platform DB — clean schema, no workflow-layer tables.
//! Replaces the monolithic state_db.rs.

pub mod adrs;
pub mod branch;
pub mod events;
pub mod jobs;
pub mod kv;
pub mod migrate_from_state_db;
pub mod notes;
pub mod schema;
pub mod workspace;

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub struct MigrationReport {
    pub db_path: PathBuf,
    pub created: bool,
    pub applied: usize,
}

/// Path to this project's SQLite DB: `~/.ship/state/<slug>/platform.db`.
///
/// Uses a separate file from `state_db`'s `ship.db` during the transition period.
/// Once `state_db` is retired, this becomes the canonical `ship.db`.
pub fn db_path(ship_dir: &Path) -> Result<PathBuf> {
    let global_dir = crate::project::get_global_dir()?;
    let slug = crate::project::project_slug_from_ship_dir(ship_dir);
    Ok(global_dir.join("state").join(slug).join("platform.db"))
}

/// Open a connection, running migrations first.
pub fn open_db(ship_dir: &Path) -> Result<SqliteConnection> {
    ensure_db(ship_dir)?;
    let path = db_path(ship_dir)?;
    connect(&path)
}

/// Run migrations without returning a connection. Idempotent.
pub fn ensure_db(ship_dir: &Path) -> Result<MigrationReport> {
    let path = db_path(ship_dir)?;
    run_migrations(&path, schema::MIGRATIONS)
}

fn connect(path: &Path) -> Result<SqliteConnection> {
    let url = sqlite_url(path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?;
    block_on(async { SqliteConnection::connect_with(&opts).await })
}

pub(crate) fn run_migrations(
    db_path: &Path,
    migrations: &[(&str, &str)],
) -> Result<MigrationReport> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let created = !db_path.exists();
    let url = sqlite_url(db_path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&opts).await })?;

    block_on(async {
        sqlx::query("PRAGMA journal_mode = WAL;")
            .execute(&mut conn)
            .await
    })?;
    block_on(async {
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&mut conn)
            .await
    })?;

    let mut applied = 0usize;
    for (version, ddl) in migrations {
        let exists = block_on(async {
            sqlx::query("SELECT version FROM schema_migrations WHERE version = ?")
                .bind(*version)
                .fetch_optional(&mut conn)
                .await
        })
        .ok()
        .flatten()
        .is_some();
        if exists {
            continue;
        }
        // Execute each statement individually — sqlx prepare only runs one statement at a time.
        for stmt in ddl.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            block_on(async { sqlx::query(stmt).execute(&mut conn).await })
                .with_context(|| format!("migration {version} failed for {}", db_path.display()))?;
        }
        block_on(async {
            sqlx::query(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?, ?)",
            )
            .bind(*version)
            .bind(Utc::now().to_rfc3339())
            .execute(&mut conn)
            .await
        })
        .with_context(|| format!("recording migration {version} failed"))?;
        applied += 1;
    }

    Ok(MigrationReport {
        db_path: db_path.to_path_buf(),
        created,
        applied,
    })
}

fn sqlite_url(path: &Path) -> String {
    let mut raw = path.to_string_lossy().replace('\\', "/");
    if !raw.starts_with('/') {
        raw = format!("/{raw}");
    }
    format!("sqlite://{raw}")
}

/// Run a future synchronously, compatible with both bare and Tokio contexts.
pub fn block_on<F, T>(future: F) -> Result<T>
where
    F: std::future::Future<Output = std::result::Result<T, sqlx::Error>>,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return tokio::task::block_in_place(|| {
            handle
                .block_on(future)
                .map_err(|e| anyhow!("SQLite operation failed: {e}"))
        });
    }
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|e| anyhow!("Failed to create SQLite runtime: {e}"))?;
    rt.block_on(future)
        .map_err(|e| anyhow!("SQLite operation failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::init_project;
    use tempfile::tempdir;

    #[test]
    fn test_ensure_db_creates_fresh() {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        let report = ensure_db(&ship_dir).unwrap();
        assert!(report.created);
        assert_eq!(report.applied, schema::MIGRATIONS.len());
    }

    #[test]
    fn test_ensure_db_idempotent() {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db(&ship_dir).unwrap();
        let report2 = ensure_db(&ship_dir).unwrap();
        assert!(!report2.created);
        assert_eq!(report2.applied, 0);
    }

    #[test]
    fn test_open_db_returns_connection() {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        let conn = open_db(&ship_dir);
        assert!(conn.is_ok());
    }
}
