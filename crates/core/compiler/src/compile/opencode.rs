use serde_json::Value as Json;

use crate::resolve::ResolvedConfig;
use crate::types::{McpServerType, Permissions};

// ── Permission translation ──────────────────────────────────────────────────

/// Translate a Ship tool pattern to an OpenCode permission key + optional glob.
///
/// Source: https://opencode.ai/docs/permissions
/// OpenCode keys: read, edit, glob, grep, list, bash, task, skill, lsp,
///   webfetch, websearch, codesearch, external_directory, doom_loop.
/// Values: "allow", "ask", "deny", or object with glob→action mappings.
fn translate_to_opencode_permission(pattern: &str) -> Option<(&'static str, Option<String>)> {
    if pattern == "*" {
        return None; // wildcard — skip, handled at top level
    }
    // Bash(cmd) → bash tool with cmd as glob pattern
    if let Some(inner) = pattern
        .strip_prefix("Bash(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return Some(("bash", Some(inner.to_string())));
    }
    if pattern == "Bash" {
        return Some(("bash", None));
    }
    // Read/Glob → read
    if matches!(pattern, "Read" | "Glob" | "LS") {
        return Some(("read", None));
    }
    if let Some(inner) = pattern
        .strip_prefix("Read(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return Some(("read", Some(inner.to_string())));
    }
    // Write/Edit → edit
    if matches!(pattern, "Write" | "Edit" | "MultiEdit") {
        return Some(("edit", None));
    }
    if let Some(inner) = pattern
        .strip_prefix("Write(")
        .and_then(|s| s.strip_suffix(')'))
        .or_else(|| {
            pattern
                .strip_prefix("Edit(")
                .and_then(|s| s.strip_suffix(')'))
        })
    {
        return Some(("edit", Some(inner.to_string())));
    }
    // Grep → grep
    if pattern == "Grep" {
        return Some(("grep", None));
    }
    // WebFetch → webfetch
    if pattern == "WebFetch" {
        return Some(("webfetch", None));
    }
    if let Some(inner) = pattern
        .strip_prefix("WebFetch(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return Some(("webfetch", Some(inner.to_string())));
    }
    // WebSearch → websearch
    if pattern == "WebSearch" {
        return Some(("websearch", None));
    }
    // mcp__server__tool — no direct OpenCode equivalent, skip
    if pattern.starts_with("mcp__") {
        return None;
    }
    None
}

/// Build the OpenCode `permission` object from Ship permissions.
///
/// Source: https://opencode.ai/docs/permissions
/// OpenCode format:
///   { "bash": "allow" }                     — simple action
///   { "bash": { "*": "ask", "git *": "allow" } } — granular with globs
fn build_opencode_permissions(permissions: &Permissions) -> Option<Json> {
    let mut perms: serde_json::Map<String, Json> = serde_json::Map::new();

    // Collect per-key entries: key → Vec<(Option<glob>, action)>
    let mut entries: std::collections::BTreeMap<&str, Vec<(Option<String>, &str)>> =
        std::collections::BTreeMap::new();

    // deny takes precedence
    for p in &permissions.tools.deny {
        if let Some((key, glob)) = translate_to_opencode_permission(p) {
            entries.entry(key).or_default().push((glob, "deny"));
        }
    }
    // ask
    for p in &permissions.tools.ask {
        if let Some((key, glob)) = translate_to_opencode_permission(p) {
            entries.entry(key).or_default().push((glob, "ask"));
        }
    }
    // allow (skip default wildcard)
    let non_default_allow = !(permissions.tools.allow.is_empty()
        || (permissions.tools.allow.len() == 1 && permissions.tools.allow[0] == "*"));
    if non_default_allow {
        for p in &permissions.tools.allow {
            if let Some((key, glob)) = translate_to_opencode_permission(p) {
                entries.entry(key).or_default().push((glob, "allow"));
            }
        }
    }

    if entries.is_empty() {
        return None;
    }

    for (key, rules) in entries {
        if rules.len() == 1 && rules[0].0.is_none() {
            // Simple: { "bash": "allow" }
            perms.insert(key.to_string(), Json::String(rules[0].1.to_string()));
        } else {
            // Granular: { "bash": { "*": "ask", "git *": "allow" } }
            let mut obj = serde_json::Map::new();
            for (glob, action) in rules {
                let glob_key = glob.unwrap_or_else(|| "*".to_string());
                obj.insert(glob_key, Json::String(action.to_string()));
            }
            perms.insert(key.to_string(), Json::Object(obj));
        }
    }

    Some(Json::Object(perms))
}

// ── MCP server entries ──────────────────────────────────────────────────────

/// Build the OpenCode `mcp` object.
///
/// Source: https://opencode.ai/docs/mcp-servers
/// OpenCode format:
///   local:  { "type": "local", "command": ["cmd", "arg1"], "environment": {...} }
///   remote: { "type": "remote", "url": "https://..." }
fn build_opencode_mcp(resolved: &ResolvedConfig) -> Json {
    let mut mcp = serde_json::Map::new();

    // Ship server always first.
    mcp.insert(
        "ship".to_string(),
        serde_json::json!({
            "type": "local",
            "command": ["ship", "mcp", "serve"]
        }),
    );

    for s in &resolved.mcp_servers {
        if s.disabled || s.id == "ship" {
            continue;
        }
        let entry = match s.server_type {
            McpServerType::Stdio => {
                let mut cmd = vec![Json::String(s.command.clone())];
                cmd.extend(s.args.iter().map(|a| Json::String(a.clone())));
                let mut e = serde_json::json!({
                    "type": "local",
                    "command": cmd,
                });
                if !s.env.is_empty() {
                    let env: serde_json::Map<String, Json> = s
                        .env
                        .iter()
                        .map(|(k, v)| (k.clone(), Json::String(v.clone())))
                        .collect();
                    e["environment"] = Json::Object(env);
                }
                if let Some(t) = s.timeout_secs {
                    e["timeout"] = serde_json::json!(t * 1000); // seconds → ms
                }
                e
            }
            McpServerType::Sse | McpServerType::Http => {
                let mut e = serde_json::json!({ "type": "remote" });
                if let Some(url) = &s.url {
                    e["url"] = Json::String(url.clone());
                }
                if let Some(t) = s.timeout_secs {
                    e["timeout"] = serde_json::json!(t * 1000);
                }
                e
            }
        };
        mcp.insert(s.id.clone(), entry);
    }

    Json::Object(mcp)
}

// ── Main build function ─────────────────────────────────────────────────────

/// Build the `opencode.json` config content.
///
/// Source: https://opencode.ai/docs/config/
/// Schema: https://opencode.ai/config.json
///
/// OpenCode uses a single `opencode.json` at the project root.
/// Ship manages: `model`, `mcp` (servers), `permission`.
/// Everything else passes through via `opencode_settings_extra`.
///
/// Always returns `Some` — the mcp block is unconditionally populated with at least the ship server.
pub(super) fn build_opencode_config_patch(resolved: &ResolvedConfig) -> Option<Json> {
    let model = resolved.model.as_deref();
    let has_extra = resolved
        .opencode_settings_extra
        .as_ref()
        .is_some_and(|v| !v.is_null());

    let permissions_patch = build_opencode_permissions(&resolved.permissions);

    // build_opencode_mcp always emits at least the ship server, so there is
    // always content to write. No early-exit guard is needed.

    let mut root = serde_json::json!({});

    // Model.
    if let Some(m) = model {
        root["model"] = Json::String(m.to_string());
    }

    // MCP servers.
    root["mcp"] = build_opencode_mcp(resolved);

    // Permissions.
    if let Some(perms) = permissions_patch {
        root["permission"] = perms;
    }

    // settings_extra — merged verbatim last (provider_settings.opencode passthrough).
    if let Some(extra) = &resolved.opencode_settings_extra
        && let Some(obj) = extra.as_object()
    {
        for (k, v) in obj {
            root[k] = v.clone();
        }
    }

    Some(root)
}
