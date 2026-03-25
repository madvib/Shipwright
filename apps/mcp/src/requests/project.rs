use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

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
    /// Note ID (nanoid returned by create_note)
    pub id: String,
    /// Scope: project (default) or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateNoteRequest {
    /// Note ID (nanoid returned by create_note)
    pub id: String,
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
pub struct SetAgentRequest {
    /// Agent ID to activate. Omit to clear active agent.
    pub id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct PushBundleRequest {
    /// JSON string containing the TransferBundle (agent, skills, dependencies).
    pub bundle: String,
}
