//! Claude Code channels push adapter.
//!
//! Sends events as `notifications/claude/channel` — the Claude Code channels
//! protocol. Events arrive in Claude's context as:
//!
//! ```xml
//! <channel source="ship" event_type="session.ended" actor_id="agent.rust-lane">
//!   { ...event payload... }
//! </channel>
//! ```
//!
//! Claude can then react: read the payload, call Ship tools, reply via mesh, etc.
//! This is the primary push mechanism for Claude Code agents.

use async_trait::async_trait;
use rmcp::{Peer, RoleServer, model::ServerNotification};
use rmcp::model::CustomNotification;
use runtime::events::EventEnvelope;
use tracing::warn;

use super::PushAdapter;

pub struct ClaudeChannelAdapter {
    peer: Peer<RoleServer>,
}

impl ClaudeChannelAdapter {
    pub fn new(peer: Peer<RoleServer>) -> Self {
        Self { peer }
    }
}

#[async_trait]
impl PushAdapter for ClaudeChannelAdapter {
    async fn push_event(&self, event: &EventEnvelope) {
        // Build the channel notification content — human-readable summary
        // that Claude can parse, plus structured meta attributes.
        let content = match serde_json::from_str::<serde_json::Value>(&event.payload_json) {
            Ok(payload) => format_event_content(event, &payload),
            Err(_) => format!("[{}] {}", event.event_type, event.entity_id),
        };

        let mut meta = serde_json::Map::new();
        meta.insert("event_type".into(), event.event_type.clone().into());
        meta.insert("entity_id".into(), event.entity_id.clone().into());
        if let Some(ref actor_id) = event.actor_id {
            meta.insert("actor_id".into(), actor_id.clone().into());
        }
        if let Some(ref workspace_id) = event.workspace_id {
            meta.insert("workspace_id".into(), workspace_id.clone().into());
        }
        if let Some(ref session_id) = event.session_id {
            meta.insert("session_id".into(), session_id.clone().into());
        }

        let params = serde_json::json!({
            "content": content,
            "meta": meta,
        });

        let notification = CustomNotification::new(
            "notifications/claude/channel",
            Some(params),
        );
        let server_notif = ServerNotification::CustomNotification(notification);

        if let Err(e) = self.peer.send_notification(server_notif).await {
            warn!("claude channel push failed: {e}");
        }
    }

    fn adapter_name(&self) -> &'static str {
        "claude-channel"
    }
}

/// Format event content for Claude's context window.
/// Keep it concise — Claude reads this as part of its conversation.
pub fn format_event_content(event: &EventEnvelope, payload: &serde_json::Value) -> String {
    match event.event_type.as_str() {
        "mesh.message" => {
            let from = payload["from"].as_str()
                .or_else(|| payload["from_agent_id"].as_str())
                .unwrap_or("unknown");
            let body = payload["body"]
                .as_str()
                .map(String::from)
                .unwrap_or_else(|| {
                    serde_json::to_string(&payload["body"]).unwrap_or_default()
                });
            format!("Message from {from}: {body}")
        }
        "session.ended" => {
            let summary = payload["summary"].as_str().unwrap_or("no summary");
            let branch = event.workspace_id.as_deref().unwrap_or("unknown");
            format!("Session ended on {branch}: {summary}")
        }
        "workspace.compiled" => {
            let branch = event.entity_id.as_str();
            format!("Workspace {branch} compiled successfully")
        }
        "workspace.compile_failed" => {
            let error = payload["error"].as_str().unwrap_or("unknown error");
            format!("Compile failed on {}: {error}", event.entity_id)
        }
        "gate.completed" => {
            let result = payload["result"].as_str().unwrap_or("unknown");
            format!("Gate review completed: {result}")
        }
        _ => {
            // Generic: event type + compact payload
            let compact = serde_json::to_string(payload).unwrap_or_default();
            let truncated: String = compact.chars().take(200).collect();
            format!("[{}] {}", event.event_type, truncated)
        }
    }
}
