#![allow(dead_code)]

use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

// ─── Request Types ────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct LogDecisionRequest {
    /// Title of the architecture decision
    pub title: String,
    /// The decision content / reasoning — what was decided and why
    pub decision: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct OpenProjectRequest {
    /// The absolute path to the project root
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TrackProjectRequest {
    /// The name of the project
    pub name: String,
    /// The absolute path to the project root
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateNoteRequest {
    /// Title of the note
    pub title: String,
    /// Optional markdown content
    pub content: Option<String>,
    /// Optional git branch to associate with this note
    pub branch: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListNotesRequest {
    /// Scope: project (default) or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetNoteRequest {
    /// Note filename (e.g. "session-summary.md")
    pub file_name: String,
    /// Scope: project (default) or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateNoteRequest {
    /// Note filename (e.g. "session-summary.md")
    pub file_name: String,
    /// Full replacement markdown content
    pub content: String,
    /// Scope: project (default) or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListSkillsRequest {
    /// Optional search filter (substring match on skill id/name/description)
    pub query: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListWorkspacesRequest {
    /// Optional status filter (e.g. "active", "idle", "archived")
    pub status: Option<String>,
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

#[derive(Deserialize, JsonSchema)]
pub struct GetSkillRequest {
    /// Skill id (without .md)
    pub id: String,
    /// Scope: effective (default), project, or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct StatusNameRequest {
    /// Status name (e.g. "review", "testing")
    pub name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateSpecRequest {
    /// Title of the spec
    pub title: String,
    /// Initial markdown content (optional — defaults to a blank template)
    pub content: Option<String>,
    /// Workspace branch/id. If omitted, uses active workspace.
    pub workspace: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetSpecRequest {
    /// Spec filename (e.g. "my-feature.md")
    pub file_name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateSpecRequest {
    /// Spec filename (e.g. "my-feature.md")
    pub file_name: String,
    /// Full replacement content
    pub content: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetAdrRequest {
    /// ADR filename (e.g. "use-postgresql.json")
    pub file_name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct SetModeRequest {
    /// Mode ID to activate. Omit to clear active mode.
    pub id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateWorkspaceToolRequest {
    /// Workspace branch/id.
    pub branch: String,
    /// Workspace type (feature, patch, service)
    pub workspace_type: Option<String>,
    /// Optional environment/profile preset ID used to seed this workspace.
    pub environment_id: Option<String>,
    /// Optional linked feature ID.
    pub feature_id: Option<String>,
    /// Optional linked target ID.
    pub target_id: Option<String>,
    /// Optional workspace mode override.
    pub mode_id: Option<String>,
    /// Whether this workspace is a git worktree.
    pub is_worktree: Option<bool>,
    /// Worktree path (required when is_worktree=true).
    pub worktree_path: Option<String>,
    /// Activate immediately after create.
    pub activate: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ActivateWorkspaceRequest {
    /// Workspace branch/id to activate.
    pub branch: String,
    /// Optional workspace mode override to apply after activation.
    pub mode_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SyncWorkspaceRequest {
    /// Workspace branch/id. If omitted, resolves from current git branch.
    pub branch: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct StartSessionRequest {
    /// Workspace branch/id. If omitted, resolves from current git branch.
    pub branch: Option<String>,
    /// Optional goal for this session.
    pub goal: Option<String>,
    /// Optional mode override for this session/workspace.
    pub mode_id: Option<String>,
    /// Optional primary provider for this session.
    pub provider_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct EndSessionRequest {
    /// Workspace branch/id. If omitted, resolves from current git branch.
    pub branch: Option<String>,
    /// End-of-session summary — what was accomplished, what changed.
    pub summary: Option<String>,
    /// Feature IDs updated during this session.
    pub updated_feature_ids: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
pub struct LogProgressRequest {
    /// Progress note — what you did, decided, or got blocked on.
    pub note: String,
    /// Workspace branch/id. If omitted, resolves from current git branch.
    pub branch: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RepairWorkspaceRequest {
    /// Workspace branch/id. If omitted, resolves from current git branch.
    pub branch: Option<String>,
    /// Preview repair without writing changes.
    pub dry_run: Option<bool>,
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
pub struct CreateTargetRequest {
    /// Target kind: "milestone" (e.g. v0.1.0) or "surface" (e.g. compiler, studio)
    pub kind: String,
    /// Short title
    pub title: String,
    /// Optional longer description
    pub description: Option<String>,
    /// One-line north star goal
    pub goal: Option<String>,
    /// Status: "active" | "planned" | "complete" | "frozen". Defaults to "active".
    pub status: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListTargetsRequest {
    /// Filter by kind: "milestone" | "surface"
    pub kind: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetTargetRequest {
    /// Target id
    pub id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateCapabilityRequest {
    /// Target this capability belongs to
    pub target_id: String,
    /// Capability title
    pub title: String,
    /// Optional milestone target id this capability is required for
    pub milestone_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct MarkCapabilityActualRequest {
    /// Capability id
    pub id: String,
    /// Evidence that proves this capability is actual (test name, commit, URL)
    pub evidence: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListCapabilitiesRequest {
    /// Filter by surface target id
    pub target_id: Option<String>,
    /// Filter by milestone id — returns capabilities across surfaces linked to this milestone
    pub milestone_id: Option<String>,
    /// Filter by status: "aspirational" | "actual"
    pub status: Option<String>,
}
