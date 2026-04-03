//! Typed session event emission.
//!
//! Each function builds an EventEnvelope, persists it to platform.db via
//! SqliteEventStore, and routes it through KernelRouter (when initialized)
//! for delivery to actor mailboxes.
//!
//! session.progress is non-elevated so it goes to kernel routing only;
//! it is still persisted to platform.db by SqliteEventStore.
//!
//! ADR GHihs2tn: all session lifecycle transitions must emit typed events.

use anyhow::Result;

use crate::db::block_on_anyhow;
use crate::events::store::EventStore;
use crate::events::types::event_types;
use crate::events::types::{SessionEnded, SessionProgress, SessionRecorded, SessionStarted};
use crate::events::validator::{CallerKind, EmitContext};
use crate::events::{EventEnvelope, SqliteEventStore};

// ── helpers ──────────────────────────────────────────────────────────────────

fn session_envelope<P: serde::Serialize>(
    event_type: &str,
    session_id: &str,
    workspace_id: &str,
    payload: &P,
) -> Result<EventEnvelope> {
    Ok(EventEnvelope::new(event_type, session_id, payload)?
        .with_context(Some(workspace_id), Some(session_id))
        .with_actor_id(session_id)
        .with_parent_actor_id(workspace_id)
        .elevate())
}

fn emit_session(envelope: EventEnvelope, workspace_id: &str) -> Result<EventEnvelope> {
    // Persist to platform.db for the reading path (CLI, sync, projections).
    SqliteEventStore::new()?.append(&envelope)?;

    // Route to actor mailboxes if the kernel router is running.
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: Some(workspace_id.to_string()),
        session_id: envelope.session_id.clone(),
    };
    if let Some(kr) = crate::events::kernel_router() {
        let _ = block_on_anyhow(async { kr.lock().await.route(envelope.clone(), &ctx).await });
    }

    Ok(envelope)
}

// ── public API ────────────────────────────────────────────────────────────────

/// Emit `session.started` to platform.db via the store.
/// Returns the envelope so callers can apply SessionProjection for
/// immediate read-back consistency.
pub fn insert_session_with_started_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionStarted,
) -> Result<EventEnvelope> {
    let envelope = session_envelope(
        event_types::SESSION_STARTED,
        session_id,
        workspace_id,
        payload,
    )?;
    emit_session(envelope, workspace_id)
}

/// Emit `session.progress` to platform.db via the store (non-elevated).
///
/// Progress events are not elevated — they are still persisted to platform.db
/// but do not appear on the platform broadcast.
pub fn insert_session_progress_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionProgress,
) -> Result<()> {
    let envelope = EventEnvelope::new(event_types::SESSION_PROGRESS, session_id, payload)?
        .with_context(Some(workspace_id), Some(session_id))
        .with_actor_id(session_id)
        .with_parent_actor_id(workspace_id);

    SqliteEventStore::new()?.append(&envelope)?;

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: Some(workspace_id.to_string()),
        session_id: Some(session_id.to_string()),
    };
    if let Some(kr) = crate::events::kernel_router() {
        let _ = block_on_anyhow(async { kr.lock().await.route(envelope, &ctx).await });
    }

    Ok(())
}

/// Emit `session.ended` to platform.db via the store.
/// Returns the envelope so callers can apply SessionProjection for
/// immediate read-back consistency.
pub fn update_session_with_ended_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionEnded,
) -> Result<EventEnvelope> {
    let envelope = session_envelope(
        event_types::SESSION_ENDED,
        session_id,
        workspace_id,
        payload,
    )?;
    emit_session(envelope, workspace_id)
}

/// Emit `session.recorded` to platform.db via the store.
/// Returns the envelope so callers can apply SessionProjection to write
/// the workspace_session_record row.
pub fn insert_session_recorded_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionRecorded,
) -> Result<EventEnvelope> {
    let envelope = session_envelope(
        event_types::SESSION_RECORDED,
        session_id,
        workspace_id,
        payload,
    )?;
    emit_session(envelope, workspace_id)
}
