//! Human gateway service — puts a human on the mesh via pluggable adapters.
//!
//! When a `job.blocked` event fires with `needs_human: true`, the gateway
//! forwards the block reason to a human via the configured adapter (Telegram,
//! Slack, etc.). Inbound replies arrive via webhook and are emitted as
//! `human.response` events by the shipd webhook handler.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::events::{ActorStore, EventEnvelope};
use crate::services::ServiceHandler;

// ── Adapter trait ─────────────────────────────────────────────────────────────

/// Outbound message sent to a human via a notification adapter.
pub struct OutboundMessage {
    pub text: String,
    pub job_id: Option<String>,
    pub metadata: serde_json::Value,
}

/// Pluggable notification adapter. Implement this to add Slack, WhatsApp, etc.
#[async_trait]
pub trait NotificationAdapter: Send + Sync {
    /// Send a message to the human. Returns a provider message ID on success.
    async fn send(&self, msg: &OutboundMessage) -> Result<String>;
    /// Stable adapter name used in log output and config.
    fn name(&self) -> &'static str;
}

// ── TelegramAdapter ───────────────────────────────────────────────────────────

/// Telegram Bot API adapter.
///
/// Configuration via environment variables:
/// - `SHIP_TELEGRAM_TOKEN` — bot token from BotFather
/// - `SHIP_TELEGRAM_CHAT_ID` — target chat ID (user or group)
pub struct TelegramAdapter {
    token: String,
    chat_id: String,
    client: reqwest::Client,
}

impl TelegramAdapter {
    pub fn new(token: String, chat_id: String) -> Self {
        Self { token, chat_id, client: reqwest::Client::new() }
    }

    /// Build from environment variables. Returns `None` if either var is absent.
    pub fn from_env() -> Option<Self> {
        let token = std::env::var("SHIP_TELEGRAM_TOKEN").ok()?;
        let chat_id = std::env::var("SHIP_TELEGRAM_CHAT_ID").ok()?;
        Some(Self::new(token, chat_id))
    }
}

#[async_trait]
impl NotificationAdapter for TelegramAdapter {
    fn name(&self) -> &'static str {
        "telegram"
    }

    async fn send(&self, msg: &OutboundMessage) -> Result<String> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.token
        );
        let body = serde_json::json!({
            "chat_id": self.chat_id,
            "text": msg.text,
        });
        let resp: serde_json::Value = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if resp["ok"].as_bool() != Some(true) {
            anyhow::bail!(
                "telegram sendMessage failed: {}",
                resp["description"].as_str().unwrap_or("unknown error")
            );
        }
        let msg_id = resp["result"]["message_id"]
            .as_i64()
            .map(|id| id.to_string())
            .unwrap_or_default();
        Ok(msg_id)
    }
}

// ── JobBlockedPayload ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct JobBlockedPayload {
    job_id: Option<String>,
    slug: Option<String>,
    blocker: Option<String>,
    needs_human: Option<bool>,
}

// ── HumanGatewayService ───────────────────────────────────────────────────────

/// Headless service that listens for `job.blocked` events and notifies a human.
pub struct HumanGatewayService {
    adapter: Arc<dyn NotificationAdapter>,
    /// Outbox for emitting events back onto the kernel (reserved for future use).
    _outbox: mpsc::UnboundedSender<EventEnvelope>,
}

impl HumanGatewayService {
    pub fn new(
        adapter: Arc<dyn NotificationAdapter>,
        outbox: mpsc::UnboundedSender<EventEnvelope>,
    ) -> Self {
        Self { adapter, _outbox: outbox }
    }

    fn handle_job_blocked(&self, event: &EventEnvelope) {
        let payload: JobBlockedPayload = match serde_json::from_str(&event.payload_json) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[human-gateway] failed to parse job.blocked payload: {e}");
                return;
            }
        };

        if payload.needs_human != Some(true) {
            return;
        }

        let job_id = payload.job_id.clone();
        let text = format!(
            "\u{1f6a7} Blocked: {}\nJob: {} ({})",
            payload.blocker.as_deref().unwrap_or("unknown"),
            payload.slug.as_deref().unwrap_or("unknown"),
            job_id.as_deref().unwrap_or("unknown"),
        );

        let msg = OutboundMessage {
            text,
            job_id,
            metadata: serde_json::json!({}),
        };

        let adapter = self.adapter.clone();
        tokio::spawn(async move {
            match adapter.send(&msg).await {
                Ok(id) => eprintln!("[human-gateway] telegram message sent, id={id}"),
                Err(e) => eprintln!("[human-gateway] adapter send failed: {e}"),
            }
        });
    }
}

impl ServiceHandler for HumanGatewayService {
    fn name(&self) -> &str {
        "human-gateway"
    }

    fn handle(&mut self, event: &EventEnvelope, _store: &ActorStore) -> Result<()> {
        if event.event_type == "job.blocked" {
            self.handle_job_blocked(event);
        }
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_blocked_payload_needs_human_false() {
        let json = r#"{"job_id":"j1","slug":"my-job","blocker":"waiting","needs_human":false}"#;
        let p: JobBlockedPayload = serde_json::from_str(json).unwrap();
        assert_eq!(p.needs_human, Some(false));
        assert_eq!(p.job_id.as_deref(), Some("j1"));
    }

    #[test]
    fn test_job_blocked_payload_needs_human_true() {
        let json = r#"{"job_id":"j2","slug":"deploy","blocker":"needs approval","needs_human":true}"#;
        let p: JobBlockedPayload = serde_json::from_str(json).unwrap();
        assert_eq!(p.needs_human, Some(true));
        assert_eq!(p.blocker.as_deref(), Some("needs approval"));
    }

    #[test]
    fn test_outbound_message_format() {
        let payload = JobBlockedPayload {
            job_id: Some("abc".to_string()),
            slug: Some("release".to_string()),
            blocker: Some("manual review required".to_string()),
            needs_human: Some(true),
        };
        let text = format!(
            "\u{1f6a7} Blocked: {}\nJob: {} ({})",
            payload.blocker.as_deref().unwrap_or("unknown"),
            payload.slug.as_deref().unwrap_or("unknown"),
            payload.job_id.as_deref().unwrap_or("unknown"),
        );
        assert!(text.contains("manual review required"));
        assert!(text.contains("release"));
        assert!(text.contains("abc"));
    }

    #[test]
    fn test_telegram_adapter_from_env_missing() {
        // Without env vars set, from_env returns None.
        // SAFETY: test-only, single-threaded context
        unsafe {
            std::env::remove_var("SHIP_TELEGRAM_TOKEN");
            std::env::remove_var("SHIP_TELEGRAM_CHAT_ID");
        }
        assert!(TelegramAdapter::from_env().is_none());
    }
}
