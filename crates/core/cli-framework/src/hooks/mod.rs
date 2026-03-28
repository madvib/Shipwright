use anyhow::{Result, anyhow};
use runtime::events::envelope::EventEnvelope;
use runtime::events::types::{
    SessionEnded, SkillCompleted, SkillFailed, SkillStarted,
    event_types,
};
use serde::Deserialize;

#[cfg(test)]
mod hooks_tests;

/// Extracts skill_id from a ship MCP tool name.
/// "mcp__ship__commit" -> Some("commit")
/// Non-ship tools -> None
pub fn extract_skill_id(tool_name: &str) -> Option<&str> {
    tool_name.strip_prefix("mcp__ship__")
}

// hook_event_name and mcp_context are deserialized for schema compatibility
// but dispatch is based on tool_name alone.
#[allow(dead_code)]
#[derive(Deserialize)]
struct ProviderHook {
    tool_name: String,
    hook_event_name: Option<String>,
    tool_response: Option<serde_json::Value>,
    mcp_context: Option<serde_json::Value>,
}

/// Parses provider hook stdin JSON and emits skill.started.
/// Returns Ok(None) for non-ship tools (ignore silently).
/// Supports Claude PreToolUse, Gemini BeforeTool, Cursor beforeMCPExecution schemas.
pub fn handle_before_tool(
    stdin_json: &str,
    actor_id: &str,
    workspace_id: &str,
) -> Result<Option<EventEnvelope>> {
    if stdin_json.is_empty() {
        return Err(anyhow!("stdin is empty"));
    }
    let hook: ProviderHook = serde_json::from_str(stdin_json)
        .map_err(|e| anyhow!("invalid JSON in hook stdin: {e}"))?;
    let skill_id = match extract_skill_id(&hook.tool_name) {
        Some(id) => id.to_string(),
        None => return Ok(None),
    };
    let event = EventEnvelope::new(
        event_types::SKILL_STARTED,
        &skill_id,
        &SkillStarted { skill_id: skill_id.clone() },
    )?
    .with_actor_id(actor_id)
    .with_context(Some(workspace_id), None);
    Ok(Some(event))
}

/// Parses provider hook stdin JSON and emits skill.completed or skill.failed.
/// Returns Ok(None) for non-ship tools.
pub fn handle_after_tool(
    stdin_json: &str,
    actor_id: &str,
    workspace_id: &str,
) -> Result<Option<EventEnvelope>> {
    if stdin_json.is_empty() {
        return Err(anyhow!("stdin is empty"));
    }
    let hook: ProviderHook = serde_json::from_str(stdin_json)
        .map_err(|e| anyhow!("invalid JSON in hook stdin: {e}"))?;
    let skill_id = match extract_skill_id(&hook.tool_name) {
        Some(id) => id.to_string(),
        None => return Ok(None),
    };
    let event = if let Some(error_msg) = hook.tool_response.as_ref().and_then(extract_error) {
        EventEnvelope::new(
            event_types::SKILL_FAILED,
            &skill_id,
            &SkillFailed { skill_id: skill_id.clone(), error: error_msg },
        )?
    } else {
        EventEnvelope::new(
            event_types::SKILL_COMPLETED,
            &skill_id,
            &SkillCompleted { skill_id: skill_id.clone(), duration_ms: None },
        )?
    };
    Ok(Some(
        event.with_actor_id(actor_id).with_context(Some(workspace_id), None),
    ))
}

/// Emits session.ended for the current actor.
pub fn handle_session_end(
    actor_id: &str,
    workspace_id: &str,
) -> Result<EventEnvelope> {
    let event = EventEnvelope::new(
        event_types::SESSION_ENDED,
        actor_id,
        &SessionEnded { summary: None, duration_secs: None, gate_result: None },
    )?
    .with_actor_id(actor_id)
    .with_context(Some(workspace_id), None);
    Ok(event)
}

fn extract_error(response: &serde_json::Value) -> Option<String> {
    let err = response.get("error")?;
    if err.is_null() {
        return None;
    }
    Some(err.as_str().unwrap_or("unknown error").to_string())
}
