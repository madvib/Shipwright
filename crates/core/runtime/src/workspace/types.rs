use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

// ---- Workspace kind --------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ShipWorkspaceKind {
    #[default]
    Feature,
    Patch,
    Service,
}

impl std::fmt::Display for ShipWorkspaceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShipWorkspaceKind::Feature => write!(f, "feature"),
            ShipWorkspaceKind::Patch => write!(f, "patch"),
            ShipWorkspaceKind::Service => write!(f, "service"),
        }
    }
}

impl std::str::FromStr for ShipWorkspaceKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        parse_workspace_kind(value)
            .ok_or_else(|| anyhow::anyhow!("Invalid workspace type: {}", value))
    }
}

pub(crate) fn parse_workspace_kind(value: &str) -> Option<ShipWorkspaceKind> {
    match value.trim().to_lowercase().as_str() {
        "feature" => Some(ShipWorkspaceKind::Feature),
        "patch" => Some(ShipWorkspaceKind::Patch),
        "service" => Some(ShipWorkspaceKind::Service),
        _ => None,
    }
}

pub(crate) fn parse_workspace_type_required(value: &str) -> Result<ShipWorkspaceKind> {
    parse_workspace_kind(value).ok_or_else(|| {
        anyhow!(
            "Invalid workspace type '{}'; expected one of: feature, patch, service",
            value
        )
    })
}

// ---- Workspace status ------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceStatus {
    #[default]
    Active,
    Archived,
}

impl std::fmt::Display for WorkspaceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceStatus::Active => write!(f, "active"),
            WorkspaceStatus::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for WorkspaceStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "active" => Ok(WorkspaceStatus::Active),
            "archived" => Ok(WorkspaceStatus::Archived),
            _ => Err(anyhow::anyhow!("Invalid workspace status: {}", value)),
        }
    }
}

pub(crate) fn parse_workspace_status_required(value: &str) -> Result<WorkspaceStatus> {
    match value.trim().to_lowercase().as_str() {
        "active" => Ok(WorkspaceStatus::Active),
        "archived" => Ok(WorkspaceStatus::Archived),
        _ => Err(anyhow!(
            "Invalid workspace status '{}'; expected one of: active, archived",
            value
        )),
    }
}

// ---- Workspace ------------------------------------------------------------

/// Workspace runtime state -- SQLite only, no frontmatter file.
/// `branch` is the workspace key and can represent either a git branch or a
/// non-git runtime workspace identifier.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Workspace {
    pub id: String,
    pub branch: String,
    #[serde(default)]
    pub workspace_type: ShipWorkspaceKind,
    #[serde(default)]
    pub status: WorkspaceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_agent: Option<String>,
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activated_at: Option<DateTime<Utc>>,
    pub is_worktree: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_hash: Option<String>,
    #[serde(default)]
    pub config_generation: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiled_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compile_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tmux_session_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Environment {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    pub permissions_json: String,
    #[serde(default)]
    pub providers: Vec<String>,
    pub hooks_json: String,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---- Process ---------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessStatus {
    #[default]
    Running,
    Paused,
    Completed,
    Error,
    Interrupted,
}

impl std::fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessStatus::Running => write!(f, "running"),
            ProcessStatus::Paused => write!(f, "paused"),
            ProcessStatus::Completed => write!(f, "completed"),
            ProcessStatus::Error => write!(f, "error"),
            ProcessStatus::Interrupted => write!(f, "interrupted"),
        }
    }
}

impl std::str::FromStr for ProcessStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "running" => Ok(ProcessStatus::Running),
            "paused" => Ok(ProcessStatus::Paused),
            "completed" => Ok(ProcessStatus::Completed),
            "error" => Ok(ProcessStatus::Error),
            "interrupted" => Ok(ProcessStatus::Interrupted),
            _ => Err(anyhow::anyhow!("Invalid process status: {}", value)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Process {
    pub id: String,
    pub workspace_id: String,
    pub status: ProcessStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
