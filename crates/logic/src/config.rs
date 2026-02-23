use crate::fs_util::write_atomic;
use crate::{SHIP_DIR_NAME, get_global_dir};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ─── Data types ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusConfig {
    pub id: String,
    pub name: String,
    #[serde(default = "default_color")]
    pub color: String,
}

fn default_color() -> String {
    "gray".to_string()
}

/// Controls which parts of .ship/ are committed to git.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GitConfig {
    /// Paths/globs that should be gitignored (relative to .ship/).
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Paths/globs that should be committed (relative to .ship/).
    #[serde(default)]
    pub commit: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AiConfig {
    pub anthropic_api_key: Option<String>,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            anthropic_api_key: None,
            model: Some("claude-haiku-4-5-20251001".to_string()),
            max_tokens: Some(1024),
        }
    }
}

impl AiConfig {
    pub fn effective_model(&self) -> &str {
        self.model.as_deref().unwrap_or("claude-haiku-4-5-20251001")
    }

    pub fn effective_max_tokens(&self) -> u32 {
        self.max_tokens.unwrap_or(1024)
    }

    pub fn resolve_api_key(&self) -> Option<String> {
        self.anthropic_api_key
            .clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectConfig {
    #[serde(default = "default_version")]
    pub version: String,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default = "default_statuses")]
    pub statuses: Vec<StatusConfig>,
    #[serde(default)]
    pub git: GitConfig,
    pub ai: Option<AiConfig>,
}

fn default_version() -> String {
    "1".to_string()
}

fn default_statuses() -> Vec<StatusConfig> {
    vec![
        StatusConfig { id: "backlog".into(),     name: "Backlog".into(),      color: "gray".into() },
        StatusConfig { id: "in-progress".into(), name: "In Progress".into(),  color: "blue".into() },
        StatusConfig { id: "review".into(),      name: "Review".into(),       color: "yellow".into() },
        StatusConfig { id: "blocked".into(),     name: "Blocked".into(),      color: "red".into() },
        StatusConfig { id: "done".into(),        name: "Done".into(),         color: "green".into() },
    ]
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            name: None,
            description: None,
            statuses: default_statuses(),
            git: GitConfig::default(),
            ai: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectDiscovery {
    pub name: String,
    pub path: PathBuf,
}

// ─── Read / Write ─────────────────────────────────────────────────────────────

pub fn get_config(project_dir: Option<PathBuf>) -> Result<ProjectConfig> {
    let mut config = ProjectConfig::default();

    if let Some(p_dir) = project_dir {
        // Try .toml first, then fall back to legacy .json
        let toml_path = p_dir.join("config.toml");
        let json_path = p_dir.join("config.json");

        if toml_path.exists() {
            let content = fs::read_to_string(&toml_path)?;
            config = toml::from_str(&content)?;
        } else if json_path.exists() {
            // Legacy JSON config — read what we can and migrate
            config = migrate_json_config(&json_path).unwrap_or_default();
        }
    }

    Ok(config)
}

pub fn save_config(config: &ProjectConfig, project_dir: Option<PathBuf>) -> Result<()> {
    let path = if let Some(p_dir) = project_dir {
        p_dir.join("config.toml")
    } else {
        get_global_dir()?.join("config.toml")
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let toml_str = toml::to_string_pretty(config)?;
    write_atomic(&path, toml_str)?;
    Ok(())
}

/// Read a legacy JSON config and convert to ProjectConfig.
fn migrate_json_config(path: &Path) -> Result<ProjectConfig> {
    #[derive(serde::Deserialize, Default)]
    struct LegacyConfig {
        statuses: Option<Vec<String>>,
    }

    let content = fs::read_to_string(path)?;
    let legacy: LegacyConfig = serde_json::from_str(&content).unwrap_or_default();

    let mut config = ProjectConfig::default();
    if let Some(status_ids) = legacy.statuses {
        config.statuses = status_ids
            .into_iter()
            .map(|id| StatusConfig {
                name: id_to_name(&id),
                color: default_color_for(&id),
                id,
            })
            .collect();
    }
    Ok(config)
}

fn id_to_name(id: &str) -> String {
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

fn default_color_for(id: &str) -> String {
    match id {
        "backlog"     => "gray".into(),
        "in-progress" => "blue".into(),
        "review"      => "yellow".into(),
        "done"        => "green".into(),
        "blocked"     => "red".into(),
        _             => "gray".into(),
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

pub fn get_project_statuses(project_dir: Option<PathBuf>) -> Result<Vec<String>> {
    let config = get_config(project_dir)?;
    Ok(config.statuses.iter().map(|s| s.id.clone()).collect())
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
        save_config(&config, project_dir)?;
    }
    Ok(())
}

pub fn remove_status(project_dir: Option<PathBuf>, status: &str) -> Result<()> {
    // Guard: refuse if any issues exist in this status folder
    if let Some(ref dir) = project_dir {
        let status_dir = dir.join("issues").join(status);
        if status_dir.exists() {
            let count = fs::read_dir(&status_dir)
                .map(|d| d.filter_map(|e| e.ok()).count())
                .unwrap_or(0);
            if count > 0 {
                return Err(anyhow!(
                    "Cannot remove status '{}': {} issue(s) still in this status. Move them first.",
                    status,
                    count
                ));
            }
        }
    }
    let mut config = get_config(project_dir.clone())?;
    config.statuses.retain(|s| s.id != status);
    save_config(&config, project_dir)?;
    Ok(())
}

pub fn get_git_config(project_dir: &Path) -> Result<GitConfig> {
    let config = get_config(Some(project_dir.to_path_buf()))?;
    Ok(config.git)
}

pub fn set_git_config(project_dir: &Path, git: GitConfig) -> Result<()> {
    let mut config = get_config(Some(project_dir.to_path_buf()))?;
    config.git = git;
    generate_gitignore(project_dir, &config.git)?;
    save_config(&config, Some(project_dir.to_path_buf()))?;
    Ok(())
}

/// Toggle a named category in/out of git commit tracking.
pub fn set_category_committed(project_dir: &Path, category: &str, commit: bool) -> Result<()> {
    let mut git = get_git_config(project_dir)?;
    if commit {
        if !git.commit.contains(&category.to_string()) {
            git.commit.push(category.to_string());
        }
        git.ignore.retain(|i| i != category);
    } else {
        git.commit.retain(|c| c != category);
        if !git.ignore.contains(&category.to_string()) {
            git.ignore.push(category.to_string());
        }
    }
    set_git_config(project_dir, git)
}

pub fn is_category_committed(git: &GitConfig, category: &str) -> bool {
    git.commit.contains(&category.to_string())
}

/// Write `.ship/.gitignore`. Everything not in `git.commit` is ignored by default.
pub fn generate_gitignore(ship_dir: &Path, git: &GitConfig) -> Result<()> {
    let known = ["issues", "adrs", "specs", "log.md", "config.toml", "templates", "plugins"];
    let mut lines = vec![
        "# Managed by Ship — edit via `ship git include/exclude`".to_string(),
        String::new(),
    ];
    for item in known {
        if !git.commit.contains(&item.to_string()) {
            lines.push(item.to_string());
        }
    }
    // Always ignore internal dirs that are never committed
    lines.push(String::new());
    lines.push("# Internal".to_string());
    lines.push("workflow/".to_string());
    let content = lines.join("\n") + "\n";
    write_atomic(&ship_dir.join(".gitignore"), content)?;
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

/// Migrate `config.json` → `config.toml` in-place (no-op if already migrated).
pub fn migrate_json_config_file(project_dir: &Path) -> Result<bool> {
    let json_path = project_dir.join("config.json");
    let toml_path = project_dir.join("config.toml");
    if !json_path.exists() || toml_path.exists() {
        return Ok(false);
    }
    let config = migrate_json_config(&json_path)?;
    save_config(&config, Some(project_dir.to_path_buf()))?;
    fs::remove_file(json_path)?;
    Ok(true)
}
