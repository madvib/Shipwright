use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ─── Status / Git ─────────────────────────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
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

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitConfig {
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub commit: Vec<String>,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            ignore: Vec::new(),
            commit: vec![
                "agents".to_string(),
                "ship.toml".to_string(),
                "templates".to_string(),
                "vision".to_string(),
            ],
        }
    }
}

// ─── AI / Provider ────────────────────────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AiConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub cli_path: Option<String>,
}

impl AiConfig {
    pub fn effective_provider(&self) -> &str {
        self.provider.as_deref().unwrap_or("claude")
    }

    pub fn effective_cli(&self) -> &str {
        self.cli_path
            .as_deref()
            .unwrap_or_else(|| self.effective_provider())
    }
}

// ─── Hooks ────────────────────────────────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum HookTrigger {
    PreToolUse,
    PostToolUse,
    Notification,
    Stop,
    SubagentStop,
    PreCompact,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HookConfig {
    pub id: String,
    pub trigger: HookTrigger,
    #[serde(default)]
    pub matcher: Option<String>,
    pub command: String,
}

// ─── Modes ────────────────────────────────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PermissionConfig {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ModeConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub active_tools: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub prompt_id: Option<String>,
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
    #[serde(default)]
    pub permissions: PermissionConfig,
    #[serde(default)]
    pub target_agents: Vec<String>,
}

// ─── MCP servers ──────────────────────────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpServerType {
    #[default]
    Stdio,
    Sse,
    Http,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpServerConfig {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(default)]
    pub server_type: McpServerType,
    pub url: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    pub timeout_secs: Option<u32>,
}

fn default_scope() -> String {
    "global".to_string()
}

// ─── Agent layer ──────────────────────────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentLayerConfig {
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub prompts: Vec<String>,
    #[serde(default)]
    pub context: Vec<String>,
}

// ─── Namespaces ───────────────────────────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NamespaceConfig {
    pub id: String,
    pub path: String,
    pub owner: String,
}

// ─── Root project config (ship.toml) ─────────────────────────────────────────

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectConfig {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default = "default_statuses")]
    pub statuses: Vec<StatusConfig>,
    #[serde(default)]
    pub providers: Vec<String>,
    pub ai: Option<AiConfig>,
    #[serde(default)]
    pub modes: Vec<ModeConfig>,
    #[serde(default)]
    pub active_agent: Option<String>,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
    #[serde(default)]
    pub namespaces: Vec<NamespaceConfig>,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(skip, default)]
    pub agent: AgentLayerConfig,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            id: String::new(),
            name: None,
            description: None,
            statuses: default_statuses(),
            providers: Vec::new(),
            ai: None,
            modes: Vec::new(),
            active_agent: None,
            mcp_servers: Vec::new(),
            hooks: Vec::new(),
            namespaces: Vec::new(),
            git: GitConfig::default(),
            agent: AgentLayerConfig::default(),
        }
    }
}

fn default_version() -> String {
    "1".to_string()
}

fn default_statuses() -> Vec<StatusConfig> {
    vec![
        StatusConfig {
            id: "backlog".to_string(),
            name: "Backlog".to_string(),
            color: "gray".to_string(),
        },
        StatusConfig {
            id: "in-progress".to_string(),
            name: "In Progress".to_string(),
            color: "blue".to_string(),
        },
        StatusConfig {
            id: "blocked".to_string(),
            name: "Blocked".to_string(),
            color: "red".to_string(),
        },
        StatusConfig {
            id: "done".to_string(),
            name: "Done".to_string(),
            color: "green".to_string(),
        },
    ]
}
