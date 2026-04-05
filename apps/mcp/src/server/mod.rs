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
    agent, event, job as job_tools, project, session, session_files, skills, workspace,
    workspace_ops,
};
use skills::{
    get_skill_vars_tool, list_skill_vars_tool,
    set_skill_var_tool,
};

#[cfg(feature = "unstable")]
use crate::tools::adr;
#[cfg(feature = "unstable")]
use crate::tools::dispatch as dispatch_tools;

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
    /// Event relay state — initialized on connection init.
    relay: std::sync::Arc<tokio::sync::Mutex<RelayState>>,
    /// Actor-scoped event store for this connection.
    pub actor_store: std::sync::Arc<tokio::sync::Mutex<Option<runtime::events::ActorStore>>>,
    /// MCP provider name from clientInfo (e.g. "claude-code", "cursor").
    pub mcp_provider: std::sync::Arc<tokio::sync::Mutex<Option<String>>>,
    /// Active session ID for this connection (set by auto_session_start).
    pub session_id: std::sync::Arc<tokio::sync::Mutex<Option<String>>>,
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
            actor_store: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            mcp_provider: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            session_id: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    async fn get_effective_project_dir(&self) -> Result<PathBuf, String> {
        project::get_effective_project_dir(&self.active_project).await
    }

    pub async fn store_peer(&self, peer: Peer<RoleServer>) {
        *self.notification_peer.lock().await = Some(peer);
    }

    /// Spawn this agent's actor in the daemon's KernelRouter.
    ///
    /// Called from `on_initialized`. Derives the actor_id from the active agent
    /// profile, falling back to `"agent.mcp"`. Creates a local `ActorStore` for
    /// event persistence but delegates mailbox ownership to the daemon.
    pub async fn spawn_agent_actor(&self) {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(_) => return,
        };

        let global_dir = match runtime::project::get_global_dir() {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("failed to resolve global dir: {e}");
                return;
            }
        };

        let actor_id = self.resolve_actor_id().await;

        // Compute skill-derived subscriptions
        let skills_list = runtime::list_skills(&project_dir).unwrap_or_default();
        let skill_subs =
            runtime::events::artifact_events::skill_event_subscriptions(&skills_list);

        let mut subscribe_namespaces = vec![
            "studio.".to_string(),
            "workspace.".to_string(),
            "session.".to_string(),
            "actor.".to_string(),
            "config.".to_string(),
            "runtime.".to_string(),
            "sync.".to_string(),
            "project.".to_string(),
            "gate.".to_string(),
            "mesh.".to_string(),
        ];
        for ns in skill_subs {
            if !subscribe_namespaces.contains(&ns) {
                subscribe_namespaces.push(ns);
            }
        }

        let config = runtime::events::ActorConfig {
            namespace: actor_id.clone(),
            write_namespaces: vec!["".to_string()],
            read_namespaces: vec!["agent.".to_string()],
            subscribe_namespaces,
        };

        // Mesh capabilities from skills
        let capabilities: Vec<String> = skills_list.iter().map(|s| s.id.clone()).collect();
        let capabilities = if capabilities.is_empty() {
            vec!["general".to_string()]
        } else {
            capabilities
        };

        // Spawn actor in daemon's kernel (includes mesh registration)
        match crate::network_client::actor_spawn(&actor_id, &config, Some(capabilities)).await {
            Ok(_) => tracing::info!(actor_id, "actor spawned in daemon kernel"),
            Err(e) => {
                tracing::warn!(actor_id, "daemon actor spawn failed (daemon may not be running): {e}");
            }
        }

        // Create local ActorStore for event persistence
        match runtime::events::ActorStore::open(
            &actor_id,
            &global_dir,
            config.write_namespaces.clone(),
            config.read_namespaces.clone(),
        ) {
            Ok(store) => {
                *self.actor_store.lock().await = Some(store);
            }
            Err(e) => {
                tracing::warn!(actor_id, "failed to open local ActorStore: {e}");
            }
        }
    }

    /// Wire the EventRelay for this connection using the daemon's SSE stream
    /// as the event source. Call once per MCP connection.
    pub async fn start_event_relay(&self) {
        let mut relay_state = self.relay.lock().await;

        // Only start once
        if relay_state.handle.is_some() {
            return;
        }

        let actor_id = self.resolve_actor_id().await;

        // Connect to daemon's SSE stream for this actor's events
        let sse_rx = match crate::network_client::mesh_subscribe(&actor_id) {
            Ok(rx) => rx,
            Err(e) => {
                tracing::warn!(actor_id, "daemon SSE subscribe failed: {e}");
                return;
            }
        };

        // Bridge SSE unbounded receiver into a bounded Mailbox
        let (tx, rx) = tokio::sync::mpsc::channel(256);
        tokio::spawn(async move {
            let mut sse_rx = sse_rx;
            while let Some(event) = sse_rx.recv().await {
                if tx.send(event).await.is_err() {
                    break; // Mailbox receiver dropped
                }
            }
        });
        let mailbox = runtime::events::Mailbox::from_receiver(rx);

        let relay = notification_relay::EventRelay::new();
        relay_state.peers = relay.peers();

        // Select push adapter based on detected provider, register as peer.
        if let Some(peer) = self.notification_peer.lock().await.clone() {
            let provider = self.mcp_provider.lock().await.clone();
            let adapter =
                crate::push::select_adapter(provider.as_deref(), peer);
            tracing::info!(adapter = adapter.adapter_name(), "push adapter selected");
            let peer_handle = notification_relay::PeerHandle {
                id: "mcp-agent".to_string(),
                actor_id: "mcp".to_string(),
                adapter,
                allowed_events: std::collections::HashSet::new(),
            };
            relay.add_peer(peer_handle).await;
        }

        relay_state.handle = Some(relay.spawn(mailbox));
    }

    /// Resolve a unique actor_id for this connection.
    ///
    /// If `SHIP_MESH_ID` is set (e.g. by the daemon job-dispatch service),
    /// it is used directly as the actor_id.
    ///
    /// Otherwise: `agent.{agent_name}.{branch}` — e.g. `agent.rust-runtime.job/fix-parser`.
    /// Falls back to `agent.{agent_name}` if branch can't be detected, and
    /// `agent.mcp.{branch}` if no agent config is set.
    async fn resolve_actor_id(&self) -> String {
        // SHIP_MESH_ID takes precedence — set by daemon when dispatching jobs.
        if let Ok(mesh_id) = std::env::var("SHIP_MESH_ID") {
            if !mesh_id.is_empty() {
                tracing::info!(actor_id = %mesh_id, "resolved actor ID from SHIP_MESH_ID");
                return mesh_id;
            }
        }

        let project_dir = self.get_effective_project_dir().await.ok();
        let agent_name = project_dir
            .as_ref()
            .and_then(|d| runtime::get_active_agent(Some(d.clone())).ok().flatten())
            .map(|a| a.id)
            .unwrap_or_else(|| "mcp".to_string());

        // Try project dir first, then CWD as fallback for branch detection.
        let branch = project_dir
            .as_ref()
            .and_then(|d| tool_gate::current_branch(d).ok())
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .and_then(|d| tool_gate::current_branch(&d).ok())
            });

        let id = match branch {
            Some(b) => format!("agent.{agent_name}.{b}"),
            None => format!("agent.{agent_name}"),
        };
        tracing::info!(actor_id = %id, "resolved actor ID");
        id
    }

    /// Automatically start or resume a session on MCP connection init.
    ///
    /// 1. Clean up stale draining sessions (grace window expired).
    /// 2. Resolve current workspace from git branch.
    /// 3. If a draining session exists for this workspace+agent, resume it.
    /// 4. Otherwise start a fresh session via the runtime.
    /// 5. Store the session_id for tool-call counting and shutdown drain.
    pub(crate) async fn auto_session_start(&self) {
        use runtime::db::session_drain;

        // Clean up sessions stuck draining longer than 120s.
        let _ = session_drain::cleanup_stale_draining(120);

        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(_) => return,
        };
        let project_root = project_dir.parent().unwrap_or(&project_dir);
        let branch = match tool_gate::current_branch(project_root) {
            Ok(b) => b,
            Err(_) => return,
        };

        // Reconcile worktree state against git before session creation.
        let _ = runtime::reconcile_workspace(
            &project_dir,
            &branch,
            project_root,
        );

        let agent_id = runtime::get_active_agent(Some(project_dir.clone()))
            .ok()
            .flatten()
            .map(|a| a.id.clone());

        let provider = self.mcp_provider.lock().await.clone();

        // Try to resume a draining session for this workspace+agent.
        let agent_key = agent_id.as_deref().unwrap_or("mcp");
        if let Ok(Some(draining)) = session_drain::find_draining_session(&branch, agent_key) {
            if session_drain::resume_session(&draining.id).is_ok() {
                tracing::info!(
                    session_id = %draining.id,
                    "resumed draining session"
                );
                *self.session_id.lock().await = Some(draining.id);
                return;
            }
        }

        // Start a fresh session via the runtime.
        match runtime::start_workspace_session(
            &project_dir,
            &branch,
            None,
            agent_id,
            provider,
        ) {
            Ok(session) => {
                tracing::info!(session_id = %session.id, "auto-started session");
                *self.session_id.lock().await = Some(session.id);
            }
            Err(e) => {
                tracing::warn!("auto_session_start failed: {e}");
            }
        }
    }

    /// Drain the active session on MCP disconnect.
    ///
    /// Transitions the session to "draining" status. If another MCP connection
    /// attaches within the grace window, `auto_session_start` will resume it.
    /// Otherwise `cleanup_stale_draining` finalizes it.
    pub(crate) async fn shutdown(&self) {
        use runtime::db::session_drain;

        if let Some(sid) = self.session_id.lock().await.take() {
            match session_drain::drain_session(&sid) {
                Ok(()) => tracing::info!(session_id = %sid, "session drained on disconnect"),
                Err(e) => tracing::warn!(session_id = %sid, "drain failed: {e}"),
            }
        }
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
        let project_root = project_dir.parent().unwrap_or(&project_dir);
        let _ = runtime::reconcile_workspace(&project_dir, &branch, project_root);
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

    // ---- Mesh ----

    #[tool(description = "Send a directed message to another agent on the mesh.")]
    async fn mesh_send(&self, Parameters(req): Parameters<MeshSendRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        match crate::network_client::mesh_send(&actor_id, &req.to, req.body).await {
            Ok(result) => result,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        description = "Broadcast a message to all agents on the mesh, optionally filtered by capability."
    )]
    async fn mesh_broadcast(&self, Parameters(req): Parameters<MeshBroadcastRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        match crate::network_client::mesh_broadcast(&actor_id, req.body, req.capability_filter).await {
            Ok(result) => result,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        description = "Discover agents on the mesh. Optionally filter by capability or status."
    )]
    async fn mesh_discover(&self, Parameters(_req): Parameters<MeshDiscoverRequest>) -> String {
        match crate::network_client::mesh_discover().await {
            Ok(result) => result,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Update this agent's status on the mesh (active, busy, idle).")]
    async fn mesh_status(&self, Parameters(req): Parameters<MeshStatusRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        match crate::network_client::mesh_status(&actor_id, &req.status).await {
            Ok(result) => result,
            Err(e) => format!("Error: {e}"),
        }
    }

    // ---- Jobs ----

    #[tool(
        description = "Create a new job. Emits job.created and returns {job_id}. \
        Jobs are event-sourced — state is derived from the event log, not stored directly."
    )]
    async fn create_job(&self, Parameters(req): Parameters<CreateJobRequest>) -> String {
        job_tools::create_job(req).await
    }

    #[tool(
        description = "Advance a job to the next status by emitting the appropriate event. \
        status: dispatched (requires worktree), gate_requested (requires gate_agent), \
        gate_passed, gate_failed (requires error), blocked (requires blocker), completed, \
        merged, failed (requires error)."
    )]
    async fn update_job(&self, Parameters(req): Parameters<UpdateJobRequest>) -> String {
        job_tools::update_job(req)
    }

    #[tool(
        description = "List all jobs. Optionally filter by status: \
        pending | dispatched | gate_pending | blocked | merged | failed."
    )]
    async fn list_jobs(&self, Parameters(req): Parameters<ListJobsRequest>) -> String {
        job_tools::list_jobs(req)
    }

    #[tool(description = "Get a single job record by job_id. Returns JSON or an error.")]
    async fn get_job(&self, Parameters(req): Parameters<GetJobRequest>) -> String {
        job_tools::get_job(req)
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
        let actor_id = self.resolve_actor_id().await;
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

        // Persist to actor-scoped store.
        {
            let store_guard = self.actor_store.lock().await;
            let Some(ref store) = *store_guard else {
                return "Error: actor not initialized — ensure on_initialized completed".to_string();
            };
            if let Err(e) = store.append(&envelope) {
                return format!("Error persisting event: {}", e);
            }
        }

        // Route via daemon's KernelRouter for subscriber delivery.
        if let Err(e) = crate::network_client::event_route(
            &envelope,
            Some(&workspace_id),
            None,
        ).await {
            return format!("Error routing event via daemon: {}", e);
        }

        self.notify_resource_updated("ship://events").await;

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

    // ---- Dispatch ----

    #[tool(
        description = "Spawn an agent: creates a git worktree, compiles provider config via \
        `ship use`, launches the agent process, and registers it on the mesh. \
        Returns the agent_id for use with steer_agent, list_agents, stop_agent."
    )]
    async fn dispatch_agent(
        &self,
        Parameters(req): Parameters<DispatchAgentRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        dispatch_tools::dispatch_agent(&project_dir, req).await
    }

    #[tool(
        description = "List all running agents managed by the dispatch service, \
        with provider, pid, thread_id, and started_at."
    )]
    async fn list_agents(&self) -> String {
        dispatch_tools::list_agents().await
    }

    #[tool(
        description = "Stop a running agent by agent_id. Kills the process and \
        deregisters it from the mesh."
    )]
    async fn stop_agent(&self, Parameters(req): Parameters<StopAgentRequest>) -> String {
        dispatch_tools::stop_agent(req).await
    }

    #[tool(
        description = "Inject a message into a running agent's stdin (Claude) or \
        via turn/steer (Codex)."
    )]
    async fn steer_agent(&self, Parameters(req): Parameters<SteerAgentRequest>) -> String {
        dispatch_tools::steer_agent(req).await
    }
}


// ---- Server entry point ----

pub async fn run_server() -> Result<()> {
    let service = ShipServer::new();
    let handle = service.clone();
    let running = service
        .serve(stdio())
        .await
        .map_err(|e| anyhow!("MCP Server initialization error: {:?}", e))?;
    running
        .waiting()
        .await
        .map_err(|e| anyhow!("MCP Server runtime error: {:?}", e))?;
    handle.shutdown().await;
    Ok(())
}

// ---- Tests ----

#[cfg(test)]
#[path = "../server_tests.rs"]
mod server_tests;
