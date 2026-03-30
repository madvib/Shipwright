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
use super::types::Workspace;

// ── helper ────────────────────────────────────────────────────────────────────

/// Apply WorkspaceProjection to platform.db synchronously.
/// Errors are logged but not propagated — the event is already persisted.
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

pub fn upsert_workspace_on_activate(_ship_dir: &Path, workspace: &Workspace) -> Result<()> {
    let payload = WorkspaceActivated {
        agent_id: workspace.active_agent.clone(),
        providers: workspace.providers.clone(),
    };
    let envelope = emit_workspace_activated(&workspace.branch, &payload)?;
    sync_workspace_projection(&envelope);
    Ok(())
}

pub fn upsert_workspace_on_compiled(
    _ship_dir: &Path,
    workspace: &Workspace,
    duration_ms: u64,
) -> Result<()> {
    let payload = WorkspaceCompiled {
        config_generation: workspace.config_generation as u32,
        duration_ms,
    };
    let envelope = emit_workspace_compiled(&workspace.branch, &payload)?;
    sync_workspace_projection(&envelope);
    Ok(())
}

pub fn upsert_workspace_on_compile_failed(
    _ship_dir: &Path,
    workspace: &Workspace,
    error: &str,
) -> Result<()> {
    let payload = WorkspaceCompileFailed { error: error.to_string() };
    let envelope = emit_workspace_compile_failed(&workspace.branch, &payload)?;
    sync_workspace_projection(&envelope);
    Ok(())
}

pub fn upsert_workspace_on_archived(_ship_dir: &Path, workspace: &Workspace) -> Result<()> {
    let payload = WorkspaceArchived {};
    let envelope = emit_workspace_archived(&workspace.branch, &payload)?;
    sync_workspace_projection(&envelope);
    Ok(())
}

pub fn upsert_workspace_on_created(_ship_dir: &Path, workspace: &Workspace) -> Result<()> {
    let workspace_type = workspace.workspace_type.to_string();
    let status = workspace.status.to_string();
    let workspace_id = if workspace.id.trim().is_empty() {
        workspace_id_from_branch(&workspace.branch)
    } else {
        workspace.id.clone()
    };
    let payload = WorkspaceCreated {
        workspace_id,
        workspace_type,
        status,
        active_agent: workspace.active_agent.clone(),
        providers: workspace.providers.clone(),
        mcp_servers: workspace.mcp_servers.clone(),
        skills: workspace.skills.clone(),
        is_worktree: workspace.is_worktree,
        worktree_path: workspace.worktree_path.clone(),
    };
    let envelope = emit_workspace_created(&workspace.branch, &payload)?;
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
    workspace: &Workspace,
    old_status: &str,
    new_status: &str,
) -> Result<()> {
    let payload = WorkspaceStatusChanged {
        old_status: old_status.to_string(),
        new_status: new_status.to_string(),
    };
    let envelope = emit_workspace_status_changed(&workspace.branch, &payload)?;
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

pub fn upsert_workspace_on_agent_changed(_ship_dir: &Path, workspace: &Workspace) -> Result<()> {
    let payload = WorkspaceAgentChanged { agent_id: workspace.active_agent.clone() };
    let envelope = emit_workspace_agent_changed(&workspace.branch, &payload)?;
    sync_workspace_projection(&envelope);
    Ok(())
}
