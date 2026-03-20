//! Platform DB — single database, single schema, no migration versioning.
//!
//! One file: `~/.ship/platform.db`.  Schema = code.

pub mod adrs;
pub mod agents;
pub mod branch;
pub mod branch_context;
pub mod events;
pub mod jobs;
pub mod kv;
pub mod managed_state;
pub mod notes;
pub mod schema;
pub mod session;
pub mod targets;
pub mod types;
pub mod workspace;
pub mod workspace_state;

use anyhow::{Context, Result, anyhow};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// The one database path: `~/.ship/platform.db`.
///
/// Never inside a project directory. Tests get automatic isolation
/// via `get_global_dir()`'s test-binary detection (per-thread temp dir).
pub fn db_path() -> Result<PathBuf> {
    Ok(crate::project::get_global_dir()?.join("platform.db"))
}

/// Open a connection, ensuring schema exists first.
///
/// On first call, migrates any `.ship/platform.db` from `ship_dir` to the
/// global location so existing data is preserved.
pub fn open_db(ship_dir: &Path) -> Result<SqliteConnection> {
    migrate_local_db_to_global(ship_dir);
    ensure_db(ship_dir)?;
    let path = db_path()?;
    connect(&path)
}

/// Ensure the schema exists. Idempotent — every statement uses
/// `CREATE TABLE IF NOT EXISTS`.
pub fn ensure_db(_ship_dir: &Path) -> Result<()> {
    let path = db_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let url = sqlite_url(&path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&opts).await })?;

    // Execute each statement individually — sqlx only runs one statement at a time.
    for stmt in schema::SCHEMA
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        block_on(async { sqlx::query(stmt).execute(&mut conn).await })
            .with_context(|| format!("schema init failed on: {}", &stmt[..stmt.len().min(80)]))?;
    }
    Ok(())
}

/// One-time migration: if `~/.ship/platform.db` is missing or empty but
/// `.ship/platform.db` inside the project has data, copy it to the global
/// location so existing data is preserved.
fn migrate_local_db_to_global(ship_dir: &Path) {
    let global_path = match db_path() {
        Ok(p) => p,
        Err(_) => return,
    };
    let global_has_data = global_path
        .metadata()
        .map(|m| m.len() > 0)
        .unwrap_or(false);
    if global_has_data {
        return;
    }
    let local_path = ship_dir.join("platform.db");
    let local_has_data = local_path
        .metadata()
        .map(|m| m.len() > 0)
        .unwrap_or(false);
    if !local_has_data {
        return;
    }
    if let Some(parent) = global_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match std::fs::copy(&local_path, &global_path) {
        Ok(_) => eprintln!(
            "migrated platform.db from {} → {}",
            local_path.display(),
            global_path.display()
        ),
        Err(e) => eprintln!(
            "warning: failed to migrate platform.db to {}: {e}",
            global_path.display()
        ),
    }
}

fn connect(path: &Path) -> Result<SqliteConnection> {
    let url = sqlite_url(path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?;
    block_on(async { SqliteConnection::connect_with(&opts).await })
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
        ensure_db(&ship_dir).unwrap();
    }

    #[test]
    fn test_ensure_db_idempotent() {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db(&ship_dir).unwrap();
        ensure_db(&ship_dir).unwrap();
    }

    #[test]
    fn test_open_db_returns_connection() {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        let conn = open_db(&ship_dir);
        assert!(conn.is_ok());
    }
}
