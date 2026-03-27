use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use super::types::{ShipWorkspaceKind, WorkspaceStatus};

// ---- Session types ---------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceSessionStatus {
    #[default]
    Active,
    Ended,
}

impl std::fmt::Display for WorkspaceSessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceSessionStatus::Active => write!(f, "active"),
            WorkspaceSessionStatus::Ended => write!(f, "ended"),
        }
    }
}

impl std::str::FromStr for WorkspaceSessionStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "active" => Ok(WorkspaceSessionStatus::Active),
            "ended" => Ok(WorkspaceSessionStatus::Ended),
            _ => Err(anyhow::anyhow!(
                "Invalid workspace session status: {}",
                value
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WorkspaceSession {
    pub id: String,
    pub workspace_id: String,
    pub workspace_branch: String,
    pub status: WorkspaceSessionStatus,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default)]
    pub updated_workspace_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_record_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiled_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compile_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_generation_at_start: Option<i64>,
    #[serde(default)]
    pub stale_context: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---- Request / report types ------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WorkspaceSessionRecord {
    pub id: String,
    pub session_id: String,
    pub workspace_id: String,
    pub workspace_branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default)]
    pub updated_workspace_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_changed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_result: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct EndWorkspaceSessionRequest {
    pub summary: Option<String>,
    pub updated_workspace_ids: Vec<String>,
    pub model: Option<String>,
    pub files_changed: Option<i64>,
    pub gate_result: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WorkspaceProviderMatrix {
    pub workspace_branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub source: String,
    pub allowed_providers: Vec<String>,
    pub supported_providers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution_error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WorkspaceRepairReport {
    pub workspace_branch: String,
    pub dry_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub status: WorkspaceStatus,
    pub providers_expected: Vec<String>,
    pub missing_provider_configs: Vec<String>,
    pub had_compile_error: bool,
    pub needs_recompile: bool,
    pub reapplied_compile: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution_error: Option<String>,
    pub actions: Vec<String>,
}

/// Input for creating or updating a workspace runtime record.
#[derive(Debug, Clone, Default)]
pub struct CreateWorkspaceRequest {
    pub branch: String,
    pub workspace_type: Option<ShipWorkspaceKind>,
    pub status: Option<WorkspaceStatus>,
    pub active_agent: Option<String>,
    pub providers: Option<Vec<String>>,
    pub mcp_servers: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
    pub is_worktree: Option<bool>,
    pub worktree_path: Option<String>,
    pub context_hash: Option<String>,
}
