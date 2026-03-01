use crate::config::{
    HookConfig, HookTrigger, McpServerConfig, McpServerType, PermissionConfig,
    get_config, get_effective_config,
};
use crate::prompt::Prompt;
use crate::prompt::get_prompt;
use crate::skill::list_effective_skills;
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ─── Model registry ───────────────────────────────────────────────────────────

/// Static model entry — all `&'static str` so it can live in a `const` array.
#[derive(Debug, Clone, Copy)]
pub struct StaticModelInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub context_window: u32,
    pub recommended: bool,
}

/// Serializable model info for Tauri/MCP.
#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub context_window: u32,
    pub recommended: bool,
}

impl ModelInfo {
    fn from_static(m: &StaticModelInfo, provider_id: &str) -> Self {
        ModelInfo {
            id: m.id.to_string(),
            name: m.name.to_string(),
            provider_id: provider_id.to_string(),
            context_window: m.context_window,
            recommended: m.recommended,
        }
    }
}

// ─── Provider registry ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Toml,
}

/// How Ship marks its own managed entries so it can remove them on teardown.
#[derive(Debug, Clone, Copy)]
pub enum ManagedMarker {
    /// Embed `"_ship": {"managed": true}` in each JSON entry.
    Inline,
    /// JSON doesn't support arbitrary metadata — track in mcp_managed_state only.
    StateFileOnly,
}

/// Where to write the active prompt / system instructions.
#[derive(Debug, Clone, Copy)]
pub enum PromptOutput {
    /// `CLAUDE.md` at project root.
    ClaudeMd,
    /// `GEMINI.md` at project root.
    GeminiMd,
    /// `AGENTS.md` at project root — Codex, Roo Code, Amp, Goose all read this.
    AgentsMd,
    None,
}

/// Where to write skill content for native agent skill directories.
#[derive(Debug, Clone, Copy)]
pub enum SkillsOutput {
    /// `.claude/skills/<id>/SKILL.md` — Claude Code native skills (.claude/commands/ is deprecated)
    ClaudeSkills,
    /// `.gemini/skills/<id>/SKILL.md` — Gemini CLI
    AgentSkills,
    /// `.agents/skills/<id>/SKILL.md` — OpenAI Codex
    CodexSkills,
    None,
}

#[derive(Debug)]
pub struct ProviderDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    /// Binary name for PATH detection.
    pub binary: &'static str,
    /// Config file path relative to project root.
    pub project_config: &'static str,
    /// Config file path relative to home directory.
    pub global_config: &'static str,
    pub config_format: ConfigFormat,
    /// Key under which MCP servers live (e.g. "mcpServers" or "mcp_servers").
    pub mcp_key: &'static str,
    /// HTTP/SSE URL field name ("url" or "httpUrl").
    pub http_url_field: &'static str,
    /// Whether to emit a `"type": "stdio"` field in stdio entries.
    pub emit_type_field: bool,
    pub managed_marker: ManagedMarker,
    pub prompt_output: PromptOutput,
    pub skills_output: SkillsOutput,
    /// Known models for this provider (static list; first `recommended` is the default).
    pub models: &'static [StaticModelInfo],
}

const CLAUDE_MODELS: &[StaticModelInfo] = &[
    StaticModelInfo { id: "claude-sonnet-4-6", name: "Claude Sonnet 4.6", context_window: 200_000, recommended: true },
    StaticModelInfo { id: "claude-opus-4-6", name: "Claude Opus 4.6", context_window: 200_000, recommended: false },
    StaticModelInfo { id: "claude-haiku-4-5", name: "Claude Haiku 4.5", context_window: 200_000, recommended: false },
];

const GEMINI_MODELS: &[StaticModelInfo] = &[
    StaticModelInfo { id: "gemini-2.5-pro", name: "Gemini 2.5 Pro", context_window: 1_000_000, recommended: true },
    StaticModelInfo { id: "gemini-2.0-flash", name: "Gemini 2.0 Flash", context_window: 1_000_000, recommended: false },
    StaticModelInfo { id: "gemini-2.0-flash-lite", name: "Gemini 2.0 Flash Lite", context_window: 1_000_000, recommended: false },
];

const CODEX_MODELS: &[StaticModelInfo] = &[
    StaticModelInfo { id: "gpt-4o", name: "GPT-4o", context_window: 128_000, recommended: true },
    StaticModelInfo { id: "gpt-4o-mini", name: "GPT-4o Mini", context_window: 128_000, recommended: false },
    StaticModelInfo { id: "o1", name: "o1", context_window: 200_000, recommended: false },
];

pub const PROVIDERS: &[ProviderDescriptor] = &[
    ProviderDescriptor {
        id: "claude",
        name: "Claude Code",
        binary: "claude",
        project_config: ".mcp.json",
        global_config: ".claude.json",
        config_format: ConfigFormat::Json,
        mcp_key: "mcpServers",
        http_url_field: "url",
        emit_type_field: true,
        managed_marker: ManagedMarker::Inline,
        prompt_output: PromptOutput::ClaudeMd,
        skills_output: SkillsOutput::ClaudeSkills,
        models: CLAUDE_MODELS,
    },
    ProviderDescriptor {
        id: "gemini",
        name: "Gemini CLI",
        binary: "gemini",
        project_config: ".gemini/settings.json",
        global_config: ".gemini/settings.json",
        config_format: ConfigFormat::Json,
        mcp_key: "mcpServers",
        http_url_field: "httpUrl",
        emit_type_field: false,
        managed_marker: ManagedMarker::Inline,
        prompt_output: PromptOutput::GeminiMd,
        skills_output: SkillsOutput::AgentSkills,
        models: GEMINI_MODELS,
    },
    ProviderDescriptor {
        id: "codex",
        name: "Codex CLI",
        binary: "codex",
        project_config: ".codex/config.toml",
        global_config: ".codex/config.toml",
        config_format: ConfigFormat::Toml,
        mcp_key: "mcp_servers",
        http_url_field: "url",
        emit_type_field: false,
        managed_marker: ManagedMarker::StateFileOnly,
        prompt_output: PromptOutput::AgentsMd,
        skills_output: SkillsOutput::CodexSkills,
        models: CODEX_MODELS,
    },
];

pub fn get_provider(id: &str) -> Option<&'static ProviderDescriptor> {
    PROVIDERS.iter().find(|p| p.id == id)
}

/// Serializable projection of `ProviderDescriptor` for MCP tools and Tauri commands.
#[derive(Serialize, Deserialize, Clone, specta::Type)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub binary: String,
    pub project_config: String,
    pub config_format: String,
    pub prompt_output: String,
    pub skills_output: String,
    /// True when this provider is listed in the project's `providers` field.
    pub enabled: bool,
    /// True when the provider's binary is found in PATH.
    pub installed: bool,
    /// Version string from `<binary> --version`, if the binary is installed.
    pub version: Option<String>,
    /// Known models for this provider.
    pub models: Vec<ModelInfo>,
}

/// Returns true if `binary` is found in the system PATH.
pub fn detect_binary(binary: &str) -> bool {
    // Use `which` on Unix; fall back to manual PATH scan.
    std::process::Command::new("which")
        .arg(binary)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or_else(|_| {
            std::env::var_os("PATH")
                .into_iter()
                .flat_map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
                .any(|dir| dir.join(binary).is_file())
        })
}

/// Returns the version string from `<binary> --version` (first line), or None.
pub fn detect_version(binary: &str) -> Option<String> {
    let out = std::process::Command::new(binary)
        .arg("--version")
        .output()
        .ok()?;
    if !out.status.success() && out.stdout.is_empty() {
        return None;
    }
    let text = String::from_utf8_lossy(if out.stdout.is_empty() { &out.stderr } else { &out.stdout });
    text.lines().next().map(|l| l.trim().to_string()).filter(|s| !s.is_empty())
}

fn provider_info(d: &ProviderDescriptor, enabled: bool) -> ProviderInfo {
    let installed = detect_binary(d.binary);
    let version = if installed { detect_version(d.binary) } else { None };
    ProviderInfo {
        id: d.id.to_string(),
        name: d.name.to_string(),
        binary: d.binary.to_string(),
        project_config: d.project_config.to_string(),
        config_format: match d.config_format {
            ConfigFormat::Json => "json".to_string(),
            ConfigFormat::Toml => "toml".to_string(),
        },
        prompt_output: match d.prompt_output {
            PromptOutput::ClaudeMd => "claude-md".to_string(),
            PromptOutput::GeminiMd => "gemini-md".to_string(),
            PromptOutput::AgentsMd => "agents-md".to_string(),
            PromptOutput::None => "none".to_string(),
        },
        skills_output: match d.skills_output {
            SkillsOutput::ClaudeSkills => "claude-skills".to_string(),
            SkillsOutput::AgentSkills => "agent-skills".to_string(),
            SkillsOutput::CodexSkills => "codex-skills".to_string(),
            SkillsOutput::None => "none".to_string(),
        },
        enabled,
        installed,
        version,
        models: d.models.iter().map(|m| ModelInfo::from_static(m, d.id)).collect(),
    }
}

/// Return all registered providers, each annotated with enabled + installed status.
pub fn list_providers(project_dir: &std::path::Path) -> anyhow::Result<Vec<ProviderInfo>> {
    let config = get_config(Some(project_dir.to_path_buf()))?;
    let enabled: std::collections::HashSet<&str> =
        config.providers.iter().map(|s| s.as_str()).collect();
    Ok(PROVIDERS
        .iter()
        .map(|d| provider_info(d, enabled.contains(d.id)))
        .collect())
}

/// Add a provider to the project's enabled list (idempotent).
/// Returns `true` if the list was changed.
pub fn enable_provider(project_dir: &std::path::Path, provider_id: &str) -> Result<bool> {
    require_provider(provider_id)?;
    let mut config = get_config(Some(project_dir.to_path_buf()))?;
    if config.providers.iter().any(|p| p == provider_id) {
        return Ok(false);
    }
    config.providers.push(provider_id.to_string());
    crate::config::save_config(&config, Some(project_dir.to_path_buf()))?;
    Ok(true)
}

/// Remove a provider from the project's enabled list.
/// Returns `true` if the list was changed.
pub fn disable_provider(project_dir: &std::path::Path, provider_id: &str) -> Result<bool> {
    require_provider(provider_id)?;
    let mut config = get_config(Some(project_dir.to_path_buf()))?;
    let before = config.providers.len();
    config.providers.retain(|p| p != provider_id);
    if config.providers.len() == before {
        return Ok(false);
    }
    crate::config::save_config(&config, Some(project_dir.to_path_buf()))?;
    Ok(true)
}

/// Detect which known providers are installed in PATH and enable them.
/// Intended for use in `ship init`. Skips already-enabled providers.
/// Returns the list of provider IDs that were newly enabled.
pub fn autodetect_providers(project_dir: &std::path::Path) -> Result<Vec<String>> {
    let mut newly_enabled = Vec::new();
    for d in PROVIDERS {
        if detect_binary(d.binary) {
            if enable_provider(project_dir, d.id)? {
                newly_enabled.push(d.id.to_string());
            }
        }
    }
    Ok(newly_enabled)
}

/// Return models for a specific provider by ID.
pub fn list_models(provider_id: &str) -> Result<Vec<ModelInfo>> {
    let d = require_provider(provider_id)?;
    Ok(d.models.iter().map(|m| ModelInfo::from_static(m, d.id)).collect())
}

fn require_provider(id: &str) -> Result<&'static ProviderDescriptor> {
    get_provider(id).ok_or_else(|| {
        let known: Vec<&str> = PROVIDERS.iter().map(|p| p.id).collect();
        anyhow!("Unknown provider '{}'. Known: {}", id, known.join(", "))
    })
}

// ─── Managed state ────────────────────────────────────────────────────────────

/// In-memory view of which server IDs Ship wrote into each provider's config.
/// Backed by the project SQLite DB (`managed_mcp_state` table).
#[derive(Debug, Default)]
struct ManagedState {
    providers: HashMap<String, ToolState>,
}

#[derive(Debug, Default, Clone)]
struct ToolState {
    managed_servers: Vec<String>,
    last_mode: Option<String>,
}

fn load_managed_state(project_dir: &Path) -> ManagedState {
    let mut state = ManagedState::default();
    for p in PROVIDERS {
        if let Ok((ids, last_mode)) = crate::state_db::get_managed_state_db(project_dir, p.id) {
            if !ids.is_empty() || last_mode.is_some() {
                state
                    .providers
                    .insert(p.id.to_string(), ToolState { managed_servers: ids, last_mode });
            }
        }
    }
    state
}

fn save_managed_state(project_dir: &Path, state: &ManagedState) -> Result<()> {
    for (provider, tool_state) in &state.providers {
        // Non-fatal: DB writes fail gracefully when called from async context.
        let _ = crate::state_db::set_managed_state_db(
            project_dir,
            provider,
            &tool_state.managed_servers,
            tool_state.last_mode.as_deref(),
        );
    }
    Ok(())
}

// ─── Sync payload ─────────────────────────────────────────────────────────────

pub struct SyncPayload {
    pub servers: Vec<McpServerConfig>,
    pub prompt: Option<Prompt>,
    pub hooks: Vec<HookConfig>,
    pub permissions: PermissionConfig,
    pub active_mode_id: Option<String>,
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Write a context file (CLAUDE.md, GEMINI.md, Codex instructions, etc.) for the given provider.
///
/// Called by the git module after building provider-agnostic Markdown content.
/// Each provider has a specific destination:
/// - Claude  → `CLAUDE.md` at project root
/// - Gemini  → `GEMINI.md` at project root
/// - Codex / Roo / Amp / Goose → `AGENTS.md` at project root
/// - Unknown provider / `PromptOutput::None` → no-op
pub fn write_context(project_root: &Path, provider_id: &str, content: &str) -> Result<()> {
    let desc = match get_provider(provider_id) {
        Some(d) => d,
        None => return Ok(()),
    };
    match desc.prompt_output {
        PromptOutput::ClaudeMd => {
            let path = project_root.join("CLAUDE.md");
            crate::fs_util::write_atomic(&path, content)?;
        }
        PromptOutput::GeminiMd => {
            let path = project_root.join("GEMINI.md");
            crate::fs_util::write_atomic(&path, content)?;
        }
        PromptOutput::AgentsMd => {
            let path = project_root.join("AGENTS.md");
            crate::fs_util::write_atomic(&path, content)?;
        }
        PromptOutput::None => {}
    }
    Ok(())
}

/// Export the active mode (or global config) to the specified provider.
pub fn export_to(project_dir: PathBuf, target: &str) -> Result<()> {
    export_to_inner(project_dir, target, None)
}

/// Like `export_to` but restricts project MCP servers to those whose IDs appear in
/// `server_filter`. Pass `None` to write all project servers (same as `export_to`).
pub fn export_to_filtered(
    project_dir: PathBuf,
    target: &str,
    server_filter: Option<&[String]>,
) -> Result<()> {
    export_to_inner(project_dir, target, server_filter)
}

fn export_to_inner(
    project_dir: PathBuf,
    target: &str,
    server_filter: Option<&[String]>,
) -> Result<()> {
    let desc = require_provider(target)?;
    let mut payload = build_payload(&project_dir)?;
    if let Some(ids) = server_filter {
        payload.servers.retain(|s| ids.contains(&s.id));
    }
    let project_root = project_dir
        .parent()
        .ok_or_else(|| anyhow!("Cannot determine project root from {:?}", project_dir))?;
    let mut state = load_managed_state(&project_dir);

    match desc.config_format {
        ConfigFormat::Json => export_json(desc, &project_dir, project_root, &payload, &mut state)?,
        ConfigFormat::Toml => export_toml(desc, &project_dir, project_root, &payload, &mut state)?,
    }

    // Skills output (provider-specific)
    match desc.skills_output {
        SkillsOutput::ClaudeSkills => export_skills_to_claude(&project_dir, project_root)?,
        SkillsOutput::AgentSkills => export_skills_to_dir(&project_dir, &project_root.join(".gemini").join("skills"))?,
        SkillsOutput::CodexSkills => export_skills_to_dir(&project_dir, &project_root.join(".agents").join("skills"))?,
        SkillsOutput::None => {}
    }

    // Hooks + permissions (Claude-only for now)
    if target == "claude"
        && (!payload.hooks.is_empty()
            || !payload.permissions.allow.is_empty()
            || !payload.permissions.deny.is_empty())
    {
        export_claude_settings(&payload.hooks, &payload.permissions)?;
    }

    save_managed_state(&project_dir, &state)?;
    Ok(())
}

/// Remove all Ship-generated config for the given provider.
pub fn teardown(project_dir: PathBuf, target: &str) -> Result<()> {
    let desc = require_provider(target)?;
    let project_root = project_dir
        .parent()
        .ok_or_else(|| anyhow!("Cannot determine project root from {:?}", project_dir))?;
    let mut state = load_managed_state(&project_dir);
    let tool_state = state.providers.entry(target.to_string()).or_default().clone();

    match desc.config_format {
        ConfigFormat::Json => {
            let config_path = project_root.join(desc.project_config);
            teardown_json(&config_path, desc.mcp_key, &desc.managed_marker, &tool_state)?;
        }
        ConfigFormat::Toml => {
            let config_path = project_root.join(desc.project_config);
            teardown_toml(&config_path, desc.mcp_key, &tool_state)?;
        }
    }

    // Remove prompt file if applicable
    match desc.prompt_output {
        PromptOutput::ClaudeMd => {
            let f = project_root.join("CLAUDE.md");
            if f.exists() {
                fs::remove_file(&f).with_context(|| format!("Failed to remove {}", f.display()))?;
            }
        }
        PromptOutput::GeminiMd => {
            let f = project_root.join("GEMINI.md");
            if f.exists() { fs::remove_file(&f).ok(); }
        }
        PromptOutput::AgentsMd => {
            let f = project_root.join("AGENTS.md");
            if f.exists() { fs::remove_file(&f).ok(); }
        }
        PromptOutput::None => {}
    }

    // Remove skill files written by Ship
    match desc.skills_output {
        SkillsOutput::ClaudeSkills => {
            remove_ship_managed_skill_dirs(&project_root.join(".claude").join("skills"));
        }
        SkillsOutput::AgentSkills => {
            remove_ship_managed_skill_dirs(&project_root.join(".gemini").join("skills"));
        }
        SkillsOutput::CodexSkills => {
            remove_ship_managed_skill_dirs(&project_root.join(".agents").join("skills"));
        }
        SkillsOutput::None => {}
    }

    // Clear managed state for this provider
    if let Some(ts) = state.providers.get_mut(target) {
        ts.managed_servers.clear();
        ts.last_mode = None;
    }
    save_managed_state(&project_dir, &state)?;
    Ok(())
}

/// Sync all target agents configured for the active mode.
pub fn sync_active_mode(project_dir: &Path) -> Result<Vec<String>> {
    let config = get_effective_config(Some(project_dir.to_path_buf()))?;
    let targets: Vec<String> = config
        .active_mode
        .as_ref()
        .and_then(|id| config.modes.iter().find(|m| &m.id == id))
        .map(|m| {
            if m.target_agents.is_empty() {
                vec!["claude".to_string()]
            } else {
                m.target_agents.clone()
            }
        })
        .unwrap_or_default();

    let mut synced = Vec::new();
    for target in &targets {
        export_to(project_dir.to_path_buf(), target)?;
        synced.push(target.clone());
    }
    Ok(synced)
}

/// Non-destructive import of MCP servers from a provider's existing config.
/// Returns count of newly-added servers.
pub fn import_from_claude(project_dir: PathBuf) -> Result<usize> {
    import_from_provider("claude", project_dir)
}

pub fn import_from_provider(provider_id: &str, project_dir: PathBuf) -> Result<usize> {
    let desc = require_provider(provider_id)?;
    if desc.config_format != ConfigFormat::Json {
        // TOML import not yet implemented — would need toml parsing path
        return Ok(0);
    }
    let path = home()?.join(desc.global_config);
    if !path.exists() {
        return Ok(0);
    }
    let root: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
    let Some(mcp_obj) = root.get(desc.mcp_key).and_then(|v| v.as_object()) else {
        return Ok(0);
    };

    let (managed, _) =
        crate::state_db::get_managed_state_db(&project_dir, provider_id).unwrap_or_default();

    let mut config = get_config(Some(project_dir.clone()))?;
    let mut added = 0usize;

    for (id, entry) in mcp_obj {
        if managed.contains(id) { continue; }
        if config.mcp_servers.iter().any(|s| &s.id == id) { continue; }

        let server_type = match entry.get("type").and_then(|v| v.as_str()) {
            Some("sse") => McpServerType::Sse,
            Some("http") => McpServerType::Http,
            _ => McpServerType::Stdio,
        };
        // Handle Gemini's httpUrl field
        let url = entry.get(desc.http_url_field)
            .or_else(|| entry.get("url"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        config.mcp_servers.push(McpServerConfig {
            id: id.clone(),
            name: id.clone(),
            command: entry.get("command").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            args: entry.get("args").and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default(),
            env: entry.get("env").and_then(|v| v.as_object())
                .map(|o| o.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect::<HashMap<_, _>>())
                .unwrap_or_default(),
            scope: "global".to_string(),
            server_type,
            url,
            disabled: entry.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false),
            timeout_secs: None,
        });
        added += 1;
    }

    if added > 0 {
        crate::config::save_config(&config, Some(project_dir))?;
    }
    Ok(added)
}

// ─── Payload builder ──────────────────────────────────────────────────────────

fn build_payload(project_dir: &Path) -> Result<SyncPayload> {
    let config = get_effective_config(Some(project_dir.to_path_buf()))?;

    if let Some(mode_id) = &config.active_mode {
        if let Some(mode) = config.modes.iter().find(|m| &m.id == mode_id) {
            let servers = if mode.mcp_servers.is_empty() {
                config.mcp_servers.clone()
            } else {
                config.mcp_servers.iter()
                    .filter(|s| mode.mcp_servers.contains(&s.id))
                    .cloned()
                    .collect()
            };
            let prompt = mode.prompt_id.as_ref().and_then(|id| get_prompt(project_dir, id).ok());
            let mut hooks = config.hooks.clone();
            hooks.extend(mode.hooks.clone());
            return Ok(SyncPayload {
                servers,
                prompt,
                hooks,
                permissions: mode.permissions.clone(),
                active_mode_id: Some(mode_id.clone()),
            });
        }
    }

    Ok(SyncPayload {
        servers: config.mcp_servers,
        prompt: None,
        hooks: config.hooks,
        permissions: Default::default(),
        active_mode_id: config.active_mode,
    })
}

// ─── Generic export ───────────────────────────────────────────────────────────

fn export_json(
    desc: &ProviderDescriptor,
    _project_dir: &Path,
    project_root: &Path,
    payload: &SyncPayload,
    state: &mut ManagedState,
) -> Result<()> {
    let config_path = project_root.join(desc.project_config);
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let existing: serde_json::Value = if config_path.exists() {
        serde_json::from_str(&fs::read_to_string(&config_path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let tool_state = state.providers.entry(desc.id.to_string()).or_default();
    let mut mcp_servers = serde_json::Map::new();

    // Preserve user-defined servers (not Ship-managed)
    if let Some(existing_mcp) = existing.get(desc.mcp_key).and_then(|v| v.as_object()) {
        for (id, entry) in existing_mcp {
            let is_managed = match desc.managed_marker {
                ManagedMarker::Inline => entry
                    .get("_ship")
                    .and_then(|v| v.get("managed"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                ManagedMarker::StateFileOnly => false,
            } || tool_state.managed_servers.contains(id);
            if !is_managed {
                mcp_servers.insert(id.clone(), entry.clone());
            }
        }
    }

    // Always inject Ship's own server
    let (ship_id, mut ship_entry) = ship_server_entry();
    if !desc.emit_type_field {
        ship_entry.as_object_mut().map(|o| o.remove("type"));
    }
    mcp_servers.insert(ship_id.to_string(), ship_entry);

    let mut written_ids = vec![ship_id.to_string()];
    for s in &payload.servers {
        if s.disabled { continue; }
        let mut entry = json_mcp_entry(desc, s);
        if matches!(desc.managed_marker, ManagedMarker::Inline) {
            entry["_ship"] = serde_json::json!({ "managed": true });
        }
        mcp_servers.insert(s.id.clone(), entry);
        written_ids.push(s.id.clone());
    }

    let mut root = existing.clone();
    if !root.is_object() { root = serde_json::json!({}); }
    root[desc.mcp_key] = serde_json::Value::Object(mcp_servers);
    crate::fs_util::write_atomic(&config_path, serde_json::to_string_pretty(&root)?)?;

    // Prompt output
    if let Some(prompt) = &payload.prompt {
        match desc.prompt_output {
            PromptOutput::GeminiMd => {
                let md = project_root.join("GEMINI.md");
                let content = format!("<!-- managed by ship — prompt: {} -->\n\n{}\n", prompt.id, prompt.content);
                crate::fs_util::write_atomic(&md, content)?;
            }
            PromptOutput::ClaudeMd | PromptOutput::AgentsMd | PromptOutput::None => {}
        }
    }

    tool_state.managed_servers = written_ids;
    tool_state.last_mode = payload.active_mode_id.clone();
    Ok(())
}

fn export_toml(
    desc: &ProviderDescriptor,
    _project_dir: &Path,
    project_root: &Path,
    payload: &SyncPayload,
    state: &mut ManagedState,
) -> Result<()> {
    let config_path = project_root.join(desc.project_config);
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let raw_existing = if config_path.exists() { fs::read_to_string(&config_path)? } else { String::new() };
    let mut doc: toml::Value = if raw_existing.is_empty() {
        toml::Value::Table(Default::default())
    } else {
        toml::from_str(&raw_existing).map_err(|e| {
            anyhow!("Cannot parse {}: {}. Note: Codex uses 'mcp_servers' (underscore).", config_path.display(), e)
        })?
    };

    let root = match &mut doc {
        toml::Value::Table(t) => t,
        _ => return Err(anyhow!("Config root is not a TOML table")),
    };

    let tool_state = state.providers.entry(desc.id.to_string()).or_default();
    let existing_mcp: toml::value::Table = root.get(desc.mcp_key)
        .and_then(|v| v.as_table()).cloned().unwrap_or_default();

    let mut new_mcp = toml::value::Table::new();
    // Preserve user servers (not Ship-managed)
    for (id, entry) in &existing_mcp {
        if !tool_state.managed_servers.contains(id) {
            new_mcp.insert(id.clone(), entry.clone());
        }
    }

    // Ship self-entry
    let mut ship_entry = toml::value::Table::new();
    ship_entry.insert("command".into(), toml::Value::String("ship".into()));
    ship_entry.insert("args".into(), toml::Value::Array(vec![toml::Value::String("mcp".into())]));
    new_mcp.insert("ship".into(), toml::Value::Table(ship_entry));
    let mut written_ids = vec!["ship".to_string()];

    for s in &payload.servers {
        if s.disabled { continue; }
        new_mcp.insert(s.id.clone(), toml_mcp_entry(desc, s));
        written_ids.push(s.id.clone());
    }

    root.insert(desc.mcp_key.to_string(), toml::Value::Table(new_mcp));

    crate::fs_util::write_atomic(&config_path, toml::to_string_pretty(&doc)?)?;

    tool_state.managed_servers = written_ids;
    tool_state.last_mode = payload.active_mode_id.clone();
    Ok(())
}

// ─── Generic teardown ─────────────────────────────────────────────────────────

fn teardown_json(
    config_path: &Path,
    mcp_key: &str,
    managed_marker: &ManagedMarker,
    tool_state: &ToolState,
) -> Result<()> {
    if !config_path.exists() { return Ok(()); }

    let existing: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(config_path)?).unwrap_or(serde_json::json!({}));

    let mut kept = serde_json::Map::new();
    if let Some(servers) = existing.get(mcp_key).and_then(|v| v.as_object()) {
        for (id, entry) in servers {
            let is_managed = match managed_marker {
                ManagedMarker::Inline => entry
                    .get("_ship")
                    .and_then(|v| v.get("managed"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                ManagedMarker::StateFileOnly => false,
            } || tool_state.managed_servers.contains(id);
            if !is_managed {
                kept.insert(id.clone(), entry.clone());
            }
        }
    }

    if kept.is_empty() {
        fs::remove_file(config_path).ok();
    } else {
        let mut root = existing.clone();
        if !root.is_object() { root = serde_json::json!({}); }
        root[mcp_key] = serde_json::Value::Object(kept);
        crate::fs_util::write_atomic(config_path, serde_json::to_string_pretty(&root)?)?;
    }
    Ok(())
}

fn teardown_toml(config_path: &Path, mcp_key: &str, tool_state: &ToolState) -> Result<()> {
    if !config_path.exists() { return Ok(()); }

    let raw = fs::read_to_string(config_path)?;
    let mut doc: toml::Value = toml::from_str(&raw).unwrap_or(toml::Value::Table(Default::default()));

    if let toml::Value::Table(root) = &mut doc {
        let existing: toml::value::Table = root.get(mcp_key)
            .and_then(|v| v.as_table()).cloned().unwrap_or_default();
        let mut kept = toml::value::Table::new();
        for (id, entry) in &existing {
            if !tool_state.managed_servers.contains(id) {
                kept.insert(id.clone(), entry.clone());
            }
        }
        root.insert(mcp_key.to_string(), toml::Value::Table(kept));
    }

    crate::fs_util::write_atomic(config_path, toml::to_string_pretty(&doc)?)?;
    Ok(())
}

// ─── Entry builders ───────────────────────────────────────────────────────────

fn ship_server_entry() -> (&'static str, serde_json::Value) {
    let entry = serde_json::json!({
        "command": "ship",
        "args": ["mcp"],
        "type": "stdio",
        "_ship": { "managed": true }
    });
    ("ship", entry)
}

fn json_mcp_entry(desc: &ProviderDescriptor, s: &McpServerConfig) -> serde_json::Value {
    match s.server_type {
        McpServerType::Stdio => {
            let mut entry = serde_json::json!({ "command": s.command });
            if desc.emit_type_field {
                entry["type"] = serde_json::json!("stdio");
            }
            if !s.args.is_empty() { entry["args"] = serde_json::json!(s.args); }
            if !s.env.is_empty() { entry["env"] = serde_json::json!(s.env); }
            entry
        }
        McpServerType::Http | McpServerType::Sse => {
            let mut entry = serde_json::json!({ desc.http_url_field: s.url });
            if desc.emit_type_field {
                let type_str = if matches!(s.server_type, McpServerType::Sse) { "sse" } else { "http" };
                entry["type"] = serde_json::json!(type_str);
            }
            if let Some(t) = s.timeout_secs {
                // Gemini timeout is in ms
                let key = if desc.http_url_field == "httpUrl" { "timeout" } else { "timeout" };
                entry[key] = serde_json::json!(if desc.http_url_field == "httpUrl" { t * 1000 } else { t });
            }
            entry
        }
    }
}

fn toml_mcp_entry(desc: &ProviderDescriptor, s: &McpServerConfig) -> toml::Value {
    let mut entry = toml::value::Table::new();
    match s.server_type {
        McpServerType::Stdio => {
            entry.insert("command".into(), toml::Value::String(s.command.clone()));
            if !s.args.is_empty() {
                entry.insert("args".into(), toml::Value::Array(
                    s.args.iter().map(|a| toml::Value::String(a.clone())).collect()
                ));
            }
            if !s.env.is_empty() {
                let env: toml::value::Table = s.env.iter()
                    .map(|(k, v)| (k.clone(), toml::Value::String(v.clone())))
                    .collect();
                entry.insert("env".into(), toml::Value::Table(env));
            }
        }
        McpServerType::Http | McpServerType::Sse => {
            if let Some(url) = &s.url {
                entry.insert(desc.http_url_field.into(), toml::Value::String(url.clone()));
            }
            // Bearer token: if env has a *_TOKEN or *_KEY, surface it
            for (k, _) in &s.env {
                if k.ends_with("_TOKEN") || k.ends_with("_KEY") {
                    entry.insert("bearer_token_env_var".into(), toml::Value::String(k.clone()));
                    break;
                }
            }
        }
    }
    if let Some(t) = s.timeout_secs {
        entry.insert("startup_timeout_sec".into(), toml::Value::Integer(t as i64));
    }
    toml::Value::Table(entry)
}

// ─── Skills ───────────────────────────────────────────────────────────────────

fn export_skills_to_claude(project_dir: &Path, project_root: &Path) -> Result<()> {
    export_skills_to_dir(project_dir, &project_root.join(".claude").join("skills"))
}

/// Write skills using the agentskills.io layout: `<skills_dir>/<skill-id>/SKILL.md`
fn export_skills_to_dir(project_dir: &Path, skills_dir: &Path) -> Result<()> {
    let skills = list_effective_skills(project_dir)?;
    if skills.is_empty() { return Ok(()); }
    fs::create_dir_all(skills_dir)?;
    for skill in &skills {
        let skill_dir = skills_dir.join(&skill.id);
        fs::create_dir_all(&skill_dir)?;
        let path = skill_dir.join("SKILL.md");
        let content = format!("<!-- managed by ship — skill: {} -->\n\n{}\n", skill.id, skill.content);
        crate::fs_util::write_atomic(&path, content)?;
    }
    Ok(())
}

/// Remove skill subdirectories that were written by Ship (identified by the
/// `<!-- managed by ship` header in their SKILL.md).
fn remove_ship_managed_skill_dirs(skills_dir: &Path) {
    if !skills_dir.exists() { return; }
    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let skill_dir = entry.path();
            if !skill_dir.is_dir() { continue; }
            let skill_md = skill_dir.join("SKILL.md");
            if skill_md.exists() {
                if let Ok(c) = fs::read_to_string(&skill_md) {
                    if c.starts_with("<!-- managed by ship") {
                        fs::remove_dir_all(&skill_dir).ok();
                    }
                }
            }
        }
    }
}

// ─── Hooks + permissions (Claude settings.json) ───────────────────────────────

fn export_claude_settings(hooks: &[HookConfig], permissions: &PermissionConfig) -> Result<()> {
    let path = home()?.join(".claude").join("settings.json");
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(&path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    let obj = root.as_object_mut().ok_or_else(|| anyhow!("~/.claude/settings.json is not an object"))?;

    if !permissions.allow.is_empty() || !permissions.deny.is_empty() {
        let perms = obj.entry("permissions").or_insert(serde_json::json!({}));
        let p = perms.as_object_mut().ok_or_else(|| anyhow!("permissions not an object"))?;
        if !permissions.allow.is_empty() { p.insert("allow".into(), serde_json::json!(permissions.allow)); }
        if !permissions.deny.is_empty() { p.insert("deny".into(), serde_json::json!(permissions.deny)); }
    }

    if !hooks.is_empty() {
        let hooks_val = obj.entry("hooks").or_insert(serde_json::json!({}));
        let hooks_map = hooks_val.as_object_mut().ok_or_else(|| anyhow!("hooks not an object"))?;
        let mut by_trigger: HashMap<&str, Vec<serde_json::Value>> = HashMap::new();
        for hook in hooks {
            let key = match hook.trigger {
                HookTrigger::PreToolUse => "PreToolUse",
                HookTrigger::PostToolUse => "PostToolUse",
                HookTrigger::Notification => "Notification",
                HookTrigger::Stop => "Stop",
                HookTrigger::SubagentStop => "SubagentStop",
                HookTrigger::PreCompact => "PreCompact",
            };
            let mut entry = serde_json::json!({ "type": "command", "command": hook.command });
            if let Some(m) = &hook.matcher { entry["matcher"] = serde_json::json!(m); }
            by_trigger.entry(key).or_default().push(entry);
        }
        for (trigger, entries) in by_trigger {
            hooks_map.insert(trigger.to_string(), serde_json::json!(entries));
        }
    }

    crate::fs_util::write_atomic(&path, serde_json::to_string_pretty(&root)?)
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn home() -> Result<PathBuf> {
    home::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{McpServerConfig, McpServerType, ProjectConfig, save_config};
    use crate::project::init_project;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn make_stdio_server(id: &str) -> McpServerConfig {
        McpServerConfig {
            id: id.to_string(), name: id.to_string(), command: "npx".to_string(),
            args: vec!["-y".to_string(), format!("@mcp/{}", id)],
            env: HashMap::new(), scope: "project".to_string(),
            server_type: McpServerType::Stdio, url: None, disabled: false, timeout_secs: None,
        }
    }

    fn make_http_server(id: &str, url: &str) -> McpServerConfig {
        McpServerConfig {
            id: id.to_string(), name: id.to_string(), command: String::new(),
            args: vec![], env: HashMap::new(), scope: "project".to_string(),
            server_type: McpServerType::Http, url: Some(url.to_string()), disabled: false, timeout_secs: None,
        }
    }

    fn project_with_servers(servers: Vec<McpServerConfig>) -> (tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let project_dir = init_project(tmp.path().to_path_buf()).unwrap();
        let mut config = ProjectConfig::default();
        config.mcp_servers = servers;
        save_config(&config, Some(project_dir.clone())).unwrap();
        (tmp, project_dir)
    }

    // ── Registry ───────────────────────────────────────────────────────────────

    #[test]
    fn all_provider_ids_are_unique() {
        let ids: Vec<_> = PROVIDERS.iter().map(|p| p.id).collect();
        let mut seen = std::collections::HashSet::new();
        for id in &ids { assert!(seen.insert(id), "duplicate provider id: {}", id); }
    }

    #[test]
    fn require_provider_errors_on_unknown() {
        let err = require_provider("vscode").unwrap_err();
        assert!(err.to_string().contains("vscode"));
        assert!(err.to_string().contains("claude"));
    }

    // ── Claude ─────────────────────────────────────────────────────────────────

    #[test]
    fn claude_writes_mcp_json_at_project_root() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        export_to(project_dir, "claude").unwrap();
        assert!(tmp.path().join(".mcp.json").exists());
    }

    #[test]
    fn claude_round_trip_stdio_server() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap()).unwrap();
        let mcp = val["mcpServers"]["github"].as_object().unwrap();
        assert_eq!(mcp["command"].as_str().unwrap(), "npx");
        assert_eq!(mcp["type"].as_str().unwrap(), "stdio");
    }

    #[test]
    fn claude_round_trip_http_server() {
        let (tmp, project_dir) = project_with_servers(vec![make_http_server("postgres", "http://localhost:5433/mcp")]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap()).unwrap();
        assert_eq!(val["mcpServers"]["postgres"]["type"].as_str().unwrap(), "http");
        assert_eq!(val["mcpServers"]["postgres"]["url"].as_str().unwrap(), "http://localhost:5433/mcp");
    }

    #[test]
    fn claude_ship_server_always_injected() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap()).unwrap();
        assert!(val["mcpServers"]["ship"].is_object());
    }

    #[test]
    fn claude_marks_managed_servers() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap()).unwrap();
        assert_eq!(val["mcpServers"]["github"]["_ship"]["managed"].as_bool(), Some(true));
    }

    #[test]
    fn claude_preserves_user_servers_across_write() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("mine")]);
        export_to(project_dir.clone(), "claude").unwrap();
        let mcp_json = tmp.path().join(".mcp.json");
        let mut val: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&mcp_json).unwrap()).unwrap();
        val["mcpServers"]["user-server"] = serde_json::json!({ "command": "user-tool", "args": [] });
        std::fs::write(&mcp_json, serde_json::to_string_pretty(&val).unwrap()).unwrap();
        export_to(project_dir, "claude").unwrap();
        let val2: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&mcp_json).unwrap()).unwrap();
        assert!(val2["mcpServers"]["user-server"].is_object(), "user server was clobbered");
    }

    #[test]
    fn claude_disabled_server_not_exported() {
        let mut s = make_stdio_server("disabled-one");
        s.disabled = true;
        let (tmp, project_dir) = project_with_servers(vec![s]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap()).unwrap();
        assert!(val["mcpServers"]["disabled-one"].is_null());
    }

    #[test]
    fn claude_managed_state_written() {
        let (_tmp, project_dir) = project_with_servers(vec![make_stdio_server("gh")]);
        export_to(project_dir.clone(), "claude").unwrap();
        let (ids, _mode) =
            crate::state_db::get_managed_state_db(&project_dir, "claude").unwrap();
        assert!(ids.contains(&"gh".to_string()), "managed server not recorded in state");
        // Clean up DB created in ~/.ship/state/ for this temp project
        std::fs::remove_file(crate::state_db::project_db_path(&project_dir).unwrap()).ok();
    }

    // ── Gemini ─────────────────────────────────────────────────────────────────

    #[test]
    fn gemini_writes_to_gemini_settings_json() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("git")]);
        export_to(project_dir, "gemini").unwrap();
        assert!(tmp.path().join(".gemini/settings.json").exists());
    }

    #[test]
    fn gemini_http_uses_httpurl_not_url() {
        let (tmp, project_dir) = project_with_servers(vec![make_http_server("figma", "https://mcp.figma.com/mcp")]);
        export_to(project_dir, "gemini").unwrap();
        let val: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".gemini/settings.json")).unwrap()).unwrap();
        assert!(val["mcpServers"]["figma"]["httpUrl"].is_string(), "Gemini must use httpUrl");
        assert!(val["mcpServers"]["figma"]["url"].is_null(), "Gemini must not use url");
    }

    #[test]
    fn gemini_preserves_non_mcp_fields() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("git")]);
        let settings_dir = tmp.path().join(".gemini");
        std::fs::create_dir_all(&settings_dir).unwrap();
        std::fs::write(settings_dir.join("settings.json"),
            r#"{"theme": "Dracula", "selectedAuthType": "gemini-api-key", "mcpServers": {}}"#).unwrap();
        export_to(project_dir, "gemini").unwrap();
        let val: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".gemini/settings.json")).unwrap()).unwrap();
        assert_eq!(val["theme"].as_str().unwrap(), "Dracula");
        assert_eq!(val["selectedAuthType"].as_str().unwrap(), "gemini-api-key");
    }

    #[test]
    fn gemini_ship_server_always_injected() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        export_to(project_dir, "gemini").unwrap();
        let val: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".gemini/settings.json")).unwrap()).unwrap();
        assert!(val["mcpServers"]["ship"].is_object());
    }

    // ── Codex ──────────────────────────────────────────────────────────────────

    #[test]
    fn codex_writes_to_codex_config_toml() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("gh")]);
        export_to(project_dir, "codex").unwrap();
        assert!(tmp.path().join(".codex/config.toml").exists());
    }

    #[test]
    fn codex_uses_mcp_servers_underscore_not_hyphen() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("gh")]);
        export_to(project_dir, "codex").unwrap();
        let content = std::fs::read_to_string(tmp.path().join(".codex/config.toml")).unwrap();
        assert!(content.contains("[mcp_servers."), "must use mcp_servers (underscore)");
        assert!(!content.contains("[mcp-servers."), "must NOT use mcp-servers (hyphen)");
    }

    #[test]
    fn codex_round_trip_stdio_server() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("gh")]);
        export_to(project_dir, "codex").unwrap();
        let val: toml::Value = toml::from_str(&std::fs::read_to_string(tmp.path().join(".codex/config.toml")).unwrap()).unwrap();
        assert_eq!(val["mcp_servers"]["gh"]["command"].as_str().unwrap(), "npx");
    }

    #[test]
    fn codex_preserves_user_servers() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("mine")]);
        export_to(project_dir.clone(), "codex").unwrap();
        let config_path = tmp.path().join(".codex/config.toml");
        let mut content = std::fs::read_to_string(&config_path).unwrap();
        content.push_str("\n[mcp_servers.user-tool]\ncommand = \"user-tool\"\n");
        std::fs::write(&config_path, &content).unwrap();
        export_to(project_dir, "codex").unwrap();
        let val: toml::Value = toml::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert!(val["mcp_servers"]["user-tool"].is_table(), "user server was clobbered");
    }

    #[test]
    fn codex_ship_server_always_injected() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        export_to(project_dir, "codex").unwrap();
        let val: toml::Value = toml::from_str(&std::fs::read_to_string(tmp.path().join(".codex/config.toml")).unwrap()).unwrap();
        assert!(val["mcp_servers"]["ship"].is_table());
    }

    // ── Import ─────────────────────────────────────────────────────────────────

    #[test]
    fn import_from_claude_adds_new_servers() {
        let (_tmp, project_dir) = project_with_servers(vec![]);
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        let mcp_obj = serde_json::json!({
            "github": { "command": "npx", "args": ["-y", "@mcp/github"], "type": "stdio" },
            "postgres": { "type": "http", "url": "http://localhost:5433/mcp" }
        });
        for (id, entry) in mcp_obj.as_object().unwrap() {
            let server_type = match entry.get("type").and_then(|v| v.as_str()) {
                Some("http") => McpServerType::Http,
                _ => McpServerType::Stdio,
            };
            config.mcp_servers.push(McpServerConfig {
                id: id.clone(), name: id.clone(),
                command: entry.get("command").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                args: entry.get("args").and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                    .unwrap_or_default(),
                env: HashMap::new(), scope: "global".to_string(), server_type,
                url: entry.get("url").and_then(|v| v.as_str()).map(str::to_string),
                disabled: false, timeout_secs: None,
            });
        }
        save_config(&config, Some(project_dir.clone())).unwrap();
        let reloaded = crate::config::get_config(Some(project_dir)).unwrap();
        assert_eq!(reloaded.mcp_servers.len(), 2);
        assert!(reloaded.mcp_servers.iter().any(|s| s.id == "github"));
        assert!(reloaded.mcp_servers.iter().any(|s| s.id == "postgres"));
    }
}
