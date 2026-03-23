//! MCP server serialization for `ship convert` output.

/// Serialize MCP server configs to a JSON map for `.ship/mcp.jsonc`.
pub(crate) fn serialize_mcp_servers(
    servers: &[compiler::McpServerConfig],
) -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    for s in servers {
        let mut entry = serde_json::json!({});
        if !s.command.is_empty() {
            entry["command"] = serde_json::json!(s.command);
        }
        if !s.args.is_empty() {
            entry["args"] = serde_json::json!(s.args);
        }
        if !s.env.is_empty() {
            entry["env"] = serde_json::json!(s.env);
        }
        if let Some(url) = &s.url {
            entry["url"] = serde_json::json!(url);
        }
        match s.server_type {
            compiler::McpServerType::Sse => {
                entry["server_type"] = serde_json::json!("sse");
            }
            compiler::McpServerType::Http => {
                entry["server_type"] = serde_json::json!("http");
            }
            _ => {}
        }
        if s.disabled {
            entry["disabled"] = serde_json::json!(true);
        }
        if let Some(t) = s.timeout_secs {
            entry["timeout_secs"] = serde_json::json!(t);
        }
        if !s.codex_enabled_tools.is_empty() {
            entry["codex_enabled_tools"] = serde_json::json!(s.codex_enabled_tools);
        }
        if !s.codex_disabled_tools.is_empty() {
            entry["codex_disabled_tools"] = serde_json::json!(s.codex_disabled_tools);
        }
        if let Some(trust) = s.gemini_trust {
            entry["gemini_trust"] = serde_json::json!(trust);
        }
        if !s.gemini_include_tools.is_empty() {
            entry["gemini_include_tools"] = serde_json::json!(s.gemini_include_tools);
        }
        if !s.gemini_exclude_tools.is_empty() {
            entry["gemini_exclude_tools"] = serde_json::json!(s.gemini_exclude_tools);
        }
        if let Some(ms) = s.gemini_timeout_ms {
            entry["gemini_timeout_ms"] = serde_json::json!(ms);
        }
        if let Some(f) = &s.cursor_env_file {
            entry["cursor_env_file"] = serde_json::json!(f);
        }
        map.insert(s.id.clone(), entry);
    }
    map
}
