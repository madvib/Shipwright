//! Lifecycle-event-aware workspace functions.
//!
//! Each function emits a typed workspace event via the EventRouter, then
//! applies WorkspaceProjection synchronously so the workspace table reflects
//! the change immediately. The async projection task provides eventual
//! consistency for any events that arrive via broadcast only.
//!
//! ADR GHihs2tn: all workspace lifecycle transitions must emit typed events.

use crate::db::workspace_events::{
    emit_workspace_activated, emit_workspace_agent_changed, emit_workspace_archived,
    emit_workspace_compile_failed, emit_workspace_compiled, emit_workspace_created,
    emit_workspace_deleted, emit_workspace_status_changed,
};
use crate::events::types::{
    WorkspaceActivated, WorkspaceAgentChanged, WorkspaceArchived, WorkspaceCompileFailed,
    WorkspaceCompiled, WorkspaceCreated, WorkspaceStatusChanged,
};
use crate::events::EventEnvelope;
use crate::projections::{Projection, WorkspaceProjection};
use anyhow::Result;
use std::path::Path;

use super::helpers::workspace_id_from_branch;

// ── helper ────────────────────────────────────────────────────────────────────

/// Apply WorkspaceProjection to platform.db synchronously.
/// Errors are logged but not propagated -- the event is already persisted.
fn sync_workspace_projection(envelope: &EventEnvelope) {
    let proj = WorkspaceProjection::new();
    match crate::db::open_db() {
        Ok(mut conn) => {
            if let Err(e) = proj.apply(envelope, &mut conn) {
                eprintln!(
                    "[workspace-upsert] projection error for {}: {}",
                    envelope.event_type, e
                );
            }
        }
        Err(e) => {
            eprintln!("[workspace-upsert] db open error: {e}");
        }
    }
}

// ── public event-emitting functions ──────────────────────────────────────────

pub fn upsert_workspace_on_activate(
    _ship_dir: &Path,
    branch: &str,
    agent_id: Option<&str>,
    providers: &[String],
) -> Result<()> {
    let payload = WorkspaceActivated {
        agent_id: agent_id.map(str::to_string),
        providers: providers.to_vec(),
    };
    let envelope = emit_workspace_activated(branch, &payload)?;
    sync_workspace_projection(&envelope);
    Ok(())
}

pub fn upsert_workspace_on_compiled(
    _ship_dir: &Path,
    branch: &str,
    config_generation: u32,
    duration_ms: u64,
) -> Result<()> {
    let payload = WorkspaceCompiled {
        config_generation,
        duration_ms,
    };
    let envelope = emit_workspace_compiled(branch, &payload)?;
    sync_workspace_projection(&envelope);
    Ok(())
}

pub fn upsert_workspace_on_compile_failed(
    _ship_dir: &Path,
    branch: &str,
    error: &str,
) -> Result<()> {
    let payload = WorkspaceCompileFailed { error: error.to_string() };
    let envelope = emit_workspace_compile_failed(branch, &payload)?;
    sync_workspace_projection(&envelope);
    Ok(())
}

pub fn upsert_workspace_on_archived(_ship_dir: &Path, branch: &str) -> Result<()> {
    let payload = WorkspaceArchived {};
    let envelope = emit_workspace_archived(branch, &payload)?;
    sync_workspace_projection(&envelope);
    Ok(())
}

pub fn upsert_workspace_on_created(
    _ship_dir: &Path,
    branch: &str,
    is_worktree: bool,
    worktree_path: Option<&str>,
    active_agent: Option<&str>,
    status: &str,
) -> Result<()> {
    let workspace_id = workspace_id_from_branch(branch);
    let payload = WorkspaceCreated {
        workspace_id,
        workspace_type: "feature".to_string(),
        status: status.to_string(),
        active_agent: active_agent.map(str::to_string),
        providers: Vec::new(),
        mcp_servers: Vec::new(),
        skills: Vec::new(),
        is_worktree,
        worktree_path: worktree_path.map(str::to_string),
    };
    let envelope = emit_workspace_created(branch, &payload)?;
    sync_workspace_projection(&envelope);
    Ok(())
}

pub fn upsert_workspace_on_deleted(_ship_dir: &Path, branch: &str) -> Result<()> {
    let envelope = emit_workspace_deleted(branch)?;
    sync_workspace_projection(&envelope);
    Ok(())
}

pub fn upsert_workspace_on_status_changed(
    _ship_dir: &Path,
    branch: &str,
    old_status: &str,
    new_status: &str,
) -> Result<()> {
    let payload = WorkspaceStatusChanged {
        old_status: old_status.to_string(),
        new_status: new_status.to_string(),
    };
    let envelope = emit_workspace_status_changed(branch, &payload)?;
    sync_workspace_projection(&envelope);
    Ok(())
}

pub fn emit_workspace_archived_event(_ship_dir: &Path, branch: &str) -> Result<()> {
    let payload = WorkspaceArchived {};
    let envelope = emit_workspace_archived(branch, &payload)?;
    sync_workspace_projection(&envelope);
    Ok(())
}

pub fn emit_workspace_agent_changed_event(
    _ship_dir: &Path,
    branch: &str,
    agent_id: Option<&str>,
) -> Result<()> {
    let payload = WorkspaceAgentChanged { agent_id: agent_id.map(str::to_string) };
    let envelope = emit_workspace_agent_changed(branch, &payload)?;
    sync_workspace_projection(&envelope);
    Ok(())
}
