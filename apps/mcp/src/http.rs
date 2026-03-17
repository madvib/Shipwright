use anyhow::{Result, anyhow};
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode, header::AUTHORIZATION},
    middleware::{self, Next},
    response::Response,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
    session::local::LocalSessionManager,
};
use std::path::Path;
use tokio_util::sync::CancellationToken;

use crate::server::ShipServer;

/// Read the bearer token from a given config.toml path.
/// Returns None if the file is absent or has no [auth] token.
pub fn read_token_from_path(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let table: toml::Value = toml::from_str(&content).ok()?;
    table
        .get("auth")?
        .get("token")?
        .as_str()
        .map(|s| s.to_string())
}

/// Read the bearer token from ~/.ship/config.toml.
pub fn read_auth_token() -> Result<Option<String>> {
    let global_dir = runtime::project::get_global_dir()?;
    let config_path = global_dir.join("config.toml");
    Ok(read_token_from_path(&config_path))
}

async fn bearer_auth(
    State(expected): State<String>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(val) if val == format!("Bearer {}", expected) => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Start the HTTP MCP server on the given port.
pub async fn run_http_server(port: u16) -> Result<()> {
    let token = read_auth_token()?;

    let ct = CancellationToken::new();
    let service: StreamableHttpService<ShipServer, LocalSessionManager> =
        StreamableHttpService::new(
            || Ok(ShipServer::new()),
            Default::default(),
            StreamableHttpServerConfig {
                cancellation_token: ct.child_token(),
                ..Default::default()
            },
        );

    let mcp_router = Router::new().nest_service("/mcp", service);

    let app = if let Some(tok) = token {
        eprintln!("ship-mcp HTTP: bearer token auth enabled");
        mcp_router.route_layer(middleware::from_fn_with_state(tok, bearer_auth))
    } else {
        eprintln!(
            "ship-mcp HTTP: WARNING — no [auth] token in ~/.ship/config.toml, server is unauthenticated"
        );
        mcp_router
    };

    let addr = format!("0.0.0.0:{port}");
    eprintln!("ship-mcp HTTP server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move { ct.cancelled_owned().await })
        .await
        .map_err(|e| anyhow!("HTTP server error: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn reads_token_from_config_toml() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        let mut f = std::fs::File::create(&config_path).unwrap();
        writeln!(f, "[auth]\ntoken = \"secret123\"").unwrap();
        let token = read_token_from_path(&config_path).unwrap();
        assert_eq!(token, "secret123");
    }

    #[test]
    fn missing_config_returns_none() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        let token = read_token_from_path(&config_path);
        assert!(token.is_none());
    }

    #[test]
    fn config_without_auth_section_returns_none() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        let mut f = std::fs::File::create(&config_path).unwrap();
        writeln!(f, "[other]\nkey = \"value\"").unwrap();
        let token = read_token_from_path(&config_path);
        assert!(token.is_none());
    }
}
