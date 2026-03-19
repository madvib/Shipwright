use crate::types::{McpServerConfig, McpServerType};

/// Build the `[mcp_servers.*]` TOML tables for `.codex/config.toml`.
///
/// Source: https://developers.openai.com/codex/mcp
/// Codex uses TOML, not JSON. Each server is a `[mcp_servers.<id>]` table.
/// Returns `None` if there are no enabled servers to write.
pub(super) fn build_codex_config_patch(
    servers: &[McpServerConfig],
    model: Option<&str>,
    sandbox: Option<&str>,
) -> Option<String> {
    let mut mcp = toml::Table::new();

    // Ship server always first
    let mut ship_entry = toml::Table::new();
    ship_entry.insert("command".into(), toml::Value::String("ship-mcp".into()));
    ship_entry.insert("args".into(), toml::Value::Array(vec![]));
    mcp.insert("ship".into(), toml::Value::Table(ship_entry));

    for s in servers {
        if s.disabled {
            continue;
        }
        let mut entry = toml::Table::new();
        match s.server_type {
            McpServerType::Stdio => {
                entry.insert("command".into(), toml::Value::String(s.command.clone()));
                if !s.args.is_empty() {
                    entry.insert(
                        "args".into(),
                        toml::Value::Array(
                            s.args.iter().map(|a| toml::Value::String(a.clone())).collect(),
                        ),
                    );
                }
                if !s.env.is_empty() {
                    let mut env_table = toml::Table::new();
                    for (k, v) in &s.env {
                        env_table.insert(k.clone(), toml::Value::String(v.clone()));
                    }
                    entry.insert("env".into(), toml::Value::Table(env_table));
                }
            }
            McpServerType::Sse | McpServerType::Http => {
                if let Some(url) = &s.url {
                    entry.insert("url".into(), toml::Value::String(url.clone()));
                }
            }
        }
        if let Some(t) = s.timeout_secs {
            entry.insert("startup_timeout_sec".into(), toml::Value::Integer(t as i64));
        }
        mcp.insert(s.id.clone(), toml::Value::Table(entry));
    }

    let mut root = toml::Table::new();
    if let Some(m) = model {
        root.insert("model".into(), toml::Value::String(m.to_string()));
    }
    if let Some(s) = sandbox {
        root.insert("sandbox".into(), toml::Value::String(s.to_string()));
    }
    root.insert("mcp_servers".into(), toml::Value::Table(mcp));
    toml::to_string(&root).ok()
}
