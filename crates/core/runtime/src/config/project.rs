use super::types::{
    AgentLayerConfig, AiConfig, GitConfig, HookConfig, NamespaceConfig, PermissionConfig,
    StatusConfig, is_agent_layer_empty,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::path::PathBuf;

// ─── Agent / MCP / Project types ─────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct AgentProfile {
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
    /// Stable project identity. Generated on `ship init`, used as the SQLite state directory key.
    /// Never changes after creation — rename the project by changing `name`, not `id`.
    #[serde(default)]
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default = "default_statuses")]
    pub statuses: Vec<StatusConfig>,
    #[serde(default)]
    pub git: GitConfig,
    pub ai: Option<AiConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modes: Vec<AgentProfile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_agent: Option<String>,
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(super) struct ProjectCoreFile {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

pub fn default_version() -> String {
    "1".to_string()
}

pub fn default_providers() -> Vec<String> {
    vec!["claude".to_string()]
}

pub fn default_statuses() -> Vec<StatusConfig> {
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

fn default_namespaces() -> Vec<NamespaceConfig> {
    super::types::default_namespaces()
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            id: String::new(),
            name: None,
            description: None,
            statuses: default_statuses(),
            git: GitConfig::default(),
            ai: None,
            modes: Vec::new(),
            mcp_servers: Vec::new(),
            active_agent: None,
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
}
