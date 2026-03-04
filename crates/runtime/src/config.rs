use crate::fs_util::write_atomic;
use crate::project::{SHIP_DIR_NAME, get_global_dir, ship_dir_from_path};
use crate::{EventAction, EventEntity, append_event};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const PRIMARY_CONFIG_FILE: &str = "ship.toml";
pub const SECONDARY_CONFIG_FILE: &str = "shipwright.toml";
pub const LEGACY_CONFIG_FILE: &str = "config.toml";

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
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct GitConfig {
    /// Paths/globs that should be gitignored (relative to .ship/).
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Paths/globs that should be committed (relative to .ship/).
    #[serde(default)]
    pub commit: Vec<String>,
}

impl Default for GitConfig {
    fn default() -> Self {
        // Opinionated alpha default:
        // - Keep delivery/context artifacts in git.
        // - Keep volatile issue execution data local unless explicitly included.
        Self {
            ignore: Vec::new(),
            commit: vec![
                "releases".to_string(),
                "features".to_string(),
                "specs".to_string(),
                "adrs".to_string(),
                "notes".to_string(),
                "agents".to_string(),
                "ship.toml".to_string(),
                "templates".to_string(),
            ],
        }
    }
}

/// Configuration for the AI pass-through CLI.
/// Ship does not call any AI APIs directly — it spawns the configured CLI binary.
/// Supported providers: "claude" (Claude Code CLI), "gemini", "codex".
/// "chatgpt" is still accepted as a backwards-compatible alias.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct AiConfig {
    /// Which AI CLI to use. Defaults to "claude".
    pub provider: Option<String>,
    /// Optional model identifier for UI/agent selection context.
    pub model: Option<String>,
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

/// Which lifecycle event triggers a hook.
#[derive(Serialize, Deserialize, Debug, Clone, Type, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum HookTrigger {
    PreToolUse,
    PostToolUse,
    Notification,
    Stop,
    SubagentStop,
    PreCompact,
}

/// A shell command executed on a lifecycle event.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct HookConfig {
    pub id: String,
    pub trigger: HookTrigger,
    /// Glob/regex pattern to match tool name (e.g. "Bash", "mcp__*"). Empty = all tools.
    #[serde(default)]
    pub matcher: Option<String>,
    /// The shell command to run
    pub command: String,
}

/// Allow/deny permission set — tool name patterns.
#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct PermissionConfig {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct AgentLayerConfig {
    /// Skill IDs to load for all sessions in this project.
    #[serde(default)]
    pub skills: Vec<String>,
    /// Prompt IDs to activate for all sessions in this project.
    #[serde(default)]
    pub prompts: Vec<String>,
    /// Context files/folders to preload for agents.
    #[serde(default)]
    pub context: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct NamespaceConfig {
    /// Stable namespace id (e.g. "project", "workflow", "agents", "plugin:ghost-issues")
    pub id: String,
    /// Directory path relative to `.ship/`
    pub path: String,
    /// Owning module or family (e.g. "project", "workflow", "agents", "plugins")
    pub owner: String,
}

fn default_namespaces() -> Vec<NamespaceConfig> {
    vec![
        NamespaceConfig {
            id: "project".to_string(),
            path: "project".to_string(),
            owner: "project".to_string(),
        },
        NamespaceConfig {
            id: "workflow".to_string(),
            path: "workflow".to_string(),
            owner: "workflow".to_string(),
        },
        NamespaceConfig {
            id: "agents".to_string(),
            path: "agents".to_string(),
            owner: "agents".to_string(),
        },
        NamespaceConfig {
            id: "generated".to_string(),
            path: "generated".to_string(),
            owner: "runtime".to_string(),
        },
    ]
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
    /// Which prompt to activate (references a .ship/agents/prompts/<id>.md)
    #[serde(default)]
    pub prompt_id: Option<String>,
    /// Hooks to apply when this mode is active
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
    /// Permission overrides for this mode
    #[serde(default)]
    pub permissions: PermissionConfig,
    /// Which agent targets to sync to (e.g. ["claude", "gemini"])
    #[serde(default)]
    pub target_agents: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpServerType {
    #[default]
    Stdio,
    Sse,
    Http,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    /// For stdio servers: the binary to run
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// "global" | "project" | "mode"
    #[serde(default = "default_scope")]
    pub scope: String,
    /// Transport type: stdio (default), sse, or http
    #[serde(default)]
    pub server_type: McpServerType,
    /// URL for SSE or HTTP servers (ignored for stdio)
    pub url: Option<String>,
    /// If true, the server is registered but not started
    #[serde(default)]
    pub disabled: bool,
    /// Optional connection timeout in seconds
    pub timeout_secs: Option<u32>,
}

fn default_scope() -> String {
    "global".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct McpConfig {
    #[serde(default)]
    pub mcp: McpSection,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct McpSection {
    #[serde(default)]
    pub servers: HashMap<String, McpServerConfig>,
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
    /// Global hooks applied regardless of active mode
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
    #[serde(default)]
    pub agent: AgentLayerConfig,
    /// Which agent providers to generate config for on branch checkout.
    /// Alpha: "claude" | "gemini" | "codex". Defaults to ["claude"].
    #[serde(default = "default_providers")]
    pub providers: Vec<String>,
    /// Claimed `.ship` namespaces. First-party modules are always present.
    #[serde(default = "default_namespaces")]
    pub namespaces: Vec<NamespaceConfig>,
}

fn default_version() -> String {
    "1".to_string()
}

fn default_providers() -> Vec<String> {
    vec!["claude".to_string()]
}

fn default_statuses() -> Vec<StatusConfig> {
    vec![
        StatusConfig {
            id: "backlog".into(),
            name: "Backlog".into(),
            color: "gray".into(),
        },
        StatusConfig {
            id: "in-progress".into(),
            name: "In Progress".into(),
            color: "blue".into(),
        },
        StatusConfig {
            id: "review".into(),
            name: "Review".into(),
            color: "yellow".into(),
        },
        StatusConfig {
            id: "blocked".into(),
            name: "Blocked".into(),
            color: "red".into(),
        },
        StatusConfig {
            id: "done".into(),
            name: "Done".into(),
            color: "green".into(),
        },
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
            hooks: Vec::new(),
            agent: AgentLayerConfig::default(),
            namespaces: default_namespaces(),
            providers: default_providers(),
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
    let config_dir = match project_dir {
        Some(dir) => dir,
        None => get_global_dir()?,
    };

    // Prefer ship.toml, then shipwright.toml, then legacy config.toml.
    let primary_path = config_dir.join(PRIMARY_CONFIG_FILE);
    let secondary_path = config_dir.join(SECONDARY_CONFIG_FILE);
    let legacy_path = config_dir.join(LEGACY_CONFIG_FILE);
    let json_path = config_dir.join("config.json");

    for path in [&primary_path, &secondary_path, &legacy_path] {
        if !path.exists() {
            continue;
        }
        let content = fs::read_to_string(path)?;
        return Ok(toml::from_str(&content)?);
    }

    if json_path.exists() {
        // Legacy JSON config — read what we can and migrate.
        return Ok(migrate_json_config(&json_path).unwrap_or_default());
    }

    Ok(ProjectConfig::default())
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

    // Project providers win; fall back to global if project is still the default ["claude"].
    if project.providers == default_providers() && global.providers != default_providers() {
        project.providers = global.providers;
    }

    project.modes = merge_modes(&global.modes, &project.modes);
    project.mcp_servers = merge_mcp_servers(&global.mcp_servers, &project.mcp_servers);
    project.hooks = merge_hooks(&global.hooks, &project.hooks);

    if project.active_mode.is_none() {
        project.active_mode = global.active_mode;
    }

    Ok(project)
}

pub fn get_mcp_config(ship_dir: &Path) -> Result<Vec<McpServerConfig>> {
    let path = crate::project::mcp_config_path(ship_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)?;
    let raw: McpConfig = toml::from_str(&content)?;

    let mut servers = Vec::new();
    for (id, mut server) in raw.mcp.servers {
        server.id = id;
        servers.push(server);
    }

    Ok(servers)
}

fn merge_string_lists(base: &[String], overlay: &[String]) -> Vec<String> {
    let mut merged = Vec::new();
    let mut seen = HashSet::new();

    for item in base.iter().chain(overlay.iter()) {
        if seen.insert(item.clone()) {
            merged.push(item.clone());
        }
    }

    merged
}

fn merge_modes(base: &[ModeConfig], overlay: &[ModeConfig]) -> Vec<ModeConfig> {
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

fn merge_mcp_servers(
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

fn merge_hooks(base: &[HookConfig], overlay: &[HookConfig]) -> Vec<HookConfig> {
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

pub fn save_config(config: &ProjectConfig, project_dir: Option<PathBuf>) -> Result<()> {
    let path = if let Some(p_dir) = project_dir {
        p_dir.join(PRIMARY_CONFIG_FILE)
    } else {
        get_global_dir()?.join(PRIMARY_CONFIG_FILE)
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
        "backlog" => "gray".into(),
        "in-progress" => "blue".into(),
        "review" => "yellow".into(),
        "done" => "green".into(),
        "blocked" => "red".into(),
        _ => "gray".into(),
    }
}

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
    // Guard: refuse if any issues exist in this status folder
    if let Some(ref dir) = project_dir {
        let status_dir = crate::project::issues_dir(dir).join(status);
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
    save_config(&config, project_dir.clone())?;
    emit_config_event(
        &project_dir,
        EventAction::Remove,
        "status",
        Some(format!("id={}", status)),
    )?;
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
    set_git_config(project_dir, git)?;
    append_event(
        project_dir,
        "logic",
        EventEntity::Config,
        if commit {
            EventAction::Set
        } else {
            EventAction::Clear
        },
        "git_category",
        Some(format!("category={}", category)),
    )?;
    Ok(())
}

pub fn is_category_committed(git: &GitConfig, category: &str) -> bool {
    git.commit.contains(&category.to_string())
}

/// Write `.ship/.gitignore`. Everything not in `git.commit` is ignored by default.
/// Keys use namespace paths (e.g. "workflow/issues", "project/adrs").
pub fn generate_gitignore(ship_dir: &Path, git: &GitConfig) -> Result<()> {
    // (key, namespace path) — key is what appears in git.commit config
    let known: &[(&str, &str)] = &[
        ("issues", "workflow/issues"),
        ("specs", "workflow/specs"),
        ("features", "project/features"),
        ("releases", "project/releases"),
        ("adrs", "project/adrs"),
        ("notes", "project/notes"),
        ("agents", "agents"),
        ("ship.toml", "ship.toml"),
        ("templates", "**/TEMPLATE.md"),
    ];
    let mut lines = vec![
        "# Managed by Ship — edit via `ship git include/exclude`".to_string(),
        String::new(),
    ];
    for (key, path) in known {
        if !git.commit.contains(&key.to_string()) {
            lines.push(path.to_string());
        }
    }
    // Generated tool outputs are always runtime-managed and local-only.
    if !lines.iter().any(|line| line == "generated/") {
        lines.push("generated/".to_string());
    }
    let content = lines.join("\n") + "\n";
    write_atomic(&ship_dir.join(".gitignore"), content)?;
    Ok(())
}

pub fn ensure_registered_namespaces(
    ship_path: &Path,
    namespaces: &[NamespaceConfig],
) -> Result<()> {
    const RESERVED_TOP_LEVEL: &[&str] = &[
        "project",
        "workflow",
        "agents",
        "generated",
        "ship.toml",
        "shipwright.toml",
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
        fs::create_dir_all(
            ship_dir_from_path(ship_path)
                .unwrap_or(ship_path.to_path_buf())
                .join(rel_path),
        )?;
    }
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
            if matches!(
                name.as_ref(),
                "Trash"
                    | ".Trash"
                    | ".DS_Store"
                    | "._*"
                    | "TemporaryItems"
                    | ".Spotlight-V100"
                    | ".fseventsd"
            ) {
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

pub fn remove_mode(project_dir: Option<PathBuf>, id: &str) -> Result<()> {
    let mut config = get_config(project_dir.clone())?;
    config.modes.retain(|m| m.id != id);
    if config.active_mode.as_deref() == Some(id) {
        config.active_mode = None;
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

pub fn set_active_mode(project_dir: Option<PathBuf>, id: Option<&str>) -> Result<()> {
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
    config.active_mode = id.map(|s| s.to_string());
    save_config(&config, project_dir.clone())?;
    // Auto-sync to configured agent targets after mode change
    if let Some(ref dir) = project_dir {
        let _ = crate::agent_export::sync_active_mode(dir);
    }
    emit_mode_event(
        &project_dir,
        if id.is_some() {
            EventAction::Set
        } else {
            EventAction::Clear
        },
        "active_mode",
        Some(format!("id={}", id.unwrap_or("none"))),
    )?;
    Ok(())
}

pub fn get_active_mode(project_dir: Option<PathBuf>) -> Result<Option<ModeConfig>> {
    let config = get_config(project_dir)?;
    Ok(config
        .active_mode
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

/// Migrate `config.json` → `ship.toml` in-place (no-op if already migrated).
pub fn migrate_json_config_file(project_dir: &Path) -> Result<bool> {
    let json_path = project_dir.join("config.json");
    let primary_path = project_dir.join(PRIMARY_CONFIG_FILE);
    let secondary_path = project_dir.join(SECONDARY_CONFIG_FILE);
    let legacy_path = project_dir.join(LEGACY_CONFIG_FILE);
    if !json_path.exists()
        || primary_path.exists()
        || secondary_path.exists()
        || legacy_path.exists()
    {
        return Ok(false);
    }
    let config = migrate_json_config(&json_path)?;
    save_config(&config, Some(project_dir.to_path_buf()))?;
    fs::remove_file(json_path)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::init_project;
    use std::collections::HashMap;
    use tempfile::tempdir;

    // ── MCP server CRUD ────────────────────────────────────────────────────────

    #[test]
    fn add_and_list_mcp_server() -> Result<()> {
        let tmp = tempdir()?;
        let dir = init_project(tmp.path().to_path_buf())?;
        let server = McpServerConfig {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@mcp/github".to_string()],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        };
        add_mcp_server(Some(dir.clone()), server)?;
        let servers = list_mcp_servers(Some(dir))?;
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].id, "github");
        Ok(())
    }

    #[test]
    fn remove_mcp_server_works() -> Result<()> {
        let tmp = tempdir()?;
        let dir = init_project(tmp.path().to_path_buf())?;
        let server = McpServerConfig {
            id: "to-remove".to_string(),
            name: "Remove Me".to_string(),
            command: "rm".to_string(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        };
        add_mcp_server(Some(dir.clone()), server)?;
        remove_mcp_server(Some(dir.clone()), "to-remove")?;
        let servers = list_mcp_servers(Some(dir))?;
        assert!(servers.is_empty());
        Ok(())
    }

    #[test]
    fn duplicate_mcp_server_rejected() -> Result<()> {
        let tmp = tempdir()?;
        let dir = init_project(tmp.path().to_path_buf())?;
        let server = McpServerConfig {
            id: "dup".to_string(),
            name: "Dup".to_string(),
            command: "x".to_string(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        };
        add_mcp_server(Some(dir.clone()), server.clone())?;
        let result = add_mcp_server(Some(dir), server);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
        Ok(())
    }

    // ── Hook CRUD ──────────────────────────────────────────────────────────────

    #[test]
    fn add_and_list_hook() -> Result<()> {
        let tmp = tempdir()?;
        let dir = init_project(tmp.path().to_path_buf())?;
        let hook = HookConfig {
            id: "log-tools".to_string(),
            trigger: HookTrigger::PostToolUse,
            matcher: Some("Bash".to_string()),
            command: "echo 'tool used'".to_string(),
        };
        add_hook(Some(dir.clone()), hook)?;
        let hooks = list_hooks(Some(dir))?;
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].id, "log-tools");
        assert_eq!(hooks[0].trigger, HookTrigger::PostToolUse);
        Ok(())
    }

    #[test]
    fn remove_hook_works() -> Result<()> {
        let tmp = tempdir()?;
        let dir = init_project(tmp.path().to_path_buf())?;
        let hook = HookConfig {
            id: "bye".to_string(),
            trigger: HookTrigger::Stop,
            matcher: None,
            command: "echo bye".to_string(),
        };
        add_hook(Some(dir.clone()), hook)?;
        remove_hook(Some(dir.clone()), "bye")?;
        assert!(list_hooks(Some(dir))?.is_empty());
        Ok(())
    }

    #[test]
    fn duplicate_hook_rejected() -> Result<()> {
        let tmp = tempdir()?;
        let dir = init_project(tmp.path().to_path_buf())?;
        let hook = HookConfig {
            id: "dup".to_string(),
            trigger: HookTrigger::PreToolUse,
            matcher: None,
            command: "x".to_string(),
        };
        add_hook(Some(dir.clone()), hook.clone())?;
        let result = add_hook(Some(dir), hook);
        assert!(result.is_err());
        Ok(())
    }

    // ── Mode CRUD ──────────────────────────────────────────────────────────────

    #[test]
    fn add_mode_and_set_active() -> Result<()> {
        let tmp = tempdir()?;
        let dir = init_project(tmp.path().to_path_buf())?;
        let mode = ModeConfig {
            id: "dev".to_string(),
            name: "Development".to_string(),
            ..Default::default()
        };
        add_mode(Some(dir.clone()), mode)?;
        set_active_mode(Some(dir.clone()), Some("dev"))?;
        let active = get_active_mode(Some(dir))?;
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, "dev");
        Ok(())
    }

    #[test]
    fn remove_active_mode_clears_active() -> Result<()> {
        let tmp = tempdir()?;
        let dir = init_project(tmp.path().to_path_buf())?;
        add_mode(
            Some(dir.clone()),
            ModeConfig {
                id: "x".to_string(),
                name: "X".to_string(),
                ..Default::default()
            },
        )?;
        set_active_mode(Some(dir.clone()), Some("x"))?;
        remove_mode(Some(dir.clone()), "x")?;
        let cfg = get_config(Some(dir))?;
        assert!(
            cfg.active_mode.is_none(),
            "active_mode should be cleared when mode removed"
        );
        Ok(())
    }

    #[test]
    fn set_nonexistent_mode_rejected() -> Result<()> {
        let tmp = tempdir()?;
        let dir = init_project(tmp.path().to_path_buf())?;
        let result = set_active_mode(Some(dir), Some("ghost"));
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn mode_with_permissions_round_trips() -> Result<()> {
        let tmp = tempdir()?;
        let dir = init_project(tmp.path().to_path_buf())?;
        let mode = ModeConfig {
            id: "restricted".to_string(),
            name: "Restricted".to_string(),
            permissions: PermissionConfig {
                allow: vec!["mcp__ship__*".to_string()],
                deny: vec!["Bash".to_string()],
            },
            ..Default::default()
        };
        add_mode(Some(dir.clone()), mode)?;
        let cfg = get_config(Some(dir))?;
        let saved = cfg.modes.iter().find(|m| m.id == "restricted").unwrap();
        assert_eq!(saved.permissions.allow, vec!["mcp__ship__*"]);
        assert_eq!(saved.permissions.deny, vec!["Bash"]);
        Ok(())
    }

    #[test]
    fn mcp_server_type_serialization_round_trips() -> Result<()> {
        let tmp = tempdir()?;
        let dir = init_project(tmp.path().to_path_buf())?;
        let http_server = McpServerConfig {
            id: "http-svc".to_string(),
            name: "HTTP".to_string(),
            command: String::new(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Http,
            url: Some("http://localhost:8080".to_string()),
            disabled: false,
            timeout_secs: Some(30),
        };
        add_mcp_server(Some(dir.clone()), http_server)?;
        let servers = list_mcp_servers(Some(dir))?;
        assert_eq!(servers[0].server_type, McpServerType::Http);
        assert_eq!(servers[0].url.as_deref(), Some("http://localhost:8080"));
        assert_eq!(servers[0].timeout_secs, Some(30));
        Ok(())
    }
}
