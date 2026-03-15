use serde::{Deserialize, Serialize};
use std::path::Path;

/// Mode TOML format — what users author in .ship/modes/<id>.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mode {
    #[serde(rename = "mode")]
    pub meta: ModeMeta,
    #[serde(default)]
    pub skills: SkillsConfig,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub permissions: ModePermissions,
    #[serde(default)]
    pub rules: RulesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeMeta {
    pub name: String,
    pub id: String,
    #[serde(default = "default_version")]
    pub version: String,
    pub description: Option<String>,
    /// Provider targets for this mode (overrides project ship.toml providers).
    #[serde(default)]
    pub providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillsConfig {
    /// Skill IDs to activate (empty = all installed skills).
    #[serde(default)]
    pub refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    /// Server IDs to activate (empty = all configured servers).
    #[serde(default)]
    pub servers: Vec<String>,
}

/// Permission overrides in a mode — merged on top of agents/permissions.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModePermissions {
    /// Preset shorthand: "ship-standard", "ship-guarded", "read-only", "full-access"
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

impl Mode {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| anyhow::anyhow!("invalid mode TOML at {}: {}", path.display(), e))
    }

    /// Template for a new mode file.
    pub fn scaffold(id: &str) -> String {
        format!(
r#"[mode]
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

/// Apply a mode's permission overrides on top of a base Permissions struct.
pub fn apply_mode_permissions(
    base: compiler::Permissions,
    mode: &Mode,
) -> compiler::Permissions {
    use compiler::{Permissions, ToolPermissions};

    let mp = &mode.permissions;

    // Apply preset
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

    // Apply explicit deny overrides
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_scaffold_parses() {
        let s = Mode::scaffold("rust-expert");
        let mode: Mode = toml::from_str(&s).expect("scaffold must be valid TOML");
        assert_eq!(mode.meta.id, "rust-expert");
        assert_eq!(mode.meta.providers, vec!["claude"]);
    }

    #[test]
    fn mode_round_trips() {
        let toml_str = r#"
[mode]
name = "Rust Expert"
id = "rust-expert"
version = "1.0.0"
providers = ["claude", "codex"]

[permissions]
preset = "ship-guarded"
tools_deny = ["mcp__*__delete*"]

[rules]
inline = "Keep things deterministic."
"#;
        let mode: Mode = toml::from_str(toml_str).unwrap();
        assert_eq!(mode.meta.providers, vec!["claude", "codex"]);
        assert_eq!(mode.permissions.preset.as_deref(), Some("ship-guarded"));
        assert_eq!(mode.rules.inline.as_deref(), Some("Keep things deterministic."));
    }

    #[test]
    fn apply_mode_permissions_guarded_adds_deny() {
        use compiler::Permissions;
        let base = Permissions::default();
        let mode_toml = r#"
[mode]
name = "Guarded"
id = "guarded"
providers = ["claude"]
[permissions]
preset = "ship-guarded"
"#;
        let mode: Mode = toml::from_str(mode_toml).unwrap();
        let result = apply_mode_permissions(base, &mode);
        assert!(result.tools.deny.contains(&"mcp__*__delete*".to_string()));
    }

    #[test]
    fn apply_mode_permissions_read_only_restricts_allow() {
        use compiler::Permissions;
        let base = Permissions::default();
        let mode_toml = r#"
[mode]
name = "ReadOnly"
id = "read-only"
providers = ["claude"]
[permissions]
preset = "read-only"
"#;
        let mode: Mode = toml::from_str(mode_toml).unwrap();
        let result = apply_mode_permissions(base, &mode);
        assert!(result.tools.allow.contains(&"Read".to_string()));
        assert!(!result.tools.allow.contains(&"*".to_string()));
    }
}
