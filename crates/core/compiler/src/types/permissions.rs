use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ToolPermissions {
    #[serde(default = "allow_all_default")]
    pub allow: Vec<String>,
    /// Patterns that trigger confirmation before tool use (Claude "ask" tier).
    #[serde(default)]
    pub ask: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FsPermissions {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CommandPermissions {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkPolicy {
    #[default]
    None,
    Localhost,
    AllowList,
    Unrestricted,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NetworkPermissions {
    #[serde(default)]
    pub policy: NetworkPolicy,
    #[serde(default)]
    pub allow_hosts: Vec<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentLimits {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cost_per_session: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
    #[serde(default)]
    pub require_confirmation: Vec<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Permissions {
    #[serde(default)]
    pub tools: ToolPermissions,
    #[serde(default)]
    pub filesystem: FsPermissions,
    #[serde(default)]
    pub commands: CommandPermissions,
    #[serde(default)]
    pub network: NetworkPermissions,
    #[serde(default)]
    pub agent: AgentLimits,
    /// Claude Code default permission mode: "default", "acceptEdits", "plan", "bypassPermissions".
    #[serde(default)]
    pub default_mode: Option<String>,
    /// Additional working directories Claude has access to.
    #[serde(default)]
    pub additional_directories: Vec<String>,
}

fn allow_all_default() -> Vec<String> {
    vec!["*".to_string()]
}
