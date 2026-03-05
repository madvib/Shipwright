use crate::config::{
    HookConfig, HookTrigger, McpServerConfig, McpServerType, get_config, get_effective_config,
};
use crate::permissions::{Permissions, get_permissions};
use crate::prompt::Prompt;
use crate::prompt::get_prompt;
use crate::skill::list_effective_skills;
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
    StaticModelInfo {
        id: "claude-sonnet-4-6",
        name: "Claude Sonnet 4.6",
        context_window: 200_000,
        recommended: true,
    },
    StaticModelInfo {
        id: "claude-opus-4-6",
        name: "Claude Opus 4.6",
        context_window: 200_000,
        recommended: false,
    },
    StaticModelInfo {
        id: "claude-haiku-4-5",
        name: "Claude Haiku 4.5",
        context_window: 200_000,
        recommended: false,
    },
];

const GEMINI_MODELS: &[StaticModelInfo] = &[
    StaticModelInfo {
        id: "gemini-2.5-pro",
        name: "Gemini 2.5 Pro",
        context_window: 1_000_000,
        recommended: true,
    },
    StaticModelInfo {
        id: "gemini-2.0-flash",
        name: "Gemini 2.0 Flash",
        context_window: 1_000_000,
        recommended: false,
    },
    StaticModelInfo {
        id: "gemini-2.0-flash-lite",
        name: "Gemini 2.0 Flash Lite",
        context_window: 1_000_000,
        recommended: false,
    },
];

const CODEX_MODELS: &[StaticModelInfo] = &[
    StaticModelInfo {
        id: "gpt-4o",
        name: "GPT-4o",
        context_window: 128_000,
        recommended: true,
    },
    StaticModelInfo {
        id: "gpt-4o-mini",
        name: "GPT-4o Mini",
        context_window: 128_000,
        recommended: false,
    },
    StaticModelInfo {
        id: "o1",
        name: "o1",
        context_window: 200_000,
        recommended: false,
    },
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
    let text = String::from_utf8_lossy(if out.stdout.is_empty() {
        &out.stderr
    } else {
        &out.stdout
    });
    text.lines()
        .next()
        .map(|l| l.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn provider_info(d: &ProviderDescriptor, enabled: bool) -> ProviderInfo {
    let installed = detect_binary(d.binary);
    let version = if installed {
        detect_version(d.binary)
    } else {
        None
    };
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
        models: d
            .models
            .iter()
            .map(|m| ModelInfo::from_static(m, d.id))
            .collect(),
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
        if detect_binary(d.binary) && enable_provider(project_dir, d.id)? {
            newly_enabled.push(d.id.to_string());
        }
    }
    Ok(newly_enabled)
}

/// Return models for a specific provider by ID.
pub fn list_models(provider_id: &str) -> Result<Vec<ModelInfo>> {
    let d = require_provider(provider_id)?;
    Ok(d.models
        .iter()
        .map(|m| ModelInfo::from_static(m, d.id))
        .collect())
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
        if let Ok((ids, last_mode)) = crate::state_db::get_managed_state_db(project_dir, p.id)
            && (!ids.is_empty() || last_mode.is_some())
        {
            state.providers.insert(
                p.id.to_string(),
                ToolState {
                    managed_servers: ids,
                    last_mode,
                },
            );
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
    pub permissions: Permissions,
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
        SkillsOutput::AgentSkills => {
            export_skills_to_dir(&project_dir, &project_root.join(".gemini").join("skills"))?
        }
        SkillsOutput::CodexSkills => {
            export_skills_to_dir(&project_dir, &project_root.join(".agents").join("skills"))?
        }
        SkillsOutput::None => {}
    }

    // Provider-native hooks + permissions.
    match target {
        "claude" => {
            if !payload.hooks.is_empty() || has_claude_permission_overrides(&payload.permissions) {
                export_claude_settings(&payload.hooks, &payload.permissions)?;
            }
        }
        "gemini" => export_gemini_workspace_policy(project_root, &payload.permissions)?,
        _ => {}
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
    let tool_state = state
        .providers
        .entry(target.to_string())
        .or_default()
        .clone();

    match desc.config_format {
        ConfigFormat::Json => {
            let config_path = project_root.join(desc.project_config);
            teardown_json(
                &config_path,
                desc.mcp_key,
                &desc.managed_marker,
                &tool_state,
            )?;
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
            if f.exists() {
                fs::remove_file(&f).ok();
            }
        }
        PromptOutput::AgentsMd => {
            let f = project_root.join("AGENTS.md");
            if f.exists() {
                fs::remove_file(&f).ok();
            }
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
    let mode_targets = config
        .active_mode
        .as_ref()
        .and_then(|id| config.modes.iter().find(|m| &m.id == id))
        .map(|m| m.target_agents.clone())
        .unwrap_or_default();
    let targets: Vec<String> = if !mode_targets.is_empty() {
        mode_targets
    } else if !config.providers.is_empty() {
        config.providers.clone()
    } else {
        vec!["claude".to_string()]
    };

    let mut seen = std::collections::HashSet::new();
    let mut synced = Vec::new();
    for target in targets {
        let normalized = target.trim().to_ascii_lowercase();
        if normalized.is_empty() || !seen.insert(normalized.clone()) {
            continue;
        }
        if get_provider(&normalized).is_none() {
            eprintln!("[ship] warning: skipping unknown target agent '{}'", target);
            continue;
        }
        export_to(project_dir.to_path_buf(), &normalized)?;
        synced.push(normalized);
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
        if managed.contains(id) {
            continue;
        }
        if config.mcp_servers.iter().any(|s| &s.id == id) {
            continue;
        }

        let server_type = match entry.get("type").and_then(|v| v.as_str()) {
            Some("sse") => McpServerType::Sse,
            Some("http") => McpServerType::Http,
            _ => McpServerType::Stdio,
        };
        // Handle Gemini's httpUrl field
        let url = entry
            .get(desc.http_url_field)
            .or_else(|| entry.get("url"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        config.mcp_servers.push(McpServerConfig {
            id: id.clone(),
            name: id.clone(),
            command: entry
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            args: entry
                .get("args")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            env: entry
                .get("env")
                .and_then(|v| v.as_object())
                .map(|o| {
                    o.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default(),
            scope: "global".to_string(),
            server_type,
            url,
            disabled: entry
                .get("disabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            timeout_secs: None,
        });
        added += 1;
    }

    if added > 0 {
        crate::config::save_config(&config, Some(project_dir))?;
    }
    Ok(added)
}

/// Import provider-native permission settings into canonical
/// `.ship/agents/permissions.toml`.
///
/// Returns `true` when permissions were imported and saved, `false` when no
/// importable permissions were found for the provider.
pub fn import_permissions_from_provider(provider_id: &str, project_dir: PathBuf) -> Result<bool> {
    let imported = match provider_id {
        "claude" => import_permissions_from_claude()?,
        "gemini" => import_permissions_from_gemini(&project_dir)?,
        "codex" => import_permissions_from_codex(&project_dir)?,
        _ => return Err(anyhow!("Unsupported provider '{}'", provider_id)),
    };

    if let Some(permissions) = imported {
        crate::permissions::save_permissions(project_dir, &permissions)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

// ─── Payload builder ──────────────────────────────────────────────────────────

fn build_payload(project_dir: &Path) -> Result<SyncPayload> {
    let config = get_effective_config(Some(project_dir.to_path_buf()))?;
    let mut effective_permissions = get_permissions(project_dir.to_path_buf())?;

    if let Some(mode_id) = &config.active_mode
        && let Some(mode) = config.modes.iter().find(|m| &m.id == mode_id)
    {
        if !mode.permissions.allow.is_empty() {
            effective_permissions.tools.allow = mode.permissions.allow.clone();
        }
        if !mode.permissions.deny.is_empty() {
            effective_permissions.tools.deny = mode.permissions.deny.clone();
        }
        let servers = if mode.mcp_servers.is_empty() {
            config.mcp_servers.clone()
        } else {
            config
                .mcp_servers
                .iter()
                .filter(|s| mode.mcp_servers.contains(&s.id))
                .cloned()
                .collect()
        };
        let prompt = mode
            .prompt_id
            .as_ref()
            .and_then(|id| get_prompt(project_dir, id).ok());
        let mut hooks = config.hooks.clone();
        hooks.extend(mode.hooks.clone());
        return Ok(SyncPayload {
            servers,
            prompt,
            hooks,
            permissions: effective_permissions,
            active_mode_id: Some(mode_id.clone()),
        });
    }

    Ok(SyncPayload {
        servers: config.mcp_servers,
        prompt: None,
        hooks: config.hooks,
        permissions: effective_permissions,
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
        if s.disabled {
            continue;
        }
        let mut entry = json_mcp_entry(desc, s);
        if matches!(desc.managed_marker, ManagedMarker::Inline) {
            entry["_ship"] = serde_json::json!({ "managed": true });
        }
        mcp_servers.insert(s.id.clone(), entry);
        written_ids.push(s.id.clone());
    }

    let mut root = existing.clone();
    if !root.is_object() {
        root = serde_json::json!({});
    }
    root[desc.mcp_key] = serde_json::Value::Object(mcp_servers);
    crate::fs_util::write_atomic(&config_path, serde_json::to_string_pretty(&root)?)?;

    // Prompt output
    if let Some(prompt) = &payload.prompt {
        match desc.prompt_output {
            PromptOutput::GeminiMd => {
                let md = project_root.join("GEMINI.md");
                let content = format!(
                    "<!-- managed by ship — prompt: {} -->\n\n{}\n",
                    prompt.id, prompt.content
                );
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

    let raw_existing = if config_path.exists() {
        fs::read_to_string(&config_path)?
    } else {
        String::new()
    };
    let mut doc: toml::Value = if raw_existing.is_empty() {
        toml::Value::Table(Default::default())
    } else {
        toml::from_str(&raw_existing).map_err(|e| {
            anyhow!(
                "Cannot parse {}: {}. Note: Codex uses 'mcp_servers' (underscore).",
                config_path.display(),
                e
            )
        })?
    };

    let root = match &mut doc {
        toml::Value::Table(t) => t,
        _ => return Err(anyhow!("Config root is not a TOML table")),
    };

    let tool_state = state.providers.entry(desc.id.to_string()).or_default();
    let existing_mcp: toml::value::Table = root
        .get(desc.mcp_key)
        .and_then(|v| v.as_table())
        .cloned()
        .unwrap_or_default();

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
    ship_entry.insert(
        "args".into(),
        toml::Value::Array(vec![
            toml::Value::String("mcp".into()),
            toml::Value::String("serve".into()),
        ]),
    );
    new_mcp.insert("ship".into(), toml::Value::Table(ship_entry));
    let mut written_ids = vec!["ship".to_string()];

    for s in &payload.servers {
        if s.disabled {
            continue;
        }
        new_mcp.insert(s.id.clone(), toml_mcp_entry(desc, s));
        written_ids.push(s.id.clone());
    }

    root.insert(desc.mcp_key.to_string(), toml::Value::Table(new_mcp));
    if desc.id == "codex" {
        apply_codex_permissions(root, &payload.permissions);
    }

    crate::fs_util::write_atomic(&config_path, toml::to_string_pretty(&doc)?)?;

    // Prompt output
    if let Some(prompt) = &payload.prompt {
        match desc.prompt_output {
            PromptOutput::AgentsMd => {
                let md = project_root.join("AGENTS.md");
                let content = format!(
                    "<!-- managed by ship — prompt: {} -->\n\n{}\n",
                    prompt.id, prompt.content
                );
                crate::fs_util::write_atomic(&md, content)?;
            }
            PromptOutput::ClaudeMd | PromptOutput::GeminiMd | PromptOutput::None => {}
        }
    }

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
    if !config_path.exists() {
        return Ok(());
    }

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
        if !root.is_object() {
            root = serde_json::json!({});
        }
        root[mcp_key] = serde_json::Value::Object(kept);
        crate::fs_util::write_atomic(config_path, serde_json::to_string_pretty(&root)?)?;
    }
    Ok(())
}

fn teardown_toml(config_path: &Path, mcp_key: &str, tool_state: &ToolState) -> Result<()> {
    if !config_path.exists() {
        return Ok(());
    }

    let raw = fs::read_to_string(config_path)?;
    let mut doc: toml::Value =
        toml::from_str(&raw).unwrap_or(toml::Value::Table(Default::default()));

    if let toml::Value::Table(root) = &mut doc {
        let existing: toml::value::Table = root
            .get(mcp_key)
            .and_then(|v| v.as_table())
            .cloned()
            .unwrap_or_default();
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
            if !s.args.is_empty() {
                entry["args"] = serde_json::json!(s.args);
            }
            if !s.env.is_empty() {
                entry["env"] = serde_json::json!(s.env);
            }
            entry
        }
        McpServerType::Http | McpServerType::Sse => {
            let mut entry = serde_json::json!({ desc.http_url_field: s.url });
            if desc.emit_type_field {
                let type_str = if matches!(s.server_type, McpServerType::Sse) {
                    "sse"
                } else {
                    "http"
                };
                entry["type"] = serde_json::json!(type_str);
            }
            if let Some(t) = s.timeout_secs {
                // Gemini timeout is in ms
                let key = "timeout";
                entry[key] = serde_json::json!(if desc.http_url_field == "httpUrl" {
                    t * 1000
                } else {
                    t
                });
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
                entry.insert(
                    "args".into(),
                    toml::Value::Array(
                        s.args
                            .iter()
                            .map(|a| toml::Value::String(a.clone()))
                            .collect(),
                    ),
                );
            }
            if !s.env.is_empty() {
                let env: toml::value::Table = s
                    .env
                    .iter()
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
            for k in s.env.keys() {
                if k.ends_with("_TOKEN") || k.ends_with("_KEY") {
                    entry.insert(
                        "bearer_token_env_var".into(),
                        toml::Value::String(k.clone()),
                    );
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

fn resolve_skills_for_export(project_dir: &Path) -> Result<Vec<crate::skill::Skill>> {
    let config = get_effective_config(Some(project_dir.to_path_buf()))?;
    let mut skills = list_effective_skills(project_dir)?;

    if let Some(active_mode_id) = config.active_mode.as_deref()
        && let Some(mode) = config.modes.iter().find(|m| m.id == active_mode_id)
        && !mode.skills.is_empty()
    {
        skills.retain(|skill| mode.skills.contains(&skill.id));
    }

    Ok(skills)
}

/// Write skills using the agentskills.io layout: `<skills_dir>/<skill-id>/SKILL.md`
fn export_skills_to_dir(project_dir: &Path, skills_dir: &Path) -> Result<()> {
    let skills = resolve_skills_for_export(project_dir)?;
    let retain_ids: HashSet<String> = skills.iter().map(|skill| skill.id.clone()).collect();
    prune_stale_managed_skill_dirs(skills_dir, &retain_ids);
    if skills.is_empty() {
        return Ok(());
    }

    fs::create_dir_all(skills_dir)?;
    for skill in &skills {
        let skill_dir = skills_dir.join(&skill.id);
        fs::create_dir_all(&skill_dir)?;
        let path = skill_dir.join("SKILL.md");
        let content = format!(
            "<!-- managed by ship — skill: {} -->\n\n{}\n",
            skill.id, skill.content
        );
        crate::fs_util::write_atomic(&path, content)?;
    }
    Ok(())
}

fn prune_stale_managed_skill_dirs(skills_dir: &Path, retain_ids: &HashSet<String>) {
    if !skills_dir.exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let skill_dir = entry.path();
            if !skill_dir.is_dir() {
                continue;
            }

            let skill_id = entry.file_name().to_string_lossy().to_string();
            if retain_ids.contains(&skill_id) {
                continue;
            }

            let skill_md = skill_dir.join("SKILL.md");
            if skill_md.exists()
                && let Ok(content) = fs::read_to_string(&skill_md)
                && content.starts_with("<!-- managed by ship")
            {
                fs::remove_dir_all(&skill_dir).ok();
            }
        }
    }
}

/// Remove skill subdirectories that were written by Ship (identified by the
/// `<!-- managed by ship` header in their SKILL.md).
fn remove_ship_managed_skill_dirs(skills_dir: &Path) {
    if !skills_dir.exists() {
        return;
    }
    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let skill_dir = entry.path();
            if !skill_dir.is_dir() {
                continue;
            }
            let skill_md = skill_dir.join("SKILL.md");
            if skill_md.exists()
                && let Ok(c) = fs::read_to_string(&skill_md)
                && c.starts_with("<!-- managed by ship")
            {
                fs::remove_dir_all(&skill_dir).ok();
            }
        }
    }
}

// ─── Hooks + permissions (provider-native mappings) ──────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct GeminiPolicyDoc {
    #[serde(rename = "rule", default)]
    rules: Vec<GeminiPolicyRule>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct GeminiPolicyRule {
    #[serde(rename = "toolName", skip_serializing_if = "Option::is_none")]
    tool_name: Option<String>,
    #[serde(rename = "mcpName", skip_serializing_if = "Option::is_none")]
    mcp_name: Option<String>,
    #[serde(rename = "commandPrefix", skip_serializing_if = "Option::is_none")]
    command_prefix: Option<String>,
    #[serde(rename = "commandRegex", skip_serializing_if = "Option::is_none")]
    command_regex: Option<String>,
    decision: String,
    priority: i32,
}

fn is_default_tool_permissions(permissions: &Permissions) -> bool {
    (permissions.tools.allow.is_empty()
        || (permissions.tools.allow.len() == 1 && permissions.tools.allow[0] == "*"))
        && permissions.tools.deny.is_empty()
}

fn has_claude_permission_overrides(permissions: &Permissions) -> bool {
    !is_default_tool_permissions(permissions)
}

fn has_gemini_policy_overrides(permissions: &Permissions) -> bool {
    !is_default_tool_permissions(permissions)
        || !permissions.commands.allow.is_empty()
        || !permissions.commands.deny.is_empty()
        || !permissions.agent.require_confirmation.is_empty()
}

fn export_claude_settings(hooks: &[HookConfig], permissions: &Permissions) -> Result<()> {
    let path = home()?.join(".claude").join("settings.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(&path)?).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    let obj = root
        .as_object_mut()
        .ok_or_else(|| anyhow!("~/.claude/settings.json is not an object"))?;

    if has_claude_permission_overrides(permissions) {
        let perms = obj.entry("permissions").or_insert(serde_json::json!({}));
        let p = perms
            .as_object_mut()
            .ok_or_else(|| anyhow!("permissions not an object"))?;
        p.insert("allow".into(), serde_json::json!(permissions.tools.allow));
        p.insert("deny".into(), serde_json::json!(permissions.tools.deny));
    }

    if !hooks.is_empty() {
        let hooks_val = obj.entry("hooks").or_insert(serde_json::json!({}));
        let hooks_map = hooks_val
            .as_object_mut()
            .ok_or_else(|| anyhow!("hooks not an object"))?;
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
            if let Some(m) = &hook.matcher {
                entry["matcher"] = serde_json::json!(m);
            }
            by_trigger.entry(key).or_default().push(entry);
        }
        for (trigger, entries) in by_trigger {
            hooks_map.insert(trigger.to_string(), serde_json::json!(entries));
        }
    }

    crate::fs_util::write_atomic(&path, serde_json::to_string_pretty(&root)?)
}

fn export_gemini_workspace_policy(project_root: &Path, permissions: &Permissions) -> Result<()> {
    let path = project_root
        .join(".gemini")
        .join("policies")
        .join("ship-permissions.toml");

    if !has_gemini_policy_overrides(permissions) {
        fs::remove_file(&path).ok();
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut rules = Vec::new();

    // Highest priority: explicit denies
    for pattern in &permissions.tools.deny {
        rules.push(GeminiPolicyRule {
            tool_name: Some(pattern.clone()),
            decision: "deny".to_string(),
            priority: 900,
            ..Default::default()
        });
    }
    for pattern in &permissions.commands.deny {
        let (prefix, regex) = command_pattern_fields(pattern);
        rules.push(GeminiPolicyRule {
            tool_name: Some("run_shell_command".to_string()),
            command_prefix: prefix,
            command_regex: regex,
            decision: "deny".to_string(),
            priority: 900,
            ..Default::default()
        });
    }

    // Mid priority: explicit confirmation
    for pattern in &permissions.agent.require_confirmation {
        let (prefix, regex) = command_pattern_fields(pattern);
        rules.push(GeminiPolicyRule {
            tool_name: Some("run_shell_command".to_string()),
            command_prefix: prefix,
            command_regex: regex,
            decision: "ask_user".to_string(),
            priority: 800,
            ..Default::default()
        });
    }

    // Lower priority: allows
    for pattern in &permissions.tools.allow {
        rules.push(GeminiPolicyRule {
            tool_name: Some(pattern.clone()),
            decision: "allow".to_string(),
            priority: 700,
            ..Default::default()
        });
    }
    for pattern in &permissions.commands.allow {
        let (prefix, regex) = command_pattern_fields(pattern);
        rules.push(GeminiPolicyRule {
            tool_name: Some("run_shell_command".to_string()),
            command_prefix: prefix,
            command_regex: regex,
            decision: "allow".to_string(),
            priority: 700,
            ..Default::default()
        });
    }

    let doc = GeminiPolicyDoc { rules };
    let body = toml::to_string_pretty(&doc)?;
    let content = format!(
        "# managed by ship\n# source: .ship/agents/permissions.toml\n\n{}",
        body
    );
    crate::fs_util::write_atomic(&path, content)
}

fn apply_codex_permissions(root: &mut toml::value::Table, permissions: &Permissions) {
    let network_access = matches!(
        permissions.network.policy,
        crate::permissions::NetworkPolicy::AllowList
            | crate::permissions::NetworkPolicy::Unrestricted
    );
    root.insert(
        "sandbox_mode".to_string(),
        toml::Value::String("workspace-write".to_string()),
    );
    let approval = if permissions.agent.require_confirmation.is_empty()
        && permissions.commands.deny.is_empty()
        && permissions.tools.deny.is_empty()
        && permissions.tools.allow.iter().any(|p| p == "*")
    {
        "on-failure"
    } else {
        "on-request"
    };
    root.insert(
        "approval_policy".to_string(),
        toml::Value::String(approval.to_string()),
    );

    if !permissions.commands.allow.is_empty() {
        root.insert(
            "allow".to_string(),
            toml::Value::Array(
                permissions
                    .commands
                    .allow
                    .iter()
                    .cloned()
                    .map(toml::Value::String)
                    .collect(),
            ),
        );
    }

    let sandbox_entry = root
        .entry("sandbox_workspace_write".to_string())
        .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
    if let Some(table) = sandbox_entry.as_table_mut() {
        table.insert(
            "network_access".to_string(),
            toml::Value::Boolean(network_access),
        );
    }

    let mut prefix_rules = read_codex_prefix_rules(root);
    for pattern in &permissions.commands.deny {
        if let Some(prefix) = command_prefix_from_pattern(pattern) {
            prefix_rules.push((prefix, "forbidden".to_string()));
        }
    }
    for pattern in &permissions.agent.require_confirmation {
        if let Some(prefix) = command_prefix_from_pattern(pattern) {
            prefix_rules.push((prefix, "prompt".to_string()));
        }
    }
    dedupe_pairs(&mut prefix_rules);
    if !prefix_rules.is_empty() {
        let rules_entry = root
            .entry("rules".to_string())
            .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
        if let Some(rules_table) = rules_entry.as_table_mut() {
            let array = prefix_rules
                .into_iter()
                .map(|(prefix, decision)| {
                    let mut table = toml::value::Table::new();
                    table.insert("prefix".to_string(), toml::Value::String(prefix));
                    table.insert("decision".to_string(), toml::Value::String(decision));
                    toml::Value::Table(table)
                })
                .collect();
            rules_table.insert("prefix_rules".to_string(), toml::Value::Array(array));
        }
    }
}

fn import_permissions_from_claude() -> Result<Option<Permissions>> {
    let path = home()?.join(".claude").join("settings.json");
    if !path.exists() {
        return Ok(None);
    }

    let root: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    let Some(perms) = root.get("permissions").and_then(|p| p.as_object()) else {
        return Ok(None);
    };
    let allow = perms
        .get("allow")
        .and_then(|v| v.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let deny = perms
        .get("deny")
        .and_then(|v| v.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if allow.is_empty() && deny.is_empty() {
        return Ok(None);
    }

    let mut permissions = Permissions::default();
    if !allow.is_empty() {
        permissions.tools.allow = allow;
    }
    permissions.tools.deny = deny;
    Ok(Some(permissions))
}

fn import_permissions_from_gemini(project_dir: &Path) -> Result<Option<Permissions>> {
    let Some(project_root) = project_dir.parent() else {
        return Ok(None);
    };
    let path = project_root
        .join(".gemini")
        .join("policies")
        .join("ship-permissions.toml");
    if !path.exists() {
        return Ok(None);
    }

    let root: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
    let rules = root
        .get("rule")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if rules.is_empty() {
        return Ok(None);
    }

    let mut permissions = Permissions::default();
    permissions.tools.allow.clear();
    permissions.tools.deny.clear();
    permissions.commands.allow.clear();
    permissions.commands.deny.clear();
    permissions.agent.require_confirmation.clear();

    for value in rules {
        let Some(rule) = value.as_table() else {
            continue;
        };
        let decision = rule
            .get("decision")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if decision.is_empty() {
            continue;
        }

        let tool_names = value_to_string_list(rule.get("toolName"));
        let mcp_names = value_to_string_list(rule.get("mcpName"));
        let command_prefixes = value_to_string_list(rule.get("commandPrefix"));
        let command_regexes = value_to_string_list(rule.get("commandRegex"));

        for command in command_prefixes {
            match decision.as_str() {
                "allow" => permissions.commands.allow.push(format!("{}*", command)),
                "deny" => permissions.commands.deny.push(format!("{}*", command)),
                "ask_user" => permissions
                    .agent
                    .require_confirmation
                    .push(format!("{}*", command)),
                _ => {}
            }
        }
        for regex in command_regexes {
            let pattern = format!("regex:{}", regex);
            match decision.as_str() {
                "allow" => permissions.commands.allow.push(pattern),
                "deny" => permissions.commands.deny.push(pattern),
                "ask_user" => permissions.agent.require_confirmation.push(pattern),
                _ => {}
            }
        }

        let mut composite_tools = Vec::new();
        if tool_names.is_empty() && mcp_names.is_empty() {
            continue;
        }
        if tool_names.is_empty() {
            for mcp_name in &mcp_names {
                composite_tools.push(format!("{}__*", mcp_name));
            }
        } else if mcp_names.is_empty() {
            composite_tools.extend(tool_names.clone());
        } else {
            for mcp_name in &mcp_names {
                for tool_name in &tool_names {
                    composite_tools.push(format!("{}__{}", mcp_name, tool_name));
                }
            }
        }

        for tool in composite_tools {
            if tool == "run_shell_command" {
                continue;
            }
            match decision.as_str() {
                "allow" => permissions.tools.allow.push(tool),
                "deny" => permissions.tools.deny.push(tool),
                _ => {}
            }
        }
    }

    dedupe_strings(&mut permissions.tools.allow);
    dedupe_strings(&mut permissions.tools.deny);
    dedupe_strings(&mut permissions.commands.allow);
    dedupe_strings(&mut permissions.commands.deny);
    dedupe_strings(&mut permissions.agent.require_confirmation);

    if permissions.tools.allow.is_empty() {
        permissions.tools.allow.push("*".to_string());
    }
    Ok(Some(permissions))
}

fn import_permissions_from_codex(project_dir: &Path) -> Result<Option<Permissions>> {
    let Some(project_root) = project_dir.parent() else {
        return Ok(None);
    };
    let path = project_root.join(".codex").join("config.toml");
    if !path.exists() {
        return Ok(None);
    }

    let root: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
    let mut imported = false;
    let mut permissions = Permissions::default();

    if let Some(allow) = root.get("allow").and_then(|v| v.as_array()) {
        permissions.commands.allow = allow
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        imported = true;
    }

    if let Some(network_access) = root
        .get("sandbox_workspace_write")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("network_access"))
        .and_then(|v| v.as_bool())
    {
        imported = true;
        permissions.network.policy = if network_access {
            crate::permissions::NetworkPolicy::Unrestricted
        } else {
            crate::permissions::NetworkPolicy::None
        };
    }

    let prefix_rules = read_codex_prefix_rules_from_value(&root);
    for (prefix, decision) in prefix_rules {
        imported = true;
        let pattern = format!("{}*", prefix);
        match decision.as_str() {
            "forbidden" => permissions.commands.deny.push(pattern),
            "prompt" => permissions.agent.require_confirmation.push(pattern),
            _ => {}
        }
    }

    if !imported {
        return Ok(None);
    }
    dedupe_strings(&mut permissions.commands.allow);
    dedupe_strings(&mut permissions.commands.deny);
    dedupe_strings(&mut permissions.agent.require_confirmation);
    Ok(Some(permissions))
}

fn command_pattern_fields(pattern: &str) -> (Option<String>, Option<String>) {
    if let Some(prefix) = command_prefix_from_pattern(pattern) {
        return (Some(prefix), None);
    }
    (None, Some(glob_to_regex(pattern)))
}

fn command_prefix_from_pattern(pattern: &str) -> Option<String> {
    let trimmed = pattern.trim();
    if !trimmed.ends_with('*') || trimmed.matches('*').count() != 1 {
        return None;
    }
    let prefix = trimmed.trim_end_matches('*').trim();
    if prefix.is_empty() {
        return None;
    }
    Some(prefix.to_string())
}

fn glob_to_regex(glob: &str) -> String {
    let mut out = String::new();
    for ch in glob.chars() {
        match ch {
            '*' => out.push_str(".*"),
            '\\' | '.' | '+' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

fn value_to_string_list(value: Option<&toml::Value>) -> Vec<String> {
    match value {
        Some(toml::Value::String(s)) => vec![s.to_string()],
        Some(toml::Value::Array(values)) => values
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

fn read_codex_prefix_rules(root: &toml::value::Table) -> Vec<(String, String)> {
    read_codex_prefix_rules_from_value(&toml::Value::Table(root.clone()))
}

fn read_codex_prefix_rules_from_value(root: &toml::Value) -> Vec<(String, String)> {
    root.get("rules")
        .and_then(|v| v.as_table())
        .and_then(|table| table.get("prefix_rules"))
        .and_then(|v| v.as_array())
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let table = entry.as_table()?;
                    let prefix = table.get("prefix")?.as_str()?.to_string();
                    let decision = table.get("decision")?.as_str()?.to_string();
                    Some((prefix, decision))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn dedupe_strings(values: &mut Vec<String>) {
    let mut seen = HashSet::new();
    values.retain(|item| seen.insert(item.clone()));
}

fn dedupe_pairs(values: &mut Vec<(String, String)>) {
    let mut seen = HashSet::new();
    values.retain(|item| seen.insert(item.clone()));
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn home() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        let path = PathBuf::from(home);
        if !path.as_os_str().is_empty() {
            return Ok(path);
        }
    }
    home::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        HookConfig, HookTrigger, McpServerConfig, McpServerType, ModeConfig, PermissionConfig,
        ProjectConfig, save_config,
    };
    use crate::permissions::{Permissions, save_permissions};
    use crate::project::init_project;
    use crate::skill::{create_skill, delete_skill};
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn make_stdio_server(id: &str) -> McpServerConfig {
        McpServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), format!("@mcp/{}", id)],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        }
    }

    fn make_http_server(id: &str, url: &str) -> McpServerConfig {
        McpServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            command: String::new(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Http,
            url: Some(url.to_string()),
            disabled: false,
            timeout_secs: None,
        }
    }

    fn project_with_servers(servers: Vec<McpServerConfig>) -> (tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let project_dir = init_project(tmp.path().to_path_buf()).unwrap();
        let config = ProjectConfig {
            mcp_servers: servers,
            ..ProjectConfig::default()
        };
        save_config(&config, Some(project_dir.clone())).unwrap();
        (tmp, project_dir)
    }

    #[test]
    fn build_payload_active_mode_filters_servers_and_applies_mode_hooks_permissions() {
        let (_tmp, project_dir) = project_with_servers(vec![
            make_stdio_server("allowed"),
            make_stdio_server("blocked"),
        ]);
        save_permissions(
            project_dir.clone(),
            &Permissions {
                tools: crate::permissions::ToolPermissions {
                    allow: vec!["Read(*)".to_string()],
                    deny: vec!["Edit(*)".to_string()],
                },
                ..Default::default()
            },
        )
        .unwrap();
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.hooks = vec![HookConfig {
            id: "project-global-hook".to_string(),
            trigger: HookTrigger::PreToolUse,
            matcher: Some("Bash".to_string()),
            command: "echo global".to_string(),
        }];
        config.modes = vec![ModeConfig {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            description: None,
            active_tools: vec![],
            mcp_servers: vec!["allowed".to_string()],
            skills: vec![],
            rules: vec![],
            prompt_id: None,
            hooks: vec![HookConfig {
                id: "mode-hook".to_string(),
                trigger: HookTrigger::PostToolUse,
                matcher: Some("Bash".to_string()),
                command: "echo mode".to_string(),
            }],
            permissions: PermissionConfig {
                allow: vec!["Bash(*)".to_string()],
                deny: vec!["WebFetch(*)".to_string()],
            },
            target_agents: vec![],
        }];
        config.active_mode = Some("focus".to_string());
        save_config(&config, Some(project_dir.clone())).unwrap();

        let payload = build_payload(&project_dir).unwrap();
        let server_ids: Vec<_> = payload.servers.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(server_ids, vec!["allowed"]);
        assert_eq!(payload.active_mode_id.as_deref(), Some("focus"));
        assert_eq!(payload.permissions.tools.allow, vec!["Bash(*)".to_string()]);
        assert_eq!(
            payload.permissions.tools.deny,
            vec!["WebFetch(*)".to_string()]
        );

        let hook_ids: Vec<_> = payload.hooks.iter().map(|h| h.id.as_str()).collect();
        let global_idx = hook_ids
            .iter()
            .position(|id| *id == "project-global-hook")
            .expect("global hook missing");
        let mode_idx = hook_ids
            .iter()
            .position(|id| *id == "mode-hook")
            .expect("mode hook missing");
        assert!(
            global_idx < mode_idx,
            "mode hooks must append after global hooks"
        );
    }

    #[test]
    fn build_payload_without_active_mode_uses_permissions_toml() {
        let (_tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        save_permissions(
            project_dir.clone(),
            &Permissions {
                tools: crate::permissions::ToolPermissions {
                    allow: vec!["Read(*)".to_string()],
                    deny: vec!["Edit(*)".to_string()],
                },
                ..Default::default()
            },
        )
        .unwrap();
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.hooks = vec![HookConfig {
            id: "project-global-hook-only".to_string(),
            trigger: HookTrigger::Notification,
            matcher: None,
            command: "echo global".to_string(),
        }];
        config.modes = vec![ModeConfig {
            id: "unused".to_string(),
            name: "Unused".to_string(),
            description: None,
            active_tools: vec![],
            mcp_servers: vec![],
            skills: vec![],
            rules: vec![],
            prompt_id: None,
            hooks: vec![HookConfig {
                id: "unused-mode-hook".to_string(),
                trigger: HookTrigger::Stop,
                matcher: None,
                command: "echo unused".to_string(),
            }],
            permissions: PermissionConfig {
                allow: vec!["Bash(*)".to_string()],
                deny: vec!["WebFetch(*)".to_string()],
            },
            target_agents: vec![],
        }];
        config.active_mode = None;
        save_config(&config, Some(project_dir.clone())).unwrap();

        let payload = build_payload(&project_dir).unwrap();
        assert_eq!(payload.active_mode_id, None);
        assert_eq!(payload.permissions.tools.allow, vec!["Read(*)".to_string()]);
        assert_eq!(payload.permissions.tools.deny, vec!["Edit(*)".to_string()]);
        assert!(
            payload
                .hooks
                .iter()
                .any(|hook| hook.id == "project-global-hook-only")
        );
        assert!(
            !payload
                .hooks
                .iter()
                .any(|hook| hook.id == "unused-mode-hook")
        );
        assert!(payload.servers.iter().any(|server| server.id == "github"));
    }

    #[test]
    fn build_payload_mode_overrides_replace_only_tool_permissions() {
        let (_tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        save_permissions(
            project_dir.clone(),
            &Permissions {
                tools: crate::permissions::ToolPermissions {
                    allow: vec!["Read(*)".to_string()],
                    deny: vec!["Edit(*)".to_string()],
                },
                network: crate::permissions::NetworkPermissions {
                    policy: crate::permissions::NetworkPolicy::AllowList,
                    allow_hosts: vec!["api.example.com".to_string()],
                },
                ..Default::default()
            },
        )
        .unwrap();

        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.modes = vec![ModeConfig {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            permissions: PermissionConfig {
                allow: vec!["Bash(*)".to_string()],
                deny: vec!["WebFetch(*)".to_string()],
            },
            ..Default::default()
        }];
        config.active_mode = Some("focus".to_string());
        save_config(&config, Some(project_dir.clone())).unwrap();

        let payload = build_payload(&project_dir).unwrap();
        assert_eq!(payload.permissions.tools.allow, vec!["Bash(*)".to_string()]);
        assert_eq!(
            payload.permissions.tools.deny,
            vec!["WebFetch(*)".to_string()]
        );
        assert_eq!(
            payload.permissions.network.policy,
            crate::permissions::NetworkPolicy::AllowList
        );
        assert_eq!(
            payload.permissions.network.allow_hosts,
            vec!["api.example.com".to_string()]
        );
    }

    #[test]
    fn sync_active_mode_uses_connected_providers_when_targets_empty() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.providers = vec!["codex".to_string()];
        config.modes = vec![ModeConfig {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            target_agents: vec![],
            ..Default::default()
        }];
        config.active_mode = Some("focus".to_string());
        save_config(&config, Some(project_dir.clone())).unwrap();

        let synced = sync_active_mode(&project_dir).unwrap();
        assert_eq!(synced, vec!["codex".to_string()]);
        assert!(tmp.path().join(".codex").join("config.toml").exists());
    }

    #[test]
    fn sync_active_mode_without_active_mode_uses_connected_providers() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.providers = vec!["gemini".to_string()];
        config.active_mode = None;
        save_config(&config, Some(project_dir.clone())).unwrap();

        let synced = sync_active_mode(&project_dir).unwrap();
        assert_eq!(synced, vec!["gemini".to_string()]);
        assert!(tmp.path().join(".gemini").join("settings.json").exists());
    }

    #[test]
    fn sync_active_mode_normalizes_targets_and_skips_unknown_values() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.modes = vec![ModeConfig {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            target_agents: vec![
                " codex ".to_string(),
                "unknown-agent".to_string(),
                "CLAUDE".to_string(),
                "claude".to_string(),
                "".to_string(),
            ],
            ..Default::default()
        }];
        config.active_mode = Some("focus".to_string());
        save_config(&config, Some(project_dir.clone())).unwrap();

        let synced = sync_active_mode(&project_dir).unwrap();
        assert_eq!(
            synced,
            vec!["codex".to_string(), "claude".to_string()],
            "targets should be normalized, deduped, and unknown providers skipped"
        );
        assert!(tmp.path().join(".codex").join("config.toml").exists());
        assert!(tmp.path().join(".mcp.json").exists());
    }

    // ── Registry ───────────────────────────────────────────────────────────────

    #[test]
    fn all_provider_ids_are_unique() {
        let ids: Vec<_> = PROVIDERS.iter().map(|p| p.id).collect();
        let mut seen = std::collections::HashSet::new();
        for id in &ids {
            assert!(seen.insert(id), "duplicate provider id: {}", id);
        }
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
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap())
                .unwrap();
        let mcp = val["mcpServers"]["github"].as_object().unwrap();
        assert_eq!(mcp["command"].as_str().unwrap(), "npx");
        assert_eq!(mcp["type"].as_str().unwrap(), "stdio");
    }

    #[test]
    fn claude_round_trip_http_server() {
        let (tmp, project_dir) = project_with_servers(vec![make_http_server(
            "postgres",
            "http://localhost:5433/mcp",
        )]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert_eq!(
            val["mcpServers"]["postgres"]["type"].as_str().unwrap(),
            "http"
        );
        assert_eq!(
            val["mcpServers"]["postgres"]["url"].as_str().unwrap(),
            "http://localhost:5433/mcp"
        );
    }

    #[test]
    fn claude_ship_server_always_injected() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert!(val["mcpServers"]["ship"].is_object());
    }

    #[test]
    fn claude_marks_managed_servers() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("github")]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert_eq!(
            val["mcpServers"]["github"]["_ship"]["managed"].as_bool(),
            Some(true)
        );
    }

    #[test]
    fn claude_preserves_user_servers_across_write() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("mine")]);
        export_to(project_dir.clone(), "claude").unwrap();
        let mcp_json = tmp.path().join(".mcp.json");
        let mut val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&mcp_json).unwrap()).unwrap();
        val["mcpServers"]["user-server"] =
            serde_json::json!({ "command": "user-tool", "args": [] });
        std::fs::write(&mcp_json, serde_json::to_string_pretty(&val).unwrap()).unwrap();
        export_to(project_dir, "claude").unwrap();
        let val2: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&mcp_json).unwrap()).unwrap();
        assert!(
            val2["mcpServers"]["user-server"].is_object(),
            "user server was clobbered"
        );
    }

    #[test]
    fn claude_disabled_server_not_exported() {
        let mut s = make_stdio_server("disabled-one");
        s.disabled = true;
        let (tmp, project_dir) = project_with_servers(vec![s]);
        export_to(project_dir, "claude").unwrap();
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert!(val["mcpServers"]["disabled-one"].is_null());
    }

    #[test]
    fn claude_managed_state_written() {
        let (_tmp, project_dir) = project_with_servers(vec![make_stdio_server("gh")]);
        export_to(project_dir.clone(), "claude").unwrap();
        let (ids, _mode) = crate::state_db::get_managed_state_db(&project_dir, "claude").unwrap();
        assert!(
            ids.contains(&"gh".to_string()),
            "managed server not recorded in state"
        );
        // Clean up DB created in ~/.ship/state/ for this temp project
        std::fs::remove_file(crate::state_db::project_db_path(&project_dir).unwrap()).ok();
    }

    #[test]
    fn claude_permissions_round_trip_imports_back_to_canonical() {
        let (_tmp, project_dir) = project_with_servers(vec![]);
        let home = tempdir().unwrap();
        unsafe {
            std::env::set_var("HOME", home.path());
        }
        save_permissions(
            project_dir.clone(),
            &Permissions {
                tools: crate::permissions::ToolPermissions {
                    allow: vec!["Bash(*)".to_string()],
                    deny: vec!["WebFetch(*)".to_string()],
                },
                ..Default::default()
            },
        )
        .unwrap();

        export_to(project_dir.clone(), "claude").unwrap();

        save_permissions(project_dir.clone(), &Permissions::default()).unwrap();
        let imported = import_permissions_from_provider("claude", project_dir.clone()).unwrap();
        assert!(imported);
        let restored = crate::permissions::get_permissions(project_dir).unwrap();
        assert_eq!(restored.tools.allow, vec!["Bash(*)".to_string()]);
        assert_eq!(restored.tools.deny, vec!["WebFetch(*)".to_string()]);
        unsafe {
            std::env::remove_var("HOME");
        }
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
        let (tmp, project_dir) =
            project_with_servers(vec![make_http_server("figma", "https://mcp.figma.com/mcp")]);
        export_to(project_dir, "gemini").unwrap();
        let val: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".gemini/settings.json")).unwrap(),
        )
        .unwrap();
        assert!(
            val["mcpServers"]["figma"]["httpUrl"].is_string(),
            "Gemini must use httpUrl"
        );
        assert!(
            val["mcpServers"]["figma"]["url"].is_null(),
            "Gemini must not use url"
        );
    }

    #[test]
    fn gemini_preserves_non_mcp_fields() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("git")]);
        let settings_dir = tmp.path().join(".gemini");
        std::fs::create_dir_all(&settings_dir).unwrap();
        std::fs::write(
            settings_dir.join("settings.json"),
            r#"{"theme": "Dracula", "selectedAuthType": "gemini-api-key", "mcpServers": {}}"#,
        )
        .unwrap();
        export_to(project_dir, "gemini").unwrap();
        let val: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".gemini/settings.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(val["theme"].as_str().unwrap(), "Dracula");
        assert_eq!(val["selectedAuthType"].as_str().unwrap(), "gemini-api-key");
    }

    #[test]
    fn gemini_ship_server_always_injected() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        export_to(project_dir, "gemini").unwrap();
        let val: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join(".gemini/settings.json")).unwrap(),
        )
        .unwrap();
        assert!(val["mcpServers"]["ship"].is_object());
    }

    #[test]
    fn gemini_exports_workspace_policy_from_permissions() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        save_permissions(
            project_dir.clone(),
            &Permissions {
                tools: crate::permissions::ToolPermissions {
                    allow: vec!["mcp__ship__*".to_string()],
                    deny: vec!["WebFetch(*)".to_string()],
                },
                commands: crate::permissions::CommandPermissions {
                    allow: vec!["git status*".to_string()],
                    deny: vec!["rm -rf *".to_string()],
                },
                agent: crate::permissions::AgentLimits {
                    require_confirmation: vec!["git push *".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();

        export_to(project_dir, "gemini").unwrap();
        let policy_path = tmp
            .path()
            .join(".gemini")
            .join("policies")
            .join("ship-permissions.toml");
        assert!(policy_path.exists());
        let content = std::fs::read_to_string(policy_path).unwrap();
        assert!(content.contains("toolName = \"run_shell_command\""));
        assert!(content.contains("commandPrefix = \"git status\""));
        assert!(content.contains("decision = \"ask_user\""));
    }

    #[test]
    fn gemini_permissions_round_trip_imports_back_to_canonical() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        let policy_dir = tmp.path().join(".gemini").join("policies");
        std::fs::create_dir_all(&policy_dir).unwrap();
        std::fs::write(
            policy_dir.join("ship-permissions.toml"),
            r#"
[[rule]]
toolName = "run_shell_command"
commandPrefix = "git "
decision = "allow"
priority = 100

[[rule]]
toolName = "run_shell_command"
commandPrefix = "rm -rf "
decision = "deny"
priority = 900

[[rule]]
toolName = "run_shell_command"
commandPrefix = "git push "
decision = "ask_user"
priority = 800

[[rule]]
toolName = "mcp__ship__*"
decision = "allow"
priority = 700
"#,
        )
        .unwrap();

        let imported = import_permissions_from_provider("gemini", project_dir.clone()).unwrap();
        assert!(imported);
        let restored = crate::permissions::get_permissions(project_dir).unwrap();
        assert!(
            restored.commands.allow.contains(&"git *".to_string()),
            "expected command allow imported from Gemini policy"
        );
        assert!(restored.commands.deny.contains(&"rm -rf *".to_string()));
        assert!(
            restored
                .agent
                .require_confirmation
                .contains(&"git push *".to_string())
        );
        assert!(restored.tools.allow.contains(&"mcp__ship__*".to_string()));
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
        assert!(
            content.contains("[mcp_servers."),
            "must use mcp_servers (underscore)"
        );
        assert!(
            !content.contains("[mcp-servers."),
            "must NOT use mcp-servers (hyphen)"
        );
    }

    #[test]
    fn codex_round_trip_stdio_server() {
        let (tmp, project_dir) = project_with_servers(vec![make_stdio_server("gh")]);
        export_to(project_dir, "codex").unwrap();
        let val: toml::Value = toml::from_str(
            &std::fs::read_to_string(tmp.path().join(".codex/config.toml")).unwrap(),
        )
        .unwrap();
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
        let val: toml::Value =
            toml::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert!(
            val["mcp_servers"]["user-tool"].is_table(),
            "user server was clobbered"
        );
    }

    #[test]
    fn codex_ship_server_always_injected() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        export_to(project_dir, "codex").unwrap();
        let val: toml::Value = toml::from_str(
            &std::fs::read_to_string(tmp.path().join(".codex/config.toml")).unwrap(),
        )
        .unwrap();
        assert!(val["mcp_servers"]["ship"].is_table());
    }

    #[test]
    fn codex_exports_permissions_to_native_fields() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        save_permissions(
            project_dir.clone(),
            &Permissions {
                commands: crate::permissions::CommandPermissions {
                    allow: vec!["cargo *".to_string()],
                    deny: vec!["rm -rf *".to_string()],
                },
                network: crate::permissions::NetworkPermissions {
                    policy: crate::permissions::NetworkPolicy::AllowList,
                    allow_hosts: vec!["github.com".to_string()],
                },
                agent: crate::permissions::AgentLimits {
                    require_confirmation: vec!["git push *".to_string()],
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();

        export_to(project_dir, "codex").unwrap();
        let val: toml::Value = toml::from_str(
            &std::fs::read_to_string(tmp.path().join(".codex/config.toml")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            val["sandbox_mode"].as_str(),
            Some("workspace-write"),
            "codex export should enforce workspace-write sandbox for mapped permissions"
        );
        assert_eq!(
            val["sandbox_workspace_write"]["network_access"].as_bool(),
            Some(true)
        );
        assert_eq!(val["approval_policy"].as_str(), Some("on-request"));
        assert!(val["rules"]["prefix_rules"].is_array());
    }

    #[test]
    fn codex_permissions_round_trip_imports_back_to_canonical() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        let codex_dir = tmp.path().join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::write(
            codex_dir.join("config.toml"),
            r#"
sandbox_mode = "workspace-write"
approval_policy = "on-request"
allow = ["cargo *"]

[sandbox_workspace_write]
network_access = false

[rules]
prefix_rules = [
  { prefix = "rm -rf ", decision = "forbidden" },
  { prefix = "git push ", decision = "prompt" }
]
"#,
        )
        .unwrap();

        let imported = import_permissions_from_provider("codex", project_dir.clone()).unwrap();
        assert!(imported);
        let restored = crate::permissions::get_permissions(project_dir).unwrap();
        assert_eq!(
            restored.network.policy,
            crate::permissions::NetworkPolicy::None
        );
        assert!(restored.commands.allow.contains(&"cargo *".to_string()));
        assert!(restored.commands.deny.contains(&"rm -rf *".to_string()));
        assert!(
            restored
                .agent
                .require_confirmation
                .contains(&"git push *".to_string())
        );
    }

    #[test]
    fn codex_export_prunes_stale_managed_skill_dirs() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        create_skill(&project_dir, "rt-live-skill", "Live", "live body").unwrap();
        create_skill(&project_dir, "rt-stale-skill", "Stale", "stale body").unwrap();

        export_to(project_dir.clone(), "codex").unwrap();
        let skills_dir = tmp.path().join(".agents").join("skills");
        let live_skill_dir = skills_dir.join("rt-live-skill");
        let stale_skill_dir = skills_dir.join("rt-stale-skill");
        assert!(live_skill_dir.join("SKILL.md").exists());
        assert!(stale_skill_dir.join("SKILL.md").exists());

        delete_skill(&project_dir, "rt-stale-skill").unwrap();
        export_to(project_dir, "codex").unwrap();

        assert!(live_skill_dir.join("SKILL.md").exists());
        assert!(
            !stale_skill_dir.exists(),
            "stale managed skill directory should be pruned on export"
        );
    }

    #[test]
    fn codex_export_applies_active_mode_skill_filter() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        create_skill(&project_dir, "rt-allowed-skill", "Allowed", "allowed body").unwrap();
        create_skill(&project_dir, "rt-blocked-skill", "Blocked", "blocked body").unwrap();

        let mut config = crate::config::get_config(Some(project_dir.clone())).unwrap();
        config.modes = vec![ModeConfig {
            id: "focus".to_string(),
            name: "Focus".to_string(),
            description: None,
            active_tools: vec![],
            mcp_servers: vec![],
            skills: vec!["rt-allowed-skill".to_string()],
            rules: vec![],
            prompt_id: None,
            hooks: vec![],
            permissions: PermissionConfig::default(),
            target_agents: vec![],
        }];
        config.active_mode = Some("focus".to_string());
        save_config(&config, Some(project_dir.clone())).unwrap();

        export_to(project_dir, "codex").unwrap();
        let skills_dir = tmp.path().join(".agents").join("skills");
        assert!(
            skills_dir
                .join("rt-allowed-skill")
                .join("SKILL.md")
                .exists()
        );
        assert!(
            !skills_dir.join("rt-blocked-skill").exists(),
            "skills excluded by active mode should not be exported"
        );
    }

    #[test]
    fn codex_export_preserves_unmanaged_skill_dirs() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        let unmanaged_dir = tmp
            .path()
            .join(".agents")
            .join("skills")
            .join("rt-unmanaged-skill");
        std::fs::create_dir_all(&unmanaged_dir).unwrap();
        let unmanaged_file = unmanaged_dir.join("SKILL.md");
        std::fs::write(&unmanaged_file, "manual skill content").unwrap();

        export_to(project_dir, "codex").unwrap();

        assert!(unmanaged_dir.exists());
        let content = std::fs::read_to_string(&unmanaged_file).unwrap();
        assert_eq!(content, "manual skill content");
    }

    #[test]
    fn codex_export_migrates_and_exports_legacy_repo_local_skills() {
        let (tmp, project_dir) = project_with_servers(vec![]);
        let legacy_skill_dir = project_dir
            .join("agents")
            .join("skills")
            .join("legacy-export");
        std::fs::create_dir_all(&legacy_skill_dir).unwrap();
        std::fs::write(
            legacy_skill_dir.join("SKILL.md"),
            r#"---
name: legacy-export
description: Legacy repo-local skill that should be migrated and exported.
---

Legacy export skill body.
"#,
        )
        .unwrap();

        export_to(project_dir, "codex").unwrap();

        let exported = tmp
            .path()
            .join(".agents")
            .join("skills")
            .join("legacy-export")
            .join("SKILL.md");
        assert!(
            exported.exists(),
            "legacy skill should be exported after migration"
        );
        let exported_body = std::fs::read_to_string(exported).unwrap();
        assert!(exported_body.contains("Legacy export skill body."));
        assert!(
            !legacy_skill_dir.exists(),
            "legacy repo-local skill should be migrated out of .ship"
        );
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
                id: id.clone(),
                name: id.clone(),
                command: entry
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                args: entry
                    .get("args")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default(),
                env: HashMap::new(),
                scope: "global".to_string(),
                server_type,
                url: entry
                    .get("url")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                disabled: false,
                timeout_secs: None,
            });
        }
        save_config(&config, Some(project_dir.clone())).unwrap();
        let reloaded = crate::config::get_config(Some(project_dir)).unwrap();
        assert_eq!(reloaded.mcp_servers.len(), 2);
        assert!(reloaded.mcp_servers.iter().any(|s| s.id == "github"));
        assert!(reloaded.mcp_servers.iter().any(|s| s.id == "postgres"));
    }
}
