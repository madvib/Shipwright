//! Supervision logic — pure, stateless, no DB access.
//!
//! The caller fetches elevated child events, passes them here, and executes
//! whatever actions are returned.  No polling loop, no background thread.

use crate::events::envelope::EventEnvelope;
use crate::events::types::{ActorCrashed, event_types};

// ── policy ────────────────────────────────────────────────────────────────────

pub struct SupervisorPolicy {
    pub max_restarts: u32,
}

impl Default for SupervisorPolicy {
    fn default() -> Self {
        Self { max_restarts: 3 }
    }
}

// ── action ────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum SupervisionAction {
    Restart { actor_id: String },
    Stop { actor_id: String, reason: String },
}

// ── evaluate ──────────────────────────────────────────────────────────────────

/// Process a batch of elevated child events and return the actions to take.
///
/// Only `actor.crashed` events are acted on.  All other event types are ignored.
/// The caller is responsible for executing each action via `actor::wake_actor`
/// or `actor::stop_actor`.
pub fn evaluate(
    events: &[EventEnvelope],
    policy: &SupervisorPolicy,
) -> Vec<SupervisionAction> {
    let mut actions = Vec::new();

    for ev in events {
        if ev.event_type != event_types::ACTOR_CRASHED {
            continue;
        }

        let actor_id = match ev.actor_id.as_deref() {
            Some(id) => id.to_string(),
            None => continue,
        };

        let crashed: ActorCrashed = match serde_json::from_str(&ev.payload_json) {
            Ok(p) => p,
            Err(_) => continue,
        };

        if crashed.restart_count < policy.max_restarts {
            actions.push(SupervisionAction::Restart { actor_id });
        } else {
            actions.push(SupervisionAction::Stop {
                actor_id,
                reason: "max_restarts exceeded".to_string(),
            });
        }
    }

    actions
}
