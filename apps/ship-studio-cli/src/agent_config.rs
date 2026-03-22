use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::loader::load_permission_preset;

/// Agent TOML format — what users author in .ship/agents/<id>.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(rename = "agent", alias = "profile")]
    pub meta: AgentMeta,
    #[serde(default)]
    pub skills: SkillsConfig,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub plugins: PluginsConfig,
    #[serde(default)]
    pub permissions: AgentPermissions,
    #[serde(default)]
    pub rules: RulesConfig,
    #[serde(default)]
    pub hooks: AgentHooks,
    /// Provider-specific settings merged verbatim into the provider's config file.
    /// `[provider_settings.claude]` → `.claude/settings.json`.
    /// Any key/value valid in that file works here — no code change required.
    #[serde(default)]
    pub provider_settings: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMeta {
    pub name: String,
    pub id: String,
    #[serde(default = "default_version")]
    pub version: String,
    pub description: Option<String>,
    /// Provider targets for this agent (overrides project ship.toml providers).
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

/// Permission overrides in an agent — merged on top of agents/permissions.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentPermissions {
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

/// Hook commands declared in an agent.
/// Each field corresponds to a Claude Code hook trigger name.
/// The compiler emits these into `.claude/settings.json` under `hooks`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentHooks {
    /// Command to run when the agent session ends (Claude Code Stop hook).
    pub stop: Option<String>,
    /// Command to run when a sub-agent session ends (Claude Code SubagentStop hook).
    pub subagent_stop: Option<String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}
fn default_plugin_scope() -> String {
    "project".to_string()
}

impl AgentConfig {
    /// Load an agent config from a file. Dispatches to JSONC or TOML based on extension.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        if crate::paths::is_jsonc_ext(path) {
            compiler::jsonc::from_jsonc_str(&content)
                .map_err(|e| anyhow::anyhow!("invalid agent JSONC at {}: {}", path.display(), e))
        } else {
            toml::from_str(&content)
                .map_err(|e| anyhow::anyhow!("invalid agent TOML at {}: {}", path.display(), e))
        }
    }

    /// Template for a new agent file in JSONC format.
    pub fn scaffold_jsonc(id: &str) -> String {
        format!(
            r#"{{
  // Agent configuration — https://getship.dev/docs/agents
  "agent": {{
    "name": "{id}",
    "id": "{id}",
    "version": "0.1.0",
    "description": "",
    "providers": ["claude"],
  }},
  "skills": {{
    // Skill IDs to activate (empty = all installed skills).
    "refs": [],
  }},
  "mcp": {{
    // MCP server IDs to activate (empty = all configured servers).
    "servers": [],
  }},
  "plugins": {{
    // "install": ["superpowers@claude-plugins-official"],
    // "scope": "project",
  }},
  "permissions": {{
    // "preset": "ship-autonomous",
    // "tools_deny": ["mcp__*__delete*"],
  }},
  "rules": {{
    // "inline": "Keep operations deterministic.\nRun tests before marking work done.",
  }},
}}"#,
            id = id,
        )
    }
}

/// Apply an agent's permission overrides on top of a base Permissions struct.
///
/// `agents_dir` is optional — when provided, named preset sections from
/// `agents/permissions.toml` (e.g. `[ship-standard]`) are consulted.
/// When absent, built-in fallback behaviour applies.
pub fn apply_agent_permissions(
    base: compiler::Permissions,
    agent: &AgentConfig,
    ship_dir: Option<&Path>,
) -> compiler::Permissions {
    use compiler::{Permissions, ToolPermissions};

    let mp = &agent.permissions;

    // Resolve the named preset from permissions.jsonc at .ship/ root.
    let preset_from_file = mp
        .preset
        .as_deref()
        .and_then(|name| ship_dir.and_then(|dir| load_permission_preset(dir, name)));

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
                    if !d.contains(p) {
                        d.push(p.clone());
                    }
                }
                d
            },
            ask: {
                let mut a = base.tools.ask.clone();
                for p in &preset.tools_ask {
                    if !a.contains(p) {
                        a.push(p.clone());
                    }
                }
                a
            },
        }
    } else {
        // Built-in fallback — no permissions.toml or section not found
        match mp.preset.as_deref() {
            Some("ship-readonly") => ToolPermissions {
                allow: vec![
                    "Read".into(),
                    "Glob".into(),
                    "LS".into(),
                    "mcp__ship__*".into(),
                    "Bash(ship *)".into(),
                ],
                deny: vec!["Write(*)".into(), "Edit(*)".into(), "Bash(rm*)".into()],
                ask: vec![],
            },
            _ => base.tools.clone(),
        }
    };

    // Agent-level additions always apply on top of preset
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

    // default_mode: agent field wins, then preset file value, then base
    let default_mode = mp
        .default_mode
        .clone()
        .or_else(|| {
            preset_from_file
                .as_ref()
                .and_then(|p| p.default_mode.clone())
        })
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
    fn agent_scaffold_jsonc_parses() {
        let s = AgentConfig::scaffold_jsonc("rust-expert");
        let agent: AgentConfig =
            compiler::jsonc::from_jsonc_str(&s).expect("scaffold must be valid JSONC");
        assert_eq!(agent.meta.id, "rust-expert");
        assert_eq!(agent.meta.providers, vec!["claude"]);
    }

    #[test]
    fn agent_key_parses() {
        let toml_str = r#"
[agent]
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
        let p: AgentConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(p.meta.id, "cli-lane");
        assert_eq!(
            p.plugins.install,
            vec!["superpowers@claude-plugins-official"]
        );
        assert_eq!(p.plugins.scope, "project");
        assert_eq!(p.permissions.preset.as_deref(), Some("ship-autonomous"));
    }

    #[test]
    fn legacy_profile_key_still_parses() {
        let toml_str = r#"
[profile]
name = "Legacy"
id = "legacy"
providers = ["claude"]
"#;
        let p: AgentConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(p.meta.id, "legacy");
    }

    #[test]
    fn apply_agent_permissions_readonly_restricts() {
        use compiler::Permissions;
        let base = Permissions::default();
        let toml_str = r#"
[agent]
name = "Reviewer"
id = "reviewer"
providers = ["claude"]
[permissions]
preset = "ship-readonly"
"#;
        let p: AgentConfig = toml::from_str(toml_str).unwrap();
        let result = apply_agent_permissions(base, &p, None);
        assert!(result.tools.deny.contains(&"Write(*)".to_string()));
        assert!(result.tools.deny.contains(&"Edit(*)".to_string()));
        assert!(result.tools.allow.contains(&"Read".to_string()));
    }

    #[test]
    fn apply_agent_permissions_readonly_restricts_allow() {
        use compiler::Permissions;
        let base = Permissions::default();
        let toml_str = r#"
[agent]
name = "ReadOnly"
id = "readonly"
providers = ["claude"]
[permissions]
preset = "ship-readonly"
"#;
        let p: AgentConfig = toml::from_str(toml_str).unwrap();
        let result = apply_agent_permissions(base, &p, None);
        assert!(result.tools.allow.contains(&"Read".to_string()));
        // ship-readonly has a narrow allow list — Grep is not included
        assert!(!result.tools.allow.contains(&"Grep".to_string()));
    }

    #[test]
    fn apply_agent_permissions_uses_permissions_toml_preset() {
        use compiler::Permissions;
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        // Write a permissions.toml with a named preset section
        std::fs::write(
            tmp.path().join("permissions.toml"),
            r#"
[custom-preset]
default_mode = "bypassPermissions"
tools_deny = ["Bash(git push --force*)"]
tools_ask = ["Bash(rm -rf*)"]
"#,
        )
        .unwrap();
        let toml_str = r#"
[agent]
name = "Custom"
id = "custom"
providers = ["claude"]
[permissions]
preset = "custom-preset"
"#;
        let p: AgentConfig = toml::from_str(toml_str).unwrap();
        let result = apply_agent_permissions(Permissions::default(), &p, Some(tmp.path()));
        assert_eq!(result.default_mode.as_deref(), Some("bypassPermissions"));
        assert!(
            result
                .tools
                .deny
                .contains(&"Bash(git push --force*)".to_string())
        );
        assert!(result.tools.ask.contains(&"Bash(rm -rf*)".to_string()));
    }
}
