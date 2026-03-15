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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpFile {
    #[serde(default)]
    pub servers: Vec<McpEntry>,
}

impl McpFile {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() { return Ok(Self::default()); }
        Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(p) = path.parent() { std::fs::create_dir_all(p)?; }
        std::fs::write(path, toml::to_string_pretty(self)?)?;
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
