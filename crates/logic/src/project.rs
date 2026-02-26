use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const SHIP_DIR_NAME: &str = ".ship";
pub const DEFAULT_STATUSES: &[&str] = &["backlog", "in-progress", "blocked", "done"];
/// Kept for backwards compatibility — prefer DEFAULT_STATUSES or get_project_statuses().
pub const ISSUE_STATUSES: &[&str] = DEFAULT_STATUSES;

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

    // 2. Traversal logic — any directory containing a .ship folder is a project
    let mut current_dir = start_dir.unwrap_or(env::current_dir()?);
    loop {
        let ship_path = current_dir.join(SHIP_DIR_NAME);
        if ship_path.exists() && ship_path.is_dir() {
            return Ok(ship_path);
        }

        // Check for legacy .project
        let legacy_path = current_dir.join(".project");
        if legacy_path.exists() && legacy_path.is_dir() {
            let ship_path = current_dir.join(SHIP_DIR_NAME);
            fs::rename(&legacy_path, &ship_path).context("Failed to migrate .project to .ship")?;
            return Ok(ship_path);
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            return Err(anyhow!(
                "Project tracking not initialized in this directory or its parents. Run `ship init` to create a .ship directory."
            ));
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ProjectRegistry {
    pub projects: Vec<ProjectEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
pub struct ProjectEntry {
    pub name: String,
    #[specta(type = String)]
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
    let canonical_path = fs::canonicalize(path)?;

    // De-duplicate entries by canonical path and keep first occurrence.
    let mut seen_target = false;
    registry.projects.retain(|project| {
        let project_path = fs::canonicalize(&project.path).unwrap_or_else(|_| project.path.clone());
        if project_path == canonical_path {
            if seen_target {
                false
            } else {
                seen_target = true;
                true
            }
        } else {
            true
        }
    });

    if let Some(existing) = registry.projects.iter_mut().find(|project| {
        fs::canonicalize(&project.path).unwrap_or_else(|_| project.path.clone()) == canonical_path
    }) {
        existing.name = name;
        existing.path = canonical_path;
    } else {
        registry.projects.push(ProjectEntry {
            name,
            path: canonical_path,
        });
    }

    save_registry(&registry)?;
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

    fs::create_dir_all(ship_path.join("issues/backlog"))?;
    fs::create_dir_all(ship_path.join("issues/in-progress"))?;
    fs::create_dir_all(ship_path.join("issues/review"))?;
    fs::create_dir_all(ship_path.join("issues/blocked"))?;
    fs::create_dir_all(ship_path.join("issues/done"))?;
    fs::create_dir_all(ship_path.join("releases"))?;
    fs::create_dir_all(ship_path.join("features"))?;
    fs::create_dir_all(ship_path.join("adrs"))?;
    fs::create_dir_all(ship_path.join("specs"))?;
    fs::create_dir_all(ship_path.join("templates"))?;

    let log_path = ship_path.join("log.md");
    if !log_path.exists() {
        fs::write(log_path, "# Project Log\n\n")?;
    }
    crate::events::ensure_event_log(&ship_path)?;

    // Write default config if not present
    let config_path = ship_path.join("config.toml");
    if !config_path.exists() {
        let config = crate::config::ProjectConfig::default();
        crate::config::save_config(&config, Some(ship_path.clone()))?;
    }

    // Write default templates
    write_default_templates(&ship_path)?;

    // Write default .gitignore (opinionated alpha defaults)
    let gitignore_path = ship_path.join(".gitignore");
    if !gitignore_path.exists() {
        let default_git = crate::config::GitConfig::default();
        crate::config::generate_gitignore(&ship_path, &default_git)?;
    }

    // Best-effort init marker in the event stream.
    let _ = crate::events::append_event(
        &ship_path,
        "logic",
        crate::events::EventEntity::Project,
        crate::events::EventAction::Init,
        "project",
        Some("Project initialized".to_string()),
    );

    Ok(ship_path)
}

fn write_default_templates(ship_path: &std::path::Path) -> Result<()> {
    let issue_tmpl = ship_path.join("templates/ISSUE.md");
    if !issue_tmpl.exists() {
        fs::write(issue_tmpl, include_str!("templates/ISSUE.md"))?;
    }
    let spec_tmpl = ship_path.join("templates/SPEC.md");
    if !spec_tmpl.exists() {
        fs::write(spec_tmpl, include_str!("templates/SPEC.md"))?;
    }
    let release_tmpl = ship_path.join("templates/RELEASE.md");
    if !release_tmpl.exists() {
        fs::write(release_tmpl, include_str!("templates/RELEASE.md"))?;
    }
    let vision_tmpl = ship_path.join("templates/VISION.md");
    if !vision_tmpl.exists() {
        fs::write(vision_tmpl, include_str!("templates/VISION.md"))?;
    }
    let feature_tmpl = ship_path.join("templates/FEATURE.md");
    if !feature_tmpl.exists() {
        fs::write(feature_tmpl, include_str!("templates/FEATURE.md"))?;
    }
    let adr_tmpl = ship_path.join("templates/ADR.md");
    if !adr_tmpl.exists() {
        fs::write(adr_tmpl, include_str!("templates/ADR.md"))?;
    }

    // Seed a project-level vision doc if missing.
    let vision_doc = ship_path.join("specs/vision.md");
    if !vision_doc.exists() {
        fs::write(vision_doc, include_str!("templates/VISION.md"))?;
    }
    Ok(())
}

fn template_file_name(kind: &str) -> Result<&'static str> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "issue" | "issues" => Ok("ISSUE.md"),
        "adr" | "adrs" => Ok("ADR.md"),
        "spec" | "specs" => Ok("SPEC.md"),
        "release" | "releases" => Ok("RELEASE.md"),
        "feature" | "features" => Ok("FEATURE.md"),
        "vision" => Ok("VISION.md"),
        _ => Err(anyhow!("Unknown template kind: {}", kind)),
    }
}

fn template_fallback(file_name: &str) -> Result<&'static str> {
    match file_name {
        "ISSUE.md" => Ok(include_str!("templates/ISSUE.md")),
        "ADR.md" => Ok(include_str!("templates/ADR.md")),
        "SPEC.md" => Ok(include_str!("templates/SPEC.md")),
        "RELEASE.md" => Ok(include_str!("templates/RELEASE.md")),
        "FEATURE.md" => Ok(include_str!("templates/FEATURE.md")),
        "VISION.md" => Ok(include_str!("templates/VISION.md")),
        _ => Err(anyhow!("No fallback for template: {}", file_name)),
    }
}

/// Reads a project template from `.ship/templates`, with built-in fallback if missing.
pub fn read_template(ship_path: &Path, kind: &str) -> Result<String> {
    let file_name = template_file_name(kind)?;
    let template_path = ship_path.join("templates").join(file_name);
    if template_path.exists() {
        return fs::read_to_string(template_path)
            .with_context(|| format!("Failed to read template: {}", file_name));
    }
    Ok(template_fallback(file_name)?.to_string())
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

// ─── Global App State ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AppState {
    #[serde(default)]
    pub active_project: Option<PathBuf>,
    #[serde(default)]
    pub recent_projects: Vec<PathBuf>,
}

pub fn load_app_state() -> Result<AppState> {
    let path = get_global_dir()?.join("app_state.json");
    if !path.exists() {
        return Ok(AppState::default());
    }
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content).context("Failed to parse app state")
}

pub fn save_app_state(state: &AppState) -> Result<()> {
    let path = get_global_dir()?.join("app_state.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn set_active_project_global(path: PathBuf) -> Result<()> {
    let mut state = load_app_state()?;
    state.active_project = Some(path.clone());
    // Add to recent projects if not already there
    if !state.recent_projects.contains(&path) {
        state.recent_projects.insert(0, path);
        // Keep only last 10
        state.recent_projects.truncate(10);
    }
    save_app_state(&state)
}

pub fn get_active_project_global() -> Result<Option<PathBuf>> {
    let state = load_app_state()?;
    Ok(state.active_project)
}

pub fn get_recent_projects_global() -> Result<Vec<PathBuf>> {
    let state = load_app_state()?;
    Ok(state.recent_projects)
}
