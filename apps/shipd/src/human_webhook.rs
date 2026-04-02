//! Telegram webhook handler.
//!
//! `POST /api/webhook/telegram` receives Telegram `Update` objects and
//! emits `human.response` events on the kernel so Commander can unblock.

use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::{info, warn};

use runtime::events::{CallerKind, EmitContext, EventEnvelope, KernelRouter};

/// Shared kernel handle used as Axum state.
pub type SharedKernel = Arc<Mutex<KernelRouter>>;

// ── Telegram update types ─────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
}

#[derive(Deserialize)]
pub struct TelegramMessage {
    pub text: Option<String>,
    pub chat: TelegramChat,
}

#[derive(Deserialize)]
pub struct TelegramChat {
    pub id: i64,
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// Receive a Telegram webhook update and emit `human.response` on the kernel.
pub async fn telegram_webhook(
    State(kernel): State<SharedKernel>,
    Json(update): Json<TelegramUpdate>,
) -> StatusCode {
    let message = match update.message {
        Some(m) => m,
        None => {
            // Non-message update (e.g. edited_message) — acknowledge and ignore.
            return StatusCode::OK;
        }
    };

    let text = message.text.unwrap_or_default();
    let chat_id = message.chat.id;

    let envelope = match EventEnvelope::new(
        "human.response",
        "human",
        &serde_json::json!({
            "source": "telegram",
            "text": text,
            "chat_id": chat_id,
        }),
    ) {
        Ok(e) => e,
        Err(e) => {
            warn!("[human-webhook] failed to build event envelope: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };

    if let Err(e) = kernel.lock().await.route(envelope, &ctx).await {
        warn!("[human-webhook] failed to route human.response: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    info!("[human-webhook] human.response emitted from telegram chat_id={chat_id}");
    StatusCode::OK
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_update_with_message() {
        let json = r#"{
            "update_id": 123456,
            "message": {
                "text": "yes, approve it",
                "chat": { "id": 9876543 }
            }
        }"#;
        let update: TelegramUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.update_id, 123456);
        let msg = update.message.unwrap();
        assert_eq!(msg.text.as_deref(), Some("yes, approve it"));
        assert_eq!(msg.chat.id, 9876543);
    }

    #[test]
    fn test_telegram_update_without_message() {
        let json = r#"{"update_id": 999}"#;
        let update: TelegramUpdate = serde_json::from_str(json).unwrap();
        assert!(update.message.is_none());
    }

    #[test]
    fn test_telegram_message_missing_text() {
        let json = r#"{
            "update_id": 1,
            "message": { "chat": { "id": 42 } }
        }"#;
        let update: TelegramUpdate = serde_json::from_str(json).unwrap();
        let msg = update.message.unwrap();
        assert!(msg.text.is_none());
    }
}
