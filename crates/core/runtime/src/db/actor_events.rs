//! Event-sourced actor state writes.
//!
//! Each public function builds an EventEnvelope, emits through the global
//! EventRouter (validate → persist to platform.db → broadcast), writes to the
//! per-workspace DB for local queries, then dispatches synchronously to
//! ActorProjection.
//!
//! ADR GHihs2tn: all actor lifecycle events are elevated=1.

use anyhow::Result;

use crate::db::workspace_db::open_workspace_db_for_id;
use crate::db::{block_on, block_on_anyhow};
use crate::events::global_router::router;
use crate::events::types::event_types;
use crate::events::types::{ActorCrashed, ActorCreated, ActorSlept, ActorStopped, ActorWoke};
use crate::events::validator::{CallerKind, EmitContext};
use crate::events::EventEnvelope;
use crate::projections::{ActorProjection, Projection};

// ── SQL constants ─────────────────────────────────────────────────────────────

const EVENT_INSERT: &str =
    "INSERT INTO events \
     (id, event_type, entity_id, actor, payload_json, version, \
      correlation_id, causation_id, workspace_id, session_id, \
      actor_id, parent_actor_id, elevated, created_at) \
     VALUES (?, ?, ?, 'ship', ?, 1, NULL, NULL, ?, NULL, ?, ?, 1, ?)";

// ── core emit ───────────────────────────────────────────────────────────────

/// Build envelope, emit via router (validate → persist → broadcast),
/// write to workspace DB, then dispatch to actor projection.
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

    // Router: validate → persist to platform.db → broadcast
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: workspace_id.map(|s| s.to_string()),
        session_id: None,
    };
    block_on_anyhow(router().emit(envelope.clone(), &ctx))?;

    // Write to workspace DB for local queries. When no workspace, the router
    // already persisted to platform.db so no additional write is needed.
    let mut conn = if let Some(ws_id) = workspace_id {
        let event_ts = envelope.created_at.to_rfc3339();
        let mut ws_conn = open_workspace_db_for_id(ws_id)?;
        block_on(async {
            sqlx::query("BEGIN IMMEDIATE").execute(&mut ws_conn).await?;
            let ev_result = sqlx::query(EVENT_INSERT)
                .bind(&envelope.id)
                .bind(&envelope.event_type)
                .bind(&envelope.entity_id)
                .bind(&envelope.payload_json)
                .bind(&envelope.workspace_id)
                .bind(&envelope.actor_id)
                .bind(&envelope.parent_actor_id)
                .bind(&event_ts)
                .execute(&mut ws_conn)
                .await;
            if let Err(e) = ev_result {
                let _ = sqlx::query("ROLLBACK").execute(&mut ws_conn).await;
                return Err(e);
            }
            sqlx::query("COMMIT").execute(&mut ws_conn).await?;
            Ok(())
        })?;
        ws_conn
    } else {
        crate::db::open_db()?
    };

    // Synchronous projection: update actors table immediately.
    let proj = ActorProjection::new();
    if proj.event_types().contains(&event_type) {
        if let Err(e) = proj.apply(&envelope, &mut conn) {
            eprintln!("[actor-events] projection error for {event_type}: {e}");
        }
    }

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
