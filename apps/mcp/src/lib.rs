mod requests;
mod server;

use anyhow::Result;
use async_trait::async_trait;

pub use server::{ShipServer, run_server};

pub struct ShipMcpApp;

#[async_trait]
impl mcp_framework::McpApp for ShipMcpApp {
    fn metadata() -> mcp_framework::McpMetadata {
        mcp_framework::McpMetadata::new("ship-mcp", "Ship MCP", env!("SHIP_MCP_VERSION_STRING"))
    }

    fn startup_banner(_metadata: mcp_framework::McpMetadata) -> Option<String> {
        None
    }

    fn shutdown_banner(
        _metadata: mcp_framework::McpMetadata,
        _context: &mcp_framework::McpRunContext,
    ) -> Option<String> {
        None
    }

    async fn serve(_context: &mcp_framework::McpRunContext) -> Result<()> {
        run_server().await
    }
}

pub async fn run() -> Result<()> {
    mcp_framework::run::<ShipMcpApp>().await
}
