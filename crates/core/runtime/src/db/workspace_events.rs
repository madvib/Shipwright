//! Event-sourced workspace state writes.
//!
//! Each function builds an EventEnvelope, persists it to platform.db via
//! SqliteEventStore, and routes it through KernelRouter (when initialized)
//! for delivery to actor mailboxes.
//!
//! ADR GHihs2tn: all workspace lifecycle transitions must emit typed events.

use anyhow::Result;

use crate::db::block_on_anyhow;
use crate::events::store::EventStore;
use crate::events::types::event_types;
use crate::events::types::{
    WorkspaceActivated, WorkspaceAgentChanged, WorkspaceArchived, WorkspaceCompileFailed,
    WorkspaceCompiled, WorkspaceCreated, WorkspaceDeleted, WorkspaceReconciled,
    WorkspaceStatusChanged,
};
use crate::events::validator::{CallerKind, EmitContext};
use crate::events::{EventEnvelope, SqliteEventStore};

fn run_tx<P: serde::Serialize>(
    branch: &str,
    event_type: &'static str,
    payload: &P,
) -> Result<EventEnvelope> {
    let envelope = EventEnvelope::new(event_type, branch, payload)?
        .with_context(Some(branch), None)
        .with_actor_id(branch)
        .elevate();

    // Persist to platform.db for the reading path (CLI, sync, projections).
    SqliteEventStore::new()?.append(&envelope)?;

    // Route to actor mailboxes if the kernel router is running.
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
