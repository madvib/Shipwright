use serde::{Deserialize, Serialize};

/// An agent parsed from `.ship/agents/<id>.toml`.
///
/// Profiles define specialist agents that the compiler emits as
/// provider-native subagent definitions (`.claude/agents/`, `.gemini/agents/`,
/// `.cursor/agents/`, `.codex/agents/`).
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
    #[cfg_attr(feature = "specta", specta(skip))]
    pub provider_settings: std::collections::HashMap<String, toml::Value>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
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

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillRefs {
    #[serde(default)]
    pub refs: Vec<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpRefs {
    #[serde(default)]
    pub servers: Vec<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginRefs {
    #[serde(default)]
    pub install: Vec<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfilePermissions {
    /// Permission preset name (e.g. "ship-standard", "ship-autonomous").
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

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileRules {
    /// Inline rules body — becomes the agent's system prompt.
    #[serde(default)]
    pub inline: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_full_profile() {
        let toml_str = r#"
[profile]
id = "code-reviewer"
name = "Code Reviewer"
version = "1.2.0"
description = "Reviews pull requests for style, correctness, and test coverage."
providers = ["claude", "cursor"]

[skills]
refs = ["review-pr", "lint-fix", "test-runner"]

[mcp]
servers = ["github", "linear"]

[plugins]
install = ["eslint-plugin", "prettier-plugin"]
scope = "workspace"

[permissions]
preset = "ship-standard"
tools_allow = ["Read", "Grep", "Bash"]
tools_ask = ["Edit", "Write"]
tools_deny = ["WebFetch"]
default_mode = "acceptEdits"

[rules]
inline = """
You are a senior code reviewer.
Focus on correctness, readability, and test coverage.
"""

[provider_settings]
claude_model = "opus"
temperature = 0.2
"#;

        let profile: AgentProfile =
            toml::from_str(toml_str).expect("failed to parse full profile");

        // profile section
        assert_eq!(profile.profile.id, "code-reviewer");
        assert_eq!(profile.profile.name, "Code Reviewer");
        assert_eq!(
            profile.profile.version.as_deref(),
            Some("1.2.0")
        );
        assert_eq!(
            profile.profile.description.as_deref(),
            Some("Reviews pull requests for style, correctness, and test coverage.")
        );
        assert_eq!(profile.profile.providers, vec!["claude", "cursor"]);

        // skills
        assert_eq!(
            profile.skills.refs,
            vec!["review-pr", "lint-fix", "test-runner"]
        );

        // mcp
        assert_eq!(profile.mcp.servers, vec!["github", "linear"]);

        // plugins
        assert_eq!(
            profile.plugins.install,
            vec!["eslint-plugin", "prettier-plugin"]
        );
        assert_eq!(profile.plugins.scope.as_deref(), Some("workspace"));

        // permissions
        assert_eq!(
            profile.permissions.preset.as_deref(),
            Some("ship-standard")
        );
        assert_eq!(
            profile.permissions.tools_allow,
            vec!["Read", "Grep", "Bash"]
        );
        assert_eq!(profile.permissions.tools_ask, vec!["Edit", "Write"]);
        assert_eq!(profile.permissions.tools_deny, vec!["WebFetch"]);
        assert_eq!(
            profile.permissions.default_mode.as_deref(),
            Some("acceptEdits")
        );

        // rules
        assert!(profile.rules.inline.as_ref().unwrap().contains("senior code reviewer"));

        // provider_settings
        assert_eq!(
            profile.provider_settings.get("claude_model").and_then(|v| v.as_str()),
            Some("opus")
        );

        // round-trip: serialize back to TOML and re-parse
        let serialized = toml::to_string(&profile).expect("failed to serialize");
        let reparsed: AgentProfile =
            toml::from_str(&serialized).expect("failed to re-parse serialized profile");
        assert_eq!(reparsed.profile.id, profile.profile.id);
        assert_eq!(reparsed.skills.refs, profile.skills.refs);
        assert_eq!(reparsed.mcp.servers, profile.mcp.servers);
        assert_eq!(reparsed.permissions.tools_allow, profile.permissions.tools_allow);
    }

    #[test]
    fn round_trip_minimal_profile() {
        let toml_str = r#"
[profile]
id = "basic-agent"
name = "Basic Agent"
"#;

        let profile: AgentProfile =
            toml::from_str(toml_str).expect("failed to parse minimal profile");

        assert_eq!(profile.profile.id, "basic-agent");
        assert_eq!(profile.profile.name, "Basic Agent");
        assert_eq!(profile.profile.version, None);
        assert_eq!(profile.profile.description, None);
        assert!(profile.profile.providers.is_empty());
        assert!(profile.skills.refs.is_empty());
        assert!(profile.mcp.servers.is_empty());
        assert!(profile.plugins.install.is_empty());
        assert_eq!(profile.plugins.scope, None);
        assert_eq!(profile.permissions.preset, None);
        assert!(profile.permissions.tools_allow.is_empty());
        assert!(profile.permissions.tools_ask.is_empty());
        assert!(profile.permissions.tools_deny.is_empty());
        assert_eq!(profile.permissions.default_mode, None);
        assert_eq!(profile.rules.inline, None);
        assert!(profile.provider_settings.is_empty());

        // round-trip
        let serialized = toml::to_string(&profile).expect("failed to serialize");
        let reparsed: AgentProfile =
            toml::from_str(&serialized).expect("failed to re-parse minimal profile");
        assert_eq!(reparsed.profile.id, "basic-agent");
        assert_eq!(reparsed.profile.name, "Basic Agent");
    }
}
