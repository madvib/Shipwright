use super::project::ProjectConfig;
use super::types::{
    AiConfig, GitConfig, HookConfig, LegacyAgentsConfigFile, NamespaceConfig, StatusConfig,
};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

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
    let Some(raw) = crate::db::agents::get_agent_runtime_settings_db()? else {
        return Ok(None);
    };

    let hooks: Vec<HookConfig> = serde_json::from_str(&raw.hooks_json).unwrap_or_default();
    let statuses: Vec<StatusConfig> = serde_json::from_str(&raw.statuses_json).unwrap_or_default();
    let statuses = if statuses.is_empty() {
        None
    } else {
        Some(statuses)
    };
    let ai = raw
        .ai_json
        .as_deref()
        .and_then(|json| serde_json::from_str::<AiConfig>(json).ok());
    let git = if raw.git_json.trim().is_empty() || raw.git_json.trim() == "{}" {
        None
    } else {
        serde_json::from_str::<GitConfig>(&raw.git_json)
            .ok()
            .map(normalize_git_config)
    };
    let namespaces: Vec<NamespaceConfig> =
        serde_json::from_str(&raw.namespaces_json).unwrap_or_default();
    let namespaces = if namespaces.is_empty() {
        None
    } else {
        Some(namespaces)
    };
    Ok(Some((
        raw.providers,
        raw.active_agent,
        hooks,
        statuses,
        ai,
        git,
        namespaces,
    )))
}

pub(super) fn save_runtime_settings(_ship_dir: &Path, config: &ProjectConfig) -> Result<()> {
    let hooks_json = serde_json::to_string(&config.hooks)?;
    let statuses_json = serde_json::to_string(&config.statuses)?;
    let ai_json = config.ai.as_ref().map(serde_json::to_string).transpose()?;
    let git_json = serde_json::to_string(&normalize_git_config(config.git.clone()))?;
    let namespaces_json = serde_json::to_string(&config.namespaces)?;
    crate::db::agents::set_agent_runtime_settings_db(
        &config.providers,
        config.active_agent.as_deref(),
        &hooks_json,
        &statuses_json,
        ai_json.as_deref(),
        &git_json,
        &namespaces_json,
    )
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
