//! Actor identity — unique instance IDs and stable labels.

#[derive(Debug, Clone)]
pub struct ActorIdentity {
    /// ULID, unique per spawn.
    pub instance_id: String,
    /// Human-readable label, stable across restarts.
    pub label: String,
    /// The kind of actor.
    pub actor_type: ActorType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorType {
    Agent,
    Service,
    App,
    Cli,
}
