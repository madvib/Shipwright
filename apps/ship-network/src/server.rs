//! NetworkServer — MCP server for cross-agent communication.
//!
//! Exposes 5 tools: mesh_register, mesh_send, mesh_broadcast, mesh_discover,
//! mesh_status. Shares a single KernelRouter across all connections so events
//! flow between agents within the daemon process.

use std::sync::Arc;

use rmcp::{
    Peer, RoleServer,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_router,
};
use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;
use runtime::events::{ActorConfig, CallerKind, EmitContext, EventEnvelope};

use crate::connections::{ConnectionGuard, EventRelay, McpEventSink, PeerHandle};

// ---- Request types ----

#[derive(Deserialize, JsonSchema)]
struct MeshRegisterRequest {
    /// Agent ID to register as (e.g. "agent.commander").
    pub agent_id: String,
    /// Capabilities this agent provides.
    pub capabilities: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
struct MeshSendRequest {
    /// Agent ID to send to.
    pub to: String,
    /// Arbitrary JSON body.
    pub body: serde_json::Value,
}

#[derive(Deserialize, JsonSchema)]
struct MeshBroadcastRequest {
    /// Arbitrary JSON body to broadcast.
    pub body: serde_json::Value,
    /// If set, only agents with this capability receive the broadcast.
    pub capability_filter: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct MeshDiscoverRequest {
    /// Filter by capability. Omit to return all agents.
    pub capability: Option<String>,
    /// Filter by status: "active", "busy", or "idle". Omit to return all.
    pub status: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct MeshStatusRequest {
    /// New status: "active", "busy", or "idle".
    pub status: String,
}

// ---- NetworkServer ----

/// One instance per HTTP session. Shares the kernel across all sessions.
#[derive(Clone)]
pub struct NetworkServer {
    tool_router: ToolRouter<Self>,
    /// Shared across all connections — the single KernelRouter for this daemon.
    kernel: Arc<tokio::sync::Mutex<runtime::events::KernelRouter>>,
    /// Per-connection MCP peer for push notifications.
    notification_peer: Arc<tokio::sync::Mutex<Option<Peer<RoleServer>>>>,
    /// Per-connection cleanup guard. Dropped when the last clone is gone.
    conn: Arc<ConnectionGuard>,
}

impl std::fmt::Debug for NetworkServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkServer").finish_non_exhaustive()
    }
}

#[tool_router]
impl NetworkServer {
    pub fn new(kernel: Arc<tokio::sync::Mutex<runtime::events::KernelRouter>>) -> Self {
        let conn = Arc::new(ConnectionGuard {
            actor_id: std::sync::Mutex::new(None),
            relay_handle: std::sync::Mutex::new(None),
            kernel: kernel.clone(),
        });
        Self {
            tool_router: Self::tool_router(),
            kernel,
            notification_peer: Arc::new(tokio::sync::Mutex::new(None)),
            conn,
        }
    }

    pub async fn store_peer(&self, peer: Peer<RoleServer>) {
        *self.notification_peer.lock().await = Some(peer);
    }

    pub fn tool_router_ref(&self) -> &ToolRouter<Self> {
        &self.tool_router
    }

    async fn actor_id(&self) -> Option<String> {
        self.conn.actor_id.lock().ok()?.clone()
    }

    async fn route(&self, envelope: EventEnvelope) -> String {
        let ctx = EmitContext {
            caller_kind: CallerKind::Mcp,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        };
        match self.kernel.lock().await.route(envelope.clone(), &ctx).await {
            Ok(()) => format!("ok: routed {}", envelope.event_type),
            Err(e) => format!("Error: {e}"),
        }
    }

    // ---- Tools ----

    #[tool(
        description = "Register or re-register this agent on the mesh. Must be called before \
        using other mesh tools. Spawns an actor and starts the push notification relay."
    )]
    async fn mesh_register(&self, Parameters(req): Parameters<MeshRegisterRequest>) -> String {
        let agent_id = req.agent_id.clone();

        // Stop previous actor and relay if re-registering
        {
            let old_id = self.conn.actor_id.lock().ok().and_then(|g| g.clone());
            if let Some(ref id) = old_id {
                let _ = self.kernel.lock().await.stop_actor(id);
            }
            if let Ok(mut h) = self.conn.relay_handle.lock() {
                if let Some(handle) = h.take() {
                    handle.abort();
                }
            }
        }

        let config = ActorConfig {
            namespace: agent_id.clone(),
            write_namespaces: vec!["".to_string()],
            read_namespaces: vec!["agent.".to_string()],
            subscribe_namespaces: vec![
                "mesh.".to_string(),
                "studio.".to_string(),
                "workspace.".to_string(),
                "session.".to_string(),
            ],
        };

        let (_, mailbox) = match self.kernel.lock().await.spawn_actor(&agent_id, config) {
            Ok(r) => r,
            Err(e) => return format!("Error: failed to spawn actor: {e}"),
        };

        // Wire event relay
        let relay = EventRelay::new();
        if let Some(peer) = self.notification_peer.lock().await.clone() {
            relay.add_peer(PeerHandle {
                sink: Box::new(McpEventSink::new(peer)),
            }).await;
        }
        let relay_handle = relay.spawn(mailbox);

        if let Ok(mut id) = self.conn.actor_id.lock() {
            *id = Some(agent_id.clone());
        }
        if let Ok(mut h) = self.conn.relay_handle.lock() {
            *h = Some(relay_handle);
        }

        // Emit mesh.register into the shared kernel
        let envelope = match EventEnvelope::new(
            "mesh.register",
            &agent_id,
            &serde_json::json!({ "agent_id": &agent_id, "capabilities": req.capabilities }),
        )
        .map(|e| e.with_actor_id(&agent_id))
        {
            Ok(e) => e,
            Err(e) => return format!("Error: failed to build event: {e}"),
        };
        let result = self.route(envelope).await;
        if result.starts_with("Error") {
            return result;
        }
        format!("registered: {agent_id}")
    }

    #[tool(description = "Send a directed message to another agent on the mesh.")]
    async fn mesh_send(&self, Parameters(req): Parameters<MeshSendRequest>) -> String {
        let Some(actor_id) = self.actor_id().await else {
            return "Error: not registered — call mesh_register first".to_string();
        };
        let envelope = match EventEnvelope::new(
            "mesh.send",
            &actor_id,
            &serde_json::json!({ "from": &actor_id, "to": &req.to, "body": req.body }),
        )
        .map(|e| e.with_actor_id(&actor_id))
        {
            Ok(e) => e,
            Err(e) => return format!("Error: {e}"),
        };
        self.route(envelope).await
    }

    #[tool(
        description = "Broadcast a message to all agents on the mesh, optionally filtered by capability."
    )]
    async fn mesh_broadcast(&self, Parameters(req): Parameters<MeshBroadcastRequest>) -> String {
        let Some(actor_id) = self.actor_id().await else {
            return "Error: not registered — call mesh_register first".to_string();
        };
        let payload = match &req.capability_filter {
            Some(cap) => serde_json::json!({
                "from": &actor_id, "body": req.body, "capability_filter": cap
            }),
            None => serde_json::json!({ "from": &actor_id, "body": req.body }),
        };
        let envelope = match EventEnvelope::new("mesh.broadcast", &actor_id, &payload)
            .map(|e| e.with_actor_id(&actor_id))
        {
            Ok(e) => e,
            Err(e) => return format!("Error: {e}"),
        };
        self.route(envelope).await
    }

    #[tool(description = "Discover agents on the mesh. Optionally filter by capability or status.")]
    async fn mesh_discover(&self, Parameters(req): Parameters<MeshDiscoverRequest>) -> String {
        let Some(actor_id) = self.actor_id().await else {
            return "Error: not registered — call mesh_register first".to_string();
        };
        let mut payload = serde_json::json!({ "from": &actor_id });
        if let Some(cap) = req.capability {
            payload["capability"] = serde_json::Value::String(cap);
        }
        if let Some(st) = req.status {
            payload["status"] = serde_json::Value::String(st);
        }
        let envelope =
            match EventEnvelope::new("mesh.discover.request", &actor_id, &payload)
                .map(|e| e.with_actor_id(&actor_id))
            {
                Ok(e) => e,
                Err(e) => return format!("Error: {e}"),
            };
        self.route(envelope).await
    }

    #[tool(description = "Update this agent's status on the mesh (active, busy, idle).")]
    async fn mesh_status(&self, Parameters(req): Parameters<MeshStatusRequest>) -> String {
        let Some(actor_id) = self.actor_id().await else {
            return "Error: not registered — call mesh_register first".to_string();
        };
        match req.status.as_str() {
            "active" | "busy" | "idle" => {}
            other => return format!("Error: unknown status '{other}' — use active|busy|idle"),
        }
        let envelope = match EventEnvelope::new(
            "mesh.status",
            &actor_id,
            &serde_json::json!({ "agent_id": &actor_id, "status": &req.status }),
        )
        .map(|e| e.with_actor_id(&actor_id))
        {
            Ok(e) => e,
            Err(e) => return format!("Error: {e}"),
        };
        self.route(envelope).await
    }
}

#[cfg(test)]
#[path = "server_tests.rs"]
mod tests;
