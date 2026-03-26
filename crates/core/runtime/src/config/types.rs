use serde::{Deserialize, Serialize};
use specta::Type;

pub const PRIMARY_CONFIG_FILE: &str = "ship.jsonc";
pub const LEGACY_CONFIG_FILE: &str = "ship.toml";

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
        // Default:
        // - Keep project docs local by default.
        // - Keep core control-plane config and always-on rules tracked.
        Self {
            ignore: Vec::new(),
            commit: vec![
                "ship.jsonc".to_string(),
                "mcp".to_string(),
                "permissions".to_string(),
                "rules".to_string(),
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
    SessionStart,
    UserPromptSubmit,
    PreToolUse,
    PermissionRequest,
    PostToolUse,
    PostToolUseFailure,
    Notification,
    SubagentStart,
    Stop,
    SubagentStop,
    PreCompact,
    BeforeTool,
    AfterTool,
    BeforeAgent,
    AfterAgent,
    SessionEnd,
    BeforeModel,
    AfterModel,
    BeforeToolSelection,
}

/// A shell command executed on a lifecycle event.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct HookConfig {
    pub id: String,
    pub trigger: HookTrigger,
    /// Glob/regex pattern to match tool name (e.g. "Bash", "mcp__*"). Empty = all tools.
    #[serde(default)]
    pub matcher: Option<String>,
    /// Optional timeout for the hook command in milliseconds.
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    /// Human-readable intent for editor UX and exported provider config.
    #[serde(default)]
    pub description: Option<String>,
    /// The shell command to run
    pub command: String,
}

/// Mode-scoped tool permission overrides.
/// These overlay canonical `.ship/permissions.jsonc` `tools.allow/deny`
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

pub fn is_agent_layer_empty(config: &AgentLayerConfig) -> bool {
    config.skills.is_empty() && config.prompts.is_empty() && config.context.is_empty()
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(super) struct LegacyAgentsConfigFile {
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub active_agent: Option<String>,
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
pub struct NamespaceConfig {
    /// Stable namespace id (e.g. "project", "agents", "generated", "plugin:ghost-issues")
    pub id: String,
    /// Directory path relative to `.ship/`
    pub path: String,
    /// Owning module or family (e.g. "project", "agents", "runtime", "plugins")
    pub owner: String,
}

// Used as a serde default in project.rs — must be pub.
pub fn default_namespaces() -> Vec<NamespaceConfig> {
    vec![
        NamespaceConfig {
            id: "project".to_string(),
            path: "project".to_string(),
            owner: "project".to_string(),
        },
        NamespaceConfig {
            id: "agents".to_string(),
            path: "agents".to_string(),
            owner: "agents".to_string(),
        },
    ]
}

