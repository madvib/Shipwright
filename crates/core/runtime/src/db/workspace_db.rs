//! Per-workspace SQLite database.
//!
//! Each workspace gets its own `events.db` at
//! `{ship_dir}/workspaces/{workspace_id}/events.db`.
//!
//! Schema: events, actors, and workspace_session tables —
//! same as platform.db, isolated per workspace.

use anyhow::{Context, Result};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::db::block_on;

/// Returns the path to the per-workspace SQLite DB file.
/// Path: `{ship_dir}/workspaces/{workspace_id}/events.db`
pub fn workspace_db_path(ship_dir: &Path, workspace_id: &str) -> PathBuf {
    ship_dir
        .join("workspaces")
        .join(workspace_id)
        .join("events.db")
}

/// Opens (or creates) the per-workspace DB, running migrations.
///
/// The workspace directory is created if it does not exist.
pub fn open_workspace_db(ship_dir: &Path, workspace_id: &str) -> Result<SqliteConnection> {
    let path = workspace_db_path(ship_dir, workspace_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create workspace db dir: {parent:?}"))?;
    }
    let url = sqlite_url(&path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&opts).await })?;

    block_on(async { sqlx::query("PRAGMA journal_mode = WAL").execute(&mut conn).await })?;
    block_on(async { sqlx::query("PRAGMA foreign_keys = ON").execute(&mut conn).await })?;

    let migrate_result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| {
            handle
                .block_on(sqlx::migrate!("./migrations/workspace").run(&mut conn))
                .map_err(|e| anyhow::anyhow!("workspace migration failed: {e}"))
        })
    } else {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .map_err(|e| anyhow::anyhow!("failed to create runtime: {e}"))?;
        rt.block_on(sqlx::migrate!("./migrations/workspace").run(&mut conn))
            .map_err(|e| anyhow::anyhow!("workspace migration failed: {e}"))
    };
    migrate_result?;

    Ok(conn)
}

/// Opens the per-workspace DB for a given workspace ID, resolving ship_dir automatically.
///
/// Equivalent to `open_workspace_db(&get_global_dir()?, workspace_id)`.
pub fn open_workspace_db_for_id(workspace_id: &str) -> Result<SqliteConnection> {
    let ship_dir = crate::project::get_global_dir()?;
    open_workspace_db(&ship_dir, workspace_id)
}

fn sqlite_url(path: &Path) -> String {
    let mut raw = path.to_string_lossy().replace('\\', "/");
    if !raw.starts_with('/') {
        raw = format!("/{raw}");
    }
    format!("sqlite://{raw}")
}
