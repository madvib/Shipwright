//! Shared types for the platform database layer.

pub type WorkspaceDbRow = (
    String,         // 0: id
    String,         // 1: workspace_type
    String,         // 2: status
    Option<String>, // 3: active_agent
    Vec<String>,    // 4: providers
    Vec<String>,    // 5: mcp_servers
    Vec<String>,    // 6: skills
    bool,           // 7: is_worktree
    Option<String>, // 8: worktree_path
    Option<String>, // 9: last_activated_at
    Option<String>, // 10: context_hash
    i64,            // 11: config_generation
    Option<String>, // 12: compiled_at
    Option<String>, // 13: compile_error
);

pub type WorkspaceDbListRow = (
    String,         // 0: branch
    String,         // 1: id
    String,         // 2: workspace_type
    String,         // 3: status
    Option<String>, // 4: active_agent
    Vec<String>,    // 5: providers
    Vec<String>,    // 6: mcp_servers
    Vec<String>,    // 7: skills
    bool,           // 8: is_worktree
    Option<String>, // 9: worktree_path
    Option<String>, // 10: last_activated_at
    Option<String>, // 11: context_hash
    i64,            // 12: config_generation
    Option<String>, // 13: compiled_at
    Option<String>, // 14: compile_error
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
