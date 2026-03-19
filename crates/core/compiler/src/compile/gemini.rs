use serde_json::Value as Json;

use crate::resolve::ResolvedConfig;
use crate::types::{HookTrigger, McpServerConfig, Permissions};

use super::provider::ProviderDescriptor;

// ── Trigger mapping ───────────────────────────────────────────────────────────

/// Map our internal `HookTrigger` to Gemini's event name.
/// Source: https://geminicli.com/docs/hooks
fn gemini_trigger(t: &HookTrigger) -> Option<&'static str> {
    match t {
        HookTrigger::PreToolUse  => Some("BeforeTool"),
        HookTrigger::PostToolUse => Some("AfterTool"),
        HookTrigger::Notification => Some("Notification"),
        HookTrigger::Stop        => Some("SessionEnd"),
        HookTrigger::PreCompact  => Some("PreCompress"),
        // No Gemini equivalent
        HookTrigger::SubagentStop => None,
    }
}

/// Translate our internal approval mode to Gemini's `defaultApprovalMode` value.
///
/// - `"default"` → `"suggest"`
/// - `"auto_edit"` → `"auto-edit"`
/// - `"plan"` → `"yolo"`
fn translate_approval_mode(val: &str) -> &str {
    match val {
        "default" => "suggest",
        "auto_edit" => "auto-edit",
        "plan" => "yolo",
        other => other,
    }
}

// ── Settings patch ────────────────────────────────────────────────────────────

/// Build the settings patch for `.gemini/settings.json`.
///
/// Includes hooks, model, and all new Gemini-specific fields.
/// Returns `None` when there is nothing to emit.
pub(super) fn build_gemini_settings_patch(resolved: &ResolvedConfig) -> Option<Json> {
    let hooks = &resolved.hooks;
    let model = resolved.model.as_deref();

    let mut by_trigger: std::collections::BTreeMap<String, Vec<Json>> =
        std::collections::BTreeMap::new();

    for h in hooks {
        // Raw event override — bypass trigger mapping.
        let event_key: Option<String> = if let Some(raw) = &h.gemini_event {
            Some(raw.clone())
        } else {
            gemini_trigger(&h.trigger).map(str::to_string)
        };
        let Some(event) = event_key else { continue };

        let hook_obj = serde_json::json!({
            "type": "command",
            "command": h.command,
        });
        let entry = if let Some(m) = &h.matcher {
            serde_json::json!({ "matcher": m, "hooks": [hook_obj] })
        } else {
            serde_json::json!({ "hooks": [hook_obj] })
        };
        by_trigger.entry(event).or_default().push(entry);
    }

    let has_hooks = !by_trigger.is_empty();
    let has_model_name = model.is_some();
    let has_approval_mode = resolved.gemini_default_approval_mode.is_some();
    let has_max_session_turns = resolved.gemini_max_session_turns.is_some();
    let has_disable_yolo = resolved.gemini_disable_yolo_mode.is_some();
    let has_disable_always_allow = resolved.gemini_disable_always_allow.is_some();
    let has_tools_sandbox = resolved.gemini_tools_sandbox.is_some();
    let has_extra = resolved.gemini_settings_extra.as_ref().is_some_and(|v| !v.is_null());

    if !has_hooks && !has_model_name && !has_approval_mode && !has_max_session_turns
        && !has_disable_yolo && !has_disable_always_allow && !has_tools_sandbox && !has_extra
    {
        return None;
    }

    let mut patch = serde_json::json!({});

    // model.name
    if let Some(m) = model {
        patch["model"] = serde_json::json!({ "name": m });
    }

    // general.*
    let has_general = has_approval_mode || has_max_session_turns;
    if has_general {
        let mut general = serde_json::json!({});
        if let Some(mode) = resolved.gemini_default_approval_mode.as_deref() {
            general["defaultApprovalMode"] = Json::String(translate_approval_mode(mode).to_string());
        }
        if let Some(turns) = resolved.gemini_max_session_turns {
            general["maxSessionTurns"] = serde_json::json!(turns);
        }
        patch["general"] = general;
    }

    // security.*
    let has_security = has_disable_yolo || has_disable_always_allow;
    if has_security {
        let mut security = serde_json::json!({});
        if let Some(v) = resolved.gemini_disable_yolo_mode {
            security["disableYoloMode"] = serde_json::json!(v);
        }
        if let Some(v) = resolved.gemini_disable_always_allow {
            security["disableAlwaysAllow"] = serde_json::json!(v);
        }
        patch["security"] = security;
    }

    // tools.sandbox
    if let Some(sandbox) = resolved.gemini_tools_sandbox.as_deref() {
        patch["tools"] = serde_json::json!({ "sandbox": sandbox });
    }

    // hooks
    if has_hooks {
        let hooks_obj: serde_json::Map<String, Json> = by_trigger
            .into_iter()
            .map(|(k, v)| (k, Json::Array(v)))
            .collect();
        patch["hooks"] = Json::Object(hooks_obj);
    }

    // settings_extra — merged verbatim last
    if let Some(extra) = &resolved.gemini_settings_extra {
        if let Some(obj) = extra.as_object() {
            for (k, v) in obj {
                patch[k] = v.clone();
            }
        }
    }

    Some(patch)
}

// ── MCP server entry (Gemini-specific fields) ─────────────────────────────────

/// Build a Gemini MCP server entry with Gemini-specific per-server fields.
pub(super) fn build_gemini_server_entry(desc: &ProviderDescriptor, s: &McpServerConfig) -> Json {
    let mut entry = super::mcp::server_entry(desc, s);

    // Gemini-specific per-server fields.
    if let Some(trust) = s.gemini_trust {
        entry["trust"] = serde_json::json!(trust);
    }
    if !s.gemini_include_tools.is_empty() {
        entry["includeTools"] = serde_json::json!(s.gemini_include_tools);
    }
    if !s.gemini_exclude_tools.is_empty() {
        entry["excludeTools"] = serde_json::json!(s.gemini_exclude_tools);
    }
    if let Some(timeout) = s.gemini_timeout_ms {
        entry["timeout"] = serde_json::json!(timeout);
    }

    entry
}

// ── Gemini MCP servers object ─────────────────────────────────────────────────

pub(super) fn build_gemini_mcp_servers(
    desc: &ProviderDescriptor,
    servers: &[McpServerConfig],
) -> Json {
    let mut map = serde_json::Map::new();

    // Ship server always first.
    map.insert("ship".to_string(), super::mcp::ship_server_entry(desc.emit_type_field));

    for s in servers {
        if s.disabled {
            continue;
        }
        map.insert(s.id.clone(), build_gemini_server_entry(desc, s));
    }

    Json::Object(map)
}

// ─── Gemini policy patch ──────────────────────────────────────────────────────

/// Translate a permission pattern to a Gemini policy `(tool, pattern)` pair.
///
/// Source: https://geminicli.com/docs/reference/policy-engine/
/// Tool names: `shell`, `mcp`, `file_read`, `file_write`, `web_fetch`
/// `pattern` is a regex string; omit (None) for any-match on that tool.
fn translate_to_gemini_policy(pattern: &str) -> Option<(&'static str, Option<String>)> {
    if pattern == "*" {
        return None; // wildcard = no restriction, skip
    }
    // Bash(cmd) → shell tool, cmd as pattern (glob → anchored regex prefix)
    if let Some(inner) = pattern.strip_prefix("Bash(").and_then(|s| s.strip_suffix(')')) {
        let re = glob_to_regex_prefix(inner);
        return Some(("shell", Some(re)));
    }
    if pattern == "Bash" {
        return Some(("shell", None));
    }
    // mcp__server__tool → mcp tool, server/tool as pattern
    if let Some(rest) = pattern.strip_prefix("mcp__") {
        if let Some(idx) = rest.find("__") {
            let server = &rest[..idx];
            let tool = &rest[idx + 2..];
            let re = format!("{}/{}", glob_to_regex(server), glob_to_regex(tool));
            return Some(("mcp", Some(re)));
        }
        return None;
    }
    // Read/Glob → file_read
    if pattern == "Read" || pattern == "Glob" || pattern == "LS" {
        return Some(("file_read", None));
    }
    if let Some(inner) = pattern.strip_prefix("Read(").and_then(|s| s.strip_suffix(')')) {
        return Some(("file_read", Some(glob_to_regex(inner))));
    }
    // Write/Edit/MultiEdit → file_write
    if matches!(pattern, "Write" | "Edit" | "MultiEdit") {
        return Some(("file_write", None));
    }
    if let Some(inner) = pattern.strip_prefix("Write(").and_then(|s| s.strip_suffix(')'))
        .or_else(|| pattern.strip_prefix("Edit(").and_then(|s| s.strip_suffix(')')))
    {
        return Some(("file_write", Some(glob_to_regex(inner))));
    }
    // WebFetch → web_fetch
    if pattern == "WebFetch" {
        return Some(("web_fetch", None));
    }
    if let Some(inner) = pattern.strip_prefix("WebFetch(").and_then(|s| s.strip_suffix(')')) {
        return Some(("web_fetch", Some(glob_to_regex(inner))));
    }
    None
}

/// Convert a simple glob pattern to a regex prefix for Gemini.
/// Handles `*` → `.*` and escapes regex metacharacters.
fn glob_to_regex(glob: &str) -> String {
    glob_to_regex_prefix(glob)
}

fn glob_to_regex_prefix(glob: &str) -> String {
    let mut out = String::new();
    for ch in glob.chars() {
        match ch {
            '*' => out.push_str(".*"),
            '.' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Build the `.gemini/policies/ship.toml` content.
///
/// Source: https://geminicli.com/docs/reference/policy-engine/
/// Returns `None` when there are no non-default permissions to emit.
pub(super) fn build_gemini_policy_patch(permissions: &Permissions) -> Option<String> {
    let mut entries: Vec<(String, Option<String>, &str)> = Vec::new(); // (tool, pattern, decision)

    // deny → "deny"
    for p in &permissions.tools.deny {
        if let Some((tool, pattern)) = translate_to_gemini_policy(p) {
            entries.push((tool.to_string(), pattern, "deny"));
        }
    }
    // ask → "ask_user"
    for p in &permissions.tools.ask {
        if let Some((tool, pattern)) = translate_to_gemini_policy(p) {
            entries.push((tool.to_string(), pattern, "ask_user"));
        }
    }
    // non-default allow → "allow"
    let non_default_allow = !(permissions.tools.allow.is_empty()
        || permissions.tools.allow.len() == 1 && permissions.tools.allow[0] == "*");
    if non_default_allow {
        for p in &permissions.tools.allow {
            if let Some((tool, pattern)) = translate_to_gemini_policy(p) {
                entries.push((tool.to_string(), pattern, "allow"));
            }
        }
    }

    if entries.is_empty() {
        return None;
    }

    let mut toml = String::new();
    toml.push_str("# Generated by Ship. Do not edit manually — run `ship sync` to regenerate.\n\n");
    for (tool, pattern, decision) in entries {
        toml.push_str("[[tool_policies]]\n");
        toml.push_str(&format!("tool = {:?}\n", tool));
        if let Some(p) = pattern {
            toml.push_str(&format!("pattern = {:?}\n", p));
        }
        toml.push_str(&format!("decision = {:?}\n", decision));
        toml.push('\n');
    }

    Some(toml)
}
