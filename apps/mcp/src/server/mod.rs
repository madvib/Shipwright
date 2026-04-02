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
    agent, event, mesh as mesh_tools, project, session, session_files, skills, workspace,
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
    /// Mailbox for this connection's actor. Taken once by start_event_relay.
    pub actor_mailbox: std::sync::Arc<tokio::sync::Mutex<Option<runtime::events::Mailbox>>>,
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
            actor_mailbox: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    async fn get_effective_project_dir(&self) -> Result<PathBuf, String> {
        project::get_effective_project_dir(&self.active_project).await
    }

    pub async fn store_peer(&self, peer: Peer<RoleServer>) {
        *self.notification_peer.lock().await = Some(peer);
    }

    /// Spawn an actor for this connection using the global KernelRouter.
    ///
    /// Called from `on_initialized`. Derives the actor_id from the active agent
    /// profile, falling back to `"agent.mcp"`.
    pub async fn spawn_agent_actor(&self) {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(_) => return,
        };

        // Initialize KernelRouter with the global ~/.ship/ dir — never the project's .ship/.
        let global_dir = match runtime::project::get_global_dir() {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("failed to resolve global dir: {e}");
                return;
            }
        };
        let kr = match runtime::events::init_kernel_router(global_dir) {
            Ok(kr) => kr,
            Err(e) => {
                tracing::warn!("failed to initialize KernelRouter: {e}");
                return;
            }
        };

        let actor_id = runtime::get_active_agent(Some(project_dir.clone()))
            .ok()
            .flatten()
            .map(|a| format!("agent.{}", a.id))
            .unwrap_or_else(|| "agent.mcp".to_string());

        // Spawn MeshService if not already running.
        spawn_mesh_service_once(&kr).await;

        // Compute skill-derived subscriptions: ship.* platform events + custom skill namespaces.
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
            // Allow writing any non-system namespace (enforced by the event tool).
            write_namespaces: vec!["".to_string()],
            read_namespaces: vec!["agent.".to_string()],
            subscribe_namespaces,
        };

        let mut kr_guard = kr.lock().await;
        let _ = kr_guard.stop_actor(&actor_id);
        match kr_guard.spawn_actor(&actor_id, config) {
            Ok((store, mailbox)) => {
                *self.actor_store.lock().await = Some(store);
                *self.actor_mailbox.lock().await = Some(mailbox);
            }
            Err(e) => {
                tracing::warn!("failed to spawn agent actor '{actor_id}': {e}");
                return;
            }
        }

        // Auto-register this agent on the mesh.
        let capabilities: Vec<String> = skills_list.iter().map(|s| s.id.clone()).collect();
        let capabilities = if capabilities.is_empty() {
            vec!["general".to_string()]
        } else {
            capabilities
        };
        let register_event = match runtime::events::EventEnvelope::new(
            "mesh.register",
            &actor_id,
            &serde_json::json!({ "agent_id": actor_id, "capabilities": capabilities }),
        ) {
            Ok(e) => e.with_actor_id(&actor_id),
            Err(e) => {
                tracing::warn!("failed to build mesh.register event: {e}");
                return;
            }
        };
        let emit_ctx = runtime::events::EmitContext {
            caller_kind: runtime::events::CallerKind::Mcp,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        };
        if let Err(e) = kr.lock().await.route(register_event, &emit_ctx).await {
            tracing::warn!("failed to route mesh.register: {e}");
        }
    }

    /// Wire the EventRelay for this connection. Takes the mailbox from the
    /// spawned actor and starts the relay task. Call once per MCP connection.
    pub async fn start_event_relay(&self) {
        let mut relay_state = self.relay.lock().await;

        // Only start once
        if relay_state.handle.is_some() {
            return;
        }

        let mailbox = self.actor_mailbox.lock().await.take();
        let Some(mailbox) = mailbox else {
            return; // Actor not yet spawned
        };

        let relay = notification_relay::EventRelay::new();

        // Share the peers Arc so we can add/remove peers later
        relay_state.peers = relay.peers();

        // Register the MCP peer as a sink if available
        if let Some(peer) = self.notification_peer.lock().await.clone() {
            let sink = event_sink::McpEventSink::new(peer);
            let peer_handle = notification_relay::PeerHandle {
                id: "mcp-agent".to_string(),
                actor_id: "mcp".to_string(),
                sink: Box::new(sink),
                allowed_events: std::collections::HashSet::new(), // system peer
            };
            relay.add_peer(peer_handle).await;
        }

        relay_state.handle = Some(relay.spawn(mailbox));
    }

    /// Resolve the actor_id for the current connection.
    async fn resolve_actor_id(&self) -> String {
        self.get_effective_project_dir()
            .await
            .ok()
            .and_then(|d| runtime::get_active_agent(Some(d)).ok().flatten())
            .map(|a| format!("agent.{}", a.id))
            .unwrap_or_else(|| "agent.mcp".to_string())
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

    // ---- Mesh ----

    #[tool(description = "Send a directed message to another agent on the mesh.")]
    async fn mesh_send(&self, Parameters(req): Parameters<MeshSendRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        let envelope = match mesh_tools::build_mesh_send(&actor_id, &req.to, req.body) {
            Ok(e) => e,
            Err(e) => return format!("Error: {e}"),
        };
        route_mesh_envelope(envelope).await
    }

    #[tool(
        description = "Broadcast a message to all agents on the mesh, optionally filtered by capability."
    )]
    async fn mesh_broadcast(&self, Parameters(req): Parameters<MeshBroadcastRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        let envelope =
            match mesh_tools::build_mesh_broadcast(&actor_id, req.body, req.capability_filter) {
                Ok(e) => e,
                Err(e) => return format!("Error: {e}"),
            };
        route_mesh_envelope(envelope).await
    }

    #[tool(
        description = "Discover agents on the mesh. Optionally filter by capability or status."
    )]
    async fn mesh_discover(&self, Parameters(req): Parameters<MeshDiscoverRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        let envelope =
            match mesh_tools::build_mesh_discover(&actor_id, req.capability, req.status) {
                Ok(e) => e,
                Err(e) => return format!("Error: {e}"),
            };
        route_mesh_envelope(envelope).await
    }

    #[tool(description = "Update this agent's status on the mesh (active, busy, idle).")]
    async fn mesh_status(&self, Parameters(req): Parameters<MeshStatusRequest>) -> String {
        let actor_id = self.resolve_actor_id().await;
        let envelope = match mesh_tools::build_mesh_status(&actor_id, &req.status) {
            Ok(e) => e,
            Err(e) => return format!("Error: {e}"),
        };
        route_mesh_envelope(envelope).await
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
            .map(|a| format!("agent.{}", a.id))
            .unwrap_or_else(|| "agent.mcp".to_string());
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

        // Route via KernelRouter for mailbox delivery.
        let ctx = runtime::events::EmitContext {
            caller_kind: runtime::events::CallerKind::Mcp,
            skill_id: None,
            workspace_id: Some(workspace_id),
            session_id: None,
        };
        let Some(kr) = runtime::events::kernel_router() else {
            return "Error: KernelRouter not initialized".to_string();
        };
        if let Err(e) = kr.lock().await.route(envelope.clone(), &ctx).await {
            return format!("Error routing event: {}", e);
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

// ---- Mesh service helpers ----

/// Guards a single MeshService spawn across all MCP connections in this process.
static MESH_SERVICE_SPAWNED: std::sync::OnceLock<()> = std::sync::OnceLock::new();

/// Spawn the MeshService into the global KernelRouter if not already present.
///
/// Idempotent — safe to call on every MCP connection, only runs once per process.
/// Also spawns a feedback task that drains the mesh's outbox and routes
/// each response event back through the kernel (directed delivery).
async fn spawn_mesh_service_once(kr: &std::sync::Arc<tokio::sync::Mutex<runtime::events::KernelRouter>>) {
    if MESH_SERVICE_SPAWNED.set(()).is_err() {
        return; // Already spawned by another connection.
    }

    let (outbox_tx, mut outbox_rx) = tokio::sync::mpsc::unbounded_channel::<runtime::events::EventEnvelope>();
    let mesh_config = runtime::events::ActorConfig {
        namespace: "service.mesh".to_string(),
        write_namespaces: vec!["mesh.".to_string()],
        read_namespaces: vec!["mesh.".to_string()],
        subscribe_namespaces: vec!["mesh.".to_string()],
    };
    let handler: Box<dyn runtime::services::ServiceHandler> =
        Box::new(runtime::services::mesh::MeshService::new(outbox_tx));

    match runtime::services::spawn_service(&mut *kr.lock().await, "service.mesh", mesh_config, handler) {
        Ok(_) => {}
        Err(e) => {
            tracing::warn!("failed to spawn MeshService: {e}");
            return;
        }
    }

    // Drain mesh outbox → kernel router (directed delivery).
    let kr_clone = kr.clone();
    tokio::spawn(async move {
        let ctx = runtime::events::EmitContext {
            caller_kind: runtime::events::CallerKind::Mcp,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        };
        while let Some(event) = outbox_rx.recv().await {
            if let Err(e) = kr_clone.lock().await.route(event, &ctx).await {
                tracing::warn!("mesh outbox routing error: {e}");
            }
        }
    });
}

/// Route a mesh EventEnvelope through the global KernelRouter.
async fn route_mesh_envelope(envelope: runtime::events::EventEnvelope) -> String {
    let Some(kr) = runtime::events::kernel_router() else {
        return "Error: KernelRouter not initialized".to_string();
    };
    let ctx = runtime::events::EmitContext {
        caller_kind: runtime::events::CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    match kr.lock().await.route(envelope.clone(), &ctx).await {
        Ok(()) => format!("ok: routed {}", envelope.event_type),
        Err(e) => format!("Error routing {}: {e}", envelope.event_type),
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
