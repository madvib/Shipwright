#![allow(dead_code)]

use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

// ─── Request Types ────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct CreateIssueRequest {
    /// The title of the issue
    pub title: String,
    /// The detailed description of the issue
    pub description: String,
    /// Initial status: backlog (default), in-progress, blocked, or done
    pub status: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetIssueRequest {
    /// Issue filename (e.g. "my-feature.md")
    pub file_name: String,
    /// Status folder to look in. If omitted, all statuses are searched.
    pub status: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateIssueRequest {
    /// Issue filename (e.g. "my-feature.md")
    pub file_name: String,
    /// Current status folder
    pub status: String,
    /// New title (optional)
    pub title: Option<String>,
    /// New description (optional)
    pub description: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DeleteIssueRequest {
    /// Issue filename (e.g. "my-feature.md")
    pub file_name: String,
    /// Status folder the issue is in
    pub status: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct MoveIssueRequest {
    /// Issue filename (e.g. "my-feature.md")
    pub file_name: String,
    /// Current status
    pub from_status: String,
    /// Target status
    pub to_status: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct SearchIssuesRequest {
    /// Text to search for in issue titles and descriptions
    pub query: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateAdrRequest {
    /// Title of the architecture decision
    pub title: String,
    /// The decision content / reasoning
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
pub struct TimeStartRequest {
    /// Issue filename (e.g. "my-feature.md")
    pub issue_file: String,
    /// Optional note for this session
    pub note: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TimeStopRequest {
    /// Optional note to attach to the completed entry
    pub note: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GenerateIssueRequest {
    /// Title or brief description of the issue to generate content for
    pub title: String,
    /// Optional extra context (e.g. related issues, tech stack)
    pub context: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GenerateAdrRequest {
    /// The problem or decision to address
    pub problem: String,
    /// Optional constraints or options already under consideration
    pub constraints: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct BrainstormRequest {
    /// Topic or area to brainstorm issues for
    pub topic: String,
    /// Number of issue suggestions to generate (default 5)
    pub count: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GitIncludeRequest {
    /// Category to change: issues, releases, features, specs, adrs, notes, agents, ship.toml, templates
    pub category: String,
    /// true = commit to git, false = local only (gitignored)
    pub commit: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct GitFeatureSyncRequest {
    /// Optional branch name. If omitted, resolves from `git branch --show-current`.
    pub branch: Option<String>,
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
pub struct CreateSkillRequest {
    /// Stable skill id (e.g. "task-policy")
    pub id: String,
    /// Human-readable skill name
    pub name: String,
    /// Skill body content; supports $ARGUMENTS placeholder
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
pub struct UpdateSkillRequest {
    /// Skill id (without .md)
    pub id: String,
    /// Optional new display name
    pub name: Option<String>,
    /// Optional replacement content
    pub content: Option<String>,
    /// Scope: project (default) or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DeleteSkillRequest {
    /// Skill id (without .md)
    pub id: String,
    /// Scope: project (default) or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ManageStatusRequest {
    /// Action: "add" or "remove"
    pub action: String,
    /// Status name (e.g. "review", "testing")
    pub name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct GhostScanRequest {
    /// Directory to scan. Defaults to the project root (parent of .ship).
    pub dir: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GhostPromoteRequest {
    /// Relative file path of the ghost issue
    pub file: String,
    /// Line number of the ghost issue
    pub line: usize,
}

#[derive(Deserialize, JsonSchema)]
pub struct StatusNameRequest {
    /// Status name (e.g. "review", "testing")
    pub name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TimeLogRequest {
    /// Issue filename
    pub issue_file: String,
    /// Duration in minutes
    pub minutes: u64,
    /// Optional note
    pub note: Option<String>,
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
pub struct CreateReleaseRequest {
    /// Version label (e.g. "v0.1.0-alpha")
    pub version: String,
    /// Initial markdown content (optional — defaults to a scaffold)
    pub content: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetReleaseRequest {
    /// Release version/id (e.g. "v0.1.0-alpha")
    pub id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateReleaseRequest {
    /// Release version/id (e.g. "v0.1.0-alpha")
    pub id: String,
    /// Full replacement content
    pub content: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateFeatureRequest {
    /// Feature title
    pub title: String,
    /// Initial markdown content (optional — defaults to a scaffold)
    pub content: Option<String>,
    /// Linked release ID (optional)
    pub release_id: Option<String>,
    /// Linked spec ID (optional)
    pub spec_id: Option<String>,
    /// Linked git branch name (optional)
    pub branch: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetFeatureRequest {
    /// Feature ID (e.g. "agent-mode-ui")
    pub id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateFeatureRequest {
    /// Feature ID
    pub id: String,
    /// Full replacement content
    pub content: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct FeatureIdRequest {
    /// Feature ID
    pub id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetAdrRequest {
    /// ADR filename (e.g. "use-postgresql.json")
    pub file_name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListEventsRequest {
    /// Only return events where seq > since
    pub since: Option<u64>,
    /// Maximum number of events to return (default 100)
    pub limit: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ConnectProviderRequest {
    /// Provider ID to enable (claude, gemini, codex)
    pub provider_id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct DisconnectProviderRequest {
    /// Provider ID to disable (claude, gemini, codex)
    pub provider_id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListModelsRequest {
    /// Provider ID (claude, gemini, codex)
    pub provider_id: String,
}
