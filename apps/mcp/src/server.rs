use anyhow::{Result, anyhow};
use rmcp::transport::stdio;
use rmcp::{
    ErrorData, RoleServer, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, tool::ToolCallContext, wrapper::Parameters},
    model::{
        CallToolRequestParams, CallToolResult, Content, Implementation,
        ListResourceTemplatesResult, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
        ProtocolVersion, ReadResourceRequestParams, ReadResourceResult, ResourceContents,
        ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
    tool, tool_router,
};
use runtime::{get_active_agent, workspace::get_active_workspace_type};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use crate::requests::*;
use crate::resources;
use crate::tools::{
    adr, agent, events, job, notes, project, session, skills, target, workspace, workspace_ops,
};

// ─── Server struct ────────────────────────────────────────────────────────────

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

    pub fn normalize_mode_tool_id(raw: &str) -> String {
        let mut normalized = raw.trim().to_ascii_lowercase().replace('-', "_");
        if let Some(stripped) = normalized.strip_prefix("ship_") {
            normalized = stripped.to_string();
        }
        if let Some(stripped) = normalized.strip_suffix("_tool") {
            normalized = stripped.to_string();
        }
        normalized
    }

    pub fn core_tools() -> &'static [&'static str] {
        &[
            "open_project", "create_note", "update_note", "create_adr",
            "activate_workspace", "create_workspace", "complete_workspace",
            "list_stale_worktrees", "set_agent",
            "list_workspaces", "start_session", "end_session", "log_progress",
            "list_skills", "create_job", "update_job", "list_jobs", "append_job_log",
            "claim_file", "get_file_owner",
            "list_events", "provider_matrix",
        ]
    }

    pub fn is_core_tool(tool_name: &str) -> bool {
        let normalized = Self::normalize_mode_tool_id(tool_name);
        Self::core_tools().contains(&normalized.as_str())
    }

    pub fn is_project_workspace_tool(_tool_name: &str) -> bool {
        false
    }

    pub fn mode_allows_tool(tool_name: &str, active_tools: &[String]) -> bool {
        if active_tools.is_empty() {
            return true;
        }
        let normalized_tool = Self::normalize_mode_tool_id(tool_name);
        active_tools
            .iter()
            .map(|t| Self::normalize_mode_tool_id(t))
            .any(|allowed| allowed == normalized_tool)
    }

    pub fn enforce_mode_tool_gate(project_dir: &Path, tool_name: &str) -> Result<(), String> {
        if Self::is_core_tool(tool_name) {
            return Ok(());
        }
        if Self::is_project_workspace_tool(tool_name) {
            let active_type = get_active_workspace_type(project_dir).unwrap_or(None);
            if matches!(active_type, Some(runtime::ShipWorkspaceKind::Service)) {
                return Ok(());
            }
        }
        let active_agent = get_active_agent(Some(project_dir.to_path_buf()))
            .map_err(|e| e.to_string())?;
        if let Some(ref mode) = active_agent {
            if Self::mode_allows_tool(tool_name, &mode.active_tools) {
                return Ok(());
            }
            let allowed = if mode.active_tools.is_empty() {
                "all tools".to_string()
            } else {
                mode.active_tools.join(", ")
            };
            return Err(format!(
                "Tool '{}' blocked by active mode '{}' (allowed: {}).",
                tool_name, mode.id, allowed
            ));
        }
        Err(format!(
            "Tool '{}' is not in the core workflow surface. \
             Activate the service workspace ('ship') or a mode with this tool in its \
             active_tools list to use it.",
            tool_name
        ))
    }

    fn resolve_workspace_branch_for_project(
        project_dir: &Path,
        branch: Option<&str>,
    ) -> Result<String, String> {
        if let Some(b) = branch {
            let trimmed = b.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
        let Some(root) = project_dir.parent() else {
            return Err("Error: Could not resolve project root".to_string());
        };
        current_branch(root).map_err(|e| e.to_string())
    }

    // ─── Project ──────────────────────────────────────────────────────────────

    #[tool(description = "Set the active project for subsequent MCP tool calls")]
    async fn open_project(&self, Parameters(req): Parameters<OpenProjectRequest>) -> String {
        let (msg, _) = project::open_project(&req.path, &self.active_project).await;
        msg
    }

    // ─── Notes ────────────────────────────────────────────────────────────────

    #[tool(description = "Create a standalone note attached to this project.")]
    async fn create_note(&self, Parameters(req): Parameters<CreateNoteRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        notes::create_note(&project_dir, &req.title, req.content, req.branch.as_deref())
    }

    #[tool(description = "Replace a note's markdown content by filename.")]
    async fn update_note(&self, Parameters(req): Parameters<UpdateNoteRequest>) -> String {
        let scope = match notes::parse_note_scope(req.scope.as_deref()) { Ok(s) => s, Err(e) => return format!("Error: {}", e) };
        use ship_docs::NoteScope;
        let dir = match scope { NoteScope::Project => match self.get_effective_project_dir().await { Ok(d) => Some(d), Err(e) => return e }, NoteScope::User => None };
        notes::update_note(scope, dir.as_deref(), &req.file_name, &req.content)
    }

    // ─── ADR ──────────────────────────────────────────────────────────────────

    #[tool(description = "Create a new Architecture Decision Record (ADR). Use when committing to a \
        technical approach, trade-off, or design choice that future contributors need to understand.")]
    async fn create_adr(&self, Parameters(req): Parameters<LogDecisionRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        adr::create_adr(&project_dir, &req.title, &req.decision)
    }

    // ─── Agent ────────────────────────────────────────────────────────────────

    #[tool(description = "Activate an agent profile by id, or clear active agent by passing null/omitting id.")]
    async fn set_agent(&self, Parameters(req): Parameters<SetModeRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        agent::set_agent(project_dir, req.id.as_deref())
    }

    // ─── Workspace ────────────────────────────────────────────────────────────

    #[tool(description = "Activate a workspace by branch/id and optionally set its mode override.")]
    async fn activate_workspace(&self, Parameters(req): Parameters<ActivateWorkspaceRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        workspace::activate_workspace(&project_dir, req)
    }

    #[tool(description = "List all workspaces for the active project. Optionally filter by status.")]
    async fn list_workspaces(&self, Parameters(req): Parameters<ListWorkspacesRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        workspace::list_workspaces(&project_dir, req)
    }

    #[tool(description = "Create a new workspace with a git worktree. For 'service' kind the worktree step is skipped.")]
    async fn create_workspace(&self, Parameters(req): Parameters<CreateWorkspaceRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        workspace::create_workspace(&project_dir, req)
    }

    #[tool(description = "Complete a workspace: writes a handoff.md and optionally prunes the git worktree.")]
    async fn complete_workspace(&self, Parameters(req): Parameters<CompleteWorkspaceRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        workspace_ops::complete_workspace(&project_dir, req)
    }

    #[tool(description = "List git worktrees that have been idle longer than idle_hours (default: 24).")]
    async fn list_stale_worktrees(&self, Parameters(req): Parameters<ListStaleWorktreesRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        workspace_ops::list_stale_worktrees(&project_dir, req)
    }

    // ─── Session ──────────────────────────────────────────────────────────────

    #[tool(description = "Start a workspace session for the active compiled context and selected provider.")]
    async fn start_session(&self, Parameters(req): Parameters<StartSessionRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        let branch = match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) { Ok(b) => b, Err(e) => return format!("Error: {}", e) };
        session::start_session(&project_dir, req, &branch)
    }

    #[tool(description = "End the active workspace session and record a summary. Emits a session-end event.")]
    async fn end_session(&self, Parameters(req): Parameters<EndSessionRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        let branch = match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) { Ok(b) => b, Err(e) => return format!("Error: {}", e) };
        session::end_session(&project_dir, req, &branch)
    }

    #[tool(description = "Record a progress note for the active session. Requires an active session.")]
    async fn log_progress(&self, Parameters(req): Parameters<LogProgressRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        let branch = match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) { Ok(b) => b, Err(e) => return format!("Error: {}", e) };
        session::log_progress(&project_dir, req, &branch)
    }

    // ─── Skills ───────────────────────────────────────────────────────────────

    #[tool(description = "List skills available to the active project. Optionally filter by search query.")]
    async fn list_skills(&self, Parameters(req): Parameters<ListSkillsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        skills::list_skills(&project_dir, req)
    }

    // ─── Targets ──────────────────────────────────────────────────────────────

    #[tool(description = "Create a target. kind='milestone' (e.g. v0.1.0) or kind='surface' (e.g. compiler, studio).")]
    async fn create_target(&self, Parameters(req): Parameters<CreateTargetRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        target::create_target(&project_dir, req)
    }

    #[tool(description = "List targets. Optionally filter by kind: 'milestone' or 'surface'.")]
    async fn list_targets(&self, Parameters(req): Parameters<ListTargetsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        target::list_targets(&project_dir, req)
    }

    #[tool(description = "Get a target with its full capability list (actual and aspirational).")]
    async fn get_target(&self, Parameters(req): Parameters<GetTargetRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        target::get_target(&project_dir, req)
    }

    #[tool(description = "Add an aspirational capability to a target. Optionally link to a milestone.")]
    async fn create_capability(&self, Parameters(req): Parameters<CreateCapabilityRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        target::create_capability(&project_dir, req)
    }

    #[tool(description = "Mark a capability as actual with evidence (test name, commit hash, or behavior).")]
    async fn mark_capability_actual(&self, Parameters(req): Parameters<MarkCapabilityActualRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        target::mark_capability_actual(&project_dir, req)
    }

    #[tool(description = "List capabilities. Filter by target_id, milestone_id, and/or status.")]
    async fn list_capabilities(&self, Parameters(req): Parameters<ListCapabilitiesRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        target::list_capabilities(&project_dir, req)
    }

    // ─── Events ───────────────────────────────────────────────────────────────

    #[tool(description = "Query the project event log. Returns JSON array of events. \
        Filter by since (ISO 8601 or relative: '1h', '24h', '7d'), actor, entity, or action. \
        Default limit: 50, max: 200.")]
    async fn list_events(&self, Parameters(req): Parameters<ListEventsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        events::list_events(&project_dir, req)
    }

    // ─── Jobs ─────────────────────────────────────────────────────────────────

    #[tool(description = "Create a new coordination job. Returns the new job id.")]
    async fn create_job(&self, Parameters(req): Parameters<CreateJobRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        job::create_job(&project_dir, req)
    }

    #[tool(description = "Update a job status, priority, assignment, or touched_files.")]
    async fn update_job(&self, Parameters(req): Parameters<UpdateJobRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        job::update_job(&project_dir, req)
    }

    #[tool(description = "List coordination jobs. Optionally filter by branch or status.")]
    async fn list_jobs(&self, Parameters(req): Parameters<ListJobsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        job::list_jobs(&project_dir, req)
    }

    #[tool(description = "Append a log message to a job's log. Level: 'info', 'warn', or 'error'.")]
    async fn append_job_log(&self, Parameters(req): Parameters<AppendJobLogRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        job::append_job_log(&project_dir, req)
    }

    #[tool(description = "Claim ownership of a file path for a job. Atomic and first-wins.")]
    async fn claim_file(&self, Parameters(req): Parameters<ClaimFileRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        job::claim_file(&project_dir, &req.job_id, &req.path)
    }

    #[tool(description = "Return the job that currently owns a file path, or 'unclaimed'.")]
    async fn get_file_owner(&self, Parameters(req): Parameters<GetFileOwnerRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await { Ok(d) => d, Err(e) => return e };
        job::get_file_owner(&project_dir, &req.path)
    }

    // ─── Provider matrix ─────────────────────────────────────────────────────

    #[tool(description = "Show the provider capability matrix with gap analysis.")]
    async fn provider_matrix(&self, Parameters(req): Parameters<ProviderMatrixRequest>) -> String {
        let mut matrix = compiler::build_matrix();
        if let Some(pid) = &req.provider {
            matrix.providers.retain(|p| p.provider_id == pid);
            if matrix.providers.is_empty() {
                return format!("Unknown provider: {}. Options: claude, gemini, codex, cursor", pid);
            }
        }
        match req.format.as_deref().unwrap_or("json") {
            "text" => compiler::render_text(&matrix),
            "diff" => compiler::render_diffable(&matrix),
            _ => serde_json::to_string_pretty(&matrix).unwrap_or_else(|e| format!("Serialization error: {}", e)),
        }
    }
}

// ─── Resource resolution ──────────────────────────────────────────────────────

impl ShipServer {
    pub async fn resolve_resource_uri(&self, uri: &str, dir: &Path) -> Option<String> {
        resources::resolve_resource_uri(uri, dir, project::get_project_info(dir)).await
    }
}

// ─── ServerHandler ────────────────────────────────────────────────────────────

impl ServerHandler for ShipServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().enable_resources().build(),
            server_info: Implementation {
                name: "Ship Project Tracker".into(),
                version: "0.2.0".into(),
                ..Default::default()
            },
            instructions: Some(
                "Ship project intelligence — three-stage workflow:\n\n\
                 PLANNING: get_project_info → create_note / create_adr\n\
                 WORKSPACE: list_workspaces → activate_workspace → set_agent\n\
                 SESSION: start_session → (work) → log_progress → end_session\n\n\
                 By default only core workflow tools are visible. To access extended tools, \
                 activate a mode that includes them in its active_tools list. \
                 Call open_project first if the project is not auto-detected. \
                 Use resources (ship://) for read-heavy workflows."
                    .into(),
            ),
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let tool_name = request.name.to_string();
        if let Ok(project_dir) = self.get_effective_project_dir().await
            && let Err(message) = Self::enforce_mode_tool_gate(&project_dir, &tool_name)
        {
            return Ok(CallToolResult::error(vec![Content::text(message)]));
        }
        self.tool_router.call(ToolCallContext::new(self, request, context)).await
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let all_tools = self.tool_router.list_all();
        let visible = if let Ok(project_dir) = self.get_effective_project_dir().await {
            let active_agent = get_active_agent(Some(project_dir.clone())).unwrap_or(None);
            let in_svc = matches!(
                get_active_workspace_type(&project_dir).unwrap_or(None),
                Some(runtime::ShipWorkspaceKind::Service)
            );
            all_tools.into_iter().filter(|t| {
                let n = t.name.as_ref();
                if Self::is_core_tool(n) { return true; }
                if in_svc && Self::is_project_workspace_tool(n) { return true; }
                active_agent.as_ref().map_or(false, |m| Self::mode_allows_tool(n, &m.active_tools))
            }).collect()
        } else {
            all_tools.into_iter().filter(|t| Self::is_core_tool(t.name.as_ref())).collect()
        };
        Ok(ListToolsResult::with_all_items(visible))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool_router.get(name).cloned()
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult::with_all_items(resources::static_resource_list()))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        Ok(ListResourceTemplatesResult::with_all_items(resources::static_resource_template_list()))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        let Ok(dir) = self.get_effective_project_dir().await else {
            return Err(ErrorData::internal_error("No active project", None));
        };
        match self.resolve_resource_uri(&request.uri, &dir).await {
            Some(text) => Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(text, &request.uri)],
            }),
            None => Err(ErrorData::resource_not_found(
                format!("Resource not found: {}", request.uri),
                None,
            )),
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn current_branch(project_root: &Path) -> Result<String> {
    let output = ProcessCommand::new("git")
        .args(["branch", "--show-current"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("Failed to determine current git branch");
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        anyhow::bail!("Current HEAD is detached; cannot map to a feature branch");
    }
    Ok(branch)
}

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

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "server_tests.rs"]
mod server_tests;
