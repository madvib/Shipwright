//! Event-sourced workspace state writes.
//!
//! Each function builds an EventEnvelope, persists the event and applies the
//! workspace projection in a single SQLite transaction on one connection.
//! Kernel routing happens after commit (fire-and-forget).
//!
//! ADR GHihs2tn: all workspace lifecycle transitions must emit typed events.

use anyhow::Result;

use crate::db::{block_on, block_on_anyhow, open_db};
use crate::events::store::append_event_with_conn;
use crate::events::types::event_types;
use crate::events::types::{
    WorkspaceActivated, WorkspaceAgentChanged, WorkspaceArchived, WorkspaceCompileFailed,
    WorkspaceCompiled, WorkspaceCreated, WorkspaceDeleted, WorkspaceReconciled,
    WorkspaceStarted, WorkspaceStatusChanged, WorkspaceTmuxAssigned,
};
use crate::events::validator::{CallerKind, EmitContext};
use crate::events::EventEnvelope;
use crate::projections::{Projection, WorkspaceProjection};

fn run_tx<P: serde::Serialize>(
    branch: &str,
    event_type: &'static str,
    payload: &P,
) -> Result<EventEnvelope> {
    let envelope = EventEnvelope::new(event_type, branch, payload)?
        .with_context(Some(branch), None)
        .with_actor_id(branch)
        .elevate();

    // Single connection, single transaction: event + projection.
    let mut conn = open_db()?;
    block_on(async { sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await })?;

    if let Err(e) = append_event_with_conn(&envelope, &mut conn) {
        let _ = block_on(async { sqlx::query("ROLLBACK").execute(&mut conn).await });
        return Err(e);
    }

    let proj = WorkspaceProjection::new();
    if let Err(e) = proj.apply(&envelope, &mut conn) {
        let _ = block_on(async { sqlx::query("ROLLBACK").execute(&mut conn).await });
        return Err(e);
    }

    block_on(async { sqlx::query("COMMIT").execute(&mut conn).await })?;

    // Route to actor mailboxes after commit (fire-and-forget).
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: Some(branch.to_string()),
        session_id: None,
    };
    if let Some(kr) = crate::events::kernel_router() {
        let _ = block_on_anyhow(async { kr.lock().await.route(envelope.clone(), &ctx).await });
    }

    Ok(envelope)
}

// ── public API ────────────────────────────────────────────────────────────────

pub fn emit_workspace_activated(branch: &str, payload: &WorkspaceActivated) -> Result<EventEnvelope> {
    run_tx(branch, event_types::WORKSPACE_ACTIVATED, payload)
}

pub fn emit_workspace_compiled(branch: &str, payload: &WorkspaceCompiled) -> Result<EventEnvelope> {
    run_tx(branch, event_types::WORKSPACE_COMPILED, payload)
}

pub fn emit_workspace_compile_failed(
    branch: &str,
    payload: &WorkspaceCompileFailed,
) -> Result<EventEnvelope> {
    run_tx(branch, event_types::WORKSPACE_COMPILE_FAILED, payload)
}

pub fn emit_workspace_archived(branch: &str, payload: &WorkspaceArchived) -> Result<EventEnvelope> {
    run_tx(branch, event_types::WORKSPACE_ARCHIVED, payload)
}

pub fn emit_workspace_created(branch: &str, payload: &WorkspaceCreated) -> Result<EventEnvelope> {
    run_tx(branch, event_types::WORKSPACE_CREATED, payload)
}

pub fn emit_workspace_status_changed(
    branch: &str,
    payload: &WorkspaceStatusChanged,
) -> Result<EventEnvelope> {
    run_tx(branch, event_types::WORKSPACE_STATUS_CHANGED, payload)
}

pub fn emit_workspace_deleted(branch: &str) -> Result<EventEnvelope> {
    let payload = WorkspaceDeleted { branch: branch.to_string() };
    run_tx(branch, event_types::WORKSPACE_DELETED, &payload)
}

pub fn emit_workspace_agent_changed(branch: &str, payload: &WorkspaceAgentChanged) -> Result<EventEnvelope> {
    run_tx(branch, event_types::WORKSPACE_AGENT_CHANGED, payload)
}

pub fn emit_workspace_reconciled(branch: &str, payload: &WorkspaceReconciled) -> Result<EventEnvelope> {
    run_tx(branch, event_types::WORKSPACE_RECONCILED, payload)
}

pub fn emit_workspace_tmux_assigned(branch: &str, payload: &WorkspaceTmuxAssigned) -> Result<EventEnvelope> {
    run_tx(branch, event_types::WORKSPACE_TMUX_ASSIGNED, payload)
}

pub fn emit_workspace_started(branch: &str, payload: &WorkspaceStarted) -> Result<EventEnvelope> {
    run_tx(branch, event_types::WORKSPACE_STARTED, payload)
}
