use rmcp::{
    ErrorData, Peer, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolRequestParams, CallToolResult, Implementation, ListToolsResult,
        PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
    },
    service::{NotificationContext, RequestContext},
    tool, tool_router,
};
use std::path::PathBuf;

use crate::requests::*;
use crate::tools::{git_info, project, session_files, skills, studio, studio_inbox};
use skills::{
    delete_skill_file, get_skill_vars_tool, list_project_skills, list_skill_vars_tool,
    set_skill_var_tool, write_skill_file,
};

// ---- Server struct ----

#[derive(Clone)]
pub struct StudioServer {
    tool_router: ToolRouter<Self>,
    pub active_project: std::sync::Arc<tokio::sync::Mutex<Option<PathBuf>>>,
    pub notification_peer: std::sync::Arc<tokio::sync::Mutex<Option<Peer<RoleServer>>>>,
    /// Actor-scoped event store for the studio actor.
    pub actor_store: std::sync::Arc<tokio::sync::Mutex<Option<runtime::events::ActorStore>>>,
    /// Mailbox for receiving cross-actor events (e.g. agent → studio).
    pub actor_mailbox: std::sync::Arc<tokio::sync::Mutex<Option<runtime::events::Mailbox>>>,
}

impl std::fmt::Debug for StudioServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StudioServer").finish_non_exhaustive()
    }
}

// ---- Studio tool registration ----

#[tool_router]
impl StudioServer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // Detect project from CWD at startup so tools work immediately
        let project_dir = runtime::project::get_project_dir(None)
            .ok()
            .map(|ship_dir| {
                // get_project_dir returns the .ship dir — resolve to project root
                if ship_dir.file_name().and_then(|n| n.to_str()) == Some(".ship") {
                    ship_dir.parent().unwrap_or(&ship_dir).to_path_buf()
                } else {
                    ship_dir
                }
            });
        if let Some(ref dir) = project_dir {
            tracing::info!("ship studio: detected project at {}", dir.display());
        } else {
            tracing::warn!("ship studio: no project detected from CWD — tools will require open_project");
        }
        Self {
            tool_router: Self::tool_router(),
            active_project: std::sync::Arc::new(tokio::sync::Mutex::new(project_dir)),
            notification_peer: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
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

    /// Spawn the studio actor via the global KernelRouter.
    ///
    /// Called from `on_initialized`. The studio actor writes `studio.*` events
    /// and subscribes to `studio.*` and `agent.*` for cross-actor delivery.
    pub async fn spawn_studio_actor(&self) {
        let ship_dir = match dirs::home_dir() {
            Some(h) => h.join(".ship"),
            None => {
                tracing::warn!("studio: cannot resolve home directory");
                return;
            }
        };

        let global_dir = match runtime::project::get_global_dir() {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("studio: failed to resolve global dir: {e}");
                return;
            }
        };
        let kr = match runtime::events::init_kernel_router(global_dir) {
            Ok(kr) => kr,
            Err(e) => {
                tracing::warn!("studio: failed to initialize KernelRouter: {e}");
                return;
            }
        };

        // Subscribe to all skill custom namespaces so Studio receives agent-emitted skill events.
        let mut subscribe_namespaces = vec!["studio.".to_string(), "agent.".to_string()];
        if let Ok(skills) = runtime::list_skills(&ship_dir) {
            for ns in runtime::events::artifact_events::skill_custom_namespaces(&skills) {
                if !subscribe_namespaces.contains(&ns) {
                    subscribe_namespaces.push(ns);
                }
            }
        }

        let config = runtime::events::ActorConfig {
            namespace: "studio".to_string(),
            write_namespaces: vec!["studio.".to_string()],
            read_namespaces: vec!["studio.".to_string()],
            subscribe_namespaces,
        };

        let mut kr_guard = kr.lock().await;
        // On reconnect the actor already exists — stop it so the old relay's
        // mailbox sender is dropped (relay task exits) then respawn fresh.
        let _ = kr_guard.stop_actor("studio");
        match kr_guard.spawn_actor("studio", config) {
            Ok((store, mailbox)) => {
                *self.actor_store.lock().await = Some(store);
                *self.actor_mailbox.lock().await = Some(mailbox);
            }
            Err(e) => {
                tracing::warn!("studio: failed to spawn actor: {e}");
            }
        }
    }

    /// Start a relay task that forwards cross-actor events (from the studio
    /// mailbox) to the connected Studio SSE peer. Call once after
    /// `spawn_studio_actor` and `store_peer`.
    pub async fn start_event_relay(&self) {
        let mailbox = self.actor_mailbox.lock().await.take();
        let Some(mailbox) = mailbox else { return };

        let peer = self.notification_peer.lock().await.clone();
        let Some(peer) = peer else { return };

        let adapter = crate::push::mcp_notification::McpNotificationAdapter::new(peer);
        let relay = crate::server::notification_relay::EventRelay::new();
        let peer_handle = crate::server::notification_relay::PeerHandle {
            id: "studio-relay".to_string(),
            actor_id: "studio".to_string(),
            adapter: Box::new(adapter),
            allowed_events: std::collections::HashSet::new(), // receive all
        };
        relay.add_peer(peer_handle).await;
        let _handle = relay.spawn(mailbox);
    }

    async fn notify_resources_changed(&self) {
        if let Some(peer) = self.notification_peer.lock().await.as_ref() {
            let _ = peer.notify_resource_list_changed().await;
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

    // ---- Studio sync ----

    #[tool(description = "Pull all local agents with resolved skills, rules, and MCP configs.")]
    async fn pull_agents(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        studio::pull_agents(&project_dir)
    }

    #[tool(description = "List agent profile IDs that exist locally in .ship/agents/.")]
    async fn list_local_agents(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        studio::list_local_agents(&project_dir)
    }

    #[tool(description = "Receive an agent config bundle from Studio and write it to .ship/.")]
    async fn push_bundle(&self, Parameters(req): Parameters<PushBundleRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let result = studio::push_bundle(&project_dir, &req.bundle);
        if !result.starts_with("Error") {
            self.notify_resources_changed().await;
        }
        result
    }

    // ---- Skills ----

    #[tool(description = "List all skills in .ship/skills/ with full resolved content.")]
    async fn list_project_skills(
        &self,
        Parameters(req): Parameters<ListProjectSkillsRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        list_project_skills(&project_dir, req)
    }

    #[tool(description = "List skills available to the active project. Optionally filter by query.")]
    async fn list_skills(&self, Parameters(req): Parameters<ListSkillsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        skills::list_skills(&project_dir, req)
    }

    #[tool(description = "Write a file into a skill directory (.ship/skills/<skill_id>/<path>).")]
    async fn write_skill_file(&self, Parameters(req): Parameters<WriteSkillFileRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let result = write_skill_file(&project_dir, req);
        if !result.starts_with("Error") {
            self.notify_resources_changed().await;
        }
        result
    }

    #[tool(description = "Delete a file from a skill directory. Refuses to delete SKILL.md.")]
    async fn delete_skill_file(
        &self,
        Parameters(req): Parameters<DeleteSkillFileRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let result = delete_skill_file(&project_dir, req);
        if !result.starts_with("Error") {
            self.notify_resources_changed().await;
        }
        result
    }

    // ---- Vars ----

    #[tool(description = "Get the merged variable state for a skill (defaults + user + project).")]
    async fn get_skill_vars(&self, Parameters(req): Parameters<GetSkillVarsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        get_skill_vars_tool(&project_dir, req)
    }

    #[tool(description = "Set a skill variable value. The variable must be declared in vars.json.")]
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

    #[tool(description = "List skills with configurable variables. Optionally filter by skill_id.")]
    async fn list_skill_vars(&self, Parameters(req): Parameters<ListSkillVarsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        list_skill_vars_tool(&project_dir, req)
    }

    // ---- Session Files ----

    #[tool(description = "List all files in .ship-session/ with path, size, modified, and type.")]
    async fn list_session_files(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        session_files::list_session_files(&project_dir)
    }

    #[tool(description = "Read a file from .ship-session/. Returns data URI for images.")]
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

    #[tool(description = "Write a file to .ship-session/. Creates parent directories as needed.")]
    async fn write_session_file(
        &self,
        Parameters(req): Parameters<WriteSessionFileRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        session_files::write_session_file(&project_dir, req)
    }

    #[tool(description = "Delete a file from .ship-session/.")]
    async fn delete_session_file(&self, Parameters(req): Parameters<ReadSessionFileRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        session_files::delete_session_file(&project_dir, &req.path)
    }

    // ---- Git info ----

    #[tool(description = "Get current branch, clean/dirty status, and a summary of changes.")]
    async fn get_git_status(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        git_info::get_git_status(&project_dir)
    }

    #[tool(description = "Get a unified diff. Defaults to unstaged changes against HEAD.")]
    async fn get_git_diff(&self, Parameters(req): Parameters<GetGitDiffRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        git_info::get_git_diff(&project_dir, req)
    }

    #[tool(description = "Get recent git commits with hash, message, author, date, and files changed.")]
    async fn get_git_log(&self, Parameters(req): Parameters<GetGitLogRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        git_info::get_git_log(&project_dir, req)
    }

    #[tool(description = "List active git worktrees with path, branch, and HEAD commit.")]
    async fn list_worktrees(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        git_info::list_worktrees(&project_dir)
    }

    // ---- Studio Events ----

    #[tool(
        description = "Emit a Studio event into the workspace event bus. \
        event_type must start with 'studio.' (e.g. 'studio.message.visual'). \
        actor is always 'studio' — not agent-controlled. \
        Payload must be self-contained: agents receive it directly with no follow-up queries. \
        Returns the persisted event id."
    )]
    async fn emit_studio_event(
        &self,
        Parameters(req): Parameters<EmitStudioEventRequest>,
    ) -> String {
        if !req.event_type.starts_with("studio.") {
            return format!(
                "Error: event_type '{}' must start with 'studio.' — \
                 only studio.* events are accepted here",
                req.event_type
            );
        }
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let workspace_id = match current_git_branch(&project_dir) {
            Ok(b) => b,
            Err(e) => return format!("Error resolving workspace: {}", e),
        };
        let mut envelope = match runtime::events::EventEnvelope::new(
            &req.event_type,
            &workspace_id,
            &req.payload,
        ) {
            Ok(e) => e,
            Err(e) => return format!("Error building event: {}", e),
        };
        envelope.actor = "studio".to_string();
        let envelope = envelope
            .with_actor_id("studio")
            .with_context(Some(&workspace_id), None);

        // Persist to Studio's actor-scoped store.
        {
            let store_guard = self.actor_store.lock().await;
            let Some(ref store) = *store_guard else {
                return "Error: studio actor not initialized — ensure on_initialized completed"
                    .to_string();
            };
            if let Err(e) = store.append(&envelope) {
                return format!("Error persisting studio event: {}", e);
            }
        }

        // Route via KernelRouter — delivers to agent mailboxes that subscribe to "studio."
        let ctx = runtime::events::EmitContext {
            caller_kind: runtime::events::CallerKind::Mcp,
            skill_id: None,
            workspace_id: Some(workspace_id.clone()),
            session_id: None,
        };
        let Some(kr) = runtime::events::kernel_router() else {
            return "Error: KernelRouter not initialized".to_string();
        };
        if let Err(e) = kr.lock().await.route(envelope.clone(), &ctx).await {
            return format!("Error routing studio event: {}", e);
        }

        // Write to .ship-session/inbox/ so connected Claude Code sessions can
        // read the event via read_session_file.  When the active workspace
        // lives in a git worktree the inbox must land in that worktree's
        // .ship-session/, not the main project root.  Failure is non-fatal.
        // target_workspace_id overrides the caller's workspace — used by agents
        // (e.g. gate) that need to notify a different session (e.g. commander).
        let inbox_workspace = req.target_workspace_id.as_deref().unwrap_or(&workspace_id);
        let inbox_root = resolve_inbox_root(&project_dir, inbox_workspace);
        match studio_inbox::write_inbox_file(
            &inbox_root,
            &req.event_type,
            &req.payload,
            &envelope.id,
        ) {
            Ok(_) => {
                self.notify_resources_changed().await;
            }
            Err(e) => {
                tracing::warn!("studio: inbox write failed (non-fatal): {e}");
            }
        }

        format!("{{\"id\":\"{}\"}}", envelope.id)
    }
}

// ---- Helpers ----

/// Resolve the directory that should contain `.ship-session/inbox/` for the
/// given workspace.
///
/// If the workspace record has a non-null `worktree_path`, the agent running
/// in that worktree reads its inbox from that path.  Otherwise the main
/// project directory is the workspace root.
///
/// Lookup failure (workspace not found, DB error) is non-fatal: we fall back
/// to `project_dir` so that inbox writes always have a valid destination.
fn resolve_inbox_root(project_dir: &std::path::Path, workspace_id: &str) -> std::path::PathBuf {
    let ship_dir = project_dir.join(".ship");
    match runtime::get_workspace(&ship_dir, workspace_id) {
        Ok(Some(ws)) => {
            if let Some(ref wt_path) = ws.worktree_path {
                let p = std::path::PathBuf::from(wt_path);
                if p.is_dir() {
                    return p;
                }
                tracing::warn!(
                    "studio: worktree_path '{}' for workspace '{}' does not exist — \
                     falling back to project_dir",
                    wt_path,
                    workspace_id
                );
            }
            project_dir.to_path_buf()
        }
        Ok(None) => {
            tracing::debug!(
                "studio: workspace '{}' not found in DB — using project_dir as inbox root",
                workspace_id
            );
            project_dir.to_path_buf()
        }
        Err(e) => {
            tracing::warn!(
                "studio: workspace lookup failed for '{}': {} — using project_dir as inbox root",
                workspace_id,
                e
            );
            project_dir.to_path_buf()
        }
    }
}

fn current_git_branch(project_dir: &std::path::Path) -> anyhow::Result<String> {
    let root = project_dir;
    let out = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(root)
        .output()?;
    anyhow::ensure!(out.status.success(), "git branch --show-current failed");
    let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
    anyhow::ensure!(!branch.is_empty(), "HEAD is detached — cannot resolve workspace");
    Ok(branch)
}

// ---- ServerHandler ----

impl ServerHandler for StudioServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "Ship Studio".into(),
                version: "0.2.0".into(),
                ..Default::default()
            },
            instructions: Some(
                "Ship Studio MCP server -- visual IDE for agents and skills.\n\n\
                 All tools are always available. No tool gating is applied.\n\
                 Call open_project first if the project is not auto-detected."
                    .into(),
            ),
        }
    }

    async fn on_initialized(&self, context: NotificationContext<RoleServer>) {
        self.store_peer(context.peer).await;
        self.spawn_studio_actor().await;
        self.start_event_relay().await;
    }

    /// Studio server has no tool gate -- all registered tools are always callable.
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        self.tool_router
            .call(rmcp::handler::server::tool::ToolCallContext::new(
                self, request, context,
            ))
            .await
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(self.tool_router.list_all()))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool_router.get(name).cloned()
    }
}

// ---- Tests ----

#[cfg(test)]
#[path = "studio_server_tests.rs"]
mod studio_server_tests;

#[cfg(test)]
#[path = "inbox_routing_tests.rs"]
mod inbox_routing_tests;
