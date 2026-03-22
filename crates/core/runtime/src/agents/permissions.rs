use crate::fs_util::write_atomic;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

// ─── Data types ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ToolPermissions {
    #[serde(default = "default_tool_allow")]
    pub allow: Vec<String>,
    #[serde(default = "default_tool_deny")]
    pub deny: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FsPermissions {
    #[serde(default = "default_filesystem_allow")]
    pub allow: Vec<String>,
    #[serde(default = "default_filesystem_deny")]
    pub deny: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct CommandPermissions {
    #[serde(default = "default_command_allow")]
    pub allow: Vec<String>,
    #[serde(default = "default_command_deny")]
    pub deny: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, Type)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkPolicy {
    #[default]
    None,
    Localhost,
    AllowList,
    Unrestricted,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct NetworkPermissions {
    #[serde(default)]
    pub policy: NetworkPolicy,
    #[serde(default)]
    pub allow_hosts: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AgentLimits {
    #[serde(default = "default_require_confirmation")]
    pub require_confirmation: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
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
}

fn default_tool_allow() -> Vec<String> {
    vec![
        "Read".to_string(),
        "Glob".to_string(),
        "Grep".to_string(),
        "mcp__ship__*".to_string(),
        "mcp__*__read*".to_string(),
        "mcp__*__list*".to_string(),
        "mcp__*__search*".to_string(),
    ]
}

fn default_tool_deny() -> Vec<String> {
    vec![
        "mcp__*__write*".to_string(),
        "mcp__*__delete*".to_string(),
        "mcp__*__exec*".to_string(),
    ]
}

fn default_filesystem_allow() -> Vec<String> {
    vec!["**/*".to_string()]
}

fn default_filesystem_deny() -> Vec<String> {
    vec![
        "/etc/**".to_string(),
        "/sys/**".to_string(),
        "/proc/**".to_string(),
        "~/.ssh/**".to_string(),
        "~/.gnupg/**".to_string(),
    ]
}

fn default_command_allow() -> Vec<String> {
    vec![
        "git status".to_string(),
        "git diff".to_string(),
        "git log".to_string(),
        "ls".to_string(),
        "cat".to_string(),
        "rg".to_string(),
        "find".to_string(),
        "pwd".to_string(),
    ]
}

fn default_command_deny() -> Vec<String> {
    vec![
        "rm -rf".to_string(),
        "git push --force".to_string(),
        "npm publish".to_string(),
        "cargo publish".to_string(),
    ]
}

fn default_require_confirmation() -> Vec<String> {
    vec![
        "mcp__*__write*".to_string(),
        "mcp__*__delete*".to_string(),
        "mcp__*__exec*".to_string(),
    ]
}

impl Default for ToolPermissions {
    fn default() -> Self {
        Self {
            allow: default_tool_allow(),
            deny: default_tool_deny(),
        }
    }
}

impl Default for FsPermissions {
    fn default() -> Self {
        Self {
            allow: default_filesystem_allow(),
            deny: default_filesystem_deny(),
        }
    }
}

impl Default for CommandPermissions {
    fn default() -> Self {
        Self {
            allow: default_command_allow(),
            deny: default_command_deny(),
        }
    }
}

impl Default for NetworkPermissions {
    fn default() -> Self {
        Self {
            policy: NetworkPolicy::None,
            allow_hosts: Vec::new(),
        }
    }
}

impl Default for AgentLimits {
    fn default() -> Self {
        Self {
            require_confirmation: default_require_confirmation(),
        }
    }
}


// ─── Paths ────────────────────────────────────────────────────────────────────

fn permissions_path(ship_dir: &std::path::Path) -> PathBuf {
    crate::project::permissions_config_path(ship_dir)
}

// ─── CRUD ─────────────────────────────────────────────────────────────────────

pub fn get_permissions(ship_dir: PathBuf) -> Result<Permissions> {
    let path = permissions_path(&ship_dir);
    if !path.exists() {
        return Ok(Permissions::default());
    }
    let content = fs::read_to_string(&path)?;
    // If the file uses named-section format (e.g. [ship-standard], [ship-autonomous]),
    // the flat Permissions struct cannot deserialize it — all fields default to empty.
    // Detect this case: if the TOML has top-level table keys that are not recognized
    // flat-format section names, treat the file as a named-preset reference and
    // return defaults rather than returning silently wrong data.
    if is_named_preset_format(&content) {
        return Ok(Permissions::default());
    }
    Ok(toml::from_str(&content)?)
}

/// Returns true when the TOML content appears to be a named-preset file
/// (top-level keys like `[ship-standard]`) rather than the flat permissions format
/// (top-level keys like `[tools]`, `[commands]`, `[filesystem]`, `[network]`, `[agent]`).
fn is_named_preset_format(content: &str) -> bool {
    let flat_keys = ["tools", "commands", "filesystem", "network", "agent", "default_mode", "additional_directories"];
    let Ok(value) = toml::from_str::<toml::Value>(content) else {
        return false;
    };
    let Some(table) = value.as_table() else {
        return false;
    };
    if table.is_empty() {
        return false;
    }
    // If none of the top-level keys are recognized flat-format keys,
    // the file is in named-preset format.
    !table.keys().any(|k| flat_keys.contains(&k.as_str()))
}

pub fn save_permissions(ship_dir: PathBuf, permissions: &Permissions) -> Result<()> {
    let path = permissions_path(&ship_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Do not overwrite a human-authored named-preset file with a flat permissions
    // struct. The named-preset format ([ship-standard], [ship-autonomous], etc.) is
    // a user-maintained reference that the compile path reads via profile presets.
    // Overwriting it would destroy the named sections on every `ship use`.
    if path.exists()
        && let Ok(existing) = fs::read_to_string(&path)
        && is_named_preset_format(&existing)
    {
        return Ok(());
    }
    write_atomic(&path, toml::to_string(permissions)?)
}

fn canonical_tool_ids() -> Vec<String> {
    let mut ids = BTreeSet::new();
    for value in default_tool_allow()
        .into_iter()
        .chain(default_tool_deny())
        .chain(default_require_confirmation())
    {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed == "*" || trimmed.starts_with("mcp__") {
            continue;
        }
        ids.insert(trimmed.to_string());
    }
    ids.into_iter().collect()
}

pub fn permission_tool_ids_for_provider(provider_id: &str) -> Vec<String> {
    let mut ids: BTreeSet<String> = canonical_tool_ids().into_iter().collect();
    if provider_id == "gemini" {
        ids.insert("run_shell_command".to_string());
    }
    ids.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn provider_tool_vocab_excludes_wildcards_and_mcp_patterns() {
        let ids = permission_tool_ids_for_provider("claude");
        assert!(!ids.iter().any(|id| id == "*"));
        assert!(!ids.iter().any(|id| id.starts_with("mcp__")));
    }

    #[test]
    fn gemini_vocab_includes_shell_tool_id() {
        let ids = permission_tool_ids_for_provider("gemini");
        assert!(ids.iter().any(|id| id == "run_shell_command"));
    }

    #[test]
    fn default_tool_deny_excludes_core_editing_tools() {
        let deny = default_tool_deny();
        // Bash, Write, Edit, MultiEdit must NOT be in the default deny list —
        // they are the core tools agents need to do their job.
        assert!(!deny.contains(&"Bash".to_string()), "Bash must not be in default deny");
        assert!(!deny.contains(&"Write".to_string()), "Write must not be in default deny");
        assert!(!deny.contains(&"Edit".to_string()), "Edit must not be in default deny");
        assert!(!deny.contains(&"MultiEdit".to_string()), "MultiEdit must not be in default deny");
    }

    #[test]
    fn get_permissions_returns_defaults_for_named_preset_format() {
        // A permissions.toml that uses named sections ([ship-standard], [ship-autonomous])
        // must not be silently misread as a flat Permissions struct. get_permissions()
        // should return Permissions::default() rather than data from the named sections.
        let tmp = TempDir::new().unwrap();
        let ship_dir = tmp.path().join(".ship");
        let agents_dir = ship_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();
        let named_section_content = r#"
[ship-standard]
default_mode = "acceptEdits"
tools_ask = ["Bash(rm -rf*)"]
tools_deny = ["Bash(git push --force*)"]

[ship-autonomous]
default_mode = "dontAsk"
tools_ask = ["Bash(*deploy*)"]
"#;
        std::fs::write(agents_dir.join("permissions.toml"), named_section_content).unwrap();
        let perms = get_permissions(ship_dir).unwrap();
        let default_perms = Permissions::default();
        // Named-preset format must return exactly the same as no file at all.
        assert_eq!(
            perms.tools.deny, default_perms.tools.deny,
            "named-preset file must return default deny rules, not rules from named sections"
        );
        assert_eq!(
            perms.tools.allow, default_perms.tools.allow,
            "named-preset file must return default allow rules"
        );
    }

    #[test]
    fn save_permissions_does_not_overwrite_named_preset_file() {
        // Saving a flat Permissions struct must not destroy a human-authored
        // named-preset permissions.toml file.
        let tmp = TempDir::new().unwrap();
        let ship_dir = tmp.path().join(".ship");
        let agents_dir = ship_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();
        let original = r#"
[ship-standard]
default_mode = "acceptEdits"
tools_deny = ["Bash(git push --force*)"]
"#;
        std::fs::write(agents_dir.join("permissions.toml"), original).unwrap();

        // Attempt to overwrite with a flat struct
        save_permissions(ship_dir, &Permissions::default()).unwrap();

        let after = std::fs::read_to_string(agents_dir.join("permissions.toml")).unwrap();
        assert_eq!(after, original, "named-preset file must not be overwritten by save_permissions");
    }

    #[test]
    fn save_permissions_writes_flat_format_when_no_existing_file() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(ship_dir.join("agents")).unwrap();

        let perms = Permissions {
            tools: ToolPermissions {
                deny: vec!["mcp__*__delete*".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        save_permissions(ship_dir.clone(), &perms).unwrap();

        let restored = get_permissions(ship_dir).unwrap();
        assert!(restored.tools.deny.contains(&"mcp__*__delete*".to_string()));
    }
}
