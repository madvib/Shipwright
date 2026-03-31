use serde_json::Value as Json;

use crate::types::{McpServerConfig, McpServerType};

use super::provider::ProviderDescriptor;

pub(super) fn build_mcp_servers(
    desc: &ProviderDescriptor,
    servers: &[McpServerConfig],
    studio_url: Option<&str>,
) -> Json {
    let mut map = serde_json::Map::new();

    // Ship's own self-hosted MCP server is always injected first.
    // When Studio integration is active, emit HTTP transport instead of stdio.
    let ship_entry = match studio_url {
        Some(url) => ship_server_entry_http(url, desc.emit_type_field),
        None => ship_server_entry(desc.emit_type_field),
    };
    map.insert("ship".to_string(), ship_entry);

    for s in servers {
        if s.disabled || s.id == "ship" {
            continue;
        }
        map.insert(s.id.clone(), server_entry(desc, s));
    }

    Json::Object(map)
}

pub(super) fn server_entry(desc: &ProviderDescriptor, s: &McpServerConfig) -> Json {
    let mut entry = serde_json::json!({});
    match s.server_type {
        McpServerType::Stdio => {
            entry["command"] = Json::String(s.command.clone());
            if !s.args.is_empty() {
                entry["args"] = serde_json::json!(s.args);
            }
            if !s.env.is_empty() {
                entry["env"] = serde_json::json!(s.env);
            }
            if desc.emit_type_field {
                entry["type"] = Json::String("stdio".to_string());
            }
        }
        McpServerType::Sse => {
            if let Some(url) = &s.url {
                entry[desc.sse_url_field] = Json::String(url.clone());
            }
            if desc.emit_type_field {
                entry["type"] = Json::String("sse".to_string());
            }
        }
        McpServerType::Http => {
            if let Some(url) = &s.url {
                entry[desc.http_url_field] = Json::String(url.clone());
            }
            if desc.emit_type_field {
                entry["type"] = Json::String("http".to_string());
            }
        }
    }
    if let Some(t) = s.timeout_secs {
        entry["startup_timeout_sec"] = serde_json::json!(t);
    }
    entry
}

pub(super) fn ship_server_entry(emit_type: bool) -> Json {
    let ship_global_dir = std::env::var("HOME")
        .map(|h| format!("{}/.ship", h))
        .unwrap_or_else(|_| String::from("~/.ship"));
    let mut e = serde_json::json!({
        "command": "ship",
        "args": ["mcp", "serve"],
        "env": {
            "SHIP_GLOBAL_DIR": ship_global_dir
        }
    });
    if emit_type {
        e["type"] = Json::String("stdio".to_string());
    }
    e
}

/// Emit an HTTP streamable-http transport entry pointing at Studio's agent endpoint.
pub(super) fn ship_server_entry_http(url: &str, emit_type: bool) -> Json {
    let mut e = serde_json::json!({ "url": url });
    if emit_type {
        e["type"] = Json::String("http".to_string());
    }
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ship_server_entry_is_stdio() {
        let entry = ship_server_entry(false);
        assert_eq!(entry["command"], "ship");
        assert_eq!(entry["args"][0], "mcp");
        assert_eq!(entry["args"][1], "serve");
        assert!(entry.get("url").is_none());
    }

    #[test]
    fn ship_server_entry_http_has_url_no_command() {
        let entry = ship_server_entry_http("http://localhost:51741/agent", false);
        assert_eq!(entry["url"], "http://localhost:51741/agent");
        assert!(entry.get("command").is_none());
        assert!(entry.get("args").is_none());
    }

    #[test]
    fn build_mcp_servers_no_studio_uses_stdio() {
        use crate::compile::provider::get_provider;
        let desc = get_provider("claude").unwrap();
        let result = build_mcp_servers(desc, &[], None);
        let ship = &result["ship"];
        assert_eq!(ship["command"], "ship");
    }

    #[test]
    fn build_mcp_servers_with_studio_uses_http() {
        use crate::compile::provider::get_provider;
        let desc = get_provider("claude").unwrap();
        let result = build_mcp_servers(desc, &[], Some("http://localhost:51741/agent"));
        let ship = &result["ship"];
        assert_eq!(ship["url"], "http://localhost:51741/agent");
        assert!(ship.get("command").is_none());
    }
}
