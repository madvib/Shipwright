use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct MeshSendRequest {
    /// Agent ID to send to (e.g. "agent.lead").
    pub to: String,
    /// Arbitrary JSON body to deliver.
    pub body: serde_json::Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct MeshBroadcastRequest {
    /// Arbitrary JSON body to broadcast.
    pub body: serde_json::Value,
    /// If set, only agents with this capability receive the broadcast.
    pub capability_filter: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct MeshDiscoverRequest {
    /// Filter by capability. Omit to return all agents.
    pub capability: Option<String>,
    /// Filter by status: "active", "busy", or "idle". Omit to return all.
    pub status: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct MeshStatusRequest {
    /// New status for this agent: "active", "busy", or "idle".
    pub status: String,
}
