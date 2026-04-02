//! ServerHandler impl for NetworkServer — rmcp connection lifecycle.

use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, Implementation, ListToolsResult,
        PaginatedRequestParams, ProtocolVersion, ServerCapabilities, ServerInfo, Tool,
    },
    service::{NotificationContext, RequestContext},
};

use crate::server::NetworkServer;

impl ServerHandler for NetworkServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "Ship Network".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            instructions: Some(
                "Ship cross-agent communication daemon.\n\n\
                 Call mesh_register first to join the mesh, then use mesh_send, \
                 mesh_broadcast, mesh_discover, and mesh_status to communicate \
                 with other agents. Push notifications arrive as ship/event \
                 custom MCP notifications."
                    .into(),
            ),
        }
    }

    async fn on_initialized(&self, context: NotificationContext<RoleServer>) {
        // Store the MCP peer for push notifications.
        // Actor spawning happens in mesh_register (requires agent_id from caller).
        self.store_peer(context.peer).await;
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        self.tool_router_ref()
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
        Ok(ListToolsResult::with_all_items(
            self.tool_router_ref().list_all(),
        ))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool_router_ref().get(name).cloned()
    }
}
