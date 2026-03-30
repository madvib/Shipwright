//! Event-sourced actor state writes.
//!
//! Each public function builds an EventEnvelope, persists it to platform.db
//! via SqliteEventStore, and routes it through KernelRouter (when initialized)
//! for delivery to actor mailboxes.
//!
//! ADR GHihs2tn: all actor lifecycle events are elevated=1.

use anyhow::Result;

use crate::db::block_on_anyhow;
use crate::events::store::EventStore;
use crate::events::types::event_types;
use crate::events::types::{ActorCrashed, ActorCreated, ActorSlept, ActorStopped, ActorWoke};
use crate::events::validator::{CallerKind, EmitContext};
use crate::events::{EventEnvelope, SqliteEventStore};

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

    // Persist to platform.db for the reading path (CLI, sync, projections).
    SqliteEventStore::new()?.append(&envelope)?;

    // Route to actor mailboxes if the kernel router is running.
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: workspace_id.map(|s| s.to_string()),
        session_id: None,
    };
    if let Some(kr) = crate::events::kernel_router() {
        let _ = block_on_anyhow(async { kr.lock().await.route(envelope, &ctx).await });
    }

    Ok(())
}

// ── public API ────────────────────────────────────────────────────────────────

pub fn emit_actor_created(
    actor_id: &str,
    payload: &ActorCreated,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    run_tx(actor_id, workspace_id, parent_actor_id, event_types::ACTOR_CREATED, payload)
}

pub fn emit_actor_woke(
    id: &str,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    run_tx(id, workspace_id, parent_actor_id, event_types::ACTOR_WOKE, &ActorWoke {})
}

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
        &ActorStopped { reason: reason.to_string() },
    )
}

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
        &ActorCrashed { error: error.to_string(), restart_count },
    )
}
