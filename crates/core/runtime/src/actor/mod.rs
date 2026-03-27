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
use crate::events::types::ActorCreated;

#[cfg(test)]
mod tests;

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
