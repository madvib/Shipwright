//! Branch context — branch ↔ entity link tracking.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use std::path::Path;

use super::{block_on, open_db};

/// Look up which linked entity is associated with `branch`.
/// Returns `(link_type, link_id)` or `None`.
pub fn get_branch_link(_ship_dir: &Path, branch: &str) -> Result<Option<(String, String)>> {
    let mut conn = open_db()?;
    let row_opt = block_on(async {
        sqlx::query("SELECT link_type, link_id FROM branch_context WHERE branch = ?")
            .bind(branch)
            .fetch_optional(&mut conn)
            .await
    })?;
    if let Some(row) = row_opt {
        Ok(Some((row.get(0), row.get(1))))
    } else {
        Ok(None)
    }
}

/// Record that `branch` is associated with `link_type` and entity id.
pub fn set_branch_link(
    _ship_dir: &Path,
    branch: &str,
    link_type: &str,
    link_id: &str,
) -> Result<()> {
    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO branch_context (branch, link_type, link_id, last_synced)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(branch) DO UPDATE SET
               link_type = excluded.link_type,
               link_id = excluded.link_id,
               last_synced = excluded.last_synced",
        )
        .bind(branch)
        .bind(link_type)
        .bind(link_id)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

/// Remove branch link mapping for `branch`.
pub fn clear_branch_link(_ship_dir: &Path, branch: &str) -> Result<()> {
    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("DELETE FROM branch_context WHERE branch = ?")
            .bind(branch)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

/// Legacy alias.
pub fn get_branch_doc(_ship_dir: &Path, branch: &str) -> Result<Option<(String, String)>> {
    get_branch_link(_ship_dir, branch)
}

/// Legacy alias.
pub fn set_branch_doc(_ship_dir: &Path, branch: &str, doc_type: &str, doc_uuid: &str) -> Result<()> {
    set_branch_link(_ship_dir, branch, doc_type, doc_uuid)
}

/// Legacy alias.
pub fn clear_branch_doc(_ship_dir: &Path, branch: &str) -> Result<()> {
    clear_branch_link(_ship_dir, branch)
}
