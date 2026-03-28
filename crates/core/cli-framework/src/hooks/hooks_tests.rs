// Tests for the `ship hook` CLI normalization layer.
//
// These tests define the contract. They will not compile until the functions
// in hooks.rs are implemented — that is the expected TDD red state.
//
// Provider schemas verified against:
//   Claude Code: hook_event_name PreToolUse/PostToolUse, fields: tool_name, tool_input, tool_response
//   Gemini CLI:  fields: tool_name, tool_input, mcp_context, tool_response (with optional error)
//   Cursor:      fields: tool_name, tool_input, hook_event_name "beforeMCPExecution", conversation_id

use super::{extract_skill_id, handle_after_tool, handle_before_tool, handle_session_end};

// ── extract_skill_id ──────────────────────────────────────────────────────────

#[test]
fn extract_skill_id_from_ship_tool_name() {
    assert_eq!(extract_skill_id("mcp__ship__commit"), Some("commit"));
}

#[test]
fn extract_skill_id_from_ship_tool_with_longer_name() {
    assert_eq!(extract_skill_id("mcp__ship__review_pr"), Some("review_pr"));
}

#[test]
fn extract_skill_id_ignores_non_ship_tools() {
    assert_eq!(extract_skill_id("mcp__filesystem__read_file"), None);
    assert_eq!(extract_skill_id("bash"), None);
}

// ── handle_before_tool ────────────────────────────────────────────────────────

#[test]
fn before_tool_claude_stdin_emits_skill_started() {
    let stdin = r#"{"tool_name":"mcp__ship__commit","tool_input":{},"hook_event_name":"PreToolUse"}"#;
    let event = handle_before_tool(stdin, "actor-1", "ws-1")
        .expect("must not error on valid Claude stdin")
        .expect("must emit an event for a ship tool");
    assert_eq!(event.event_type, "skill.started");
    assert_eq!(event.actor_id, Some("actor-1".to_string()));
    assert_eq!(event.workspace_id, Some("ws-1".to_string()));
    assert!(
        event.payload_json.contains("\"skill_id\""),
        "payload must contain a skill_id key"
    );
    assert!(
        event.payload_json.contains("\"commit\""),
        "payload must contain the extracted skill id value"
    );
}

#[test]
fn before_tool_non_ship_tool_emits_nothing() {
    let stdin = r#"{"tool_name":"bash","tool_input":{},"hook_event_name":"PreToolUse"}"#;
    let result = handle_before_tool(stdin, "actor-1", "ws-1")
        .expect("must not error on valid stdin");
    assert!(result.is_none(), "non-ship tool must emit nothing");
}

// ── handle_after_tool ─────────────────────────────────────────────────────────

#[test]
fn after_tool_success_emits_skill_completed() {
    let stdin = r#"{"tool_name":"mcp__ship__commit","tool_input":{},"tool_response":{"output":"ok"},"hook_event_name":"PostToolUse"}"#;
    let event = handle_after_tool(stdin, "actor-1", "ws-1")
        .expect("must not error on valid Claude PostToolUse stdin")
        .expect("must emit an event for a ship tool");
    assert_eq!(event.event_type, "skill.completed");
}

#[test]
fn after_tool_error_emits_skill_failed() {
    let stdin = r#"{"tool_name":"mcp__ship__commit","tool_input":{},"tool_response":{"error":"permission denied"},"hook_event_name":"PostToolUse"}"#;
    let event = handle_after_tool(stdin, "actor-1", "ws-1")
        .expect("must not error on valid Claude PostToolUse stdin")
        .expect("must emit an event for a ship tool");
    assert_eq!(event.event_type, "skill.failed");
    assert!(
        event.payload_json.contains("permission denied"),
        "payload must carry the error message"
    );
}

// ── provider normalization ────────────────────────────────────────────────────

#[test]
fn gemini_before_tool_normalized() {
    // Gemini CLI BeforeTool schema: tool_name, tool_input, mcp_context (no hook_event_name).
    // Source: geminicli.com/docs/hooks/reference/
    let stdin = r#"{"tool_name":"mcp__ship__commit","tool_input":{},"mcp_context":{}}"#;
    let event = handle_before_tool(stdin, "actor-1", "ws-1")
        .expect("must not error on valid Gemini BeforeTool stdin")
        .expect("must emit an event for a ship tool");
    assert_eq!(event.event_type, "skill.started");
    assert!(
        event.payload_json.contains("\"commit\""),
        "payload must contain the extracted skill id"
    );
}

#[test]
fn cursor_before_mcp_execution_normalized() {
    // Cursor beforeMCPExecution schema: tool_name, tool_input, hook_event_name, conversation_id.
    // Source: cursor.com/docs/hooks
    let stdin = r#"{"tool_name":"mcp__ship__commit","tool_input":{},"hook_event_name":"beforeMCPExecution","conversation_id":"conv-1"}"#;
    let event = handle_before_tool(stdin, "actor-1", "ws-1")
        .expect("must not error on valid Cursor beforeMCPExecution stdin")
        .expect("must emit an event for a ship tool");
    assert_eq!(event.event_type, "skill.started");
}

// ── handle_session_end ────────────────────────────────────────────────────────

#[test]
fn session_end_emits_correct_event() {
    let event = handle_session_end("actor-1", "ws-1")
        .expect("handle_session_end must not error");
    assert_eq!(event.event_type, "session.ended");
    assert_eq!(event.actor_id, Some("actor-1".to_string()));
}

// ── error handling ────────────────────────────────────────────────────────────

#[test]
fn hook_handler_returns_err_for_empty_stdin() {
    let result = handle_before_tool("", "actor-1", "ws-1");
    assert!(result.is_err(), "empty stdin must return an error, not panic");
    assert!(
        !result.unwrap_err().to_string().is_empty(),
        "error message must not be empty"
    );
}

#[test]
fn hook_handler_returns_err_for_invalid_json() {
    let result = handle_before_tool("not json at all", "actor-1", "ws-1");
    assert!(result.is_err(), "invalid JSON must return an error, not panic");
}
