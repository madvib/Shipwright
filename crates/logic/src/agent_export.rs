use crate::config::{get_config, McpServerConfig};
use anyhow::{Result, anyhow};
use std::fs;
use std::path::PathBuf;

/// Export the project's MCP server registry to the specified AI client's config.
/// Merges with any existing entries — does not clobber other settings.
pub fn export_to(project_dir: PathBuf, target: &str) -> Result<()> {
    let config = get_config(Some(project_dir))?;
    let servers = config.mcp_servers;
    match target {
        "claude" => export_claude(&servers),
        "codex" => export_codex(&servers),
        "gemini" => export_gemini(&servers),
        other => Err(anyhow!(
            "Unknown target '{}': use claude, codex, or gemini",
            other
        )),
    }
}

fn home() -> Result<std::path::PathBuf> {
    home::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))
}

fn mcp_entry_json(s: &McpServerConfig) -> serde_json::Value {
    let mut entry = serde_json::json!({ "command": s.command });
    if !s.args.is_empty() {
        entry["args"] = serde_json::Value::Array(
            s.args.iter().map(|a| serde_json::Value::String(a.clone())).collect(),
        );
    }
    if !s.env.is_empty() {
        let env: serde_json::Map<String, serde_json::Value> = s
            .env
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();
        entry["env"] = serde_json::Value::Object(env);
    }
    entry
}

/// ~/.claude.json → { "mcpServers": { "<id>": { "command": "...", "args": [...] } } }
fn export_claude(servers: &[McpServerConfig]) -> Result<()> {
    let path = home()?.join(".claude.json");
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(&path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let mcp = root
        .as_object_mut()
        .ok_or_else(|| anyhow!("~/.claude.json is not a JSON object"))?
        .entry("mcpServers")
        .or_insert(serde_json::json!({}));

    let mcp_obj = mcp
        .as_object_mut()
        .ok_or_else(|| anyhow!("mcpServers is not an object"))?;

    for s in servers {
        mcp_obj.insert(s.id.clone(), mcp_entry_json(s));
    }

    crate::fs_util::write_atomic(&path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

/// ~/.gemini/settings.json → same shape as Claude
fn export_gemini(servers: &[McpServerConfig]) -> Result<()> {
    let path = home()?.join(".gemini/settings.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(&path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let mcp = root
        .as_object_mut()
        .ok_or_else(|| anyhow!("~/.gemini/settings.json is not a JSON object"))?
        .entry("mcpServers")
        .or_insert(serde_json::json!({}));

    let mcp_obj = mcp
        .as_object_mut()
        .ok_or_else(|| anyhow!("mcpServers is not an object"))?;

    for s in servers {
        mcp_obj.insert(s.id.clone(), mcp_entry_json(s));
    }

    crate::fs_util::write_atomic(&path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

/// ~/.codex/config.toml → [mcp-servers.<id>] tables
fn export_codex(servers: &[McpServerConfig]) -> Result<()> {
    let path = home()?.join(".codex/config.toml");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut doc: toml::Value = if path.exists() {
        toml::from_str(&fs::read_to_string(&path)?)
            .unwrap_or(toml::Value::Table(Default::default()))
    } else {
        toml::Value::Table(Default::default())
    };

    let root = match &mut doc {
        toml::Value::Table(t) => t,
        _ => return Err(anyhow!("~/.codex/config.toml root is not a table")),
    };

    let mcp_servers = root
        .entry("mcp-servers".to_string())
        .or_insert(toml::Value::Table(Default::default()));

    let mcp_table = match mcp_servers {
        toml::Value::Table(t) => t,
        _ => return Err(anyhow!("[mcp-servers] is not a table")),
    };

    for s in servers {
        let mut entry = toml::value::Table::new();
        entry.insert("command".to_string(), toml::Value::String(s.command.clone()));
        if !s.args.is_empty() {
            entry.insert(
                "args".to_string(),
                toml::Value::Array(
                    s.args.iter().map(|a| toml::Value::String(a.clone())).collect(),
                ),
            );
        }
        mcp_table.insert(s.id.clone(), toml::Value::Table(entry));
    }

    crate::fs_util::write_atomic(&path, toml::to_string_pretty(&doc)?)?;
    Ok(())
}
