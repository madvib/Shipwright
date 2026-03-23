mod handler;
mod tool_gate;

use anyhow::{Result, anyhow};
use rmcp::transport::stdio;
use rmcp::{
    ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_router,
};
use std::path::PathBuf;

use crate::requests::*;
use crate::tools::{
    adr, agent, events, job, notes, project, session, skills, target, workspace, workspace_ops,
};
use target::{
    delete_capability as tool_delete_capability, update_capability as tool_update_capability,
    update_target as tool_update_target,
};

// ---- Server struct ----

#[derive(Debug, Clone)]
pub struct ShipServer {
    tool_router: ToolRouter<Self>,
    pub active_project: std::sync::Arc<tokio::sync::Mutex<Option<PathBuf>>>,
}

#[tool_router]
impl ShipServer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            active_project: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    async fn get_effective_project_dir(&self) -> Result<PathBuf, String> {
        project::get_effective_project_dir(&self.active_project).await
    }

    // ---- Project ----

    #[tool(description = "Set the active project for subsequent MCP tool calls")]
    async fn open_project(&self, Parameters(req): Parameters<OpenProjectRequest>) -> String {
        let (msg, _) = project::open_project(&req.path, &self.active_project).await;
        msg
    }

    // ---- Notes ----

    #[tool(description = "Create a standalone note attached to this project.")]
    async fn create_note(&self, Parameters(req): Parameters<CreateNoteRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        notes::create_note(&project_dir, &req.title, req.content, req.branch.as_deref())
    }

    #[tool(description = "Replace a note's markdown content by filename.")]
    async fn update_note(&self, Parameters(req): Parameters<UpdateNoteRequest>) -> String {
        let scope = match notes::parse_note_scope(req.scope.as_deref()) {
            Ok(s) => s,
            Err(e) => return format!("Error: {}", e),
        };
        use crate::tools::notes::NoteScope;
        let dir = match scope {
            NoteScope::Project => match self.get_effective_project_dir().await {
                Ok(d) => Some(d),
                Err(e) => return e,
            },
            NoteScope::User => None,
        };
        notes::update_note(scope, dir.as_deref(), &req.file_name, &req.content)
    }

    // ---- ADR ----

    #[tool(
        description = "Create a new Architecture Decision Record (ADR). Use when committing to a \
        technical approach, trade-off, or design choice that future contributors need to understand."
    )]
    async fn create_adr(&self, Parameters(req): Parameters<LogDecisionRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        adr::create_adr(&project_dir, &req.title, &req.decision)
    }

    // ---- Agent ----

    #[tool(
        description = "Activate an agent profile by id, or clear active agent by passing null/omitting id."
    )]
    async fn set_agent(&self, Parameters(req): Parameters<SetAgentRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        agent::set_agent(project_dir, req.id.as_deref())
    }

    // ---- Workspace ----

    #[tool(description = "Activate a workspace by branch/id and optionally set its mode override.")]
    async fn activate_workspace(
        &self,
        Parameters(req): Parameters<ActivateWorkspaceRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        workspace::activate_workspace(&project_dir, req)
    }

    #[tool(
        description = "List all workspaces for the active project. Optionally filter by status."
    )]
    async fn list_workspaces(&self, Parameters(req): Parameters<ListWorkspacesRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        workspace::list_workspaces(&project_dir, req)
    }

    #[tool(
        description = "Create a new workspace with a git worktree. For 'service' kind the worktree step is skipped."
    )]
    async fn create_workspace(
        &self,
        Parameters(req): Parameters<CreateWorkspaceRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        workspace::create_workspace(&project_dir, req)
    }

    #[tool(
        description = "Complete a workspace: writes a handoff.md and optionally prunes the git worktree."
    )]
    async fn complete_workspace(
        &self,
        Parameters(req): Parameters<CompleteWorkspaceRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        workspace_ops::complete_workspace(&project_dir, req)
    }

    #[tool(
        description = "List git worktrees that have been idle longer than idle_hours (default: 24)."
    )]
    async fn list_stale_worktrees(
        &self,
        Parameters(req): Parameters<ListStaleWorktreesRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        workspace_ops::list_stale_worktrees(&project_dir, req)
    }

    // ---- Session ----

    #[tool(
        description = "Start a workspace session for the active compiled context and selected provider."
    )]
    async fn start_session(&self, Parameters(req): Parameters<StartSessionRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let branch =
            match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) {
                Ok(b) => b,
                Err(e) => return format!("Error: {}", e),
            };
        session::start_session(&project_dir, req, &branch)
    }

    #[tool(
        description = "End the active workspace session and record a summary. Emits a session-end event."
    )]
    async fn end_session(&self, Parameters(req): Parameters<EndSessionRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let branch =
            match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) {
                Ok(b) => b,
                Err(e) => return format!("Error: {}", e),
            };
        session::end_session(&project_dir, req, &branch)
    }

    #[tool(
        description = "Record a progress note for the active session. Requires an active session."
    )]
    async fn log_progress(&self, Parameters(req): Parameters<LogProgressRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let branch =
            match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) {
                Ok(b) => b,
                Err(e) => return format!("Error: {}", e),
            };
        session::log_progress(&project_dir, req, &branch)
    }

    // ---- Skills ----

    #[tool(
        description = "List skills available to the active project. Optionally filter by search query."
    )]
    async fn list_skills(&self, Parameters(req): Parameters<ListSkillsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        skills::list_skills(&project_dir, req)
    }

    // ---- Targets ----

    #[tool(
        description = "Create a target. kind='milestone' (e.g. v0.1.0) or kind='surface' (e.g. compiler, studio). \
        Accepts phase, due_date, body_markdown, and file_scope for full intent document."
    )]
    async fn create_target(&self, Parameters(req): Parameters<CreateTargetRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        target::create_target(&project_dir, req)
    }

    #[tool(
        description = "Update a target's metadata or long-form body_markdown. Patch-style: only provided fields change."
    )]
    async fn update_target(&self, Parameters(req): Parameters<UpdateTargetRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        tool_update_target(&project_dir, req)
    }

    #[tool(description = "List targets. Optionally filter by kind: 'milestone' or 'surface'.")]
    async fn list_targets(&self, Parameters(req): Parameters<ListTargetsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        target::list_targets(&project_dir, req)
    }

    #[tool(
        description = "Get a target with its full capability progress board (done / in-progress / planned)."
    )]
    async fn get_target(&self, Parameters(req): Parameters<GetTargetRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        target::get_target(&project_dir, req)
    }

    #[tool(
        description = "Add a capability to a target. Accepts phase, acceptance_criteria, file_scope, assigned_to, priority."
    )]
    async fn create_capability(
        &self,
        Parameters(req): Parameters<CreateCapabilityRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        target::create_capability(&project_dir, req)
    }

    #[tool(
        description = "Update a capability's fields. Patch-style. Status: aspirational | in_progress | actual."
    )]
    async fn update_capability(
        &self,
        Parameters(req): Parameters<UpdateCapabilityRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        tool_update_capability(&project_dir, req)
    }

    #[tool(description = "Delete a capability by id. Returns confirmation or not-found.")]
    async fn delete_capability(
        &self,
        Parameters(req): Parameters<DeleteCapabilityRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        tool_delete_capability(&project_dir, req)
    }

    #[tool(
        description = "Mark a capability as actual with evidence (test name, commit hash, or behavior)."
    )]
    async fn mark_capability_actual(
        &self,
        Parameters(req): Parameters<MarkCapabilityActualRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        target::mark_capability_actual(&project_dir, req)
    }

    #[tool(
        description = "List capabilities. Filter by target_id, milestone_id, status, and/or phase."
    )]
    async fn list_capabilities(
        &self,
        Parameters(req): Parameters<ListCapabilitiesRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        target::list_capabilities(&project_dir, req)
    }

    // ---- Events ----

    #[tool(
        description = "Query the project event log. Returns JSON array of events. \
        Filter by since (ISO 8601 or relative: '1h', '24h', '7d'), actor, entity, or action. \
        Default limit: 50, max: 200."
    )]
    async fn list_events(&self, Parameters(req): Parameters<ListEventsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        events::list_events(&project_dir, req)
    }

    // ---- Jobs ----

    #[tool(description = "Create a new coordination job. Returns the new job id.")]
    async fn create_job(&self, Parameters(req): Parameters<CreateJobRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        job::create_job(&project_dir, req)
    }

    #[tool(description = "Update a job status, priority, assignment, or touched_files.")]
    async fn update_job(&self, Parameters(req): Parameters<UpdateJobRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        job::update_job(&project_dir, req)
    }

    #[tool(description = "List coordination jobs. Optionally filter by branch or status.")]
    async fn list_jobs(&self, Parameters(req): Parameters<ListJobsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        job::list_jobs(&project_dir, req)
    }

    #[tool(description = "Append a log message to a job's log. Level: 'info', 'warn', or 'error'.")]
    async fn append_job_log(&self, Parameters(req): Parameters<AppendJobLogRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        job::append_job_log(&project_dir, req)
    }

    #[tool(description = "Claim ownership of a file path for a job. Atomic and first-wins.")]
    async fn claim_file(&self, Parameters(req): Parameters<ClaimFileRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        job::claim_file(&project_dir, &req.job_id, &req.path)
    }

    #[tool(description = "Return the job that currently owns a file path, or 'unclaimed'.")]
    async fn get_file_owner(&self, Parameters(req): Parameters<GetFileOwnerRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        job::get_file_owner(&project_dir, &req.path)
    }

    // ---- Provider matrix ----

    #[tool(description = "Show the provider capability matrix with gap analysis.")]
    async fn provider_matrix(&self, Parameters(req): Parameters<ProviderMatrixRequest>) -> String {
        let mut matrix = compiler::build_matrix();
        if let Some(pid) = &req.provider {
            matrix.providers.retain(|p| p.provider_id == pid);
            if matrix.providers.is_empty() {
                return format!(
                    "Unknown provider: {}. Options: claude, gemini, codex, cursor",
                    pid
                );
            }
        }
        match req.format.as_deref().unwrap_or("json") {
            "text" => compiler::render_text(&matrix),
            "diff" => compiler::render_diffable(&matrix),
            _ => serde_json::to_string_pretty(&matrix)
                .unwrap_or_else(|e| format!("Serialization error: {}", e)),
        }
    }
}

// ---- Server entry point ----

pub async fn run_server() -> Result<()> {
    let service = ShipServer::new();
    let running = service
        .serve(stdio())
        .await
        .map_err(|e| anyhow!("MCP Server initialization error: {:?}", e))?;
    running
        .waiting()
        .await
        .map_err(|e| anyhow!("MCP Server runtime error: {:?}", e))?;
    Ok(())
}

// ---- Tests ----

#[cfg(test)]
#[path = "../server_tests.rs"]
mod server_tests;
