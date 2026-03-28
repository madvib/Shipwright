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
use crate::tools::{project, session_files, skills, studio};
use skills::{
    delete_skill_file, get_skill_vars_tool, list_project_skills, list_skill_vars_tool,
    set_skill_var_tool, write_skill_file,
};

// ---- Server struct ----

#[derive(Debug, Clone)]
pub struct StudioServer {
    tool_router: ToolRouter<Self>,
    pub active_project: std::sync::Arc<tokio::sync::Mutex<Option<PathBuf>>>,
    pub notification_peer: std::sync::Arc<tokio::sync::Mutex<Option<Peer<RoleServer>>>>,
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
        }
    }

    async fn get_effective_project_dir(&self) -> Result<PathBuf, String> {
        project::get_effective_project_dir(&self.active_project).await
    }

    pub async fn store_peer(&self, peer: Peer<RoleServer>) {
        *self.notification_peer.lock().await = Some(peer);
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
