use serde_json::Value as Json;

use crate::types::{HookConfig, HookTrigger, Permissions};

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

/// Build the settings patch for `.gemini/settings.json`.
/// Includes `hooks` and `model` when present.
/// Returns `None` when there is nothing to emit.
pub(super) fn build_gemini_settings_patch(
    hooks: &[HookConfig],
    model: Option<&str>,
) -> Option<Json> {
    let mut by_trigger: std::collections::BTreeMap<&str, Vec<Json>> =
        std::collections::BTreeMap::new();

    for h in hooks {
        let Some(event) = gemini_trigger(&h.trigger) else { continue };
        let mut hook_obj = serde_json::json!({
            "type": "command",
            "command": h.command,
        });
        if let Some(m) = &h.matcher {
            hook_obj["matcher"] = Json::String(m.clone());
        }
        // Gemini schema: { "matcher": "...", "hooks": [{ "type": "command", "command": "..." }] }
        let entry_with_matcher = if let Some(m) = &h.matcher {
            serde_json::json!({ "matcher": m, "hooks": [hook_obj] })
        } else {
            serde_json::json!({ "hooks": [hook_obj] })
        };
        by_trigger.entry(event).or_default().push(entry_with_matcher);
    }

    let has_hooks = !by_trigger.is_empty();
    let has_model = model.is_some();

    if !has_hooks && !has_model {
        return None;
    }

    let mut patch = serde_json::json!({});

    if has_hooks {
        let hooks_obj: serde_json::Map<String, Json> = by_trigger
            .into_iter()
            .map(|(k, v)| (k.to_string(), Json::Array(v)))
            .collect();
        patch["hooks"] = Json::Object(hooks_obj);
    }

    if let Some(m) = model {
        patch["model"] = Json::String(m.to_string());
    }

    Some(patch)
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
