//! Shared types for the platform database layer.

pub type WorkspaceDbRow = (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    String,
    bool,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
    Option<String>,
    Option<String>,
);

pub type WorkspaceDbListRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    String,
    bool,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
    Option<String>,
    Option<String>,
);

pub struct WorkspaceUpsert<'a> {
    pub branch: &'a str,
    pub workspace_id: &'a str,
    pub workspace_type: &'a str,
    pub status: &'a str,
    pub environment_id: Option<&'a str>,
    pub feature_id: Option<&'a str>,
    pub target_id: Option<&'a str>,
    pub active_agent: Option<&'a str>,
    pub providers: &'a [String],
    pub mcp_servers: &'a [String],
    pub skills: &'a [String],
    pub resolved_at: &'a str,
    pub is_worktree: bool,
    pub worktree_path: Option<&'a str>,
    pub last_activated_at: Option<&'a str>,
    pub context_hash: Option<&'a str>,
    pub config_generation: i64,
    pub compiled_at: Option<&'a str>,
    pub compile_error: Option<&'a str>,
}

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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentRuntimeSettingsDb {
    pub providers: Vec<String>,
    pub active_agent: Option<String>,
    pub hooks_json: String,
    pub statuses_json: String,
    pub ai_json: Option<String>,
    pub git_json: String,
    pub namespaces_json: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentConfigDb {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub active_tools_json: String,
    pub mcp_refs_json: String,
    pub skill_refs_json: String,
    pub rule_refs_json: String,
    pub prompt_id: Option<String>,
    pub hooks_json: String,
    pub permissions_json: String,
    pub target_agents_json: String,
}
