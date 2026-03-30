use anyhow::anyhow;
use runtime::events::{EventEnvelope, RESERVED_NAMESPACES};

/// Build and validate an agent-emitted domain event.
///
/// `actor_id` and `workspace_id` are injected by the MCP handler from connection context.
/// Agents cannot supply or override these values.
pub fn handle_ship_event(
    actor_id: &str,
    workspace_id: &str,
    event_type: &str,
    payload: serde_json::Value,
    elevated: bool,
) -> anyhow::Result<EventEnvelope> {
    if event_type.is_empty() {
        return Err(anyhow!("event_type must not be empty"));
    }
    if !event_type.contains('.') {
        return Err(anyhow!(
            "event_type '{}' must be namespaced with a dot (e.g. 'deployment.completed')",
            event_type
        ));
    }
    for prefix in RESERVED_NAMESPACES {
        if event_type.starts_with(prefix) {
            return Err(anyhow!(
                "event_type '{}' is reserved: '{}' prefix is platform-controlled",
                event_type,
                prefix
            ));
        }
    }

    let mut envelope = EventEnvelope::new(event_type, workspace_id, &payload)?
        .with_actor_id(actor_id)
        .with_context(Some(workspace_id), None);
    if elevated {
        envelope = envelope.elevate();
    }
    Ok(envelope)
}

#[cfg(test)]
#[path = "event_tests.rs"]
mod event_tests;
