use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct ListWorkspacesRequest {
    /// Optional status filter (e.g. "active", "idle", "archived")
    pub status: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateWorkspaceRequest {
    /// Human-readable name for the workspace
    pub name: String,
    /// Optional preset ID to activate in this workspace
    pub preset_id: Option<String>,
    /// Branch name. If omitted, derived from name (slugified).
    pub branch: Option<String>,
    /// Base branch to create worktree from. Defaults to "main".
    pub base_branch: Option<String>,
    /// File scope -- paths this workspace should edit (e.g. "crates/")
    pub file_scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ActivateWorkspaceRequest {
    /// Workspace branch/id to activate.
    pub branch: String,
    /// Optional workspace agent override to apply after activation.
    pub agent_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CompleteWorkspaceRequest {
    /// Workspace id (branch name) to complete
    pub workspace_id: String,
    /// Summary of what was accomplished — written to handoff.md
    pub summary: String,
    /// Whether to prune the worktree on completion. Defaults true for imperative, false for declarative/service.
    pub prune_worktree: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListStaleWorktreesRequest {
    /// Idle threshold in hours. Worktrees not modified within this window are returned. Defaults to 24.
    pub idle_hours: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
pub struct StartSessionRequest {
    /// Workspace branch/id. If omitted, resolves from current git branch.
    pub branch: Option<String>,
    /// Optional goal for this session.
    pub goal: Option<String>,
    /// Optional agent override for this session/workspace.
    pub agent_id: Option<String>,
    /// Optional primary provider for this session.
    pub provider_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct EndSessionRequest {
    /// Workspace branch/id. If omitted, resolves from current git branch.
    pub branch: Option<String>,
    /// End-of-session summary — what was accomplished, what changed.
    pub summary: Option<String>,
    /// Workspace IDs updated during this session.
    pub updated_workspace_ids: Option<Vec<String>>,
    /// Model ID used during the session (e.g. "claude-opus-4-20250514").
    pub model: Option<String>,
    /// Count of files modified during the session.
    pub files_changed: Option<i64>,
    /// Gate result for gated sessions: "pass", "fail", or null.
    pub gate_result: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct LogProgressRequest {
    /// Progress note — what you did, decided, or got blocked on.
    pub note: String,
    /// Workspace branch/id. If omitted, resolves from current git branch.
    pub branch: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetSessionRequest {
    /// Workspace branch/id. If omitted, resolves from current git branch.
    pub branch: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListSessionsRequest {
    /// Filter by branch. If omitted, returns sessions across all branches.
    pub branch: Option<String>,
    /// Maximum number of sessions to return. Default 20, max 100.
    pub limit: Option<u32>,
}

