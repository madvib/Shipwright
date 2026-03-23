//! Gemini CLI decompiler — parse `.gemini/settings.json`, `GEMINI.md`, policies into a
//! [`ProjectLibrary`].

use std::collections::HashMap;
use std::path::Path;

use serde_json::Value as Json;

use crate::types::{McpServerConfig, McpServerType, Rule};
use crate::ProjectLibrary;

use super::gemini_policies::parse_gemini_policies;
use super::json_string_array;

/// Known top-level keys in `.gemini/settings.json` that map to structured Ship fields.
const KNOWN_SETTINGS_KEYS: &[&str] = &[
    "model",
    "general",
    "security",
    "tools",
    "hooks",
    "mcpServers",
];

/// Parse Gemini CLI native config files and produce a partial [`ProjectLibrary`].
///
/// Reads (if present):
/// - `.gemini/settings.json` → model, hooks, MCP servers, gemini settings, provider_defaults
/// - `GEMINI.md` → rules
/// - `.gemini/policies/*.toml` → permissions
pub fn decompile_gemini(project_root: &Path) -> ProjectLibrary {
    let mut library = ProjectLibrary::default();

    // ── .gemini/settings.json ────────────────────────────────────────────────
    let settings_path = project_root.join(".gemini").join("settings.json");
    if let Ok(content) = std::fs::read_to_string(&settings_path)
        && let Ok(json) = serde_json::from_str::<Json>(&content)
    {
        parse_gemini_settings(&mut library, &json);
    }

    // ── GEMINI.md ────────────────────────────────────────────────────────────
    let gemini_md = project_root.join("GEMINI.md");
    if let Ok(content) = std::fs::read_to_string(&gemini_md) {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            library.rules.push(Rule {
                file_name: "GEMINI.md".to_string(),
                content: trimmed.to_string(),
                always_apply: true,
                globs: vec![],
                description: None,
            });
        }
    }

    // ── .gemini/policies/*.toml → permissions ────────────────────────────────
    let policies_dir = project_root.join(".gemini").join("policies");
    if policies_dir.is_dir() {
        library.permissions = parse_gemini_policies(&policies_dir);
    }

    library
}

fn parse_gemini_settings(library: &mut ProjectLibrary, settings: &Json) {
    let obj = match settings.as_object() {
        Some(o) => o,
        None => return,
    };

    // ── model.name ───────────────────────────────────────────────────────────
    if let Some(name) = obj
        .get("model")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
    {
        library.model = Some(name.to_string());
    }

    // ── general.* ────────────────────────────────────────────────────────────
    if let Some(general) = obj.get("general").and_then(|v| v.as_object()) {
        if let Some(mode) = general.get("defaultApprovalMode").and_then(|v| v.as_str()) {
            library.gemini_default_approval_mode = Some(reverse_approval_mode(mode));
        }
        if let Some(turns) = general.get("maxSessionTurns").and_then(|v| v.as_u64()) {
            library.gemini_max_session_turns = Some(turns as u32);
        }
    }

    // ── security.* ───────────────────────────────────────────────────────────
    if let Some(security) = obj.get("security").and_then(|v| v.as_object()) {
        if let Some(v) = security.get("disableYoloMode").and_then(|v| v.as_bool()) {
            library.gemini_disable_yolo_mode = Some(v);
        }
        if let Some(v) = security.get("disableAlwaysAllow").and_then(|v| v.as_bool()) {
            library.gemini_disable_always_allow = Some(v);
        }
    }

    // ── tools.sandbox ────────────────────────────────────────────────────────
    if let Some(sandbox) = obj
        .get("tools")
        .and_then(|v| v.get("sandbox"))
        .and_then(|v| v.as_str())
    {
        library.gemini_tools_sandbox = Some(sandbox.to_string());
    }

    // ── hooks ────────────────────────────────────────────────────────────────
    // Gemini hooks go into provider_defaults as-is (provider-native format for v0.1).
    // We don't reverse-map them to Ship HookConfig since hooks are provider-native.

    // ── mcpServers ───────────────────────────────────────────────────────────
    if let Some(servers) = obj.get("mcpServers").and_then(|v| v.as_object()) {
        for (id, entry) in servers {
            if let Some(server) = parse_gemini_mcp_server(id, entry) {
                library.mcp_servers.push(server);
            }
        }
    }

    // ── Provider defaults — everything we don't recognize ────────────────────
    let mut extra = serde_json::Map::new();
    for (k, v) in obj {
        if !KNOWN_SETTINGS_KEYS.contains(&k.as_str()) {
            extra.insert(k.clone(), v.clone());
        }
    }
    // Also preserve hooks in provider_defaults (provider-native)
    if let Some(hooks) = obj.get("hooks") {
        extra.insert("hooks".to_string(), hooks.clone());
    }
    if !extra.is_empty() {
        library
            .provider_defaults
            .insert("gemini".to_string(), Json::Object(extra));
    }
}

fn parse_gemini_mcp_server(id: &str, entry: &Json) -> Option<McpServerConfig> {
    let obj = entry.as_object()?;

    let command = obj
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let url = obj.get("url").and_then(|v| v.as_str()).map(String::from);
    let http_url = obj
        .get("httpUrl")
        .and_then(|v| v.as_str())
        .map(String::from);

    let server_type = if http_url.is_some() {
        McpServerType::Http
    } else if url.is_some() && command.is_empty() {
        McpServerType::Sse
    } else {
        McpServerType::Stdio
    };

    let effective_url = http_url.or(url);

    let args = json_string_array(obj.get("args"));

    let env: HashMap<String, String> = obj
        .get("env")
        .and_then(|v| v.as_object())
        .map(|o| {
            o.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    let trust = obj.get("trust").and_then(|v| v.as_bool());
    let include_tools = json_string_array(obj.get("includeTools"));
    let exclude_tools = json_string_array(obj.get("excludeTools"));
    let timeout_ms = obj.get("timeout").and_then(|v| v.as_u64()).map(|t| t as u32);

    Some(McpServerConfig {
        id: id.to_string(),
        name: id.to_string(),
        command,
        args,
        env,
        scope: "project".to_string(),
        server_type,
        url: effective_url,
        disabled: false,
        timeout_secs: None,
        codex_enabled_tools: vec![],
        codex_disabled_tools: vec![],
        gemini_trust: trust,
        gemini_include_tools: include_tools,
        gemini_exclude_tools: exclude_tools,
        gemini_timeout_ms: timeout_ms,
        cursor_env_file: None,
    })
}

// ── Reverse translations ─────────────────────────────────────────────────────

fn reverse_approval_mode(val: &str) -> String {
    match val {
        "suggest" => "default",
        "auto-edit" => "auto_edit",
        "yolo" => "plan",
        other => other,
    }
    .to_string()
}
