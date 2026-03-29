//! Event-sourced actor state writes.
//!
//! Each public function inserts a typed event into the `events` table inside a
//! BEGIN/COMMIT block on the per-workspace DB, then dispatches the event through
//! the workspace EventBus so projections (e.g. ActorProjection) maintain derived
//! state.
//!
//! ADR GHihs2tn: write path is BEGIN IMMEDIATE → INSERT INTO events → COMMIT →
//! dispatch. All actor lifecycle events are elevated=1.

use std::sync::OnceLock;

use anyhow::Result;

use crate::db::workspace_db::open_workspace_db_for_id;
use crate::db::{block_on, open_db};
use crate::events::types::event_types;
use crate::events::types::{ActorCrashed, ActorCreated, ActorSlept, ActorStopped, ActorWoke};
use crate::events::EventEnvelope;
use crate::projections::{ActorProjection, EventBus};

// ── SQL constants ─────────────────────────────────────────────────────────────

const EVENT_INSERT: &str =
    "INSERT INTO events \
     (id, event_type, entity_id, actor, payload_json, version, \
      correlation_id, causation_id, workspace_id, session_id, \
      actor_id, parent_actor_id, elevated, created_at) \
     VALUES (?, ?, ?, 'ship', ?, 1, NULL, NULL, ?, NULL, ?, ?, 1, ?)";

// ── workspace event bus ──────────────────────────────────────────────────────

static WORKSPACE_BUS: OnceLock<EventBus> = OnceLock::new();

fn workspace_bus() -> &'static EventBus {
    WORKSPACE_BUS.get_or_init(|| {
        let mut bus = EventBus::new();
        bus.register(Box::new(ActorProjection::new()));
        bus
    })
}

// ── core transactional write ─────────────────────────────────────────────────

/// Insert event in a BEGIN/COMMIT block on the workspace DB, then dispatch.
///
/// The projection handler maintains the actors table — there is no direct
/// actors UPSERT in this path.
fn run_tx<P: serde::Serialize>(
    actor_id: &str,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
    event_type: &'static str,
    payload: &P,
) -> Result<()> {
    let mut envelope = EventEnvelope::new(event_type, actor_id, payload)?
        .with_context(workspace_id, None)
        .with_actor_id(actor_id)
        .elevate();
    if let Some(parent) = parent_actor_id {
        envelope = envelope.with_parent_actor_id(parent);
    }

    let event_ts = envelope.created_at.to_rfc3339();

    let mut conn = match workspace_id {
        Some(ws_id) => open_workspace_db_for_id(ws_id)?,
        None => open_db()?,
    };
    block_on(async {
        sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;

        let ev_result = sqlx::query(EVENT_INSERT)
            .bind(&envelope.id)
            .bind(&envelope.event_type)
            .bind(&envelope.entity_id)
            .bind(&envelope.payload_json)
            .bind(&envelope.workspace_id)
            .bind(&envelope.actor_id)
            .bind(&envelope.parent_actor_id)
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
    workspace_bus().dispatch(&envelope, &mut conn);

    Ok(())
}

// ── public API ────────────────────────────────────────────────────────────────

/// Emit `actor.created` and let the projection insert actor state.
pub fn emit_actor_created(
    actor_id: &str,
    payload: &ActorCreated,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    run_tx(
        actor_id,
        workspace_id,
        parent_actor_id,
        event_types::ACTOR_CREATED,
        payload,
    )
}

/// Emit `actor.woke` and let the projection update status to active.
pub fn emit_actor_woke(
    id: &str,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    run_tx(id, workspace_id, parent_actor_id, event_types::ACTOR_WOKE, &ActorWoke {})
}

/// Emit `actor.slept` and let the projection update status to sleeping.
pub fn emit_actor_slept(
    id: &str,
    idle_secs: u64,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    run_tx(
        id,
        workspace_id,
        parent_actor_id,
        event_types::ACTOR_SLEPT,
        &ActorSlept { idle_secs },
    )
}

/// Emit `actor.stopped` and let the projection update status to stopped.
pub fn emit_actor_stopped(
    id: &str,
    reason: &str,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    run_tx(
        id,
        workspace_id,
        parent_actor_id,
        event_types::ACTOR_STOPPED,
        &ActorStopped {
            reason: reason.to_string(),
        },
    )
}

/// Emit `actor.crashed` and let the projection update status + restart_count.
pub fn emit_actor_crashed(
    id: &str,
    error: &str,
    restart_count: u32,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    run_tx(
        id,
        workspace_id,
        parent_actor_id,
        event_types::ACTOR_CRASHED,
        &ActorCrashed {
            error: error.to_string(),
            restart_count,
        },
    )
}
