//! Claude MCP server parsing — shared between `.mcp.json` and Claude settings.

use std::collections::HashMap;

use serde_json::Value as Json;

use crate::types::{McpServerConfig, McpServerType};

use super::json_string_array;

/// Parse `.mcp.json` format into MCP server configs.
pub(super) fn parse_mcp_json(json: &Json) -> Vec<McpServerConfig> {
    let servers_obj = json
        .get("mcpServers")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let mut servers = Vec::new();

    for (id, entry) in &servers_obj {
        let entry_obj = match entry.as_object() {
            Some(o) => o,
            None => continue,
        };

        let command = entry_obj
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let args = json_string_array(entry_obj.get("args"));

        let env: HashMap<String, String> = entry_obj
            .get("env")
            .and_then(|v| v.as_object())
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        let url = entry_obj
            .get("url")
            .and_then(|v| v.as_str())
            .map(String::from);

        let server_type = if url.is_some() && command.is_empty() {
            McpServerType::Sse
        } else {
            McpServerType::Stdio
        };

        let disabled = entry_obj
            .get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let timeout_secs = entry_obj
            .get("timeout")
            .and_then(|v| v.as_u64())
            .map(|t| t as u32);

        servers.push(McpServerConfig {
            id: id.clone(),
            name: id.clone(),
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
        });
    }

    servers
}
