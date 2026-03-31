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

#[derive(Deserialize, JsonSchema)]
pub struct GetSkillVarsRequest {
    /// Skill id (directory name under .ship/skills/)
    pub skill_id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct SetSkillVarRequest {
    /// Skill id (directory name under .ship/skills/)
    pub skill_id: String,
    /// Variable name as declared in vars.json
    pub key: String,
    /// New value as a JSON string (e.g. `"\"gitmoji\""`, `"true"`, `"42"`)
    pub value_json: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListSkillVarsRequest {
    /// Optional skill id filter — if omitted, lists all skills with vars.json
    pub skill_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct WriteSkillFileRequest {
    /// Skill directory name (e.g. "tdd", "browse")
    pub skill_id: String,
    /// Relative path within the skill directory (e.g. "SKILL.md", "assets/vars.json")
    pub file_path: String,
    /// File content to write
    pub content: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct DeleteSkillFileRequest {
    /// Skill directory name (e.g. "tdd", "browse")
    pub skill_id: String,
    /// Relative path within the skill directory (e.g. "assets/vars.json")
    pub file_path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListProjectSkillsRequest {
    /// Optional search filter (substring match on skill id/name/description)
    pub query: Option<String>,
}

// ---- Git info ----

#[derive(Deserialize, JsonSchema)]
pub struct GetGitDiffRequest {
    /// Base ref for diff (default: HEAD for unstaged, or "main" for branch diff)
    pub base: Option<String>,
    /// Limit diff to a specific file path
    pub path: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetGitLogRequest {
    /// Number of commits to return (default: 20)
    pub limit: Option<u32>,
    /// Limit log to a specific file path
    pub path: Option<String>,
}

// ---- Session files ----

#[derive(Deserialize, JsonSchema)]
pub struct ReadSessionFileRequest {
    /// Relative path within .ship-session/ (e.g. "canvas.html", "screenshots/issue-001.png")
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct WriteSessionFileRequest {
    /// Relative path within .ship-session/ (e.g. "canvas.html", "screenshots/issue-001.png")
    pub path: String,
    /// File content (text for text files, base64-encoded string for binary files)
    pub content: String,
}
