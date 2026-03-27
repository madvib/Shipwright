//! Actor lifecycle public API.
//!
//! Thin module over `db::actor_events`.  Each function writes the actors
//! operational row and emits a typed event in one atomic transaction.
//!
//! ADR GHihs2tn: write path is BEGIN IMMEDIATE → actors → events → COMMIT.

use anyhow::Result;

use crate::db::actor_events::{
    ActorUpsert, insert_actor_created, update_actor_crashed, update_actor_slept,
    update_actor_stopped, update_actor_woke,
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

/// Create actor row and emit `actor.created` atomically.
pub fn create_actor(upsert: ActorUpsert<'_>) -> Result<()> {
    let payload = ActorCreated {
        kind: upsert.kind.to_string(),
        environment_type: upsert.environment_type.to_string(),
    };
    insert_actor_created(&upsert, &payload)
}

/// Transition actor to `active` and emit `actor.woke` atomically.
pub fn wake_actor(
    id: &str,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    update_actor_woke(id, workspace_id, parent_actor_id)
}

/// Transition actor to `sleeping` and emit `actor.slept` atomically.
pub fn sleep_actor(
    id: &str,
    idle_secs: u64,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    update_actor_slept(id, idle_secs, workspace_id, parent_actor_id)
}

/// Transition actor to `stopped` and emit `actor.stopped` atomically.
pub fn stop_actor(
    id: &str,
    reason: &str,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    update_actor_stopped(id, reason, workspace_id, parent_actor_id)
}

/// Transition actor to `crashed`, persist restart_count, and emit `actor.crashed` atomically.
pub fn crash_actor(
    id: &str,
    error: &str,
    restart_count: u32,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    update_actor_crashed(id, error, restart_count, workspace_id, parent_actor_id)
}

/// Fetch all elevated events for children of `supervisor_actor_id`, evaluate
/// the supervision policy, and execute resulting actions (restart or stop).
///
/// Returns the list of actions taken so callers can observe what happened.
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
