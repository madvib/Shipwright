use serde::{Deserialize, Serialize};
use std::path::Path;

/// Preset TOML format — what users author in .ship/agents/presets/<id>.toml
/// Also accepts the legacy [mode] key for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    /// New key: [preset]. If absent, falls back to [mode].
    #[serde(rename = "preset", alias = "mode")]
    pub meta: PresetMeta,
    #[serde(default)]
    pub skills: SkillsConfig,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub plugins: PluginsConfig,
    #[serde(default)]
    pub permissions: PresetPermissions,
    #[serde(default)]
    pub rules: RulesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetMeta {
    pub name: String,
    pub id: String,
    #[serde(default = "default_version")]
    pub version: String,
    pub description: Option<String>,
    /// Provider targets for this preset (overrides project ship.toml providers).
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

/// Permission overrides in a preset — merged on top of agents/permissions.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetPermissions {
    /// Preset shorthand: "ship-standard" | "ship-guarded" | "read-only" | "full-access"
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

fn default_version() -> String { "0.1.0".to_string() }
fn default_plugin_scope() -> String { "project".to_string() }

impl Preset {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("invalid preset TOML at {}: {}", path.display(), e))
    }

    /// Template for a new preset file.
    pub fn scaffold(id: &str) -> String {
        format!(
r#"[preset]
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
# preset = "ship-guarded"   # ship-standard | ship-guarded | read-only | full-access
# tools_deny = ["mcp__*__delete*"]
# default_mode = "plan"     # default | acceptEdits | plan | bypassPermissions

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

/// Apply a preset's permission overrides on top of a base Permissions struct.
pub fn apply_preset_permissions(
    base: compiler::Permissions,
    preset: &Preset,
) -> compiler::Permissions {
    use compiler::{Permissions, ToolPermissions};

    let mp = &preset.permissions;

    let mut tools = match mp.preset.as_deref() {
        Some("read-only") => ToolPermissions {
            allow: vec!["Read".into(), "Glob".into(), "LS".into()],
            deny: vec![],
            ask: vec![],
        },
        Some("ship-guarded") => ToolPermissions {
            allow: base.tools.allow.clone(),
            deny: {
                let mut d = base.tools.deny.clone();
                d.extend(["mcp__*__delete*".into(), "mcp__*__drop*".into()]);
                d
            },
            ask: base.tools.ask.clone(),
        },
        Some("full-access") => ToolPermissions {
            allow: vec!["*".into()],
            deny: vec![],
            ask: vec![],
        },
        _ => base.tools.clone(),
    };

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

    Permissions {
        tools,
        default_mode: mp.default_mode.clone().or(base.default_mode),
        ..base
    }
}

/// Backward compat alias — callers using the old `Mode` name still compile.
pub type Mode = Preset;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_scaffold_parses() {
        let s = Preset::scaffold("rust-expert");
        let preset: Preset = toml::from_str(&s).expect("scaffold must be valid TOML");
        assert_eq!(preset.meta.id, "rust-expert");
        assert_eq!(preset.meta.providers, vec!["claude"]);
    }

    #[test]
    fn preset_key_parses() {
        let toml_str = r#"
[preset]
name = "CLI Lane"
id = "cli-lane"
providers = ["claude"]

[plugins]
install = ["superpowers@claude-plugins-official"]
scope = "project"

[permissions]
preset = "ship-guarded"
tools_deny = ["Bash(rm -rf *)"]
"#;
        let p: Preset = toml::from_str(toml_str).unwrap();
        assert_eq!(p.meta.id, "cli-lane");
        assert_eq!(p.plugins.install, vec!["superpowers@claude-plugins-official"]);
        assert_eq!(p.plugins.scope, "project");
        assert_eq!(p.permissions.preset.as_deref(), Some("ship-guarded"));
    }

    #[test]
    fn mode_key_backward_compat() {
        let toml_str = r#"
[mode]
name = "Rust Expert"
id = "rust-expert"
providers = ["claude", "codex"]

[permissions]
preset = "ship-guarded"
tools_deny = ["mcp__*__delete*"]

[rules]
inline = "Keep things deterministic."
"#;
        let p: Preset = toml::from_str(toml_str).unwrap();
        assert_eq!(p.meta.providers, vec!["claude", "codex"]);
        assert_eq!(p.permissions.preset.as_deref(), Some("ship-guarded"));
        assert_eq!(p.rules.inline.as_deref(), Some("Keep things deterministic."));
    }

    #[test]
    fn apply_preset_permissions_guarded_adds_deny() {
        use compiler::Permissions;
        let base = Permissions::default();
        let toml_str = r#"
[preset]
name = "Guarded"
id = "guarded"
providers = ["claude"]
[permissions]
preset = "ship-guarded"
"#;
        let p: Preset = toml::from_str(toml_str).unwrap();
        let result = apply_preset_permissions(base, &p);
        assert!(result.tools.deny.contains(&"mcp__*__delete*".to_string()));
    }

    #[test]
    fn apply_preset_permissions_read_only_restricts_allow() {
        use compiler::Permissions;
        let base = Permissions::default();
        let toml_str = r#"
[preset]
name = "ReadOnly"
id = "read-only"
providers = ["claude"]
[permissions]
preset = "read-only"
"#;
        let p: Preset = toml::from_str(toml_str).unwrap();
        let result = apply_preset_permissions(base, &p);
        assert!(result.tools.allow.contains(&"Read".to_string()));
        assert!(!result.tools.allow.contains(&"*".to_string()));
    }
}
