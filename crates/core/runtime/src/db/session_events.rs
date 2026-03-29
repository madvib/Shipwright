//! Transactional session state + typed event emission.
//!
//! Session lifecycle events (started, ended) are written to platform.db where
//! the global EventBus dispatches them to SessionProjection, which maintains the
//! workspace_session row. The same events are also written to the per-workspace
//! DB for workspace-scoped event queries.
//!
//! Progress events go only to the workspace DB — they are not elevated.
//!
//! ADR GHihs2tn: all session lifecycle transitions must emit typed events.
//! Session is a child actor of its workspace — `parent_actor_id` = workspace ID.

use std::sync::OnceLock;

use anyhow::{Context, Result};
use chrono::Utc;
use ulid::Ulid;

use crate::db::workspace_db::open_workspace_db_for_id;
use crate::db::{block_on, open_db};
use crate::events::types::event_types;
use crate::events::types::{SessionEnded, SessionProgress, SessionStarted};
use crate::events::EventEnvelope;
use crate::projections::{EventBus, SessionProjection};

// ── SQL constants ─────────────────────────────────────────────────────────────

// entity_id, workspace_id, session_id, actor_id, and parent_actor_id are all
// set explicitly per-event. `elevated` is passed as a bind parameter.
const EVENT_INSERT: &str =
    "INSERT INTO events \
     (id, event_type, entity_id, actor, payload_json, version, \
      correlation_id, causation_id, workspace_id, session_id, \
      actor_id, parent_actor_id, elevated, created_at) \
     VALUES (?, ?, ?, 'ship', ?, 1, NULL, NULL, ?, ?, ?, ?, ?, ?)";

// ── global event bus (session) ───────────────────────────────────────────────

static SESSION_BUS: OnceLock<EventBus> = OnceLock::new();

fn session_bus() -> &'static EventBus {
    SESSION_BUS.get_or_init(|| {
        let mut bus = EventBus::new();
        bus.register(Box::new(SessionProjection::new()));
        bus
    })
}

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

/// Insert event into platform.db and dispatch to session projection.
fn write_platform_event(envelope: &EventEnvelope) -> Result<()> {
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
            .bind(&envelope.session_id)
            .bind(&envelope.actor_id)
            .bind(&envelope.parent_actor_id)
            .bind(1_i64) // elevated
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

    // Dispatch to session projection after successful commit.
    session_bus().dispatch(envelope, &mut conn);
    Ok(())
}

/// Insert event into the per-workspace DB for local event queries.
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
            .bind(1_i64) // elevated
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

/// Emit `session.started` to platform.db (projection inserts row) and workspace DB.
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

    write_platform_event(&envelope)?;
    write_workspace_event(&envelope, workspace_id)?;
    Ok(())
}

/// Emit `session.progress` event to the workspace DB only.
///
/// Progress events are not elevated — too noisy to bubble to workspace level.
pub fn insert_session_progress_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionProgress,
) -> Result<()> {
    let payload_json = serde_json::to_string(payload)
        .context("failed to serialise SessionProgress payload")?;
    let event_id = Ulid::new().to_string();
    let event_ts = Utc::now().to_rfc3339();
    let session_id = session_id.to_string();
    let workspace_id = workspace_id.to_string();

    let mut conn = open_workspace_db_for_id(&workspace_id)?;
    block_on(async {
        sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;

        let ev_result = sqlx::query(EVENT_INSERT)
            .bind(&event_id)
            .bind(event_types::SESSION_PROGRESS)
            .bind(&session_id)   // entity_id = session ID
            .bind(&payload_json)
            .bind(&workspace_id) // workspace_id
            .bind(&session_id)   // session_id
            .bind(&session_id)   // actor_id = session ID
            .bind(&workspace_id) // parent_actor_id = workspace ID
            .bind(0_i64)         // elevated = 0
            .bind(&event_ts)
            .execute(&mut conn)
            .await;

        if let Err(e) = ev_result {
            let _ = sqlx::query("ROLLBACK").execute(&mut conn).await;
            return Err(e);
        }

        sqlx::query("COMMIT").execute(&mut conn).await?;
        Ok(())
    })
}

/// Emit `session.ended` to platform.db (projection updates row) and workspace DB.
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

    write_platform_event(&envelope)?;
    write_workspace_event(&envelope, workspace_id)?;
    Ok(())
}
