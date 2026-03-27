use anyhow::Result;
use crate::fs_util::write_atomic;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use super::project::{McpConfig, McpSection, McpServerConfig};

pub fn get_mcp_config(ship_dir: &Path) -> Result<Vec<McpServerConfig>> {
    let path = crate::project::mcp_config_path(ship_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)?;
    let raw: McpConfig = compiler::jsonc::from_jsonc_str(&content)?;

    let mut servers = Vec::new();
    for (id, mut server) in raw.mcp.servers {
        server.id = id;
        servers.push(server);
    }
    servers.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(servers)
}

pub(super) fn save_mcp_config(ship_dir: &Path, servers: &[McpServerConfig]) -> Result<()> {
    let path = crate::project::mcp_config_path(ship_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut by_id: HashMap<String, McpServerConfig> = HashMap::new();
    for server in servers {
        let mut cloned = server.clone();
        cloned.id.clear();
        by_id.insert(server.id.clone(), cloned);
    }

    let raw = McpConfig {
        mcp: McpSection { servers: by_id },
    };
    write_atomic(&path, serde_json::to_string_pretty(&raw)?)?;
    Ok(())
}
