//! `ship mcp serve` — run the Ship MCP server in-process.
//!
//! stdio mode (default): Claude Code spawns `ship mcp serve` directly.
//! HTTP mode (--http):   a long-running daemon for CI/CD or remote agents.
//!
//! Logging is already initialised by the CLI before this is called;
//! everything writes to ~/.ship/logs/ship.log via the tracing subscriber.

use anyhow::Result;

pub fn run(http: bool, port: u16) -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    if http {
        rt.block_on(mcp::run_http_server(port))
    } else {
        rt.block_on(mcp::run())
    }
}
