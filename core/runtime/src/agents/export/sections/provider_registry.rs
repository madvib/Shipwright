use crate::config::{
    HookConfig, HookTrigger, McpServerConfig, McpServerType, ModeConfig, ProjectConfig,
    get_config, get_effective_config,
};
use crate::permissions::{Permissions, get_permissions};
use crate::skill::{get_effective_skill, list_effective_skills};
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
