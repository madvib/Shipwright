//! Event-sourced workspace state writes.
//!
//! Each public function builds an EventEnvelope and emits it through the global
//! EventRouter (validate → persist → broadcast), then dispatches synchronously
//! to WorkspaceProjection so derived state is available immediately.
//!
//! ADR GHihs2tn: all workspace lifecycle transitions must emit typed events.

use anyhow::Result;

use crate::db::{block_on_anyhow, open_db};
use crate::events::global_router::router;
use crate::events::types::event_types;
use crate::events::types::{
    WorkspaceActivated, WorkspaceAgentChanged, WorkspaceArchived, WorkspaceCompileFailed,
    WorkspaceCompiled, WorkspaceCreated, WorkspaceDeleted, WorkspaceStatusChanged,
};
use crate::events::validator::{CallerKind, EmitContext};
use crate::events::EventEnvelope;
use crate::projections::{Projection, WorkspaceProjection};

// ── core emit ────────────────────────────────────────────────────────────────

/// Build envelope, emit via router, then dispatch to workspace projection.
fn run_tx<P: serde::Serialize>(
    branch: &str,
    event_type: &'static str,
    payload: &P,
) -> Result<()> {
    let envelope = EventEnvelope::new(event_type, branch, payload)?
        .with_context(Some(branch), None)
        .with_actor_id(branch)
        .elevate();

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: Some(branch.to_string()),
        session_id: None,
    };

    // Router: validate → persist → broadcast
    block_on_anyhow(router().emit(envelope.clone(), &ctx))?;

    // Synchronous projection: update workspace table immediately so callers
    // can read the state right after emit returns.
    let proj = WorkspaceProjection::new();
    if proj.event_types().contains(&event_type) {
        let mut conn = open_db()?;
        if let Err(e) = proj.apply(&envelope, &mut conn) {
            eprintln!("[workspace-events] projection error for {event_type}: {e}");
        }
    }

    Ok(())
}

// ── public API ────────────────────────────────────────────────────────────────

/// Emit `workspace.activated` and let the projection update workspace state.
pub fn emit_workspace_activated(branch: &str, payload: &WorkspaceActivated) -> Result<()> {
    run_tx(branch, event_types::WORKSPACE_ACTIVATED, payload)
}

/// Emit `workspace.compiled` and let the projection update workspace state.
pub fn emit_workspace_compiled(branch: &str, payload: &WorkspaceCompiled) -> Result<()> {
    run_tx(branch, event_types::WORKSPACE_COMPILED, payload)
}

/// Emit `workspace.compile_failed` and let the projection update workspace state.
pub fn emit_workspace_compile_failed(
    branch: &str,
    payload: &WorkspaceCompileFailed,
) -> Result<()> {
    run_tx(branch, event_types::WORKSPACE_COMPILE_FAILED, payload)
}

/// Emit `workspace.archived` and let the projection update workspace state.
pub fn emit_workspace_archived(branch: &str, payload: &WorkspaceArchived) -> Result<()> {
    run_tx(branch, event_types::WORKSPACE_ARCHIVED, payload)
}

/// Emit `workspace.created` and let the projection insert workspace state.
pub fn emit_workspace_created(branch: &str, payload: &WorkspaceCreated) -> Result<()> {
    run_tx(branch, event_types::WORKSPACE_CREATED, payload)
}

/// Emit `workspace.status_changed` and let the projection update workspace state.
pub fn emit_workspace_status_changed(
    branch: &str,
    payload: &WorkspaceStatusChanged,
) -> Result<()> {
    run_tx(branch, event_types::WORKSPACE_STATUS_CHANGED, payload)
}

/// Emit `workspace.deleted` for a branch that is about to be (or was) deleted.
pub fn emit_workspace_deleted(branch: &str) -> Result<()> {
    let payload = WorkspaceDeleted {
        branch: branch.to_string(),
    };
    run_tx(branch, event_types::WORKSPACE_DELETED, &payload)
}

/// Emit `workspace.agent_changed` and let the projection update workspace state.
pub fn emit_workspace_agent_changed(branch: &str, payload: &WorkspaceAgentChanged) -> Result<()> {
    run_tx(branch, event_types::WORKSPACE_AGENT_CHANGED, payload)
}
