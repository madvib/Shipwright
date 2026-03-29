//! Event-sourced workspace state writes.
//!
//! Each public function inserts a typed event into the `events` table inside a
//! BEGIN/COMMIT block, then dispatches the event through the global EventBus so
//! projections (e.g. WorkspaceProjection) maintain derived state.
//!
//! ADR GHihs2tn: all workspace lifecycle transitions must emit typed events.

use std::sync::OnceLock;

use anyhow::Result;

use crate::db::{block_on, open_db};
use crate::events::types::event_types;
use crate::events::types::{
    WorkspaceActivated, WorkspaceAgentChanged, WorkspaceArchived, WorkspaceCompileFailed,
    WorkspaceCompiled, WorkspaceCreated, WorkspaceDeleted, WorkspaceStatusChanged,
};
use crate::events::EventEnvelope;
use crate::projections::{EventBus, SessionProjection, WorkspaceProjection};

// ── SQL constants ─────────────────────────────────────────────────────────────

const EVENT_INSERT: &str =
    "INSERT INTO events \
     (id, event_type, entity_id, actor, payload_json, version, \
      correlation_id, causation_id, workspace_id, session_id, \
      actor_id, parent_actor_id, elevated, created_at) \
     VALUES (?, ?, ?, 'ship', ?, 1, NULL, NULL, ?, NULL, ?, NULL, 1, ?)";

// ── global event bus ──────────────────────────────────────────────────────────

static EVENT_BUS: OnceLock<EventBus> = OnceLock::new();

fn global_bus() -> &'static EventBus {
    EVENT_BUS.get_or_init(|| {
        let mut bus = EventBus::new();
        bus.register(Box::new(WorkspaceProjection::new()));
        bus.register(Box::new(SessionProjection::new()));
        bus
    })
}

// ── core transactional write ──────────────────────────────────────────────────

/// Insert event in a BEGIN/COMMIT block, then dispatch to projections.
///
/// The projection handler maintains the workspace table — there is no direct
/// workspace UPSERT in this path.
fn run_tx<P: serde::Serialize>(
    branch: &str,
    event_type: &'static str,
    payload: &P,
) -> Result<()> {
    let envelope = EventEnvelope::new(event_type, branch, payload)?
        .with_context(Some(branch), None)
        .with_actor_id(branch)
        .elevate();

    let event_ts = envelope.created_at.to_rfc3339();

    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;

        let ev_result = sqlx::query(EVENT_INSERT)
            .bind(&envelope.id)
            .bind(&envelope.event_type)
            .bind(&envelope.entity_id)
            .bind(&envelope.payload_json)
            .bind(&envelope.workspace_id)
            .bind(&envelope.actor_id)
            .bind(&event_ts)
            .execute(&mut conn)
            .await;

        if let Err(e) = ev_result {
            let _ = sqlx::query("ROLLBACK").execute(&mut conn).await;
            return Err(e);
        }

        sqlx::query("COMMIT").execute(&mut conn).await?;
        Ok(())
    })?;

    // Dispatch to projections after successful commit.
    global_bus().dispatch(&envelope, &mut conn);

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
