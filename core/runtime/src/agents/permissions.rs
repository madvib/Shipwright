use crate::fs_util::write_atomic;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use specta::Type;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cost_per_session: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
    #[serde(default = "default_require_confirmation")]
    pub require_confirmation: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
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
        "Bash".to_string(),
        "Write".to_string(),
        "Edit".to_string(),
        "MultiEdit".to_string(),
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
        "Bash".to_string(),
        "Write".to_string(),
        "Edit".to_string(),
        "MultiEdit".to_string(),
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
            max_cost_per_session: Some(5.0),
            max_turns: Some(50),
            require_confirmation: default_require_confirmation(),
        }
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            tools: ToolPermissions::default(),
            filesystem: FsPermissions::default(),
            commands: CommandPermissions::default(),
            network: NetworkPermissions::default(),
            agent: AgentLimits::default(),
        }
    }
}

// ─── Paths ────────────────────────────────────────────────────────────────────

fn permissions_path(ship_dir: &std::path::Path) -> PathBuf {
    ship_dir.join("agents").join("permissions.toml")
}

// ─── CRUD ─────────────────────────────────────────────────────────────────────

pub fn get_permissions(ship_dir: PathBuf) -> Result<Permissions> {
    let path = permissions_path(&ship_dir);
    if !path.exists() {
        return Ok(Permissions::default());
    }
    let content = fs::read_to_string(&path)?;
    Ok(toml::from_str(&content)?)
}

pub fn save_permissions(ship_dir: PathBuf, permissions: &Permissions) -> Result<()> {
    let path = permissions_path(&ship_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    write_atomic(&path, toml::to_string(permissions)?)
}
