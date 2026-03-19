use std::collections::HashMap;

use serde_json::Value as Json;

use crate::resolve::ResolvedConfig;
use crate::types::{HookConfig, HookTrigger, McpServerConfig, McpServerType, Permissions, Rule};

// ─── Cursor hooks patch ───────────────────────────────────────────────────────

/// Map our internal `HookTrigger` to Cursor's event name(s).
/// Source: https://cursor.com/docs/agent/hooks
/// Cursor splits PreToolUse into beforeMCPExecution / beforeShellExecution;
/// we emit both so hooks fire regardless of tool type.
fn cursor_triggers(t: &HookTrigger) -> &'static [&'static str] {
    match t {
        HookTrigger::PreToolUse  => &["beforeMCPExecution", "beforeShellExecution"],
        HookTrigger::PostToolUse => &["afterMCPExecution", "afterShellExecution"],
        HookTrigger::Stop        => &["sessionEnd"],
        // No Cursor equivalents
        HookTrigger::Notification | HookTrigger::SubagentStop | HookTrigger::PreCompact => &[],
    }
}

/// Build the full `.cursor/hooks.json` content.
/// Returns `None` when there are no mappable hooks.
pub(super) fn build_cursor_hooks_patch(hooks: &[HookConfig]) -> Option<Json> {
    let mut by_event: std::collections::BTreeMap<String, Vec<Json>> =
        std::collections::BTreeMap::new();

    for h in hooks {
        if let Some(raw_event) = &h.cursor_event {
            // Raw event override — emit directly, skip trigger mapping.
            let mut entry = serde_json::json!({ "command": h.command });
            if let Some(m) = &h.matcher {
                entry["matcher"] = Json::String(m.clone());
            }
            by_event.entry(raw_event.clone()).or_default().push(entry);
            continue;
        }

        for &event in cursor_triggers(&h.trigger) {
            let mut entry = serde_json::json!({ "command": h.command });
            if let Some(m) = &h.matcher {
                entry["matcher"] = Json::String(m.clone());
            }
            by_event.entry(event.to_string()).or_default().push(entry);
        }
    }

    if by_event.is_empty() {
        return None;
    }

    let obj: serde_json::Map<String, Json> = by_event
        .into_iter()
        .map(|(k, v)| (k, Json::Array(v)))
        .collect();

    Some(Json::Object(obj))
}

// ─── Cursor permissions ───────────────────────────────────────────────────────

/// Translate a pattern to Cursor's typed permission format.
///
/// Source: https://cursor.com/docs/cli/reference/permissions
/// Cursor uses: `Shell(cmd)`, `Read(glob)`, `Write(glob)`, `WebFetch(domain)`, `Mcp(server:tool)`.
/// These apply to both Cursor IDE agents and the Cursor CLI.
///
/// The bare wildcard `"*"` is intentionally NOT translated to the permissive set
/// `[Shell(*), Read(*), Write(*), WebFetch(*), Mcp(*:*)]`. Cursor's default without
/// a config is interactive/prompt — `"*"` must be an explicit UI choice, not a silent default.
///
/// Returns `None` for patterns with no Cursor equivalent (they are silently dropped).
pub fn translate_to_cursor_permission(pattern: &str) -> Option<String> {
    // Bare wildcard — must not be auto-expanded. The caller handles the permissive preset.
    if pattern == "*" {
        return None;
    }
    // mcp__server__tool → Mcp(server:tool)
    // e.g. mcp__ship__read_notes → Mcp(ship:read_notes)
    //      mcp__*__delete* → Mcp(*:delete*)
    if let Some(rest) = pattern.strip_prefix("mcp__") {
        if let Some(idx) = rest.find("__") {
            let server = &rest[..idx];
            let tool = &rest[idx + 2..];
            return Some(format!("Mcp({server}:{tool})"));
        }
        return None; // malformed — skip
    }

    // Phase 3: Read(glob) and Write(glob)/Edit(glob) pass-through.
    if let Some(inner) = pattern.strip_prefix("Read(").and_then(|s| s.strip_suffix(')')) {
        return Some(format!("Read({inner})"));
    }
    if let Some(inner) = pattern.strip_prefix("Write(").and_then(|s| s.strip_suffix(')')) {
        return Some(format!("Write({inner})"));
    }
    if let Some(inner) = pattern.strip_prefix("Edit(").and_then(|s| s.strip_suffix(')')) {
        return Some(format!("Write({inner})"));
    }

    // Bash(cmd) → Shell(cmd)
    if let Some(inner) = pattern.strip_prefix("Bash(").and_then(|s| s.strip_suffix(')')) {
        return Some(format!("Shell({inner})"));
    }
    if pattern == "Bash" {
        return Some("Shell(*)".to_string());
    }
    // Read/Glob/LS → Read(*)
    if matches!(pattern, "Read" | "Glob" | "LS") {
        return Some("Read(*)".to_string());
    }
    // Write/Edit/MultiEdit → Write(*)
    if matches!(pattern, "Write" | "Edit" | "MultiEdit") {
        return Some("Write(*)".to_string());
    }
    // WebFetch(domain) — pass through; bare → wildcard
    if pattern == "WebFetch" {
        return Some("WebFetch(*)".to_string());
    }
    if pattern.starts_with("WebFetch(") {
        return Some(pattern.to_string());
    }
    // Unknown tool pattern — drop rather than emit garbage
    None
}

/// The explicit permissive allow list for Cursor.
/// Cursor has no implicit allow-all; this must be emitted only when the user
/// has deliberately chosen "permissive" mode (with appropriate warnings in the UI).
pub const CURSOR_PERMISSIVE_ALLOW: &[&str] =
    &["Shell(*)", "Read(*)", "Write(*)", "WebFetch(*)", "Mcp(*:*)"];

/// Build the `.cursor/cli.json` permissions content.
///
/// Returns `None` when there are no non-default permissions to emit.
/// IMPORTANT: `allow = ["*"]` (our internal allow-all) does NOT automatically expand
/// to the permissive Cursor set — that must be an explicit user action in the UI.
pub(super) fn build_cursor_cli_permissions(permissions: &Permissions) -> Option<Json> {
    let non_default_allow = !(permissions.tools.allow.is_empty()
        || permissions.tools.allow.len() == 1 && permissions.tools.allow[0] == "*");

    let allow_patterns: Vec<String> = if non_default_allow {
        permissions.tools.allow
            .iter()
            .filter_map(|p| translate_to_cursor_permission(p))
            .collect()
    } else {
        vec![]
    };

    let deny_patterns: Vec<String> = permissions.tools.deny
        .iter()
        .filter_map(|p| translate_to_cursor_permission(p))
        .collect();

    if allow_patterns.is_empty() && deny_patterns.is_empty() {
        return None;
    }

    let mut perms = serde_json::json!({});
    if !allow_patterns.is_empty() {
        perms["allow"] = serde_json::json!(allow_patterns);
    }
    if !deny_patterns.is_empty() {
        perms["deny"] = serde_json::json!(deny_patterns);
    }

    Some(serde_json::json!({ "version": 1, "permissions": perms }))
}

// ─── Cursor environment.json ──────────────────────────────────────────────────

/// Build the `.cursor/environment.json` content from `cursor_environment`.
/// Returns `None` when not set.
pub(super) fn build_cursor_environment(resolved: &ResolvedConfig) -> Option<Json> {
    resolved.cursor_environment.clone()
}

// ─── Cursor cli.json (settings_extra merge) ───────────────────────────────────

/// Build the full `.cursor/cli.json` by merging permissions and cursor_settings_extra.
pub(super) fn build_cursor_cli_json(resolved: &ResolvedConfig) -> Option<Json> {
    let permissions_json = build_cursor_cli_permissions(&resolved.permissions);
    let has_extra = resolved.cursor_settings_extra.as_ref().is_some_and(|v| !v.is_null());

    if permissions_json.is_none() && !has_extra {
        return None;
    }

    let mut out = permissions_json.unwrap_or_else(|| serde_json::json!({ "version": 1 }));

    // Merge cursor_settings_extra verbatim.
    if let Some(extra) = &resolved.cursor_settings_extra {
        if let Some(obj) = extra.as_object() {
            for (k, v) in obj {
                out[k] = v.clone();
            }
        }
    }

    Some(out)
}

// ─── Cursor MCP servers ───────────────────────────────────────────────────────

/// Build Cursor MCP server entry with Cursor-specific per-server fields.
pub(super) fn build_cursor_server_entry(
    desc: &super::provider::ProviderDescriptor,
    s: &McpServerConfig,
) -> Json {
    let mut entry = super::mcp::server_entry(desc, s);

    // envFile for stdio servers.
    if matches!(s.server_type, McpServerType::Stdio) {
        if let Some(env_file) = &s.cursor_env_file {
            entry["envFile"] = Json::String(env_file.clone());
        }
    }

    entry
}

/// Build the Cursor MCP servers object with Cursor-specific fields.
pub(super) fn build_cursor_mcp_servers(
    desc: &super::provider::ProviderDescriptor,
    servers: &[McpServerConfig],
) -> Json {
    let mut map = serde_json::Map::new();

    // Ship server always first.
    map.insert("ship".to_string(), super::mcp::ship_server_entry(desc.emit_type_field));

    for s in servers {
        if s.disabled {
            continue;
        }
        map.insert(s.id.clone(), build_cursor_server_entry(desc, s));
    }

    Json::Object(map)
}

// ─── Cursor rule files (.cursor/rules/*.mdc) ──────────────────────────────────

/// Build per-file `.cursor/rules/<name>.mdc` entries with YAML frontmatter.
pub(super) fn build_cursor_rule_files(rules: &[Rule]) -> HashMap<String, String> {
    rules
        .iter()
        .filter(|r| !r.content.trim().is_empty())
        .map(|rule| {
            let stem = rule
                .file_name
                .trim_end_matches(".md")
                .trim_end_matches(".mdc");
            let path = format!(".cursor/rules/{}.mdc", stem);
            let content = format_cursor_rule(rule);
            (path, content)
        })
        .collect()
}

fn format_cursor_rule(rule: &Rule) -> String {
    let mut fm = String::new();
    fm.push_str("---\n");
    if let Some(desc) = &rule.description {
        fm.push_str(&format!("description: {:?}\n", desc));
    }
    if !rule.globs.is_empty() {
        fm.push_str("globs:\n");
        for g in &rule.globs {
            fm.push_str(&format!("  - {}\n", g));
        }
    }
    fm.push_str(&format!("alwaysApply: {}\n", rule.always_apply));
    fm.push_str("---\n\n");
    fm.push_str(rule.content.trim());
    fm
}
