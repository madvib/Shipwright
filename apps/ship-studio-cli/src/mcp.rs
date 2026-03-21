//! MCP server management backed by `.ship/agents/mcp.toml`.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::paths::agents_mcp_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEntry {
    pub id: String,
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_type: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub disabled: bool,
}

fn default_scope() -> String { "project".to_string() }

/// On-disk format: `[mcp.servers.<key>]` tables keyed by server id.
#[derive(Debug, Clone, Deserialize, Default)]
struct RawMcpFile {
    #[serde(default)]
    mcp: RawMcpSection,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawMcpSection {
    #[serde(default)]
    servers: HashMap<String, McpEntry>,
}

/// Legacy flat array format: `servers = [{ id = "ship", ... }]`
#[derive(Debug, Clone, Deserialize, Default)]
struct LegacyMcpFile {
    #[serde(default)]
    servers: Vec<McpEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct McpFile {
    pub servers: Vec<McpEntry>,
}

impl McpFile {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() { return Ok(Self::default()); }
        let text = std::fs::read_to_string(path)?;

        if crate::paths::is_jsonc_ext(path) {
            // JSONC format: { "mcp": { "servers": { "<id>": {...} } } }
            if let Ok(raw) = compiler::jsonc::from_jsonc_str::<RawMcpFile>(&text)
                && !raw.mcp.servers.is_empty()
            {
                let servers = raw.mcp.servers.into_iter().map(|(key, mut entry)| {
                    if entry.id.is_empty() { entry.id = key; }
                    entry
                }).collect();
                return Ok(Self { servers });
            }
            if let Ok(legacy) = compiler::jsonc::from_jsonc_str::<LegacyMcpFile>(&text) {
                return Ok(Self { servers: legacy.servers });
            }
            return Ok(Self::default());
        }

        // TOML format
        // Try keyed table format first: [mcp.servers.<key>]
        if let Ok(raw) = toml::from_str::<RawMcpFile>(&text)
            && !raw.mcp.servers.is_empty()
        {
            let servers = raw.mcp.servers.into_iter().map(|(key, mut entry)| {
                if entry.id.is_empty() { entry.id = key; }
                entry
            }).collect();
            return Ok(Self { servers });
        }
        // Fallback: flat array format { servers = [{...}] }
        if let Ok(legacy) = toml::from_str::<LegacyMcpFile>(&text) {
            return Ok(Self { servers: legacy.servers });
        }
        Ok(Self::default())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(p) = path.parent() { std::fs::create_dir_all(p)?; }

        if crate::paths::is_jsonc_ext(path) {
            // Write as JSONC: { "mcp": { "servers": { "<id>": {...} } } }
            let mut servers_map = serde_json::Map::new();
            for s in &self.servers {
                let val = serde_json::to_value(s)?;
                servers_map.insert(s.id.clone(), val);
            }
            let root = serde_json::json!({ "mcp": { "servers": servers_map } });
            std::fs::write(path, serde_json::to_string_pretty(&root)?)?;
            return Ok(());
        }

        // Write back in TOML keyed table format: [mcp.servers.<id>]
        let mut map = HashMap::new();
        for s in &self.servers {
            map.insert(s.id.clone(), s.clone());
        }
        let raw = toml::Value::Table({
            let mut root = toml::map::Map::new();
            let mut mcp = toml::map::Map::new();
            let servers_val = toml::Value::try_from(map)?;
            mcp.insert("servers".into(), servers_val);
            root.insert("mcp".into(), toml::Value::Table(mcp));
            root
        });
        std::fs::write(path, toml::to_string_pretty(&raw)?)?;
        Ok(())
    }
}

pub fn add_http(id: &str, name: Option<String>, url: &str) -> Result<()> {
    let path = agents_mcp_path();
    let mut file = McpFile::load(&path)?;
    if file.servers.iter().any(|s| s.id == id) {
        anyhow::bail!("MCP server '{}' already registered. Remove it first.", id);
    }
    file.servers.push(McpEntry {
        id: id.to_string(), name,
        command: None, args: vec![], env: HashMap::new(),
        url: Some(url.to_string()),
        scope: "project".into(), server_type: Some("http".into()), disabled: false,
    });
    file.save(&path)?;
    println!("✓ registered MCP server '{}'", id);
    Ok(())
}

pub fn add_stdio(id: &str, name: Option<String>, command: &str, args: Vec<String>) -> Result<()> {
    let path = agents_mcp_path();
    let mut file = McpFile::load(&path)?;
    if file.servers.iter().any(|s| s.id == id) {
        anyhow::bail!("MCP server '{}' already registered. Remove it first.", id);
    }
    file.servers.push(McpEntry {
        id: id.to_string(), name,
        command: Some(command.to_string()), args,
        env: HashMap::new(), url: None,
        scope: "project".into(), server_type: Some("stdio".into()), disabled: false,
    });
    file.save(&path)?;
    println!("✓ registered MCP server '{}'", id);
    Ok(())
}

pub fn list() -> Result<()> {
    let path = agents_mcp_path();
    let file = McpFile::load(&path)?;
    if file.servers.is_empty() {
        println!("No MCP servers configured.");
        println!("Add one with: ship mcp add-stdio <id> <command> [args...]");
        return Ok(());
    }
    println!("MCP servers:");
    for s in &file.servers {
        let transport = if s.url.is_some() { "http/sse" } else { "stdio" };
        let status = if s.disabled { " (disabled)" } else { "" };
        let name = s.name.as_deref().unwrap_or(&s.id);
        println!("  {} — {} [{}]{}", s.id, name, transport, status);
    }
    Ok(())
}

pub fn remove(id: &str) -> Result<()> {
    let path = agents_mcp_path();
    let mut file = McpFile::load(&path)?;
    let before = file.servers.len();
    file.servers.retain(|s| s.id != id);
    if file.servers.len() == before {
        anyhow::bail!("MCP server '{}' not found", id);
    }
    file.save(&path)?;
    println!("✓ removed MCP server '{}'", id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn mcp_path(tmp: &TempDir) -> std::path::PathBuf {
        tmp.path().join("mcp.toml")
    }

    #[test]
    fn add_stdio_creates_entry() {
        let tmp = TempDir::new().unwrap();
        let path = mcp_path(&tmp);
        let mut file = McpFile::load(&path).unwrap();
        file.servers.push(McpEntry {
            id: "github".into(), name: None,
            command: Some("npx".into()), args: vec!["-y".into(), "@mcp/github".into()],
            env: HashMap::new(), url: None,
            scope: "project".into(), server_type: Some("stdio".into()), disabled: false,
        });
        file.save(&path).unwrap();

        let back = McpFile::load(&path).unwrap();
        assert_eq!(back.servers.len(), 1);
        assert_eq!(back.servers[0].id, "github");
        assert_eq!(back.servers[0].command.as_deref(), Some("npx"));
    }

    #[test]
    fn mcp_file_round_trips() {
        let tmp = TempDir::new().unwrap();
        let path = mcp_path(&tmp);
        let original = McpFile {
            servers: vec![
                McpEntry {
                    id: "linear".into(), name: Some("Linear".into()),
                    command: Some("npx".into()), args: vec!["-y".into(), "@mcp/linear".into()],
                    env: HashMap::new(), url: None,
                    scope: "project".into(), server_type: Some("stdio".into()), disabled: false,
                },
            ],
        };
        original.save(&path).unwrap();
        let back = McpFile::load(&path).unwrap();
        assert_eq!(back.servers[0].name.as_deref(), Some("Linear"));
    }

    #[test]
    fn load_empty_when_no_file() {
        let tmp = TempDir::new().unwrap();
        let file = McpFile::load(&tmp.path().join("nonexistent.toml")).unwrap();
        assert!(file.servers.is_empty());
    }
}
