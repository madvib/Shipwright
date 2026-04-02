//! Generic MCP notification push adapter (fallback).
//!
//! Sends events as `ship/event` custom MCP notifications with the full
//! EventEnvelope serialized as params. Used for providers that don't have
//! a dedicated push mechanism (Cursor, Codex, OpenCode, Gemini).
//!
//! Providers can poll for events via MCP tools as an alternative.

use async_trait::async_trait;
use rmcp::model::CustomNotification;
use rmcp::{Peer, RoleServer, model::ServerNotification};
use runtime::events::EventEnvelope;
use tracing::warn;

use super::PushAdapter;

pub struct McpNotificationAdapter {
    peer: Peer<RoleServer>,
}

impl McpNotificationAdapter {
    pub fn new(peer: Peer<RoleServer>) -> Self {
        Self { peer }
    }
}

#[async_trait]
impl PushAdapter for McpNotificationAdapter {
    async fn push_event(&self, event: &EventEnvelope) {
        let params = match serde_json::to_value(event) {
            Ok(v) => v,
            Err(e) => {
                warn!("failed to serialize event: {e}");
                return;
            }
        };

        let notification = CustomNotification::new("ship/event", Some(params));
        let server_notif = ServerNotification::CustomNotification(notification);

        if let Err(e) = self.peer.send_notification(server_notif).await {
            warn!("ship/event push failed: {e}");
        }
    }

    fn adapter_name(&self) -> &'static str {
        "mcp-notification"
    }
}
