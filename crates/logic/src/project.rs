use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

pub const SHIP_DIR_NAME: &str = ".ship";

/// Resolves the .ship directory by searching upwards from the given directory.
/// Also checks for legacy `.project` and migrates it to `.ship` if found.
/// Supports `SHIP_DIR` environment variable override.
pub fn get_project_dir(start_dir: Option<PathBuf>) -> Result<PathBuf> {
    // 1. Check for environment variable override
    if let Ok(env_path) = env::var("SHIP_DIR") {
        let path = PathBuf::from(env_path);
        if path.exists() && path.is_dir() {
            return Ok(path);
        }
    }

    // 2. Traversal logic
    let mut current_dir = start_dir.unwrap_or(env::current_dir()?);
    loop {
        let ship_path = current_dir.join(SHIP_DIR_NAME);
        if ship_path.exists() && ship_path.is_dir() {
            // Check if it's a project dir (has Issues or ADR) or just a global dir
            if ship_path.join("Issues").is_dir() || ship_path.join("ADR").is_dir() {
                return Ok(ship_path);
            }
        }

        // Check for legacy .project
        let legacy_path = current_dir.join(".project");
        if legacy_path.exists() && legacy_path.is_dir() {
            // Found legacy .project, so migrate it to .ship
            let ship_path = current_dir.join(SHIP_DIR_NAME);
            fs::rename(&legacy_path, &ship_path).context("Failed to migrate .project to .ship")?;
            return Ok(ship_path);
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            // Last resort: search in global config directory if not found in project tree
            let global = get_global_dir()?;
            if global.exists() && global.join("Issues").is_dir() {
                return Ok(global);
            }

            return Err(anyhow!(
                "Project tracking not initialized in this directory or its parents. Run `ship init` to create a .ship directory."
            ));
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectRegistry {
    pub projects: Vec<ProjectEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProjectEntry {
    pub name: String,
    pub path: PathBuf,
}

pub fn get_registry_path() -> Result<PathBuf> {
    Ok(get_global_dir()?.join("projects.json"))
}

pub fn load_registry() -> Result<ProjectRegistry> {
    let path = get_registry_path()?;
    if !path.exists() {
        return Ok(ProjectRegistry {
            projects: Vec::new(),
        });
    }
    let content = fs::read_to_string(path)?;
    let registry: ProjectRegistry = serde_json::from_str(&content)?;
    Ok(registry)
}

pub fn save_registry(registry: &ProjectRegistry) -> Result<()> {
    let path = get_registry_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(registry)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn register_project(name: String, path: PathBuf) -> Result<()> {
    let mut registry = load_registry()?;
    let entry = ProjectEntry {
        name,
        path: fs::canonicalize(path)?,
    };

    if !registry.projects.contains(&entry) {
        registry.projects.push(entry);
        save_registry(&registry)?;
    }
    Ok(())
}

pub fn unregister_project(path: PathBuf) -> Result<()> {
    let mut registry = load_registry()?;
    let path = fs::canonicalize(path)?;
    registry.projects.retain(|p| p.path != path);
    save_registry(&registry)?;
    Ok(())
}

pub fn list_registered_projects() -> Result<Vec<ProjectEntry>> {
    let registry = load_registry()?;
    Ok(registry.projects)
}

/// Returns the global config directory (~/.ship)
pub fn get_global_dir() -> Result<PathBuf> {
    home::home_dir()
        .map(|h| h.join(SHIP_DIR_NAME))
        .ok_or_else(|| anyhow!("Could not find home directory"))
}

/// Initializes the .ship directory structure in the given directory.
pub fn init_project(base_dir: PathBuf) -> Result<PathBuf> {
    let ship_path = base_dir.join(SHIP_DIR_NAME);

    fs::create_dir_all(ship_path.join("Issues/backlog"))?;
    fs::create_dir_all(ship_path.join("Issues/blocked"))?;
    fs::create_dir_all(ship_path.join("Issues/done"))?;
    fs::create_dir_all(ship_path.join("Issues/in-progress"))?;
    fs::create_dir_all(ship_path.join("ADR"))?;
    fs::create_dir_all(ship_path.join("templates"))?;

    let log_path = ship_path.join("log.md");
    if !log_path.exists() {
        fs::write(log_path, "# Project Log\n\n")?;
    }

    let readme_path = ship_path.join("README.md");
    if !readme_path.exists() {
        fs::write(readme_path, "# Project Tracking\n\nManaged by Ship CLI.")?;
    }

    Ok(ship_path)
}

pub fn sanitize_file_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .to_lowercase()
}

/// Returns the human-readable project name from the parent directory of a .ship path.
pub fn get_project_name(ship_path: &std::path::Path) -> String {
    ship_path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown Project".to_string())
}
