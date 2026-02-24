+++
id = "bc6b6c3f-d3a2-4ef8-af15-6d44a2aa011d"
title = "MCP server registry — centralized with mode and project scoping"
created = "2026-02-24T04:10:05.473302747Z"
updated = "2026-02-24T04:10:05.473303747Z"
tags = []
links = []
+++

## What
Implement the MCP server registry in logic crate — CRUD operations over the `[[mcp-servers]]` entries in `.ship/config.toml`. Each server entry defines which modes it's active in. This powers the config export that writes provider-specific MCP configs.

## Data Model (from config schema v2)

```rust
#[derive(Serialize, Deserialize, Clone, Debug, Type)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Mode IDs this server is active in, or ["all"] for all modes
    #[serde(default = "default_all_modes")]
    pub modes: Vec<String>,
}
```

## Logic functions

```rust
// logic/src/mcp_registry.rs
pub fn list_mcp_servers(project_dir: PathBuf) -> Result<Vec<McpServerConfig>>
pub fn add_mcp_server(project_dir: PathBuf, server: McpServerConfig) -> Result<()>
pub fn update_mcp_server(project_dir: PathBuf, id: &str, server: McpServerConfig) -> Result<()>
pub fn remove_mcp_server(project_dir: PathBuf, id: &str) -> Result<()>

/// Returns only servers active in the given mode
pub fn active_servers_for_mode(project_dir: PathBuf, mode_id: &str) -> Result<Vec<McpServerConfig>>
```

## MCP tools to add

- `list_mcp_servers` — list all registered servers
- `add_mcp_server` — register a new server
- `remove_mcp_server` — remove by id

## Tauri commands to add

- `list_mcp_servers_cmd`
- `add_mcp_server_cmd`
- `remove_mcp_server_cmd`

## Notes
- Ship's own MCP server is always bootstrapped in the registry on `ship init` with `modes = ["all"]`
- No validation of whether the server command actually exists — that's the user's problem
- `env` values support `$VAR` substitution at export time, not at storage time