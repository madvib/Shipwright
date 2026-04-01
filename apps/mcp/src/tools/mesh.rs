//! Mesh MCP tool helpers — build EventEnvelopes for mesh operations.
//!
//! Async routing is handled by the server tool handlers. These functions
//! are sync: they validate inputs and construct the EventEnvelope only.

use anyhow::{Result, anyhow};
use runtime::events::EventEnvelope;

/// Build a `mesh.send` envelope — directed message to one agent.
pub fn build_mesh_send(
    actor_id: &str,
    to: &str,
    body: serde_json::Value,
) -> Result<EventEnvelope> {
    if to.is_empty() {
        return Err(anyhow!("mesh_send: 'to' must not be empty"));
    }
    EventEnvelope::new(
        "mesh.send",
        actor_id,
        &serde_json::json!({ "from": actor_id, "to": to, "body": body }),
    )
    .map(|e| e.with_actor_id(actor_id))
}

/// Build a `mesh.broadcast` envelope — message to all (or capability-filtered) agents.
pub fn build_mesh_broadcast(
    actor_id: &str,
    body: serde_json::Value,
    capability_filter: Option<String>,
) -> Result<EventEnvelope> {
    let payload = match capability_filter {
        Some(ref cap) => serde_json::json!({
            "from": actor_id,
            "body": body,
            "capability_filter": cap,
        }),
        None => serde_json::json!({
            "from": actor_id,
            "body": body,
        }),
    };
    EventEnvelope::new("mesh.broadcast", actor_id, &payload)
        .map(|e| e.with_actor_id(actor_id))
}

/// Build a `mesh.discover.request` envelope — query registered agents.
pub fn build_mesh_discover(
    actor_id: &str,
    capability: Option<String>,
    status: Option<String>,
) -> Result<EventEnvelope> {
    let mut payload = serde_json::json!({ "from": actor_id });
    if let Some(cap) = capability {
        payload["capability"] = serde_json::Value::String(cap);
    }
    if let Some(st) = status {
        payload["status"] = serde_json::Value::String(st);
    }
    EventEnvelope::new("mesh.discover.request", actor_id, &payload)
        .map(|e| e.with_actor_id(actor_id))
}

/// Build a `mesh.status` envelope — update this agent's status.
pub fn build_mesh_status(actor_id: &str, status: &str) -> Result<EventEnvelope> {
    match status {
        "active" | "busy" | "idle" => {}
        other => return Err(anyhow!("mesh_status: unknown status '{other}' (active|busy|idle)")),
    }
    EventEnvelope::new(
        "mesh.status",
        actor_id,
        &serde_json::json!({ "agent_id": actor_id, "status": status }),
    )
    .map(|e| e.with_actor_id(actor_id))
}
