use serde_json::json;

use super::handle_ship_event;
use runtime::events::RESERVED_NAMESPACES;

#[test]
fn ship_event_accepts_valid_domain_event() {
    let event = handle_ship_event(
        "actor-1",
        "ws-1",
        "deployment.completed",
        json!({"env": "prod"}),
        false,
    )
    .expect("valid domain event must be accepted");

    assert_eq!(event.event_type, "deployment.completed");
    assert_eq!(event.actor_id.as_deref(), Some("actor-1"));
    assert_eq!(event.workspace_id.as_deref(), Some("ws-1"));
    assert!(!event.elevated);
}

#[test]
fn ship_event_rejects_reserved_actor_type() {
    let result = handle_ship_event("actor-1", "ws-1", "actor.created", json!({}), false);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("reserved"), "error must mention 'reserved': {msg}");
}

#[test]
fn ship_event_rejects_reserved_session_type() {
    let result = handle_ship_event("actor-1", "ws-1", "session.started", json!({}), false);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("reserved"), "error must mention 'reserved': {msg}");
}

#[test]
fn ship_event_rejects_reserved_skill_type() {
    let result = handle_ship_event("actor-1", "ws-1", "skill.started", json!({}), false);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("reserved"), "error must mention 'reserved': {msg}");
}

#[test]
fn ship_event_rejects_reserved_workspace_type() {
    let result = handle_ship_event("actor-1", "ws-1", "workspace.activated", json!({}), false);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("reserved"), "error must mention 'reserved': {msg}");
}

#[test]
fn ship_event_elevated_flag_propagated() {
    let event = handle_ship_event("actor-1", "ws-1", "test.result", json!({}), true)
        .expect("elevated domain event must be accepted");

    assert!(event.elevated, "elevated flag must be set on the returned event");
}

#[test]
fn ship_event_rejects_empty_event_type() {
    let result = handle_ship_event("actor-1", "ws-1", "", json!({}), false);
    assert!(result.is_err(), "empty event_type must be rejected");
}

#[test]
fn ship_event_rejects_event_type_without_dot() {
    // Event types must be namespaced (e.g. "deployment.completed", not "deployment")
    let result = handle_ship_event("actor-1", "ws-1", "nodot", json!({}), false);
    assert!(result.is_err(), "un-namespaced event_type must be rejected");
}

#[test]
fn ship_event_payload_preserved() {
    let input = json!({"key": "value", "count": 42});
    let event = handle_ship_event("actor-1", "ws-1", "test.payload", input.clone(), false)
        .expect("valid domain event must be accepted");

    let stored: serde_json::Value =
        serde_json::from_str(&event.payload_json).expect("payload_json must be valid JSON");
    assert_eq!(stored, input, "stored payload must round-trip to original value");
}

#[test]
fn ship_event_actor_id_cannot_be_overridden() {
    // actor_id is injected from MCP context, not from the agent payload.
    // A payload field named "actor_id" must not override the context value.
    let event = handle_ship_event(
        "context-actor",
        "ws-1",
        "test.event",
        json!({"actor_id": "evil-override"}),
        false,
    )
    .expect("valid domain event must be accepted");

    assert_eq!(
        event.actor_id.as_deref(),
        Some("context-actor"),
        "actor_id must come from MCP context, not agent payload",
    );
}

#[test]
fn ship_event_rejects_reserved_gate_type() {
    let result = handle_ship_event("actor-1", "ws-1", "gate.passed", json!({}), false);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("reserved"), "error must mention 'reserved': {msg}");
}

#[test]
fn ship_event_rejects_reserved_job_type() {
    let result = handle_ship_event("actor-1", "ws-1", "job.created", json!({}), false);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("reserved"), "error must mention 'reserved': {msg}");
}

#[test]
fn ship_event_rejects_studio_namespace() {
    // Agents must not emit studio.* events — only StudioServer may do so
    let result = handle_ship_event("actor-1", "ws-1", "studio.message.visual", json!({}), false);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("reserved"), "error must mention 'reserved': {msg}");
}

#[test]
fn reserved_type_list_is_complete() {
    let required = [
        "actor.", "config.", "gate.", "job.", "project.",
        "runtime.", "session.", "skill.", "studio.",
        "sync.", "workspace.",
    ];
    for prefix in required {
        assert!(
            RESERVED_NAMESPACES.contains(&prefix),
            "RESERVED_NAMESPACES must contain '{prefix}'",
        );
    }
}
