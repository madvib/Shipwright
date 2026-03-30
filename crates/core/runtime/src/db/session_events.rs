//! Typed session event emission.
//!
//! Each function builds an EventEnvelope and emits through the global
//! EventRouter (validate → persist to platform.db → broadcast). That is
//! the only write — no workspace DB writes, no inline projection.
//!
//! Callers that need immediate read-back (e.g. session_lifecycle.rs) are
//! responsible for applying SessionProjection synchronously after the emit.
//!
//! session.progress is non-elevated so it goes to the workspace broadcast
//! channel only, but IS persisted to platform.db by the router.
//!
//! ADR GHihs2tn: all session lifecycle transitions must emit typed events.

use anyhow::{Context, Result};

use crate::db::block_on_anyhow;
use crate::events::global_router::router;
use crate::events::types::event_types;
use crate::events::types::{SessionEnded, SessionProgress, SessionStarted};
use crate::events::validator::{CallerKind, EmitContext};
use crate::events::EventEnvelope;

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
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: Some(workspace_id.to_string()),
        session_id: envelope.session_id.clone(),
    };
    block_on_anyhow(router().emit(envelope.clone(), &ctx))?;
    Ok(envelope)
}

// ── public API ────────────────────────────────────────────────────────────────

/// Emit `session.started` to platform.db via the router.
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

/// Emit `session.progress` to platform.db via the router (non-elevated).
///
/// Progress events are not elevated — they go to the workspace broadcast
/// channel only, not the platform channel. They ARE persisted to platform.db.
pub fn insert_session_progress_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionProgress,
) -> Result<()> {
    let payload_json = serde_json::to_string(payload)
        .context("failed to serialise SessionProgress payload")?;

    let envelope = EventEnvelope::new(event_types::SESSION_PROGRESS, session_id, &payload)?
        .with_context(Some(workspace_id), Some(session_id))
        .with_actor_id(session_id)
        .with_parent_actor_id(workspace_id);
    // Note: NOT elevated — session.progress stays on workspace channel
    let _ = payload_json; // payload serialised inside EventEnvelope::new above

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: Some(workspace_id.to_string()),
        session_id: Some(session_id.to_string()),
    };
    block_on_anyhow(router().emit(envelope, &ctx))
}

/// Emit `session.ended` to platform.db via the router.
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
