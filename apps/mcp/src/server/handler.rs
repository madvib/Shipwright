use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, Content, Implementation,
        ListResourceTemplatesResult, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
        ProtocolVersion, ReadResourceRequestParams, ReadResourceResult, ResourceContents,
        ServerCapabilities, ServerInfo, SubscribeRequestParams, Tool, UnsubscribeRequestParams,
    },
    service::{NotificationContext, RequestContext},
};
use runtime::get_active_agent;
use std::path::Path;

use crate::resource_resolver;
use crate::resources;
use crate::tools::project;

use super::ShipServer;

// ---- Resource resolution ----

impl ShipServer {
    pub async fn resolve_resource_uri(&self, uri: &str, dir: &Path) -> Option<String> {
        resource_resolver::resolve_resource_uri(uri, dir, project::get_project_info(dir)).await
    }
}

// ---- ServerHandler ----

impl ServerHandler for ShipServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: {
                let mut caps = ServerCapabilities::builder()
                    .enable_tools()
                    .enable_resources()
                    .enable_resources_list_changed()
                    .enable_resources_subscribe()
                    .build();
                // Declare claude/channel capability for push notifications.
                // Non-Claude providers ignore experimental capabilities they
                // don't understand — this is safe to declare unconditionally.
                let mut experimental = std::collections::BTreeMap::new();
                experimental.insert(
                    "claude/channel".to_string(),
                    serde_json::Map::new(),
                );
                caps.experimental = Some(experimental);
                caps
            },
            server_info: Implementation {
                name: "Ship Project Tracker".into(),
                version: "0.2.0".into(),
                ..Default::default()
            },
            instructions: Some(
                "Ship project intelligence — three-stage workflow:\n\n\
                 PLANNING: get_project_info → create_adr\n\
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

    async fn on_initialized(&self, context: NotificationContext<RoleServer>) {
        // Capture provider from MCP handshake before moving the peer.
        if let Some(client_info) = context.peer.peer_info() {
            *self.mcp_provider.lock().await = Some(client_info.client_info.name.clone());
        }
        self.store_peer(context.peer).await;

        // Spawn lifecycle tasks in background so on_initialized returns promptly.
        // This unblocks tools/list and other requests while DB/kernel init runs.
        let server = self.clone();
        tokio::spawn(async move {
            server.auto_session_start().await;
            server.spawn_agent_actor().await;
            server.start_event_relay().await;
        });
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
        // Increment tool call count for session analytics.
        if let Some(sid) = self.session_id.lock().await.as_deref() {
            let _ = runtime::db::session_drain::increment_tool_count(sid);
        }
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
        let all_tools = self.tool_router.list_all();
        let visible = if let Ok(project_dir) = self.get_effective_project_dir().await {
            let active_agent = get_active_agent(Some(project_dir.clone())).unwrap_or(None);
            all_tools
                .into_iter()
                .filter(|t| {
                    let n = t.name.as_ref();
                    if Self::is_core_tool(n) {
                        return true;
                    }
                    if Self::is_project_workspace_tool(n) {
                        return true;
                    }
                    active_agent
                        .as_ref()
                        .is_some_and(|m| Self::mode_allows_tool(n, &m.active_tools))
                })
                .collect()
        } else {
            all_tools
                .into_iter()
                .filter(|t| Self::is_core_tool(t.name.as_ref()))
                .collect()
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
        Ok(ListResourcesResult::with_all_items(
            resources::static_resource_list(),
        ))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        Ok(ListResourceTemplatesResult::with_all_items(
            resources::static_resource_template_list(),
        ))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        let Ok(dir) = self.get_effective_project_dir().await else {
            return Err(ErrorData::internal_error("No active project", None));
        };

        // Handle ship://session/* resources — read files from .ship-session/
        if let Some(rel_path) = request.uri.strip_prefix("ship://session/") {
            let session_dir = dir.join(".ship-session");
            let file_path = session_dir.join(rel_path);
            // Prevent path traversal
            if !file_path.starts_with(&session_dir) {
                return Err(ErrorData::invalid_params("Path traversal not allowed", None));
            }
            return match std::fs::read_to_string(&file_path) {
                Ok(content) => Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(content, &request.uri)],
                }),
                Err(_) => Err(ErrorData::resource_not_found(
                    format!("Session file not found: {}", rel_path),
                    None,
                )),
            };
        }

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

    async fn subscribe(
        &self,
        request: SubscribeRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), ErrorData> {
        self.subscriptions.lock().await.insert(request.uri);
        Ok(())
    }

    async fn unsubscribe(
        &self,
        request: UnsubscribeRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), ErrorData> {
        self.subscriptions.lock().await.remove(&request.uri);
        Ok(())
    }
}
