use serde::{Deserialize, Serialize};

/// An agent profile parsed from `.ship/agents/profiles/<id>.toml`.
///
/// Profiles define specialist agents that the compiler emits as
/// provider-native subagent definitions (`.claude/agents/`, `.gemini/agents/`,
/// `.cursor/agents/`, `.codex/agents/`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub profile: ProfileMeta,
    #[serde(default)]
    pub skills: SkillRefs,
    #[serde(default)]
    pub mcp: McpRefs,
    #[serde(default)]
    pub plugins: PluginRefs,
    #[serde(default)]
    pub permissions: ProfilePermissions,
    #[serde(default)]
    pub rules: ProfileRules,
    #[serde(default)]
    pub provider_settings: std::collections::HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMeta {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub providers: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillRefs {
    #[serde(default)]
    pub refs: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpRefs {
    #[serde(default)]
    pub servers: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginRefs {
    #[serde(default)]
    pub install: Vec<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfilePermissions {
    /// Permission preset name (e.g. "ship-standard", "ship-guarded").
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub tools_allow: Vec<String>,
    #[serde(default)]
    pub tools_ask: Vec<String>,
    #[serde(default)]
    pub tools_deny: Vec<String>,
    /// Claude Code permission mode: "default", "acceptEdits", "plan", "bypassPermissions".
    #[serde(default)]
    pub default_mode: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileRules {
    /// Inline rules body — becomes the agent's system prompt.
    #[serde(default)]
    pub inline: Option<String>,
}
