use super::io::{get_config, get_effective_config, save_config};
use super::project::{AgentProfile, McpServerConfig};
use super::runtime_settings::{default_color_for, id_to_name};
use super::types::{HookConfig, NamespaceConfig, StatusConfig};
use crate::{EventAction, EventEntity, append_event};
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};

// ─── Helpers ──────────────────────────────────────────────────────────────────

pub fn get_project_statuses(project_dir: Option<PathBuf>) -> Result<Vec<String>> {
    let config = get_config(project_dir)?;
    Ok(config.statuses.iter().map(|s| s.id.clone()).collect())
}

fn emit_config_event(
    project_dir: &Option<PathBuf>,
    action: EventAction,
    subject: &str,
    details: Option<String>,
) -> Result<()> {
    if let Some(dir) = project_dir {
        append_event(
            dir,
            "logic",
            EventEntity::Config,
            action,
            subject.to_string(),
            details,
        )?;
    }
    Ok(())
}

fn emit_mode_event(
    project_dir: &Option<PathBuf>,
    action: EventAction,
    subject: &str,
    details: Option<String>,
) -> Result<()> {
    if let Some(dir) = project_dir {
        append_event(
            dir,
            "logic",
            EventEntity::Mode,
            action,
            subject.to_string(),
            details,
        )?;
    }
    Ok(())
}

pub fn add_status(project_dir: Option<PathBuf>, status: &str) -> Result<()> {
    let sanitized = status.to_lowercase().replace(' ', "-");
    let mut config = get_config(project_dir.clone())?;
    if !config.statuses.iter().any(|s| s.id == sanitized) {
        config.statuses.push(StatusConfig {
            id: sanitized.clone(),
            name: id_to_name(&sanitized),
            color: default_color_for(&sanitized),
        });
        save_config(&config, project_dir.clone())?;
        emit_config_event(
            &project_dir,
            EventAction::Add,
            "status",
            Some(format!("id={}", sanitized)),
        )?;
    }
    Ok(())
}

pub fn remove_status(project_dir: Option<PathBuf>, status: &str) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    config.statuses.retain(|s| s.id != status);
    save_config(&config, project_dir.clone())?;
    emit_config_event(
        &project_dir,
        EventAction::Remove,
        "status",
        Some(format!("id={}", status)),
    )?;
    Ok(())
}

pub fn ensure_registered_namespaces(
    ship_path: &Path,
    namespaces: &[NamespaceConfig],
) -> Result<()> {
    const RESERVED_TOP_LEVEL: &[&str] = &[
        "project",
        "agents",
        "ship.jsonc",
        "ship.toml",
        "config.toml",
        "log.md",
        "templates",
        "plugins",
    ];

    for ns in namespaces {
        let rel = ns.path.trim();
        if rel.is_empty() {
            continue;
        }
        let rel_path = Path::new(rel);
        if rel_path.is_absolute()
            || rel_path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(anyhow!(
                "Invalid namespace path '{}' for namespace '{}'",
                ns.path,
                ns.id
            ));
        }
        if ns.id.starts_with("plugin:") {
            let mut components = rel_path.components();
            let first = components
                .next()
                .and_then(|c| c.as_os_str().to_str())
                .ok_or_else(|| anyhow!("Plugin namespace '{}' has an invalid path", ns.id))?;
            if components.next().is_some() {
                return Err(anyhow!(
                    "Plugin namespace '{}' must claim a top-level directory only",
                    ns.id
                ));
            }
            if RESERVED_TOP_LEVEL.contains(&first) {
                return Err(anyhow!(
                    "Plugin namespace '{}' cannot claim reserved path '{}'",
                    ns.id,
                    first
                ));
            }
        }
        std::fs::create_dir_all(
            crate::project::ship_dir_from_path(ship_path)
                .unwrap_or(ship_path.to_path_buf())
                .join(rel_path),
        )?;
    }
    Ok(())
}

// ─── Mode CRUD ────────────────────────────────────────────────────────────────

pub fn add_agent(project_dir: Option<PathBuf>, mode: AgentProfile) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    if config.modes.iter().any(|m| m.id == mode.id) {
        return Err(anyhow!("Mode '{}' already exists", mode.id));
    }
    let mode_id = mode.id.clone();
    config.modes.push(mode);
    save_config(&config, project_dir.clone())?;
    emit_mode_event(
        &project_dir,
        EventAction::Add,
        "mode",
        Some(format!("id={}", mode_id)),
    )?;
    Ok(())
}

pub fn remove_agent(project_dir: Option<PathBuf>, id: &str) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    config.modes.retain(|m| m.id != id);
    if config.active_agent.as_deref() == Some(id) {
        config.active_agent = None;
    }
    save_config(&config, project_dir.clone())?;
    emit_mode_event(
        &project_dir,
        EventAction::Remove,
        "mode",
        Some(format!("id={}", id)),
    )?;
    Ok(())
}

pub fn set_active_agent(project_dir: Option<PathBuf>, id: Option<&str>) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    if let Some(mode_id) = id {
        let mode_exists = match &project_dir {
            Some(dir) => get_effective_config(Some(dir.clone()))?
                .modes
                .iter()
                .any(|m| m.id == mode_id),
            None => config.modes.iter().any(|m| m.id == mode_id),
        };
        if !mode_exists {
            return Err(anyhow!("Mode '{}' not found", mode_id));
        }
    }
    config.active_agent = id.map(|s| s.to_string());
    save_config(&config, project_dir.clone())?;
    // Auto-sync to configured agent targets after mode change
    if let Some(ref dir) = project_dir
        && let Err(error) = crate::agents::export::sync_active_agent(dir)
    {
        eprintln!("[ship] warning: active mode sync failed: {}", error);
    }
    emit_mode_event(
        &project_dir,
        if id.is_some() {
            EventAction::Set
        } else {
            EventAction::Clear
        },
        "active_agent",
        Some(format!("id={}", id.unwrap_or("none"))),
    )?;
    Ok(())
}

pub fn get_active_agent(project_dir: Option<PathBuf>) -> Result<Option<AgentProfile>> {
    let config = get_config(project_dir)?;
    Ok(config
        .active_agent
        .as_ref()
        .and_then(|id| config.modes.into_iter().find(|m| &m.id == id)))
}

// ─── MCP Server Registry CRUD ─────────────────────────────────────────────────

pub fn add_mcp_server(project_dir: Option<PathBuf>, server: McpServerConfig) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    if config.mcp_servers.iter().any(|s| s.id == server.id) {
        return Err(anyhow!("MCP server '{}' already exists", server.id));
    }
    config.mcp_servers.push(server);
    save_config(&config, project_dir)
}

pub fn remove_mcp_server(project_dir: Option<PathBuf>, id: &str) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    config.mcp_servers.retain(|s| s.id != id);
    save_config(&config, project_dir)
}

pub fn list_mcp_servers(project_dir: Option<PathBuf>) -> Result<Vec<McpServerConfig>> {
    let config = get_config(project_dir)?;
    Ok(config.mcp_servers)
}

// ─── Hook CRUD ────────────────────────────────────────────────────────────────

pub fn add_hook(project_dir: Option<PathBuf>, hook: HookConfig) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    if config.hooks.iter().any(|h| h.id == hook.id) {
        return Err(anyhow!("Hook '{}' already exists", hook.id));
    }
    config.hooks.push(hook);
    save_config(&config, project_dir)
}

pub fn remove_hook(project_dir: Option<PathBuf>, id: &str) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    config.hooks.retain(|h| h.id != id);
    save_config(&config, project_dir)
}

pub fn list_hooks(project_dir: Option<PathBuf>) -> Result<Vec<HookConfig>> {
    let config = get_config(project_dir)?;
    Ok(config.hooks)
}
