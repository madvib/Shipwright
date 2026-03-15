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
    /// Scope: project (default) or user
    pub scope: Option<String>,
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
    /// Scope: effective (default), project, or user
    pub scope: Option<String>,
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
