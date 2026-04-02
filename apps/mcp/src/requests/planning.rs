use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct ShipEventRequest {
    /// Domain event type, namespaced with a dot (e.g. "deployment.completed").
    /// Reserved prefixes (actor.*, session.*, skill.*, workspace.*, gate.*, job.*, config.*, project.*) are rejected.
    pub event_type: String,
    /// Arbitrary JSON payload for the event.
    pub payload: serde_json::Value,
    /// Mark this event as elevated (supervisor-level). Default: false.
    pub elevated: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct EmitStudioEventRequest {
    /// Event type, must start with "studio." (e.g. "studio.message.visual").
    pub event_type: String,
    /// Arbitrary JSON payload. Must be self-contained — agents receive this directly.
    pub payload: serde_json::Value,
    /// Route the inbox write to this workspace instead of the caller's workspace.
    /// Use when an agent (e.g. gate) needs to notify a different session (e.g. commander).
    pub target_workspace_id: Option<String>,
}
