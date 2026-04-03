//! Shared types for the platform database layer.

pub type WorkspaceDbRow = (
    String,         // 0: id
    String,         // 1: status
    bool,           // 2: is_worktree
    Option<String>, // 3: worktree_path
    Option<String>, // 4: active_agent
    Option<String>, // 5: last_activated_at
);

pub type WorkspaceDbListRow = (
    String,         // 0: branch
    String,         // 1: id
    String,         // 2: status
    bool,           // 3: is_worktree
    Option<String>, // 4: worktree_path
    Option<String>, // 5: active_agent
    Option<String>, // 6: last_activated_at
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSessionDb {
    pub id: String,
    pub workspace_id: String,
    pub workspace_branch: String,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub agent_id: Option<String>,
    pub primary_provider: Option<String>,
    pub goal: Option<String>,
    pub summary: Option<String>,
    pub updated_workspace_ids: Vec<String>,
    pub compiled_at: Option<String>,
    pub compile_error: Option<String>,
    pub config_generation_at_start: Option<i64>,
    pub tool_call_count: i64,
    pub drained_at: Option<String>,
    pub mcp_provider: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSessionRecordDb {
    pub id: String,
    pub session_id: String,
    pub workspace_id: String,
    pub workspace_branch: String,
    pub summary: Option<String>,
    pub updated_workspace_ids: Vec<String>,
    pub duration_secs: Option<i64>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub agent_id: Option<String>,
    pub files_changed: Option<i64>,
    pub gate_result: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentArtifactRegistryDb {
    pub uuid: String,
    pub kind: String,
    pub external_id: String,
    pub name: String,
    pub source_path: String,
    pub content_hash: String,
}

