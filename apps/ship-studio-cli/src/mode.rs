use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::loader::load_permission_preset;

/// Profile TOML format — what users author in .ship/agents/profiles/<id>.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(rename = "profile")]
    pub meta: ProfileMeta,
    #[serde(default)]
    pub skills: SkillsConfig,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub plugins: PluginsConfig,
    #[serde(default)]
    pub permissions: ProfilePermissions,
    #[serde(default)]
    pub rules: RulesConfig,
    #[serde(default)]
    pub hooks: ProfileHooks,
    /// Provider-specific settings merged verbatim into the provider's config file.
    /// `[provider_settings.claude]` → `.claude/settings.json`.
    /// Any key/value valid in that file works here — no code change required.
    #[serde(default)]
    pub provider_settings: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMeta {
    pub name: String,
    pub id: String,
    #[serde(default = "default_version")]
    pub version: String,
    pub description: Option<String>,
    /// Provider targets for this profile (overrides project ship.toml providers).
    #[serde(default)]
    pub providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillsConfig {
    #[serde(default)]
    pub refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginsConfig {
    /// Plugin IDs to install (e.g. "superpowers@claude-plugins-official").
    #[serde(default)]
    pub install: Vec<String>,
    /// Scope for plugin installation: "project" | "user".
    #[serde(default = "default_plugin_scope")]
    pub scope: String,
}

/// Permission overrides in a profile — merged on top of agents/permissions.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfilePermissions {
    /// Permission tier shorthand: "ship-readonly" | "ship-standard" | "ship-autonomous" | "ship-elevated"
    pub preset: Option<String>,
    #[serde(default)]
    pub tools_deny: Vec<String>,
    #[serde(default)]
    pub tools_ask: Vec<String>,
    pub default_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RulesConfig {
    /// Inline always-on rules appended after agents/rules/*.md
    pub inline: Option<String>,
}

/// Hook commands declared in a profile.
/// Each field corresponds to a Claude Code hook trigger name.
/// The compiler emits these into `.claude/settings.json` under `hooks`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileHooks {
    /// Command to run when the agent session ends (Claude Code Stop hook).
    pub stop: Option<String>,
    /// Command to run when a sub-agent session ends (Claude Code SubagentStop hook).
    pub subagent_stop: Option<String>,
}

fn default_version() -> String { "0.1.0".to_string() }
fn default_plugin_scope() -> String { "project".to_string() }

impl Profile {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("invalid profile TOML at {}: {}", path.display(), e))
    }

    /// Template for a new profile file.
    pub fn scaffold(id: &str) -> String {
        format!(
r#"[profile]
name = "{name}"
id = "{id}"
version = "0.1.0"
description = ""
providers = ["claude"]

[skills]
# Skill IDs to activate (empty = all installed skills).
refs = []

[mcp]
# MCP server IDs to activate (empty = all configured servers).
servers = []

[plugins]
# install = ["superpowers@claude-plugins-official"]
# scope = "project"   # project | user

[permissions]
# preset = "ship-autonomous"   # ship-readonly | ship-standard | ship-autonomous | ship-elevated
# tools_deny = ["mcp__*__delete*"]

[rules]
# inline = """
# Keep operations deterministic.
# Run tests before marking work done.
# """
"#,
            name = id,
            id = id,
        )
    }
}

/// Apply a profile's permission overrides on top of a base Permissions struct.
///
/// `agents_dir` is optional — when provided, named preset sections from
/// `agents/permissions.toml` (e.g. `[ship-standard]`) are consulted.
/// When absent, built-in fallback behaviour applies.
pub fn apply_profile_permissions(
    base: compiler::Permissions,
    profile: &Profile,
    agents_dir: Option<&Path>,
) -> compiler::Permissions {
    use compiler::{Permissions, ToolPermissions};

    let mp = &profile.permissions;

    // Resolve the named preset. Try permissions.toml first, then built-in fallbacks.
    let preset_from_file = mp.preset.as_deref().and_then(|name| {
        agents_dir.and_then(|dir| load_permission_preset(dir, name))
    });

    let mut tools = if let Some(ref preset) = preset_from_file {
        // Use data from permissions.toml section
        ToolPermissions {
            allow: if preset.tools_allow.is_empty() {
                base.tools.allow.clone()
            } else {
                preset.tools_allow.clone()
            },
            deny: {
                let mut d = base.tools.deny.clone();
                for p in &preset.tools_deny {
                    if !d.contains(p) { d.push(p.clone()); }
                }
                d
            },
            ask: {
                let mut a = base.tools.ask.clone();
                for p in &preset.tools_ask {
                    if !a.contains(p) { a.push(p.clone()); }
                }
                a
            },
        }
    } else {
        // Built-in fallback — no permissions.toml or section not found
        // Start from preset in permissions.toml (resolved via file), fall back to built-in
        match mp.preset.as_deref() {
            Some("ship-readonly") => ToolPermissions {
                allow: vec!["Read".into(), "Glob".into(), "LS".into(), "mcp__ship__*".into(), "Bash(ship *)".into()],
                deny: vec!["Write(*)".into(), "Edit(*)".into(), "Bash(rm*)".into()],
                ask: vec![],
            },
            _ => base.tools.clone(),
        }
    };

    // Profile-level additions always apply on top of preset
    for p in &mp.tools_deny {
        if !tools.deny.contains(p) {
            tools.deny.push(p.clone());
        }
    }
    for p in &mp.tools_ask {
        if !tools.ask.contains(p) {
            tools.ask.push(p.clone());
        }
    }

    // default_mode: profile field wins, then preset file value, then base
    let default_mode = mp.default_mode.clone()
        .or_else(|| preset_from_file.as_ref().and_then(|p| p.default_mode.clone()))
        .or(base.default_mode);

    Permissions {
        tools,
        default_mode,
        ..base
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_scaffold_parses() {
        let s = Profile::scaffold("rust-expert");
        let profile: Profile = toml::from_str(&s).expect("scaffold must be valid TOML");
        assert_eq!(profile.meta.id, "rust-expert");
        assert_eq!(profile.meta.providers, vec!["claude"]);
    }

    #[test]
    fn profile_key_parses() {
        let toml_str = r#"
[profile]
name = "CLI Lane"
id = "cli-lane"
providers = ["claude"]

[plugins]
install = ["superpowers@claude-plugins-official"]
scope = "project"

[permissions]
preset = "ship-autonomous"
tools_deny = ["Bash(rm -rf *)"]
"#;
        let p: Profile = toml::from_str(toml_str).unwrap();
        assert_eq!(p.meta.id, "cli-lane");
        assert_eq!(p.plugins.install, vec!["superpowers@claude-plugins-official"]);
        assert_eq!(p.plugins.scope, "project");
        assert_eq!(p.permissions.preset.as_deref(), Some("ship-autonomous"));
    }

    #[test]
    fn apply_profile_permissions_readonly_restricts() {
        use compiler::Permissions;
        let base = Permissions::default();
        let toml_str = r#"
[profile]
name = "Reviewer"
id = "reviewer"
providers = ["claude"]
[permissions]
preset = "ship-readonly"
"#;
        let p: Profile = toml::from_str(toml_str).unwrap();
        let result = apply_profile_permissions(base, &p, None);
        assert!(result.tools.deny.contains(&"Write(*)".to_string()));
        assert!(result.tools.deny.contains(&"Edit(*)".to_string()));
        assert!(result.tools.allow.contains(&"Read".to_string()));
    }

    #[test]
    fn apply_profile_permissions_readonly_restricts_allow() {
        use compiler::Permissions;
        let base = Permissions::default();
        let toml_str = r#"
[profile]
name = "ReadOnly"
id = "readonly"
providers = ["claude"]
[permissions]
preset = "ship-readonly"
"#;
        let p: Profile = toml::from_str(toml_str).unwrap();
        let result = apply_profile_permissions(base, &p, None);
        assert!(result.tools.allow.contains(&"Read".to_string()));
        // ship-readonly has a narrow allow list — Grep is not included
        assert!(!result.tools.allow.contains(&"Grep".to_string()));
    }

    #[test]
    fn apply_profile_permissions_uses_permissions_toml_preset() {
        use compiler::Permissions;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        // Write a permissions.toml with a named preset section
        std::fs::write(tmp.path().join("permissions.toml"), r#"
[custom-preset]
default_mode = "bypassPermissions"
tools_deny = ["Bash(git push --force*)"]
tools_ask = ["Bash(rm -rf*)"]
"#).unwrap();
        let toml_str = r#"
[profile]
name = "Custom"
id = "custom"
providers = ["claude"]
[permissions]
preset = "custom-preset"
"#;
        let p: Profile = toml::from_str(toml_str).unwrap();
        let result = apply_profile_permissions(Permissions::default(), &p, Some(tmp.path()));
        assert_eq!(result.default_mode.as_deref(), Some("bypassPermissions"));
        assert!(result.tools.deny.contains(&"Bash(git push --force*)".to_string()));
        assert!(result.tools.ask.contains(&"Bash(rm -rf*)".to_string()));
    }
}
