pub mod http;
mod requests;
mod resource_resolver;
mod resources;
mod server;
pub mod studio_server;
mod tools;
mod util;

use anyhow::Result;
use async_trait::async_trait;

pub use http::{run_http_server, run_studio_http_server};
pub use server::{ShipServer, run_server};
pub use studio_server::StudioServer;

pub struct ShipMcpApp;

#[async_trait]
impl mcp_framework::McpApp for ShipMcpApp {
    fn metadata() -> mcp_framework::McpMetadata {
        mcp_framework::McpMetadata::new("ship", "Ship MCP", env!("SHIP_MCP_VERSION_STRING"))
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

#[cfg(test)]
mod resource_tests;

#[cfg(test)]
mod project_skills_tests;
#[cfg(test)]
mod skill_file_tests;
