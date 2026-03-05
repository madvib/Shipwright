use crate::fs_util::write_atomic;
use crate::project::{SHIP_DIR_NAME, get_global_dir, ship_dir_from_path};
use crate::{EventAction, EventEntity, append_event};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
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

/// Mode-scoped tool permission overrides.
/// These overlay canonical `.ship/agents/permissions.toml` `tools.allow/deny`
/// when a mode is active.
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
    /// Legacy alias for global instruction skill IDs.
    /// Prompts are modeled as skills in Ship.
    #[serde(default)]
    pub prompts: Vec<String>,
    /// Context files/folders to preload for agents.
    #[serde(default)]
    pub context: Vec<String>,
}

fn is_agent_layer_empty(config: &AgentLayerConfig) -> bool {
    config.skills.is_empty() && config.prompts.is_empty() && config.context.is_empty()
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct LegacyAgentsConfigFile {
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub active_mode: Option<String>,
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
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
    /// Skill IDs active in this mode (empty = all)
    #[serde(default)]
    pub skills: Vec<String>,
    /// Rule IDs active in this mode (empty = all). Rule IDs map to rule file
    /// names without the `.md` suffix.
    #[serde(default)]
    pub rules: Vec<String>,
    /// Legacy field name for mode-level instruction skill selection.
    /// This now references a skill ID.
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
    #[serde(default)]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modes: Vec<ModeConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_mode: Option<String>,
    /// Global hooks applied regardless of active mode
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hooks: Vec<HookConfig>,
    #[serde(default, skip_serializing_if = "is_agent_layer_empty")]
    pub agent: AgentLayerConfig,
    /// Which agent providers to generate config for on branch checkout.
    /// Alpha: "claude" | "gemini" | "codex". Defaults to ["claude"].
    #[serde(default = "default_providers", skip_serializing_if = "Vec::is_empty")]
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
    let is_project = project_dir.is_some();
    let config_dir = match project_dir {
        Some(dir) => dir,
        None => get_global_dir()?,
    };

    // Prefer ship.toml, then shipwright.toml, then legacy config.toml.
    let primary_path = config_dir.join(PRIMARY_CONFIG_FILE);
    let secondary_path = config_dir.join(SECONDARY_CONFIG_FILE);
    let legacy_path = config_dir.join(LEGACY_CONFIG_FILE);
    let json_path = config_dir.join("config.json");

    let mut config = None;
    for path in [&primary_path, &secondary_path, &legacy_path] {
        if !path.exists() {
            continue;
        }
        let content = fs::read_to_string(path)?;
        config = Some(toml::from_str(&content)?);
        break;
    }

    let mut config = if let Some(config) = config {
        config
    } else if json_path.exists() {
        // Legacy JSON config — read what we can and migrate.
        migrate_json_config(&json_path).unwrap_or_default()
    } else {
        ProjectConfig::default()
    };

    if is_project {
        if let Some((providers, active_mode, hooks)) = get_runtime_settings(&config_dir)? {
            config.providers = providers;
            config.active_mode = active_mode;
            config.hooks = hooks;
        } else if let Some(legacy) = get_legacy_agents_config(&config_dir)? {
            // One-time compatibility path: bootstrap SQLite runtime settings from
            // legacy .ship/agents/config.toml if present.
            config.providers = legacy.providers;
            config.active_mode = legacy.active_mode;
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
    servers.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(servers)
}

fn save_mcp_config(ship_dir: &Path, servers: &[McpServerConfig]) -> Result<()> {
    let path = crate::project::mcp_config_path(ship_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut by_id: HashMap<String, McpServerConfig> = HashMap::new();
    for server in servers {
        let mut cloned = server.clone();
        cloned.id.clear();
        by_id.insert(server.id.clone(), cloned);
    }

    let raw = McpConfig {
        mcp: McpSection { servers: by_id },
    };
    write_atomic(&path, toml::to_string_pretty(&raw)?)?;
    Ok(())
}

const ARTIFACT_KIND_MCP: &str = "mcp";
const ARTIFACT_KIND_SKILL: &str = "skill";
const ARTIFACT_KIND_RULE: &str = "rule";

fn stable_hash(value: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn normalize_rule_external_id(id: &str) -> String {
    let normalized = id.trim().trim_end_matches(".md");
    if let Some((prefix, rest)) = normalized.split_once('-')
        && !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_digit())
    {
        return rest.to_string();
    }
    normalized.to_string()
}

fn sync_agent_artifact_registry(ship_dir: &Path) -> Result<()> {
    for skill in crate::skill::list_skills(ship_dir)? {
        let path = crate::project::skills_dir(ship_dir)
            .join(&skill.id)
            .join("SKILL.md");
        let digest = stable_hash(&skill.content);
        crate::state_db::upsert_agent_artifact_registry_db(
            ship_dir,
            ARTIFACT_KIND_SKILL,
            &skill.id,
            &skill.name,
            &path.to_string_lossy(),
            &digest,
        )?;
    }

    for rule in crate::rule::list_rules(ship_dir.to_path_buf())? {
        let external_id = normalize_rule_external_id(&rule.file_name);
        let digest = stable_hash(&rule.content);
        crate::state_db::upsert_agent_artifact_registry_db(
            ship_dir,
            ARTIFACT_KIND_RULE,
            &external_id,
            &rule.file_name,
            &rule.path,
            &digest,
        )?;
    }

    for server in get_mcp_config(ship_dir)? {
        let digest = stable_hash(&toml::to_string(&server)?);
        crate::state_db::upsert_agent_artifact_registry_db(
            ship_dir,
            ARTIFACT_KIND_MCP,
            &server.id,
            &server.name,
            &crate::project::mcp_config_path(ship_dir).to_string_lossy(),
            &digest,
        )?;
    }

    Ok(())
}

fn resolve_refs_to_external_ids(
    ship_dir: &Path,
    kind: &str,
    refs: &[String],
) -> Result<Vec<String>> {
    let mut resolved = Vec::new();
    let mut seen = HashSet::new();
    for reference in refs {
        if let Some(entry) =
            crate::state_db::get_agent_artifact_registry_by_uuid_db(ship_dir, kind, reference)?
        {
            let external_id = if kind == ARTIFACT_KIND_RULE {
                normalize_rule_external_id(&entry.external_id)
            } else {
                entry.external_id
            };
            if seen.insert(external_id.clone()) {
                resolved.push(external_id);
            }
            continue;
        }

        let lookup = if kind == ARTIFACT_KIND_RULE {
            normalize_rule_external_id(reference)
        } else {
            reference.clone()
        };
        if let Some(entry) =
            crate::state_db::get_agent_artifact_registry_by_external_id_db(ship_dir, kind, &lookup)?
            && seen.insert(entry.external_id.clone())
        {
            resolved.push(entry.external_id);
        }
    }
    Ok(resolved)
}

fn resolve_external_ids_to_refs(
    ship_dir: &Path,
    kind: &str,
    external_ids: &[String],
) -> Result<Vec<String>> {
    let mut refs = Vec::new();
    let mut seen = HashSet::new();
    for id in external_ids {
        let lookup = if kind == ARTIFACT_KIND_RULE {
            normalize_rule_external_id(id)
        } else {
            id.clone()
        };
        if let Some(entry) =
            crate::state_db::get_agent_artifact_registry_by_external_id_db(ship_dir, kind, &lookup)?
            && seen.insert(entry.uuid.clone())
        {
            refs.push(entry.uuid);
        }
    }
    Ok(refs)
}

fn get_modes_config(ship_dir: &Path) -> Result<Vec<ModeConfig>> {
    sync_agent_artifact_registry(ship_dir)?;

    let mode_rows = crate::state_db::list_agent_modes_db(ship_dir)?;
    let mut modes = Vec::new();
    for row in mode_rows {
        let active_tools: Vec<String> =
            serde_json::from_str(&row.active_tools_json).unwrap_or_default();
        let mcp_refs: Vec<String> = serde_json::from_str(&row.mcp_refs_json).unwrap_or_default();
        let skill_refs: Vec<String> =
            serde_json::from_str(&row.skill_refs_json).unwrap_or_default();
        let rule_refs: Vec<String> = serde_json::from_str(&row.rule_refs_json).unwrap_or_default();
        let hooks: Vec<HookConfig> = serde_json::from_str(&row.hooks_json).unwrap_or_default();
        let permissions: PermissionConfig =
            serde_json::from_str(&row.permissions_json).unwrap_or_default();
        let target_agents: Vec<String> =
            serde_json::from_str(&row.target_agents_json).unwrap_or_default();

        modes.push(ModeConfig {
            id: row.id,
            name: row.name,
            description: row.description,
            active_tools,
            mcp_servers: resolve_refs_to_external_ids(ship_dir, ARTIFACT_KIND_MCP, &mcp_refs)?,
            skills: resolve_refs_to_external_ids(ship_dir, ARTIFACT_KIND_SKILL, &skill_refs)?,
            rules: resolve_refs_to_external_ids(ship_dir, ARTIFACT_KIND_RULE, &rule_refs)?,
            prompt_id: row.prompt_id,
            hooks,
            permissions,
            target_agents,
        });
    }
    modes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(modes)
}

fn save_modes_config(ship_dir: &Path, modes: &[ModeConfig]) -> Result<()> {
    sync_agent_artifact_registry(ship_dir)?;

    let existing_ids: HashSet<String> = crate::state_db::list_agent_modes_db(ship_dir)?
        .into_iter()
        .map(|row| row.id)
        .collect();
    let mut next_ids = HashSet::new();

    for mode in modes {
        next_ids.insert(mode.id.clone());
        let db_mode = crate::state_db::AgentModeDb {
            id: mode.id.clone(),
            name: mode.name.clone(),
            description: mode.description.clone(),
            active_tools_json: serde_json::to_string(&mode.active_tools)?,
            mcp_refs_json: serde_json::to_string(&resolve_external_ids_to_refs(
                ship_dir,
                ARTIFACT_KIND_MCP,
                &mode.mcp_servers,
            )?)?,
            skill_refs_json: serde_json::to_string(&resolve_external_ids_to_refs(
                ship_dir,
                ARTIFACT_KIND_SKILL,
                &mode.skills,
            )?)?,
            rule_refs_json: serde_json::to_string(&resolve_external_ids_to_refs(
                ship_dir,
                ARTIFACT_KIND_RULE,
                &mode.rules,
            )?)?,
            prompt_id: mode.prompt_id.clone(),
            hooks_json: serde_json::to_string(&mode.hooks)?,
            permissions_json: serde_json::to_string(&mode.permissions)?,
            target_agents_json: serde_json::to_string(&mode.target_agents)?,
        };
        crate::state_db::upsert_agent_mode_db(ship_dir, &db_mode)?;
    }

    for id in existing_ids {
        if !next_ids.contains(&id) {
            crate::state_db::delete_agent_mode_db(ship_dir, &id)?;
        }
    }

    Ok(())
}

fn get_legacy_agents_config(ship_dir: &Path) -> Result<Option<LegacyAgentsConfigFile>> {
    let path = legacy_agents_config_path(ship_dir);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    let parsed: LegacyAgentsConfigFile = toml::from_str(&content)?;
    Ok(Some(parsed))
}

fn remove_legacy_agents_config(ship_dir: &Path) -> Result<()> {
    let path = legacy_agents_config_path(ship_dir);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn legacy_agents_config_path(ship_dir: &Path) -> PathBuf {
    ship_dir.join("agents").join("config.toml")
}

fn get_runtime_settings(
    ship_dir: &Path,
) -> Result<Option<(Vec<String>, Option<String>, Vec<HookConfig>)>> {
    let Some(raw) = crate::state_db::get_agent_runtime_settings_db(ship_dir)? else {
        return Ok(None);
    };

    let hooks: Vec<HookConfig> = serde_json::from_str(&raw.hooks_json).unwrap_or_default();
    Ok(Some((raw.providers, raw.active_mode, hooks)))
}

fn save_runtime_settings(ship_dir: &Path, config: &ProjectConfig) -> Result<()> {
    let hooks_json = serde_json::to_string(&config.hooks)?;
    crate::state_db::set_agent_runtime_settings_db(
        ship_dir,
        &config.providers,
        config.active_mode.as_deref(),
        &hooks_json,
    )
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
        write_atomic(&path, toml_str)?;
        return Ok(());
    }

    // Project runtime settings + mode bindings live in SQLite.
    save_runtime_settings(&config_dir, config)?;
    // File-backed catalog state (mcp/skills/rules) is indexed into SQLite for mode refs.
    save_mcp_config(&config_dir, &config.mcp_servers)?;
    save_modes_config(&config_dir, &config.modes)?;

    // Keep ship.toml focused on core project/workflow config.
    let mut core = config.clone();
    core.modes.clear();
    core.mcp_servers.clear();
    core.active_mode = None;
    core.hooks.clear();
    core.agent = AgentLayerConfig::default();
    core.providers.clear();

    let toml_str = toml::to_string_pretty(&core)?;
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
        if let Err(error) = crate::agents::export::sync_active_mode(dir) {
            eprintln!("[ship] warning: active mode sync failed: {}", error);
        }
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

    #[test]
    fn save_config_keeps_ship_toml_free_of_agent_sections() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = tmp.path().join(".ship");
        fs::create_dir_all(&ship_dir)?;

        let mut config = ProjectConfig::default();
        config.providers = vec!["claude".to_string(), "codex".to_string()];
        config.active_mode = Some("planning".to_string());
        config.hooks = vec![HookConfig {
            id: "audit".to_string(),
            trigger: HookTrigger::PostToolUse,
            matcher: Some("Bash".to_string()),
            command: "echo audit".to_string(),
        }];
        config.agent = AgentLayerConfig {
            skills: vec!["task-policy".to_string()],
            prompts: vec![],
            context: vec!["project/README.md".to_string()],
        };
        config.modes = vec![ModeConfig {
            id: "planning".to_string(),
            name: "Planning".to_string(),
            ..Default::default()
        }];
        config.mcp_servers = vec![McpServerConfig {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-github".to_string(),
            ],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: Some(30),
        }];

        save_config(&config, Some(ship_dir.clone()))?;

        let ship_toml = fs::read_to_string(ship_dir.join("ship.toml"))?;
        assert!(
            !ship_toml.contains("[[modes]]"),
            "ship.toml must not persist mode definitions"
        );
        assert!(
            !ship_toml.contains("[[mcp_servers]]"),
            "ship.toml must not persist MCP servers"
        );
        assert!(
            !ship_toml.contains("[agent]"),
            "ship.toml must not persist agent block"
        );
        assert!(
            !ship_toml.contains("providers ="),
            "ship.toml must not persist providers"
        );
        assert!(
            !ship_toml.contains("active_mode ="),
            "ship.toml must not persist active_mode"
        );

        assert!(
            !ship_dir.join("agents").join("config.toml").exists(),
            "legacy agents/config.toml should not be written"
        );

        let runtime_settings = crate::state_db::get_agent_runtime_settings_db(&ship_dir)?
            .expect("expected runtime settings row");
        assert_eq!(
            runtime_settings.providers,
            vec!["claude".to_string(), "codex".to_string()]
        );
        assert_eq!(runtime_settings.active_mode.as_deref(), Some("planning"));
        assert!(runtime_settings.hooks_json.contains("\"audit\""));

        let mode_rows = crate::state_db::list_agent_modes_db(&ship_dir)?;
        assert_eq!(mode_rows.len(), 1);
        assert_eq!(mode_rows[0].id, "planning");

        let mcp_cfg = fs::read_to_string(ship_dir.join("agents").join("mcp.toml"))?;
        assert!(mcp_cfg.contains("[mcp.servers.github]"));
        Ok(())
    }

    #[test]
    fn get_config_round_trips_agent_sidecars() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = tmp.path().join(".ship");
        fs::create_dir_all(&ship_dir)?;

        let mut config = ProjectConfig::default();
        config.providers = vec!["gemini".to_string()];
        config.active_mode = Some("focus".to_string());
        config.modes = vec![ModeConfig {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            mcp_servers: vec!["github".to_string()],
            ..Default::default()
        }];
        config.mcp_servers = vec![McpServerConfig {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            command: "npx".to_string(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        }];

        save_config(&config, Some(ship_dir.clone()))?;
        let loaded = get_config(Some(ship_dir))?;

        assert_eq!(loaded.providers, vec!["gemini".to_string()]);
        assert_eq!(loaded.active_mode.as_deref(), Some("focus"));
        assert!(loaded.agent.skills.is_empty());
        assert_eq!(loaded.modes.len(), 1);
        assert_eq!(loaded.modes[0].id, "focus");
        assert_eq!(loaded.mcp_servers.len(), 1);
        assert_eq!(loaded.mcp_servers[0].id, "github");
        Ok(())
    }
}
