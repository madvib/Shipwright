use super::{handle_after_tool, handle_before_tool, handle_session_end, handle_session_start};
use runtime::events::types::event_types;

#[test]
fn session_start_emits_correct_event() {
    let event = handle_session_start("actor-1", "ws-1").expect("handle_session_start failed");
    assert_eq!(event.event_type, event_types::SESSION_STARTED);
    assert_eq!(event.actor_id.as_deref(), Some("actor-1"));
}

#[test]
fn session_start_workspace_id_in_context() {
    let event = handle_session_start("actor-x", "ws-x").expect("handle_session_start failed");
    assert_eq!(event.workspace_id.as_deref(), Some("ws-x"));
}

#[test]
fn session_end_emits_correct_event() {
    let event = handle_session_end("actor-1", "ws-1").unwrap();
    assert_eq!(event.event_type, event_types::SESSION_ENDED);
    assert_eq!(event.actor_id.as_deref(), Some("actor-1"));
}

#[test]
fn before_tool_returns_none_for_non_ship_tool() {
    let json = r#"{"tool_name": "bash", "hook_event_name": "PreToolUse"}"#;
    let result = handle_before_tool(json, "a", "w").unwrap();
    assert!(result.is_none());
}

#[test]
fn before_tool_emits_skill_started_for_ship_tool() {
    let json = r#"{"tool_name": "mcp__ship__commit", "hook_event_name": "PreToolUse"}"#;
    let event = handle_before_tool(json, "a", "w").unwrap().expect("expected event");
    assert_eq!(event.event_type, event_types::SKILL_STARTED);
}

#[test]
fn after_tool_emits_skill_completed_for_ship_tool() {
    let json = r#"{"tool_name": "mcp__ship__commit", "hook_event_name": "PostToolUse"}"#;
    let event = handle_after_tool(json, "a", "w").unwrap().expect("expected event");
    assert_eq!(event.event_type, event_types::SKILL_COMPLETED);
}

#[test]
fn after_tool_emits_skill_failed_when_error_present() {
    let json = r#"{"tool_name": "mcp__ship__commit", "hook_event_name": "PostToolUse", "tool_response": {"error": "something failed"}}"#;
    let event = handle_after_tool(json, "a", "w").unwrap().expect("expected event");
    assert_eq!(event.event_type, event_types::SKILL_FAILED);
}

#[test]
fn before_tool_errors_on_empty_stdin() {
    assert!(handle_before_tool("", "a", "w").is_err());
}
