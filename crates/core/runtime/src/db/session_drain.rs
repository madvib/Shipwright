//! Session drain primitives for MCP lifecycle management.
//!
//! Write operations emit events; projections apply the state changes.
//! Read operations query the workspace_session projection table directly.

use anyhow::{Result, anyhow};
use chrono::Utc;
use sqlx::Row;

use super::types::WorkspaceSessionDb;
use super::{block_on, open_db};
use crate::db::session_events::emit_session_drain_event;
use crate::events::types::{
    SessionDrainAborted, SessionDrainCompleted, SessionDrainStarted,
    SessionToolCountIncremented, event_types,
};

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

/// Transition an active session to "draining" status via event emission.
pub fn drain_session(session_id: &str) -> Result<()> {
    let workspace_id = lookup_session_workspace(session_id)?;
    let now = Utc::now().to_rfc3339();
    let payload = SessionDrainStarted { drained_at: now };
    emit_session_drain_event(
        event_types::SESSION_DRAIN_STARTED,
        session_id,
        &workspace_id,
        &payload,
    )?;
    Ok(())
}

/// Resume a draining session back to active via event emission.
pub fn resume_session(session_id: &str) -> Result<()> {
    let workspace_id = lookup_session_workspace(session_id)?;
    let now = Utc::now().to_rfc3339();
    let payload = SessionDrainAborted { resumed_at: now };
    emit_session_drain_event(
        event_types::SESSION_DRAIN_ABORTED,
        session_id,
        &workspace_id,
        &payload,
    )?;
    Ok(())
}

/// Close sessions stuck in "draining" longer than `grace_secs`.
/// Returns the number of sessions finalized.
pub fn cleanup_stale_draining(grace_secs: i64) -> Result<u64> {
    let mut conn = open_db()?;
    let cutoff = (Utc::now() - chrono::Duration::seconds(grace_secs)).to_rfc3339();
    let rows = block_on(async {
        sqlx::query(
            "SELECT id, workspace_id FROM workspace_session \
             WHERE status = 'draining' AND drained_at < ?",
        )
        .bind(&cutoff)
        .fetch_all(&mut conn)
        .await
    })?;
    let count = rows.len() as u64;
    let now = Utc::now().to_rfc3339();
    for row in &rows {
        let sid: String = row.get(0);
        let wid: String = row.get(1);
        let payload = SessionDrainCompleted { ended_at: now.clone() };
        let _ = emit_session_drain_event(
            event_types::SESSION_DRAIN_COMPLETED,
            &sid,
            &wid,
            &payload,
        );
    }
    Ok(count)
}

/// Increment tool_call_count for a session via event emission.
pub fn increment_tool_count(session_id: &str) -> Result<()> {
    let workspace_id = lookup_session_workspace(session_id)?;
    let payload = SessionToolCountIncremented {};
    emit_session_drain_event(
        event_types::SESSION_TOOL_COUNT_INCREMENTED,
        session_id,
        &workspace_id,
        &payload,
    )?;
    Ok(())
}

/// Look up the workspace_id for a session.
fn lookup_session_workspace(session_id: &str) -> Result<String> {
    let mut conn = open_db()?;
    let wid = block_on(async {
        sqlx::query_scalar::<_, String>(
            "SELECT workspace_id FROM workspace_session WHERE id = ?",
        )
        .bind(session_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    wid.ok_or_else(|| anyhow!("session '{session_id}' not found"))
}
