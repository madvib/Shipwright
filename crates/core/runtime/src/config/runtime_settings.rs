use super::project::ProjectConfig;
use super::types::{
    AiConfig, GitConfig, HookConfig, LegacyAgentsConfigFile, NamespaceConfig, StatusConfig,
};
use crate::db::kv;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

const NS: &str = "runtime";

pub(super) fn normalize_git_config(mut git: GitConfig) -> GitConfig {
    if git.commit.iter().any(|entry| entry == "agents") {
        for entry in ["mcp", "permissions", "rules"] {
            if !git.commit.iter().any(|existing| existing == entry) {
                git.commit.push(entry.to_string());
            }
        }
        git.commit.retain(|entry| entry != "agents");
    }

    if git.ignore.iter().any(|entry| entry == "agents") {
        for entry in ["mcp", "permissions", "rules"] {
            if !git.ignore.iter().any(|existing| existing == entry) {
                git.ignore.push(entry.to_string());
            }
        }
    }

    git
}

#[allow(clippy::type_complexity)]
pub(super) fn get_runtime_settings(
    _ship_dir: &Path,
) -> Result<
    Option<(
        Vec<String>,
        Option<String>,
        Vec<HookConfig>,
        Option<Vec<StatusConfig>>,
        Option<AiConfig>,
        Option<GitConfig>,
        Option<Vec<NamespaceConfig>>,
    )>,
> {
    let providers: Vec<String> = kv::get(NS, "providers")?
        .map(|v| serde_json::from_value(v).unwrap_or_default())
        .unwrap_or_default();
    let active_agent: Option<String> = kv::get(NS, "active_agent")?
        .and_then(|v| serde_json::from_value(v).ok());
    let hooks: Vec<HookConfig> = kv::get(NS, "hooks")?
        .map(|v| serde_json::from_value(v).unwrap_or_default())
        .unwrap_or_default();

    // If no keys exist at all, return None (fresh project).
    if providers.is_empty() && active_agent.is_none() && hooks.is_empty() {
        let has_any = kv::get(NS, "providers")?.is_some()
            || kv::get(NS, "active_agent")?.is_some()
            || kv::get(NS, "hooks")?.is_some();
        if !has_any {
            return Ok(None);
        }
    }

    let statuses: Vec<StatusConfig> = kv::get(NS, "statuses")?
        .map(|v| serde_json::from_value(v).unwrap_or_default())
        .unwrap_or_default();
    let statuses = if statuses.is_empty() { None } else { Some(statuses) };

    let ai: Option<AiConfig> = kv::get(NS, "ai")?
        .and_then(|v| serde_json::from_value(v).ok());

    let git_val = kv::get(NS, "git")?;
    let git = git_val
        .and_then(|v| serde_json::from_value::<GitConfig>(v).ok())
        .map(normalize_git_config);

    let namespaces: Vec<NamespaceConfig> = kv::get(NS, "namespaces")?
        .map(|v| serde_json::from_value(v).unwrap_or_default())
        .unwrap_or_default();
    let namespaces = if namespaces.is_empty() { None } else { Some(namespaces) };

    Ok(Some((
        providers,
        active_agent,
        hooks,
        statuses,
        ai,
        git,
        namespaces,
    )))
}

pub(super) fn save_runtime_settings(_ship_dir: &Path, config: &ProjectConfig) -> Result<()> {
    kv::set(NS, "providers", &serde_json::to_value(&config.providers)?)?;
    kv::set(NS, "active_agent", &serde_json::to_value(&config.active_agent)?)?;
    kv::set(NS, "hooks", &serde_json::to_value(&config.hooks)?)?;
    kv::set(NS, "statuses", &serde_json::to_value(&config.statuses)?)?;
    kv::set(NS, "ai", &serde_json::to_value(&config.ai)?)?;
    kv::set(NS, "git", &serde_json::to_value(&normalize_git_config(config.git.clone()))?)?;
    kv::set(NS, "namespaces", &serde_json::to_value(&config.namespaces)?)?;
    Ok(())
}

pub(super) fn get_legacy_agents_config(ship_dir: &Path) -> Result<Option<LegacyAgentsConfigFile>> {
    let path = legacy_agents_config_path(ship_dir);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    let parsed: LegacyAgentsConfigFile = toml::from_str(&content)?;
    Ok(Some(parsed))
}

pub(super) fn remove_legacy_agents_config(ship_dir: &Path) -> Result<()> {
    let path = legacy_agents_config_path(ship_dir);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn legacy_agents_config_path(ship_dir: &Path) -> PathBuf {
    ship_dir.join("agents").join("config.toml")
}

pub(super) fn migrate_json_config(path: &Path) -> Result<ProjectConfig> {
    #[derive(serde::Deserialize, Default)]
    struct LegacyConfig {
        statuses: Option<Vec<String>>,
    }

    let content = fs::read_to_string(path)?;
    let legacy: LegacyConfig = serde_json::from_str(&content).unwrap_or_default();

    let statuses = legacy
        .statuses
        .unwrap_or_default()
        .into_iter()
        .map(|id| StatusConfig {
            name: id_to_name(&id),
            color: default_color_for(&id),
            id,
        })
        .collect();
    Ok(ProjectConfig {
        statuses,
        ..Default::default()
    })
}

pub(super) fn id_to_name(id: &str) -> String {
    id.split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn default_color_for(id: &str) -> String {
    match id {
        "backlog" => "gray".into(),
        "in-progress" => "blue".into(),
        "review" => "yellow".into(),
        "done" => "green".into(),
        "blocked" => "red".into(),
        _ => "gray".into(),
    }
}
