//! Platform DB — single database, single schema, no migration versioning.
//!
//! One file: `~/.ship/platform.db`.  Schema = code.

pub mod adrs;
pub mod agents;
pub mod branch;
pub mod branch_context;
pub mod events;
#[cfg(test)]
mod events_tests;
pub mod file_claims;
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
use sqlx::{Connection, Row, SqliteConnection};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Bump when the schema changes to invalidate the fast-path.
const SCHEMA_VERSION: i32 = 1;

/// The one database path: `~/.ship/platform.db`.
///
/// Never inside a project directory. Tests get automatic isolation
/// via `get_global_dir()`'s test-binary detection (per-thread temp dir).
pub fn db_path() -> Result<PathBuf> {
    Ok(crate::project::get_global_dir()?.join("platform.db"))
}

/// Open a connection, ensuring schema exists first.
pub fn open_db(ship_dir: &Path) -> Result<SqliteConnection> {
    ensure_db(ship_dir)?;
    let path = db_path()?;
    connect(&path)
}

/// Ensure the schema exists. Idempotent — every statement uses
/// `CREATE TABLE IF NOT EXISTS`. Uses `PRAGMA user_version` as a
/// fast-path: if the version matches, skip all DDL.
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

    // Fast-path: schema already initialized.
    let version: i32 = block_on(async {
        sqlx::query("PRAGMA user_version")
            .fetch_one(&mut conn)
            .await
    })
    .map(|row| row.get::<i32, _>(0))
    .unwrap_or(0);
    if version == SCHEMA_VERSION {
        return Ok(());
    }

    // Execute each statement individually — sqlx only runs one statement at a time.
    // Strip SQL comments first since they may contain semicolons.
    for part in schema::SCHEMA_PARTS {
        let stripped: String = part
            .lines()
            .filter(|l| !l.trim_start().starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");
        for stmt in stripped.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            block_on(async { sqlx::query(stmt).execute(&mut conn).await }).with_context(|| {
                format!("schema init failed on: {}", &stmt[..stmt.len().min(80)])
            })?;
        }
    }

    // Stamp so subsequent calls take the fast-path.
    block_on(async {
        sqlx::query(&format!("PRAGMA user_version = {SCHEMA_VERSION}"))
            .execute(&mut conn)
            .await
    })?;
    Ok(())
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
