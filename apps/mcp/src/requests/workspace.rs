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
    /// Workspace kind: "imperative" | "declarative" | "service"
    pub kind: String,
    /// Optional preset ID to activate in this workspace
    pub preset_id: Option<String>,
    /// Branch name. If omitted, derived from name (slugified).
    pub branch: Option<String>,
    /// Base branch to create worktree from. Defaults to "main".
    pub base_branch: Option<String>,
    /// File scope — paths this workspace should edit (e.g. "crates/")
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
pub struct CreateJobRequest {
    /// Job kind (e.g. "review", "build", "deploy")
    pub kind: String,
    /// Human-readable description of the work
    pub description: String,
    /// Git branch this job is associated with
    pub branch: Option<String>,
    /// Workspace id/branch that requested this job
    pub requesting_workspace: Option<String>,
    /// Agent id or workspace this job is assigned to
    pub assigned_to: Option<String>,
    /// Scheduling priority — higher numbers run first (default 0)
    pub priority: Option<i32>,
    /// Job id that must complete before this one can start
    pub blocked_by: Option<String>,
    /// File paths this job intends to touch (informational; use claim_file for ownership)
    pub touched_files: Option<Vec<String>>,
    /// Capability id this job is delivering (e.g. "csycGZPJ")
    pub capability_id: Option<String>,
    /// File or directory paths the agent may touch (scope declaration, informational)
    pub scope: Option<Vec<String>>,
    /// File paths / prefixes the agent is allowed to touch — enforced by `ship gate`
    pub file_scope: Option<Vec<String>>,
    /// Acceptance criteria checklist items
    pub acceptance_criteria: Option<Vec<String>>,
    /// Profile name to compile / activate in the worktree
    pub preset_hint: Option<String>,
    /// Human-readable worktree label (used as symlink name)
    pub symlink_name: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateJobRequest {
    /// Job id to update
    pub id: String,
    /// New status: "pending" | "running" | "complete" | "failed"
    pub status: Option<String>,
    /// Reassign to a different agent or workspace
    pub assigned_to: Option<String>,
    /// Update scheduling priority
    pub priority: Option<i32>,
    /// Set or clear the blocking job id
    pub blocked_by: Option<String>,
    /// Replace the touched_files list
    pub touched_files: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ClaimFileRequest {
    /// Job id claiming ownership of the file
    pub job_id: String,
    /// File path to claim (relative to project root)
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetFileOwnerRequest {
    /// File path to look up (relative to project root)
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListJobsRequest {
    /// Filter by branch
    pub branch: Option<String>,
    /// Filter by status: "pending" | "running" | "complete" | "failed"
    pub status: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AppendJobLogRequest {
    /// Job id to append a log entry to
    pub job_id: String,
    /// Log message
    pub message: String,
    /// Log level: "info" | "warn" | "error" (informational only, stored in message prefix)
    pub level: Option<String>,
}
