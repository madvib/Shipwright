//! Workspace state CRUD — the primary workspace persistence layer.
//!
//! These functions are used by `crate::workspace` for all workspace lifecycle
//! operations.  They write to the unified `workspace` table in platform.db.

use anyhow::Result;
use sqlx::Row;

use super::types::{WorkspaceDbListRow, WorkspaceDbRow};
use super::{block_on, open_db};

/// Retrieve the workspace record by id (or branch as fallback), or None if none exists.
pub fn get_workspace_by_id_db(id: &str) -> Result<Option<(String, WorkspaceDbRow)>> {
    let mut conn = open_db()?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT branch, COALESCE(id, branch), status, is_worktree, worktree_path, \
             active_agent, last_activated_at \
             FROM workspace WHERE id = ? OR branch = ? LIMIT 1",
        )
        .bind(id)
        .bind(id)
        .fetch_optional(&mut conn)
        .await
    })?;
    if let Some(row) = row_opt {
        let branch: String = row.get(0);
        let rec_id: String = row.get(1);
        let status: String = row.get(2);
        let is_worktree: i64 = row.get(3);
        let worktree_path: Option<String> = row.get(4);
        let active_agent: Option<String> = row.get(5);
        let last_activated_at: Option<String> = row.get(6);
        Ok(Some((
            branch,
            (
                rec_id,
                status,
                is_worktree != 0,
                worktree_path,
                active_agent,
                last_activated_at,
            ),
        )))
    } else {
        Ok(None)
    }
}

/// Retrieve the workspace record for the given branch, or None if none exists.
pub fn get_workspace_db(branch: &str) -> Result<Option<WorkspaceDbRow>> {
    let mut conn = open_db()?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT COALESCE(id, branch), status, is_worktree, worktree_path, \
             active_agent, last_activated_at \
             FROM workspace WHERE branch = ?",
        )
        .bind(branch)
        .fetch_optional(&mut conn)
        .await
    })?;
    if let Some(row) = row_opt {
        let id: String = row.get(0);
        let status: String = row.get(1);
        let is_worktree: i64 = row.get(2);
        let worktree_path: Option<String> = row.get(3);
        let active_agent: Option<String> = row.get(4);
        let last_activated_at: Option<String> = row.get(5);
        Ok(Some((
            id,
            status,
            is_worktree != 0,
            worktree_path,
            active_agent,
            last_activated_at,
        )))
    } else {
        Ok(None)
    }
}

pub fn list_workspaces_db() -> Result<Vec<WorkspaceDbListRow>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT branch, COALESCE(id, branch), status, is_worktree, worktree_path, \
             active_agent, last_activated_at \
             FROM workspace \
             ORDER BY \
               CASE status \
                 WHEN 'active' THEN 0 \
                 WHEN 'archived' THEN 1 \
                 ELSE 2 \
               END, \
               last_activated_at DESC",
        )
        .fetch_all(&mut conn)
        .await
    })?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let branch: String = row.get(0);
        let id: String = row.get(1);
        let status: String = row.get(2);
        let is_worktree: i64 = row.get(3);
        let worktree_path: Option<String> = row.get(4);
        let active_agent: Option<String> = row.get(5);
        let last_activated_at: Option<String> = row.get(6);

        result.push((
            branch,
            id,
            status,
            is_worktree != 0,
            worktree_path,
            active_agent,
            last_activated_at,
        ));
    }
    Ok(result)
}

/// Set tmux_session_name for the workspace identified by branch.
///
/// Returns `true` if the workspace was found and updated, `false` if not found.
pub fn set_workspace_tmux_session_db(branch: &str, session_name: Option<&str>) -> Result<bool> {
    let mut conn = open_db()?;
    let rows_affected = block_on(async {
        sqlx::query("UPDATE workspace SET tmux_session_name = ? WHERE branch = ?")
            .bind(session_name)
            .bind(branch)
            .execute(&mut conn)
            .await
    })?
    .rows_affected();
    Ok(rows_affected > 0)
}

/// Update both worktree_path and tmux_session_name for a workspace in one call.
/// Sets is_worktree = 1 implicitly. Returns true if the workspace was found and updated.
pub fn set_workspace_started_db(
    branch: &str,
    worktree_path: &str,
    tmux_session: &str,
) -> Result<bool> {
    let mut conn = open_db()?;
    let rows_affected = block_on(async {
        sqlx::query(
            "UPDATE workspace SET worktree_path = ?, tmux_session_name = ?, is_worktree = 1 WHERE branch = ?",
        )
        .bind(worktree_path)
        .bind(tmux_session)
        .bind(branch)
        .execute(&mut conn)
        .await
    })?
    .rows_affected();
    Ok(rows_affected > 0)
}

/// Delete workspace state for a branch, including any session history.
pub fn delete_workspace_db(branch: &str) -> Result<bool> {
    let mut conn = open_db()?;
    let workspace_id = block_on(async {
        sqlx::query_scalar::<_, String>(
            "SELECT COALESCE(id, branch) FROM workspace WHERE branch = ?",
        )
        .bind(branch)
        .fetch_optional(&mut conn)
        .await
    })?;

    let Some(workspace_id) = workspace_id else {
        return Ok(false);
    };

    let deleted = block_on(async {
        sqlx::query("DELETE FROM workspace_session WHERE workspace_id = ? OR workspace_branch = ?")
            .bind(&workspace_id)
            .bind(branch)
            .execute(&mut conn)
            .await?;

        let result = sqlx::query("DELETE FROM workspace WHERE branch = ?")
            .bind(branch)
            .execute(&mut conn)
            .await?;

        Ok::<bool, sqlx::Error>(result.rows_affected() > 0)
    })?;

    Ok(deleted)
}

/// Retrieve the tmux_session_name for a workspace by id or branch.
pub fn get_workspace_tmux_session_db(id: &str) -> Result<Option<String>> {
    let mut conn = open_db()?;
    let row_opt = block_on(async {
        sqlx::query_scalar::<_, Option<String>>(
            "SELECT tmux_session_name FROM workspace WHERE id = ? OR branch = ? LIMIT 1",
        )
        .bind(id)
        .bind(id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row_opt.flatten())
}

/// Retrieve the current config_generation for a workspace by id or branch.
pub fn get_workspace_config_generation_db(id: &str) -> Result<Option<i64>> {
    let mut conn = open_db()?;
    let row_opt = block_on(async {
        sqlx::query_scalar::<_, i64>(
            "SELECT config_generation FROM workspace WHERE id = ? OR branch = ? LIMIT 1",
        )
        .bind(id)
        .bind(id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row_opt)
}

/// Mark any currently active workspace as archived except `active_branch`.
///
/// Emits a `workspace.archived` event per demoted workspace.
pub fn demote_other_active_workspaces_db(active_branch: &str) -> Result<()> {
    use crate::db::workspace_events::emit_workspace_archived;

    let mut conn = open_db()?;
    // Collect branches to demote before mutating.
    let branches_to_demote: Vec<String> = block_on(async {
        sqlx::query_scalar::<_, String>(
            "SELECT branch FROM workspace WHERE status = 'active' AND branch != ?",
        )
        .bind(active_branch)
        .fetch_all(&mut conn)
        .await
    })?;

    // Emit workspace.archived for each demoted workspace.
    // The WorkspaceProjection handler updates the workspace row.
    for branch in &branches_to_demote {
        emit_workspace_archived(branch, &crate::events::types::WorkspaceArchived {})?;
    }

    Ok(())
}
