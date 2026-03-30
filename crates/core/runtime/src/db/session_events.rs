//! Transactional session state + typed event emission.
//!
//! Session lifecycle events (started, ended) are emitted through the global
//! EventRouter, which validates, persists to platform.db, and broadcasts.
//! The same events are also written to the per-workspace DB for local queries.
//!
//! Progress events are non-elevated and go only to the workspace DB + workspace
//! broadcast channel.
//!
//! ADR GHihs2tn: all session lifecycle transitions must emit typed events.
//! Session is a child actor of its workspace — `parent_actor_id` = workspace ID.

use anyhow::{Context, Result};
use chrono::Utc;
use ulid::Ulid;

use crate::db::workspace_db::open_workspace_db_for_id;
use crate::db::{block_on, block_on_anyhow};
use crate::events::global_router::router;
use crate::events::types::event_types;
use crate::events::types::{SessionEnded, SessionProgress, SessionStarted};
use crate::events::validator::{CallerKind, EmitContext};
use crate::events::EventEnvelope;
use crate::projections::{Projection, SessionProjection};

// ── SQL constants ─────────────────────────────────────────────────────────────

const EVENT_INSERT: &str =
    "INSERT INTO events \
     (id, event_type, entity_id, actor, payload_json, version, \
      correlation_id, causation_id, workspace_id, session_id, \
      actor_id, parent_actor_id, elevated, created_at) \
     VALUES (?, ?, ?, 'ship', ?, 1, NULL, NULL, ?, ?, ?, ?, ?, ?)";

// ── helpers ──────────────────────────────────────────────────────────────────

/// Build a session event envelope.
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

/// Emit via the global EventRouter (validate → persist to platform.db → broadcast),
/// then dispatch synchronously to SessionProjection.
fn emit_via_router(envelope: &EventEnvelope, workspace_id: &str) -> Result<()> {
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: Some(workspace_id.to_string()),
        session_id: envelope.session_id.clone(),
    };
    block_on_anyhow(router().emit(envelope.clone(), &ctx))?;

    // Synchronous projection: update session table immediately.
    let proj = SessionProjection::new();
    if proj.event_types().contains(&envelope.event_type.as_str()) {
        let mut conn = crate::db::open_db()?;
        if let Err(e) = proj.apply(envelope, &mut conn) {
            eprintln!(
                "[session-events] projection error for {}: {e}",
                envelope.event_type
            );
        }
    }
    Ok(())
}

/// Write event to the per-workspace DB for local event queries.
fn write_workspace_event(envelope: &EventEnvelope, workspace_id: &str) -> Result<()> {
    let event_ts = envelope.created_at.to_rfc3339();

    let mut ws_conn = open_workspace_db_for_id(workspace_id)?;
    block_on(async {
        sqlx::query("BEGIN IMMEDIATE").execute(&mut ws_conn).await?;

        let ev_result = sqlx::query(EVENT_INSERT)
            .bind(&envelope.id)
            .bind(&envelope.event_type)
            .bind(&envelope.entity_id)
            .bind(&envelope.payload_json)
            .bind(&envelope.workspace_id)
            .bind(&envelope.session_id)
            .bind(&envelope.actor_id)
            .bind(&envelope.parent_actor_id)
            .bind(envelope.elevated as i64)
            .bind(&event_ts)
            .execute(&mut ws_conn)
            .await;

        if let Err(e) = ev_result {
            let _ = sqlx::query("ROLLBACK").execute(&mut ws_conn).await;
            return Err(e);
        }

        sqlx::query("COMMIT").execute(&mut ws_conn).await?;
        Ok(())
    })
}

// ── public API ────────────────────────────────────────────────────────────────

/// Emit `session.started` to platform.db (via router) and workspace DB.
pub fn insert_session_with_started_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionStarted,
) -> Result<()> {
    let envelope = session_envelope(
        event_types::SESSION_STARTED,
        session_id,
        workspace_id,
        payload,
    )?;

    emit_via_router(&envelope, workspace_id)?;
    write_workspace_event(&envelope, workspace_id)?;
    Ok(())
}

/// Emit `session.progress` event to the workspace DB only.
///
/// Progress events are not elevated — too noisy to bubble to platform scope.
/// They are still broadcast on the workspace channel via the router.
pub fn insert_session_progress_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionProgress,
) -> Result<()> {
    let payload_json = serde_json::to_string(payload)
        .context("failed to serialise SessionProgress payload")?;
    let event_id = Ulid::new().to_string();

    let envelope = EventEnvelope {
        id: event_id,
        event_type: event_types::SESSION_PROGRESS.to_string(),
        entity_id: session_id.to_string(),
        actor: "ship".to_string(),
        payload_json,
        version: 1,
        correlation_id: None,
        causation_id: None,
        workspace_id: Some(workspace_id.to_string()),
        session_id: Some(session_id.to_string()),
        actor_id: Some(session_id.to_string()),
        parent_actor_id: Some(workspace_id.to_string()),
        elevated: false,
        created_at: Utc::now(),
    };

    // Write to workspace DB only — progress is too noisy for platform.db.
    write_workspace_event(&envelope, workspace_id)
}

/// Emit `session.ended` to platform.db (via router) and workspace DB.
pub fn update_session_with_ended_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionEnded,
) -> Result<()> {
    let envelope = session_envelope(
        event_types::SESSION_ENDED,
        session_id,
        workspace_id,
        payload,
    )?;

    emit_via_router(&envelope, workspace_id)?;
    write_workspace_event(&envelope, workspace_id)?;
    Ok(())
}
