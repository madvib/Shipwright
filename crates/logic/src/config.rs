use crate::{SHIP_DIR_NAME, get_global_dir};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    pub theme: Option<String>,
    pub author: Option<String>,
    pub notifications_enabled: Option<bool>,
    pub default_status: Option<String>,
    pub statuses: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectDiscovery {
    pub name: String,
    pub path: PathBuf,
}

pub fn get_config(project_dir: Option<PathBuf>) -> Result<Config> {
    let global_path = get_global_dir()?.join("config.json");
    let mut config = if global_path.exists() {
        let content = fs::read_to_string(&global_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    };

    if let Some(p_dir) = project_dir {
        let p_config_path = p_dir.join("config.json");
        if p_config_path.exists() {
            let p_content = fs::read_to_string(&p_config_path)?;
            let p_config: Config = serde_json::from_str(&p_content)?;
            // Layer project config over global
            if p_config.theme.is_some() {
                config.theme = p_config.theme;
            }
            if p_config.author.is_some() {
                config.author = p_config.author;
            }
            if p_config.notifications_enabled.is_some() {
                config.notifications_enabled = p_config.notifications_enabled;
            }
            if p_config.default_status.is_some() {
                config.default_status = p_config.default_status;
            }
            if p_config.statuses.is_some() {
                config.statuses = p_config.statuses;
            }
        }
    }

    // Ensure default statuses if none provided
    if config.statuses.is_none() {
        config.statuses = Some(vec![
            "backlog".to_string(),
            "blocked".to_string(),
            "in-progress".to_string(),
            "done".to_string(),
        ]);
    }

    Ok(config)
}

pub fn save_config(config: &Config, project_dir: Option<PathBuf>) -> Result<()> {
    let path = if let Some(p_dir) = project_dir {
        p_dir.join("config.json")
    } else {
        get_global_dir()?.join("config.json")
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn discover_projects(root: PathBuf) -> Result<Vec<ProjectDiscovery>> {
    let mut projects = Vec::new();
    if !root.is_dir() {
        return Ok(projects);
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let ship_dir = path.join(SHIP_DIR_NAME);
            if ship_dir.exists() && ship_dir.is_dir() {
                projects.push(ProjectDiscovery {
                    name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into(),
                    path: ship_dir,
                });
            }
        }
    }
    Ok(projects)
}
