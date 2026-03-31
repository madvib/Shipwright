//! Actor permissions — emit and subscribe enforcement.

use anyhow::{Result, anyhow};

/// Permissions granted to an actor at spawn time.
#[derive(Debug, Clone)]
pub struct ActorPermissions {
    /// Namespace prefixes the actor can emit to.
    pub emit: Vec<String>,
    /// Allowed subscriptions with delivery scope.
    pub subscribe: Vec<PermittedSubscription>,
}

/// A namespace subscription with a delivery scope constraint.
#[derive(Debug, Clone)]
pub struct PermittedSubscription {
    /// Namespace prefix (e.g. `"studio."`).
    pub namespace: String,
    /// How events in this namespace are filtered for this actor.
    pub scope: DeliveryScope,
}

/// Controls which events within a subscribed namespace reach the actor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryScope {
    /// All events in namespace.
    Global,
    /// Only events matching the actor's workspace_id.
    Workspace,
    /// Only events with matching target_actor_id.
    Directed,
    /// Only elevated events.
    Elevated,
}

impl ActorPermissions {
    /// Validate that all declared namespaces are non-empty.
    pub fn validate(&self) -> Result<()> {
        for ns in &self.emit {
            if ns.is_empty() {
                return Err(anyhow!("empty namespace in emit list"));
            }
        }
        for sub in &self.subscribe {
            if sub.namespace.is_empty() {
                return Err(anyhow!("empty namespace in subscribe list"));
            }
        }
        Ok(())
    }

    /// Check whether the actor is permitted to emit an event with the given type.
    pub fn can_emit(&self, event_type: &str) -> bool {
        self.emit.iter().any(|ns| event_type.starts_with(ns.as_str()))
    }
}
