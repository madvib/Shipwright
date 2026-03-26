//! Workspace state CRUD — the primary workspace persistence layer.
//!
//! These functions are used by `crate::workspace` for all workspace lifecycle
//! operations.  They write to the unified `workspace` table in platform.db.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::Row;

use super::types::{WorkspaceDbListRow, WorkspaceDbRow, WorkspaceUpsert};
use super::{block_on, open_db};

/// Retrieve the workspace record for the given branch, or None if none exists.
pub fn get_workspace_db(branch: &str) -> Result<Option<WorkspaceDbRow>> {
    let mut conn = open_db()?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT COALESCE(id, branch), workspace_type, status, active_agent, \
             providers_json, mcp_servers_json, skills_json, is_worktree, worktree_path, \
             last_activated_at, context_hash, COALESCE(config_generation, 0), compiled_at, compile_error \
             FROM workspace WHERE branch = ?",
        )
        .bind(branch)
        .fetch_optional(&mut conn)
        .await
    })?;
    if let Some(row) = row_opt {
        let id: String = row.get(0);
        let workspace_type: String = row.get(1);
        let status: String = row.get(2);
        let active_agent: Option<String> = row.get(3);
        let providers_json: String = row.get(4);
        let mcp_servers_json: String = row.get(5);
        let skills_json: String = row.get(6);
        let is_worktree: i64 = row.get(7);
        let worktree_path: Option<String> = row.get(8);
        let last_activated_at: Option<String> = row.get(9);
        let context_hash: Option<String> = row.get(10);
        let config_generation: i64 = row.get(11);
        let compiled_at: Option<String> = row.get(12);
        let compile_error: Option<String> = row.get(13);
        let providers: Vec<String> = serde_json::from_str(&providers_json).unwrap_or_default();
        let mcp_servers: Vec<String> = serde_json::from_str(&mcp_servers_json).unwrap_or_default();
        let skills: Vec<String> = serde_json::from_str(&skills_json).unwrap_or_default();
        Ok(Some((
            id,
            workspace_type,
            status,
            active_agent,
            providers,
            mcp_servers,
            skills,
            is_worktree != 0,
            worktree_path,
            last_activated_at,
            context_hash,
            config_generation,
            compiled_at,
            compile_error,
        )))
    } else {
        Ok(None)
    }
}

pub fn list_workspaces_db() -> Result<Vec<WorkspaceDbListRow>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT branch, COALESCE(id, branch), workspace_type, status, active_agent, \
             providers_json, mcp_servers_json, skills_json, is_worktree, worktree_path, \
             last_activated_at, context_hash, COALESCE(config_generation, 0), compiled_at, compile_error \
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
        let workspace_type: String = row.get(2);
        let status: String = row.get(3);
        let active_agent: Option<String> = row.get(4);
        let providers_json: String = row.get(5);
        let mcp_servers_json: String = row.get(6);
        let skills_json: String = row.get(7);
        let is_worktree: i64 = row.get(8);
        let worktree_path: Option<String> = row.get(9);
        let last_activated_at: Option<String> = row.get(10);
        let context_hash: Option<String> = row.get(11);
        let config_generation: i64 = row.get(12);
        let compiled_at: Option<String> = row.get(13);
        let compile_error: Option<String> = row.get(14);
        let providers: Vec<String> = serde_json::from_str(&providers_json).unwrap_or_default();
        let mcp_servers: Vec<String> = serde_json::from_str(&mcp_servers_json).unwrap_or_default();
        let skills: Vec<String> = serde_json::from_str(&skills_json).unwrap_or_default();

        result.push((
            branch,
            id,
            workspace_type,
            status,
            active_agent,
            providers,
            mcp_servers,
            skills,
            is_worktree != 0,
            worktree_path,
            last_activated_at,
            context_hash,
            config_generation,
            compiled_at,
            compile_error,
        ));
    }
    Ok(result)
}

/// Upsert the workspace record for the given branch.
pub fn upsert_workspace_db(record: WorkspaceUpsert<'_>) -> Result<()> {
    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    let providers_json = serde_json::to_string(record.providers)
        .with_context(|| "Failed to serialize workspace providers")?;
    let mcp_servers_json = serde_json::to_string(record.mcp_servers)
        .with_context(|| "Failed to serialize workspace mcp servers")?;
    let skills_json = serde_json::to_string(record.skills)
        .with_context(|| "Failed to serialize workspace skills")?;
    block_on(async {
        sqlx::query(
            "INSERT INTO workspace (branch, id, workspace_type, status, active_agent, \
             providers_json, mcp_servers_json, skills_json, is_worktree, worktree_path, \
             last_activated_at, context_hash, config_generation, compiled_at, compile_error, \
             created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(branch) DO UPDATE SET \
               id                = excluded.id, \
               workspace_type    = excluded.workspace_type, \
               status            = excluded.status, \
               active_agent      = excluded.active_agent, \
               providers_json    = excluded.providers_json, \
               mcp_servers_json  = excluded.mcp_servers_json, \
               skills_json       = excluded.skills_json, \
               is_worktree       = excluded.is_worktree, \
               worktree_path     = excluded.worktree_path, \
               last_activated_at = excluded.last_activated_at, \
               context_hash      = excluded.context_hash, \
               config_generation = excluded.config_generation, \
               compiled_at       = excluded.compiled_at, \
               compile_error     = excluded.compile_error, \
               updated_at        = excluded.updated_at",
        )
        .bind(record.branch)
        .bind(record.workspace_id)
        .bind(record.workspace_type)
        .bind(record.status)
        .bind(record.active_agent)
        .bind(&providers_json)
        .bind(&mcp_servers_json)
        .bind(&skills_json)
        .bind(if record.is_worktree { 1i64 } else { 0i64 })
        .bind(record.worktree_path)
        .bind(record.last_activated_at)
        .bind(record.context_hash)
        .bind(record.config_generation)
        .bind(record.compiled_at)
        .bind(record.compile_error)
        .bind(&now)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
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

/// Mark any currently active workspace as idle except `active_branch`.
pub fn demote_other_active_workspaces_db(active_branch: &str) -> Result<()> {
    let mut conn = open_db()?;
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET status = 'archived' WHERE status = 'active' AND branch != ?",
        )
        .bind(active_branch)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}
