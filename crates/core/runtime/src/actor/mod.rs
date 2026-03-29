//! Actor lifecycle public API.
//!
//! Thin module over `db::actor_events`. Each function emits a typed event;
//! the ActorProjection maintains the actors table from those events.

use anyhow::Result;

use crate::db::actor_events::{
    emit_actor_created, emit_actor_crashed, emit_actor_slept, emit_actor_stopped, emit_actor_woke,
};
use crate::events::filter::EventFilter;
use crate::events::store::{EventStore, SqliteEventStore};
use crate::events::types::ActorCreated;

pub mod supervisor;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_supervision;

// ── public API ────────────────────────────────────────────────────────────────

/// Emit `actor.created` — the projection inserts the actors row.
pub fn create_actor(
    id: &str,
    kind: &str,
    environment_type: &str,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    let payload = ActorCreated {
        kind: kind.to_string(),
        environment_type: environment_type.to_string(),
        workspace_id: workspace_id.map(str::to_string),
        parent_actor_id: parent_actor_id.map(str::to_string),
        restart_count: 0,
    };
    emit_actor_created(id, &payload, workspace_id, parent_actor_id)
}

/// Transition actor to `active` via `actor.woke` event.
pub fn wake_actor(
    id: &str,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_woke(id, workspace_id, parent_actor_id)
}

/// Transition actor to `sleeping` via `actor.slept` event.
pub fn sleep_actor(
    id: &str,
    idle_secs: u64,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_slept(id, idle_secs, workspace_id, parent_actor_id)
}

/// Transition actor to `stopped` via `actor.stopped` event.
pub fn stop_actor(
    id: &str,
    reason: &str,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_stopped(id, reason, workspace_id, parent_actor_id)
}

/// Transition actor to `crashed` + persist restart_count via `actor.crashed` event.
pub fn crash_actor(
    id: &str,
    error: &str,
    restart_count: u32,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_crashed(id, error, restart_count, workspace_id, parent_actor_id)
}

/// Fetch all elevated events for children of `supervisor_actor_id`, evaluate
/// the supervision policy, and execute resulting actions (restart or stop).
pub fn run_supervision(
    supervisor_actor_id: &str,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
    policy: &supervisor::SupervisorPolicy,
) -> Result<Vec<supervisor::SupervisionAction>> {
    let store = SqliteEventStore::new()?;
    let events = store.query(&EventFilter {
        parent_actor_id: Some(supervisor_actor_id.to_string()),
        elevated_only: true,
        ..Default::default()
    })?;

    let actions = supervisor::evaluate(&events, policy);

    for action in &actions {
        match action {
            supervisor::SupervisionAction::Restart { actor_id } => {
                wake_actor(actor_id, workspace_id, parent_actor_id)?;
            }
            supervisor::SupervisionAction::Stop { actor_id, reason } => {
                stop_actor(actor_id, reason, workspace_id, parent_actor_id)?;
            }
        }
    }

    Ok(actions)
}
