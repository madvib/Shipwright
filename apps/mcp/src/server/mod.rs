pub mod event_sink;
mod handler;
pub mod notification_relay;
mod tool_gate;

use anyhow::{Result, anyhow};
use rmcp::transport::stdio;
use rmcp::{
    Peer, RoleServer, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_router,
};
use std::path::PathBuf;

use crate::requests::*;
use crate::tools::{
    agent, event, project, session, session_files, skills, workspace, workspace_ops,
};
use skills::{
    get_skill_vars_tool, list_skill_vars_tool,
    set_skill_var_tool,
};

#[cfg(feature = "unstable")]
use crate::tools::{adr, job, notes, target};
#[cfg(feature = "unstable")]
use target::{
    delete_capability as tool_delete_capability, update_capability as tool_update_capability,
    update_target as tool_update_target,
};

// ---- Server struct ----

/// Holds event relay state. Not Debug/Clone — stored behind Arc.
struct RelayState {
    /// Shared peer list for the event relay (add/remove peers after spawn).
    peers: std::sync::Arc<tokio::sync::RwLock<Vec<notification_relay::PeerHandle>>>,
    /// Handle to the spawned relay task (kept alive for the server lifetime).
    handle: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Clone)]
pub struct ShipServer {
    tool_router: ToolRouter<Self>,
    pub active_project: std::sync::Arc<tokio::sync::Mutex<Option<PathBuf>>>,
    pub notification_peer: std::sync::Arc<tokio::sync::Mutex<Option<Peer<RoleServer>>>>,
    /// URIs the client has subscribed to via resources/subscribe
    pub subscriptions: std::sync::Arc<tokio::sync::Mutex<std::collections::HashSet<String>>>,
    /// Event relay state — initialized lazily on first workspace activation.
    relay: std::sync::Arc<tokio::sync::Mutex<RelayState>>,
}

impl std::fmt::Debug for ShipServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShipServer").finish_non_exhaustive()
    }
}

// ---- Stable tool registration ----

#[tool_router]
impl ShipServer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let router = {
            #[allow(unused_mut)]
            let mut r = Self::tool_router();
            #[cfg(feature = "unstable")]
            r.merge(Self::unstable_tool_router());
            r
        };
        // Detect project from CWD at startup
        let project_dir = runtime::project::get_project_dir(None)
            .ok()
            .map(|ship_dir| {
                if ship_dir.file_name().and_then(|n| n.to_str()) == Some(".ship") {
                    ship_dir.parent().unwrap_or(&ship_dir).to_path_buf()
                } else {
                    ship_dir
                }
            });
        Self {
            tool_router: router,
            active_project: std::sync::Arc::new(tokio::sync::Mutex::new(project_dir)),
            notification_peer: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            subscriptions: std::sync::Arc::new(tokio::sync::Mutex::new(
                std::collections::HashSet::new(),
            )),
            relay: std::sync::Arc::new(tokio::sync::Mutex::new(RelayState {
                peers: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
                handle: None,
            })),
        }
    }

    async fn get_effective_project_dir(&self) -> Result<PathBuf, String> {
        project::get_effective_project_dir(&self.active_project).await
    }

    pub async fn store_peer(&self, peer: Peer<RoleServer>) {
        *self.notification_peer.lock().await = Some(peer);
    }

    /// Wire the EventRelay for a workspace. Subscribes to the workspace bus
    /// and spawns the relay task. Call once per MCP connection lifecycle.
    pub async fn start_event_relay(&self, workspace_id: &str) {
        let mut relay_state = self.relay.lock().await;

        // Only start once
        if relay_state.handle.is_some() {
            return;
        }

        let Ok(event_router) = std::panic::catch_unwind(runtime::events::router) else {
            return; // Router not initialized
        };

        let rx = event_router.subscribe_workspace(workspace_id).await;
        let relay = notification_relay::EventRelay::new();

        // Share the peers Arc so we can add/remove peers later
        relay_state.peers = relay.peers();

        // Register the MCP peer as a sink if available
        if let Some(peer) = self.notification_peer.lock().await.clone() {
            let sink = event_sink::McpEventSink::new(peer);
            let peer_handle = notification_relay::PeerHandle {
                id: format!("mcp-{workspace_id}"),
                actor_id: "mcp".to_string(),
                sink: Box::new(sink),
                allowed_events: std::collections::HashSet::new(), // system peer
            };
            relay.add_peer(peer_handle).await;
        }

        relay_state.handle = Some(relay.spawn(rx));
    }

    async fn notify_resources_changed(&self) {
        if let Some(peer) = self.notification_peer.lock().await.as_ref() {
            let _ = peer.notify_resource_list_changed().await;
        }
    }

    /// Push a resource update notification if the client is subscribed to this URI.
    pub async fn notify_resource_updated(&self, uri: &str) {
        let subscribed = self.subscriptions.lock().await.contains(uri);
        if subscribed {
            if let Some(peer) = self.notification_peer.lock().await.as_ref() {
                let _ = peer
                    .notify_resource_updated(rmcp::model::ResourceUpdatedNotificationParam {
                        uri: uri.to_string(),
                    })
                    .await;
            }
        }
    }

    #[cfg(test)]
    pub fn registered_tool_names(&self) -> Vec<String> {
        self.tool_router
            .list_all()
            .into_iter()
            .map(|t| t.name.to_string())
            .collect()
    }

    // ---- Project ----

    #[tool(description = "Set the active project for subsequent MCP tool calls")]
    async fn open_project(&self, Parameters(req): Parameters<OpenProjectRequest>) -> String {
        let (msg, resolved) = project::open_project(&req.path, &self.active_project).await;
        if resolved.is_some() {
            self.notify_resources_changed().await;
        }
        msg
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

    // ---- Studio sync ----

    // Studio-only tools (pull_agents, list_local_agents, push_bundle) are on
    // StudioServer, not here. Agents don't need to pull/push their own config.

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

    #[tool(
        description = "Get the active session for a workspace branch. Returns session JSON or 'No active session'."
    )]
    async fn get_session(&self, Parameters(req): Parameters<GetSessionRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let branch =
            match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) {
                Ok(b) => b,
                Err(e) => return format!("Error: {}", e),
            };
        session::get_session(&project_dir, req, &branch)
    }

    #[tool(
        description = "List session history for a branch. Returns all branches if branch is omitted. \
        Default limit: 20, max: 100."
    )]
    async fn list_sessions(&self, Parameters(req): Parameters<ListSessionsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        session::list_sessions(&project_dir, req)
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

    #[tool(
        description = "Get the merged variable state for a skill (defaults + user state + project state). \
        Returns JSON object of var name → current value."
    )]
    async fn get_skill_vars(&self, Parameters(req): Parameters<GetSkillVarsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        get_skill_vars_tool(&project_dir, req)
    }

    #[tool(
        description = "Set a skill variable value. Pass value_json as a JSON-encoded string \
        (e.g. '\"gitmoji\"' for strings, 'true' for bools, '42' for numbers). \
        The variable must be declared in the skill's vars.json."
    )]
    async fn set_skill_var(&self, Parameters(req): Parameters<SetSkillVarRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let result = set_skill_var_tool(&project_dir, req);
        if !result.starts_with("Error") {
            self.notify_resources_changed().await;
        }
        result
    }

    #[tool(
        description = "List all skills that have configurable variables (vars.json). \
        Optionally filter to a single skill_id. Shows current value for each var."
    )]
    async fn list_skill_vars(&self, Parameters(req): Parameters<ListSkillVarsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        list_skill_vars_tool(&project_dir, req)
    }

    // Studio-only tools (write_skill_file, delete_skill_file, list_project_skills)
    // are on StudioServer. Agents use list_skills and skill_vars for their needs.

    // ---- Session Files ----

    #[tool(
        description = "Write a file to .ship-session/. Fires a resource update notification \
        so subscribed clients (Studio, agents) react immediately. \
        Path is relative to .ship-session/ (e.g. 'canvas.html', 'vitest/report.html')."
    )]
    async fn write_session_file(
        &self,
        Parameters(req): Parameters<WriteSessionFileRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let path = req.path.clone();
        let result = session_files::write_session_file(&project_dir, req);
        if !result.starts_with("Error") {
            let uri = format!("ship://session/{}", path);
            self.notify_resource_updated(&uri).await;
            self.notify_resources_changed().await;
        }
        result
    }

    #[tool(
        description = "Read a file from .ship-session/. Returns text content or base64 for binary files."
    )]
    async fn read_session_file(
        &self,
        Parameters(req): Parameters<ReadSessionFileRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        session_files::read_session_file(&project_dir, req)
    }

    #[tool(
        description = "List all files in .ship-session/ with metadata (name, path, type, size)."
    )]
    async fn list_session_files(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        session_files::list_session_files(&project_dir)
    }

    // ---- Events ----

    #[tool(
        description = "Emit a domain event. Reserved types (actor.*, session.*, skill.*, \
        workspace.*, gate.*, job.*, config.*, project.*) are rejected. \
        actor_id and workspace_id are injected from connection context — not agent-controlled."
    )]
    async fn event(&self, Parameters(req): Parameters<ShipEventRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let actor_id = runtime::get_active_agent(Some(project_dir.clone()))
            .ok()
            .flatten()
            .map(|a| a.id)
            .unwrap_or_else(|| "mcp".to_string());
        let workspace_id =
            tool_gate::current_branch(project_dir.parent().unwrap_or(&project_dir))
                .unwrap_or_else(|_| "unknown".to_string());
        let envelope = match event::handle_ship_event(
            &actor_id,
            &workspace_id,
            &req.event_type,
            req.payload,
            req.elevated.unwrap_or(false),
        ) {
            Ok(e) => e,
            Err(e) => return format!("Error: {}", e),
        };
        let ctx = runtime::events::EmitContext {
            caller_kind: runtime::events::CallerKind::Mcp,
            skill_id: None,
            workspace_id: Some(workspace_id),
            session_id: None,
        };
        let router = match std::panic::catch_unwind(runtime::events::router) {
            Ok(r) => r,
            Err(_) => return "Error: EventRouter not initialized".to_string(),
        };
        if let Err(e) = router.emit(envelope.clone(), &ctx).await {
            return format!("Error emitting event: {}", e);
        }
        match serde_json::to_string(&envelope) {
            Ok(json) => json,
            Err(_) => format!("Event emitted: {}", envelope.id),
        }
    }

}

// ---- Unstable tool registration ----

#[cfg(feature = "unstable")]
#[tool_router(router = unstable_tool_router)]
impl ShipServer {
    // ---- Notes ----

    #[tool(description = "Create a standalone note attached to this project.")]
    async fn create_note(&self, Parameters(req): Parameters<CreateNoteRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        notes::create_note(&project_dir, &req.title, req.content, req.branch.as_deref())
    }

    #[tool(description = "Replace a note's markdown content by ID.")]
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
        notes::update_note(scope, dir.as_deref(), &req.id, &req.content)
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
