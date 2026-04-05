//! Typed session event emission.
//!
//! Each function builds an EventEnvelope, persists the event and applies the
//! session projection in a single SQLite transaction on one connection.
//! Kernel routing happens after commit (fire-and-forget).
//!
//! session.progress is non-elevated so it goes to kernel routing only;
//! it is still persisted to platform.db by the transactional path.
//!
//! ADR GHihs2tn: all session lifecycle transitions must emit typed events.

use anyhow::Result;

use crate::db::{block_on, block_on_anyhow, open_db};
use crate::events::store::append_event_with_conn;
use crate::events::types::event_types;
use crate::events::types::{SessionEnded, SessionProgress, SessionRecorded, SessionStarted};
use crate::events::validator::{CallerKind, EmitContext};
use crate::events::EventEnvelope;
use crate::projections::{Projection, SessionProjection};

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

/// Persist event + apply projection in one transaction, then route to kernel.
fn emit_session(envelope: EventEnvelope, workspace_id: &str) -> Result<EventEnvelope> {
    let mut conn = open_db()?;
    block_on(async { sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await })?;

    if let Err(e) = append_event_with_conn(&envelope, &mut conn) {
        let _ = block_on(async { sqlx::query("ROLLBACK").execute(&mut conn).await });
        return Err(e);
    }

    let proj = SessionProjection::new();
    if let Err(e) = proj.apply(&envelope, &mut conn) {
        let _ = block_on(async { sqlx::query("ROLLBACK").execute(&mut conn).await });
        return Err(e);
    }

    block_on(async { sqlx::query("COMMIT").execute(&mut conn).await })?;

    // Route to actor mailboxes after commit (fire-and-forget).
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

/// Emit `session.progress` — non-elevated, no projection handler.
pub fn insert_session_progress_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionProgress,
) -> Result<()> {
    let envelope = EventEnvelope::new(event_types::SESSION_PROGRESS, session_id, payload)?
        .with_context(Some(workspace_id), Some(session_id))
        .with_actor_id(session_id)
        .with_parent_actor_id(workspace_id);

    let mut conn = open_db()?;
    append_event_with_conn(&envelope, &mut conn)?;

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

/// Emit a session drain/tool-count event transactionally.
pub fn emit_session_drain_event<P: serde::Serialize>(
    event_type: &str,
    session_id: &str,
    workspace_id: &str,
    payload: &P,
) -> Result<EventEnvelope> {
    let envelope = session_envelope(event_type, session_id, workspace_id, payload)?;
    emit_session(envelope, workspace_id)
}
