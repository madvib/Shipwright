use anyhow::Result;
use crate::fs_util::write_atomic;
use std::collections::HashSet;
use std::path::Path;
use super::project::{AgentProfile, McpServerConfig, ProjectCoreFile, ProjectConfig};
use super::types::HookConfig;

pub(super) fn merge_string_lists(base: &[String], overlay: &[String]) -> Vec<String> {
    let mut merged = Vec::new();
    let mut seen = HashSet::new();

    for item in base.iter().chain(overlay.iter()) {
        if seen.insert(item.clone()) {
            merged.push(item.clone());
        }
    }

    merged
}

pub(super) fn merge_modes(base: &[AgentProfile], overlay: &[AgentProfile]) -> Vec<AgentProfile> {
    let mut merged = base.to_vec();
    for mode in overlay {
        if let Some(existing) = merged.iter_mut().find(|m| m.id == mode.id) {
            *existing = mode.clone();
        } else {
            merged.push(mode.clone());
        }
    }
    merged
}

pub(super) fn merge_mcp_servers(
    base: &[McpServerConfig],
    overlay: &[McpServerConfig],
) -> Vec<McpServerConfig> {
    let mut merged = base.to_vec();
    for server in overlay {
        if let Some(existing) = merged.iter_mut().find(|s| s.id == server.id) {
            *existing = server.clone();
        } else {
            merged.push(server.clone());
        }
    }
    merged
}

pub(super) fn merge_hooks(base: &[HookConfig], overlay: &[HookConfig]) -> Vec<HookConfig> {
    let mut merged = base.to_vec();
    for hook in overlay {
        if let Some(existing) = merged.iter_mut().find(|h| h.id == hook.id) {
            *existing = hook.clone();
        } else {
            merged.push(hook.clone());
        }
    }
    merged
}

pub(super) fn project_core_file(config: &ProjectConfig) -> ProjectCoreFile {
    ProjectCoreFile {
        version: config.version.clone(),
        id: config.id.clone(),
        name: config.name.clone(),
        description: config.description.clone(),
    }
}

pub(super) fn write_project_core_config(path: &Path, config: &ProjectConfig) -> Result<()> {
    let core = project_core_file(config);
    let json_str = serde_json::to_string_pretty(&core)?;
    write_atomic(path, json_str)?;
    Ok(())
}
