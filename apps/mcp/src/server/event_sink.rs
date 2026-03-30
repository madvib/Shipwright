//! MCP event sink — sends `ship/event` custom notifications to connected peers.

use async_trait::async_trait;
use rmcp::model::CustomNotification;
use rmcp::{Peer, RoleServer, model::ServerNotification};
use runtime::events::EventEnvelope;
use tracing::warn;

use super::notification_relay::EventSink;

/// Adapter that implements [`EventSink`] for an rmcp `Peer<RoleServer>`.
///
/// Sends events as `ship/event` custom MCP notifications with the full
/// EventEnvelope serialized as the `params` payload.
pub struct McpEventSink {
    peer: Peer<RoleServer>,
}

impl McpEventSink {
    pub fn new(peer: Peer<RoleServer>) -> Self {
        Self { peer }
    }
}

#[async_trait]
impl EventSink for McpEventSink {
    async fn send_event(&self, event: &EventEnvelope) {
        let params = match serde_json::to_value(event) {
            Ok(v) => v,
            Err(e) => {
                warn!("failed to serialize event for MCP notification: {e}");
                return;
            }
        };

        let notification = CustomNotification::new("ship/event", Some(params));
        let server_notif = ServerNotification::CustomNotification(notification);

        if let Err(e) = self.peer.send_notification(server_notif).await {
            warn!("failed to send ship/event notification: {e}");
        }
    }
}
