use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use super::mcp::{get_mcp_config, save_mcp_config};
use super::merge::{merge_hooks, merge_mcp_servers, merge_modes, merge_string_lists, write_project_core_config};
use super::modes::{get_modes_config, save_modes_config};
use super::project::{ProjectConfig, default_providers};
use super::runtime_settings::{
    get_legacy_agents_config, get_runtime_settings, migrate_json_config,
    normalize_git_config, remove_legacy_agents_config, save_runtime_settings,
};
use super::types::{PRIMARY_CONFIG_FILE, LEGACY_CONFIG_FILE};
use crate::project::get_global_dir;

pub fn get_config(project_dir: Option<PathBuf>) -> Result<ProjectConfig> {
    let is_project = project_dir.is_some();
    let config_dir = match project_dir {
        Some(dir) => dir,
        None => get_global_dir()?,
    };

    // Prefer ship.jsonc, then legacy ship.toml.
    let primary_path = config_dir.join(PRIMARY_CONFIG_FILE);
    let legacy_path = config_dir.join(LEGACY_CONFIG_FILE);
    let json_path = config_dir.join("config.json");

    let mut config = None;
    if primary_path.exists() {
        let content = fs::read_to_string(&primary_path)?;
        config = Some(compiler::jsonc::from_jsonc_str(&content)?);
    } else if legacy_path.exists() {
        let content = fs::read_to_string(&legacy_path)?;
        config = Some(toml::from_str(&content)?);
    }

    let mut config = if let Some(config) = config {
        config
    } else if json_path.exists() {
        // Legacy JSON config — read what we can and migrate.
        migrate_json_config(&json_path).unwrap_or_default()
    } else {
        ProjectConfig::default()
    };
    config.git = normalize_git_config(config.git);

    if is_project {
        if let Some((providers, active_agent, hooks, statuses, ai, git, namespaces)) =
            get_runtime_settings(&config_dir)?
        {
            config.providers = providers;
            config.active_agent = active_agent;
            config.hooks = hooks;
            if let Some(statuses) = statuses {
                config.statuses = statuses;
            }
            if ai.is_some() {
                config.ai = ai;
            }
            if let Some(git) = git {
                config.git = git;
            }
            if let Some(namespaces) = namespaces {
                config.namespaces = namespaces;
            }
        } else if let Some(legacy) = get_legacy_agents_config(&config_dir)? {
            // One-time compatibility path: bootstrap SQLite runtime settings from
            // legacy .ship/agents/config.toml if present.
            config.providers = legacy.providers;
            config.active_agent = legacy.active_agent;
            config.hooks = legacy.hooks;
            save_runtime_settings(&config_dir, &config)?;
            remove_legacy_agents_config(&config_dir)?;
        }
    }

    if is_project {
        let modes = get_modes_config(&config_dir)?;
        if !modes.is_empty() {
            config.modes = modes;
        }
    }

    let servers = get_mcp_config(&config_dir)?;
    if !servers.is_empty() {
        config.mcp_servers = servers;
    }

    Ok(config)
}

/// Returns a merged view of global + project config.
/// Project values win; missing project AI/agent/mode/MCP values inherit from global.
pub fn get_effective_config(project_dir: Option<PathBuf>) -> Result<ProjectConfig> {
    let global = get_config(None)?;
    let Some(project_dir) = project_dir else {
        return Ok(global);
    };

    let mut project = get_config(Some(project_dir))?;

    if project.ai.is_none() {
        project.ai = global.ai;
    }

    project.agent.skills = merge_string_lists(&global.agent.skills, &project.agent.skills);
    project.agent.prompts = merge_string_lists(&global.agent.prompts, &project.agent.prompts);
    project.agent.context = merge_string_lists(&global.agent.context, &project.agent.context);

    // Project providers win; fall back to global when project does not specify a real override.
    if (project.providers.is_empty() || project.providers == default_providers())
        && !global.providers.is_empty()
    {
        project.providers = global.providers;
    }

    project.modes = merge_modes(&global.modes, &project.modes);
    project.mcp_servers = merge_mcp_servers(&global.mcp_servers, &project.mcp_servers);
    project.hooks = merge_hooks(&global.hooks, &project.hooks);

    if project.active_agent.is_none() {
        project.active_agent = global.active_agent;
    }

    Ok(project)
}

pub fn save_config(config: &ProjectConfig, project_dir: Option<PathBuf>) -> Result<()> {
    let config_dir = if let Some(p_dir) = project_dir.clone() {
        p_dir
    } else {
        get_global_dir()?
    };
    let path = config_dir.join(PRIMARY_CONFIG_FILE);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if project_dir.is_none() {
        // Global config stays file-backed for now.
        let toml_str = toml::to_string_pretty(config)?;
        crate::fs_util::write_atomic(&path, toml_str)?;
        return Ok(());
    }

    let mut effective = config.clone();
    if effective.id.trim().is_empty() {
        #[derive(serde::Deserialize, Default)]
        struct MinConfig {
            #[serde(default)]
            id: String,
        }
        if let Ok(content) = fs::read_to_string(&path) {
            let parsed: MinConfig = compiler::jsonc::from_jsonc_str(&content).unwrap_or_default();
            if !parsed.id.trim().is_empty() {
                effective.id = parsed.id;
            }
        }
    }
    if effective.id.trim().is_empty() {
        effective.id = crate::gen_nanoid();
    }

    // Bootstrap ship.jsonc with project identity before any SQLite-backed writes.
    // save_runtime_settings/open_project_db resolve the DB key from ship.jsonc:id.
    if !path.exists() {
        write_project_core_config(&path, &effective)?;
    }

    // Project runtime settings + mode bindings live in SQLite.
    save_runtime_settings(&config_dir, &effective)?;
    // File-backed catalog state (mcp/skills/rules) is indexed into SQLite for mode refs.
    save_mcp_config(&config_dir, &effective.mcp_servers)?;
    save_modes_config(&config_dir, &effective.modes)?;

    // Keep ship.jsonc focused on stable project identity only.
    write_project_core_config(&path, &effective)?;
    Ok(())
}

