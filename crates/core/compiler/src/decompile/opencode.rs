//! OpenCode decompiler — parse `opencode.json` and `AGENTS.md` into a [`ProjectLibrary`].
//!
//! Source: https://opencode.ai/docs/plugins/
//! Config schema: https://opencode.ai/config.json

use std::collections::HashMap;
use std::path::Path;

use serde_json::Value as Json;

use crate::types::{McpServerConfig, McpServerType, Rule};
use crate::ProjectLibrary;

/// Known top-level keys in `opencode.json` that map to structured Ship fields.
const KNOWN_CONFIG_KEYS: &[&str] = &["mcp", "mcpServers", "model", "permission", "$schema"];

/// Parse OpenCode native config files and produce a partial [`ProjectLibrary`].
///
/// Reads (if present):
/// - `opencode.json` → MCP servers, model, provider_defaults
/// - `AGENTS.md` → rules
pub fn decompile_opencode(project_root: &Path) -> ProjectLibrary {
    let mut library = ProjectLibrary::default();

    // ── opencode.json ────────────────────────────────────────────────────────
    let config_path = project_root.join("opencode.json");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(json) = serde_json::from_str::<Json>(&content) {
            parse_opencode_config(&mut library, &json);
        }
    }

    // ── .opencode/config.json (alternate location) ───────────────────────────
    let alt_config = project_root.join(".opencode").join("config.json");
    if library.mcp_servers.is_empty() {
        if let Ok(content) = std::fs::read_to_string(&alt_config) {
            if let Ok(json) = serde_json::from_str::<Json>(&content) {
                parse_opencode_config(&mut library, &json);
            }
        }
    }

    // ── AGENTS.md ────────────────────────────────────────────────────────────
    let agents_md = project_root.join("AGENTS.md");
    if let Ok(content) = std::fs::read_to_string(&agents_md) {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            library.rules.push(Rule {
                file_name: "AGENTS.md".to_string(),
                content: trimmed.to_string(),
                always_apply: true,
                globs: vec![],
                description: None,
            });
        }
    }

    library
}

fn parse_opencode_config(library: &mut ProjectLibrary, config: &Json) {
    let obj = match config.as_object() {
        Some(o) => o,
        None => return,
    };

    // ── Model ────────────────────────────────────────────────────────────────
    if let Some(Json::String(m)) = obj.get("model") {
        library.model = Some(m.clone());
    }

    // ── mcp (OpenCode uses "mcp", not "mcpServers") ─────────────────────────
    let mcp_obj = obj
        .get("mcp")
        .or_else(|| obj.get("mcpServers")) // fallback for legacy configs
        .and_then(|v| v.as_object());
    if let Some(servers) = mcp_obj {
        for (id, entry) in servers {
            if let Some(server) = parse_opencode_mcp_server(id, entry) {
                library.mcp_servers.push(server);
            }
        }
    }

    // ── Provider defaults — everything we don't recognize ────────────────────
    let mut extra = serde_json::Map::new();
    for (k, v) in obj {
        if !KNOWN_CONFIG_KEYS.contains(&k.as_str()) {
            extra.insert(k.clone(), v.clone());
        }
    }
    if !extra.is_empty() {
        library
            .provider_defaults
            .insert("opencode".to_string(), Json::Object(extra));
    }
}

/// Parse an OpenCode MCP server entry.
///
/// OpenCode format:
///   local:  { "type": "local", "command": ["cmd", "arg1"], "environment": {...} }
///   remote: { "type": "remote", "url": "https://..." }
fn parse_opencode_mcp_server(id: &str, entry: &Json) -> Option<McpServerConfig> {
    let obj = entry.as_object()?;

    // OpenCode uses "type": "local"/"remote" to distinguish transport.
    let type_str = obj.get("type").and_then(|v| v.as_str()).unwrap_or("local");
    let is_remote = type_str == "remote";

    // Local: command is an array ["cmd", "arg1", ...].
    let (command, args) = if !is_remote {
        match obj.get("command").and_then(|v| v.as_array()) {
            Some(arr) => {
                let strs: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                let cmd = strs.first().cloned().unwrap_or_default();
                let rest = strs.into_iter().skip(1).collect();
                (cmd, rest)
            }
            // Fallback: command as string (legacy/generic format)
            None => {
                let cmd = obj
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let args = json_string_array(obj.get("args"));
                (cmd, args)
            }
        }
    } else {
        (String::new(), vec![])
    };

    let url = obj.get("url").and_then(|v| v.as_str()).map(String::from);

    let server_type = if is_remote {
        McpServerType::Sse
    } else {
        McpServerType::Stdio
    };

    // OpenCode uses "environment" (not "env") for local servers.
    let env: HashMap<String, String> = obj
        .get("environment")
        .or_else(|| obj.get("env")) // fallback for generic format
        .and_then(|v| v.as_object())
        .map(|o| {
            o.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    // OpenCode timeout is in milliseconds.
    let timeout_secs = obj
        .get("timeout")
        .and_then(|v| v.as_u64())
        .map(|ms| (ms / 1000) as u32);

    let disabled = obj
        .get("enabled")
        .and_then(|v| v.as_bool())
        .map(|e| !e)
        .unwrap_or(false);

    Some(McpServerConfig {
        id: id.to_string(),
        name: id.to_string(),
        command,
        args,
        env,
        scope: "project".to_string(),
        server_type,
        url,
        disabled,
        timeout_secs,
        codex_enabled_tools: vec![],
        codex_disabled_tools: vec![],
        gemini_trust: None,
        gemini_include_tools: vec![],
        gemini_exclude_tools: vec![],
        gemini_timeout_ms: None,
        cursor_env_file: None,
    })
}

fn json_string_array(val: Option<&Json>) -> Vec<String> {
    val.and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}
