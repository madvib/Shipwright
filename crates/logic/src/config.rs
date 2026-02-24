use crate::fs_util::write_atomic;
use crate::{SHIP_DIR_NAME, get_global_dir};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ─── Data types ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
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
#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct GitConfig {
    /// Paths/globs that should be gitignored (relative to .ship/).
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Paths/globs that should be committed (relative to .ship/).
    #[serde(default)]
    pub commit: Vec<String>,
}

/// Configuration for the AI pass-through CLI.
/// Ship does not call any AI APIs directly — it spawns the configured CLI binary.
/// Supported providers: "claude" (Claude Code CLI), "gemini", "codex".
#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct AiConfig {
    /// Which AI CLI to use. Defaults to "claude".
    pub provider: Option<String>,
    /// Override the binary path if it's not on PATH. Defaults to the provider name.
    pub cli_path: Option<String>,
}

impl AiConfig {
    pub fn effective_provider(&self) -> &str {
        self.provider.as_deref().unwrap_or("claude")
    }

    /// The binary to invoke — cli_path override, or falls back to the provider name.
    pub fn effective_cli(&self) -> &str {
        self.cli_path
            .as_deref()
            .unwrap_or_else(|| self.effective_provider())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct ModeConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Tool IDs visible in this mode (empty = all)
    #[serde(default)]
    pub active_tools: Vec<String>,
    /// MCP server IDs active in this mode (empty = all)
    #[serde(default)]
    pub mcp_servers: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// "global" | "project" | "mode"
    #[serde(default = "default_scope")]
    pub scope: String,
}

fn default_scope() -> String {
    "global".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
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
    #[serde(default)]
    pub modes: Vec<ModeConfig>,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default)]
    pub active_mode: Option<String>,
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
            modes: Vec::new(),
            mcp_servers: Vec::new(),
            active_mode: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ProjectDiscovery {
    pub name: String,
    /// Stored as PathBuf internally; serialized as a string path on the wire.
    #[specta(type = String)]
    pub path: PathBuf,
    #[serde(default)]
    pub issue_count: usize,
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
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            // Skip hidden, system, and archive directories
            if name.starts_with('.') && name != ".ship" {
                continue;
            }
            if matches!(name.as_ref(), "Trash" | ".Trash" | ".DS_Store" | "._*" | "TemporaryItems" | ".Spotlight-V100" | ".fseventsd") {
                continue;
            }
            let ship_dir = path.join(SHIP_DIR_NAME);
            if ship_dir.exists() && ship_dir.is_dir() {
                projects.push(ProjectDiscovery {
                    name: name.into_owned(),
                    path: ship_dir,
                    issue_count: 0,
                });
            }
        }
    }
    Ok(projects)
}

// ─── Mode CRUD ────────────────────────────────────────────────────────────────

pub fn add_mode(project_dir: Option<PathBuf>, mode: ModeConfig) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    if config.modes.iter().any(|m| m.id == mode.id) {
        return Err(anyhow!("Mode '{}' already exists", mode.id));
    }
    config.modes.push(mode);
    save_config(&config, project_dir)
}

pub fn remove_mode(project_dir: Option<PathBuf>, id: &str) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    config.modes.retain(|m| m.id != id);
    if config.active_mode.as_deref() == Some(id) {
        config.active_mode = None;
    }
    save_config(&config, project_dir)
}

pub fn set_active_mode(project_dir: Option<PathBuf>, id: Option<&str>) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    if let Some(mode_id) = id {
        if !config.modes.iter().any(|m| m.id == mode_id) {
            return Err(anyhow!("Mode '{}' not found", mode_id));
        }
    }
    config.active_mode = id.map(|s| s.to_string());
    save_config(&config, project_dir)
}

pub fn get_active_mode(project_dir: Option<PathBuf>) -> Result<Option<ModeConfig>> {
    let config = get_config(project_dir)?;
    Ok(config.active_mode.as_ref().and_then(|id| {
        config.modes.into_iter().find(|m| &m.id == id)
    }))
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
