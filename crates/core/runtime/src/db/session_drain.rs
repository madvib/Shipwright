//! Session drain primitives for MCP lifecycle management.
//!
//! These functions use direct SQL instead of the event/projection path.
//! This is intentional: drain operations are on the MCP hot path and must
//! complete quickly. The event log is append-only and would add unnecessary
//! I/O for what are essentially status transitions.

use anyhow::{Result, anyhow};
use chrono::Utc;
use sqlx::Row;

use super::types::WorkspaceSessionDb;
use super::{block_on, open_db};

const S_COLS: &str =
    "id, workspace_id, workspace_branch, status, started_at, ended_at, \
     agent_id, primary_provider, goal, summary, updated_workspace_ids_json, \
     compiled_at, compile_error, config_generation_at_start, \
     tool_call_count, drained_at, mcp_provider, created_at, updated_at";

fn parse_row(row: &sqlx::sqlite::SqliteRow) -> WorkspaceSessionDb {
    let uwids_json: String = row.get(10);
    WorkspaceSessionDb {
        id: row.get(0),
        workspace_id: row.get(1),
        workspace_branch: row.get(2),
        status: row.get(3),
        started_at: row.get(4),
        ended_at: row.get(5),
        agent_id: row.get(6),
        primary_provider: row.get(7),
        goal: row.get(8),
        summary: row.get(9),
        updated_workspace_ids: serde_json::from_str(&uwids_json).unwrap_or_default(),
        compiled_at: row.get(11),
        compile_error: row.get(12),
        config_generation_at_start: row.get(13),
        tool_call_count: row.get(14),
        drained_at: row.get(15),
        mcp_provider: row.get(16),
        created_at: row.get(17),
        updated_at: row.get(18),
    }
}

/// Find a session in "draining" status for the given workspace + agent.
pub fn find_draining_session(
    workspace_id: &str,
    agent_id: &str,
) -> Result<Option<WorkspaceSessionDb>> {
    let mut conn = open_db()?;
    let row = block_on(async {
        sqlx::query(&format!(
            "SELECT {S_COLS} FROM workspace_session \
             WHERE workspace_id = ? AND agent_id = ? AND status = 'draining' \
             ORDER BY started_at DESC LIMIT 1"
        ))
        .bind(workspace_id)
        .bind(agent_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(parse_row))
}

/// Transition an active session to "draining" status.
/// Idempotent: draining sessions remain draining.
pub fn drain_session(session_id: &str) -> Result<()> {
    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    let affected = block_on(async {
        sqlx::query(
            "UPDATE workspace_session \
             SET status = 'draining', drained_at = ?, updated_at = ? \
             WHERE id = ? AND status IN ('active', 'draining')",
        )
        .bind(&now)
        .bind(&now)
        .bind(session_id)
        .execute(&mut conn)
        .await
    })?
    .rows_affected();
    if affected == 0 {
        return Err(anyhow!("session '{session_id}' not found or already ended"));
    }
    Ok(())
}

/// Resume a draining session back to active.
/// Idempotent: active sessions remain active.
pub fn resume_session(session_id: &str) -> Result<()> {
    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    let affected = block_on(async {
        sqlx::query(
            "UPDATE workspace_session \
             SET status = 'active', drained_at = NULL, updated_at = ? \
             WHERE id = ? AND status IN ('draining', 'active')",
        )
        .bind(&now)
        .bind(session_id)
        .execute(&mut conn)
        .await
    })?
    .rows_affected();
    if affected == 0 {
        return Err(anyhow!("session '{session_id}' not found or already ended"));
    }
    Ok(())
}

/// Close sessions stuck in "draining" longer than `grace_secs`.
/// Returns the number of sessions finalized.
pub fn cleanup_stale_draining(grace_secs: i64) -> Result<u64> {
    let mut conn = open_db()?;
    let cutoff = (Utc::now() - chrono::Duration::seconds(grace_secs)).to_rfc3339();
    let now = Utc::now().to_rfc3339();
    let result = block_on(async {
        sqlx::query(
            "UPDATE workspace_session \
             SET status = 'ended', ended_at = ?, updated_at = ? \
             WHERE status = 'draining' AND drained_at < ?",
        )
        .bind(&now)
        .bind(&now)
        .bind(&cutoff)
        .execute(&mut conn)
        .await
    })?;
    Ok(result.rows_affected())
}

/// Atomically increment tool_call_count for a session.
pub fn increment_tool_count(session_id: &str) -> Result<()> {
    let mut conn = open_db()?;
    block_on(async {
        sqlx::query(
            "UPDATE workspace_session \
             SET tool_call_count = tool_call_count + 1 \
             WHERE id = ?",
        )
        .bind(session_id)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}
