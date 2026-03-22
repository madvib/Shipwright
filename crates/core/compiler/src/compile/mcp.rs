use serde_json::Value as Json;

use crate::types::{McpServerConfig, McpServerType};

use super::provider::ProviderDescriptor;

pub(super) fn build_mcp_servers(desc: &ProviderDescriptor, servers: &[McpServerConfig]) -> Json {
    let mut map = serde_json::Map::new();

    // Ship's own self-hosted MCP server is always injected first.
    map.insert("ship".to_string(), ship_server_entry(desc.emit_type_field));

    for s in servers {
        if s.disabled {
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
    let mut e = serde_json::json!({
        "command": "ship",
        "args": ["mcp", "serve"]
    });
    if emit_type {
        e["type"] = Json::String("stdio".to_string());
    }
    e
}
