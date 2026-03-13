use crate::config::{
    HookConfig, HookTrigger, McpServerConfig, McpServerType, get_config, get_effective_config,
};
use crate::permissions::{Permissions, get_permissions};
use crate::project::{SHIP_DIR_NAME, ship_dir_from_path};
use crate::skill::list_effective_skills;
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Serializable model info for Tauri/MCP.
#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub context_window: u32,
    pub recommended: bool,
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
}

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
        emit_type_field: true,
        managed_marker: ManagedMarker::Inline,
        prompt_output: PromptOutput::GeminiMd,
        skills_output: SkillsOutput::AgentSkills,
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
    pub global_config: String,
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
    resolve_binary_path(binary).is_some()
}

/// Returns the version string from `<binary> --version` (first line), or None.
pub fn detect_version(binary: &str) -> Option<String> {
    let command = resolve_binary_path(binary).unwrap_or_else(|| PathBuf::from(binary));
    let out = std::process::Command::new(command)
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

fn resolve_binary_path(binary: &str) -> Option<PathBuf> {
    let mut dirs = Vec::<PathBuf>::new();
    let mut seen = HashSet::<PathBuf>::new();

    if let Some(path_env) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_env) {
            if seen.insert(dir.clone()) {
                dirs.push(dir);
            }
        }
    }

    for dir in fallback_binary_dirs() {
        if seen.insert(dir.clone()) {
            dirs.push(dir);
        }
    }

    for dir in dirs {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        for ext in ["exe", "cmd", "bat", "com"] {
            let with_ext = dir.join(format!("{}.{}", binary, ext));
            if with_ext.is_file() {
                return Some(with_ext);
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn fallback_binary_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/opt/local/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        dirs.push(home.join(".cargo/bin"));
        dirs.push(home.join(".local/bin"));
        dirs.push(home.join("bin"));
    }
    dirs
}

#[cfg(all(not(target_os = "macos"), not(windows)))]
fn fallback_binary_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        dirs.push(home.join(".cargo/bin"));
        dirs.push(home.join(".local/bin"));
        dirs.push(home.join("bin"));
    }
    dirs
}

#[cfg(windows)]
fn fallback_binary_dirs() -> Vec<PathBuf> {
    Vec::new()
}

fn provider_info(d: &ProviderDescriptor, enabled: bool, project_dir: Option<&Path>) -> ProviderInfo {
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
        global_config: d.global_config.to_string(),
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
        models: discover_models(d, project_dir),
    }
}

/// Return all registered providers, each annotated with enabled + installed status.
pub fn list_providers(project_dir: &std::path::Path) -> anyhow::Result<Vec<ProviderInfo>> {
    let config = get_config(Some(project_dir.to_path_buf()))?;
    let enabled: std::collections::HashSet<&str> =
        config.providers.iter().map(|s| s.as_str()).collect();
    Ok(PROVIDERS
        .iter()
        .map(|d| provider_info(d, enabled.contains(d.id), Some(project_dir)))
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
    Ok(discover_models(d, resolve_project_dir_for_models().as_deref()))
}

fn require_provider(id: &str) -> Result<&'static ProviderDescriptor> {
    get_provider(id).ok_or_else(|| {
        let known: Vec<&str> = PROVIDERS.iter().map(|p| p.id).collect();
        anyhow!("Unknown provider '{}'. Known: {}", id, known.join(", "))
    })
}

fn resolve_project_dir_for_models() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    if cwd
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == SHIP_DIR_NAME)
    {
        return Some(cwd);
    }
    let direct = cwd.join(SHIP_DIR_NAME);
    if direct.is_dir() {
        return Some(direct);
    }
    ship_dir_from_path(&cwd)
}

fn discover_models(desc: &ProviderDescriptor, project_dir: Option<&Path>) -> Vec<ModelInfo> {
    let mut models = Vec::new();
    let mut seen = HashSet::new();

    // Provider env hints (explicitly set by users in shell/startup).
    for env_key in provider_model_env_keys(desc.id) {
        if let Ok(value) = std::env::var(env_key) {
            push_model(&mut models, &mut seen, desc.id, value.trim(), true);
        }
    }

    // Model IDs from provider-native config files.
    for path in provider_model_config_paths(desc, project_dir) {
        match desc.config_format {
            ConfigFormat::Json => {
                if let Ok(raw) = fs::read_to_string(&path)
                    && let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw)
                {
                    collect_json_models(desc.id, &json, &mut models, &mut seen);
                }
            }
            ConfigFormat::Toml => {
                if let Ok(raw) = fs::read_to_string(&path)
                    && let Ok(toml) = toml::from_str::<toml::Value>(&raw)
                {
                    collect_toml_models(desc.id, &toml, &mut models, &mut seen);
                }
            }
        }
    }

    // Ship-side preference for generation model.
    if let Some(project_dir) = project_dir
        && let Ok(config) = get_config(Some(project_dir.to_path_buf()))
        && let Some(ai) = config.ai.as_ref()
        && ai.effective_provider() == desc.id
        && let Some(model) = ai.model.as_deref()
    {
        push_model(&mut models, &mut seen, desc.id, model.trim(), false);
    }

    models
}

fn provider_model_env_keys(provider_id: &str) -> &'static [&'static str] {
    match provider_id {
        "claude" => &["ANTHROPIC_MODEL", "CLAUDE_CODE_MODEL"],
        "gemini" => &["GEMINI_MODEL"],
        "codex" => &["OPENAI_MODEL", "CODEX_MODEL"],
        _ => &[],
    }
}

fn provider_model_config_paths(desc: &ProviderDescriptor, project_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(project_dir) = project_dir
        && let Some(project_root) = project_dir.parent()
    {
        paths.push(project_root.join(desc.project_config));
    }
    // Fallback to HOME for providers whose global config is not under ~/.ship.
    if let Some(home) = std::env::var_os("HOME") {
        paths.push(PathBuf::from(home).join(desc.global_config));
    }
    let mut deduped = Vec::new();
    let mut seen = HashSet::new();
    for path in paths {
        if seen.insert(path.clone()) {
            deduped.push(path);
        }
    }
    deduped
}

fn push_model(
    models: &mut Vec<ModelInfo>,
    seen: &mut HashSet<String>,
    provider_id: &str,
    raw_id: &str,
    recommended: bool,
) {
    let id = raw_id.trim();
    if id.is_empty() {
        return;
    }
    if !seen.insert(id.to_string()) {
        // Promote to recommended if any source marks it as default.
        if recommended
            && let Some(existing) = models.iter_mut().find(|entry| entry.id == id)
        {
            existing.recommended = true;
        }
        return;
    }
    models.push(ModelInfo {
        id: id.to_string(),
        name: id.to_string(),
        provider_id: provider_id.to_string(),
        context_window: 0,
        recommended,
    });
}

fn collect_json_models(
    provider_id: &str,
    root: &serde_json::Value,
    models: &mut Vec<ModelInfo>,
    seen: &mut HashSet<String>,
) {
    if let Some(model) = root.get("model").and_then(|v| v.as_str()) {
        push_model(models, seen, provider_id, model, true);
    }
    if let Some(model) = root
        .get("model")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
    {
        push_model(models, seen, provider_id, model, true);
    }

    // Claude model aliases.
    if let Some(aliases) = root.get("modelAliases").and_then(|v| v.as_object()) {
        for (alias, value) in aliases {
            push_model(models, seen, provider_id, alias, false);
            if let Some(target) = value.as_str() {
                push_model(models, seen, provider_id, target, false);
            }
        }
    }

    // Gemini model configs.
    if let Some(model_configs) = root.get("modelConfigs").and_then(|v| v.as_object()) {
        for (name, entry) in model_configs {
            push_model(models, seen, provider_id, name, false);
            if let Some(model_name) = entry.get("name").and_then(|v| v.as_str()) {
                push_model(models, seen, provider_id, model_name, false);
            }
            if let Some(model_name) = entry.get("model").and_then(|v| v.as_str()) {
                push_model(models, seen, provider_id, model_name, false);
            }
        }
    }
}

fn collect_toml_models(
    provider_id: &str,
    root: &toml::Value,
    models: &mut Vec<ModelInfo>,
    seen: &mut HashSet<String>,
) {
    if let Some(model) = root.get("model").and_then(|v| v.as_str()) {
        push_model(models, seen, provider_id, model, true);
    }
    if let Some(model) = root.get("model_name").and_then(|v| v.as_str()) {
        push_model(models, seen, provider_id, model, false);
    }
    if let Some(table) = root.get("model_providers").and_then(|v| v.as_table()) {
        for (_, provider_entry) in table {
            if let Some(model) = provider_entry.get("model").and_then(|v| v.as_str()) {
                push_model(models, seen, provider_id, model, false);
            }
            if let Some(array) = provider_entry.get("models").and_then(|v| v.as_array()) {
                for model in array.iter().filter_map(|item| item.as_str()) {
                    push_model(models, seen, provider_id, model, false);
                }
            }
        }
    }
}
