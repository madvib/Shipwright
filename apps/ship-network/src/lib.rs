//! Ship Network daemon library.
//!
//! Exposes `run_network` for starting the HTTP/SSE MCP daemon and
//! `network_status` / `network_stop` for CLI subcommands.

mod connections;
mod handler;
mod server;

pub use server::NetworkServer;

use anyhow::{Result, anyhow};
use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serde::Deserialize;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// Start the network daemon and block until shutdown (SIGINT/SIGTERM).
pub async fn run_network(host: String, port: u16) -> Result<()> {
    let global_dir = runtime::project::get_global_dir()?;
    let kernel = runtime::events::init_kernel_router(global_dir.clone())
        .map_err(|e| anyhow!("failed to initialize KernelRouter: {e}"))?;

    connections::spawn_mesh_service(&kernel).await?;

    write_port_file(&global_dir, port)?;
    write_pid_file(&global_dir)?;

    let ct = CancellationToken::new();
    let ct_shutdown = ct.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        ct_shutdown.cancel();
    });

    let kernel_factory = kernel.clone();
    let service: StreamableHttpService<NetworkServer, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(NetworkServer::new(kernel_factory.clone())),
            Default::default(),
            StreamableHttpServerConfig {
                cancellation_token: ct.child_token(),
                sse_retry: None,
                sse_keep_alive: None,
                ..Default::default()
            },
        );

    let app = Router::new()
        .nest_service("/mcp", service)
        .route("/api/mesh/broadcast", axum::routing::post(api_mesh_broadcast))
        .with_state(kernel.clone())
        .layer(axum::middleware::from_fn(cors_middleware));

    let addr = format!("{host}:{port}");
    eprintln!("ship network: listening on http://{addr}/mcp");
    tracing::info!("ship network: listening on http://{addr}/mcp");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow!("failed to bind {addr}: {e}"))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async move { ct.cancelled_owned().await })
        .await
        .map_err(|e| anyhow!("server error: {e}"))?;

    let _ = std::fs::remove_file(global_dir.join("network.port"));
    let _ = std::fs::remove_file(global_dir.join("network.pid"));
    tracing::info!("ship network: shutdown complete");
    Ok(())
}

/// Check whether the network daemon is running by inspecting ~/.ship/network.port.
pub fn network_status() -> Result<()> {
    let global_dir = runtime::project::get_global_dir()?;
    match std::fs::read_to_string(global_dir.join("network.port")) {
        Ok(port) => println!("ship network: running on port {}", port.trim()),
        Err(_) => println!("ship network: not running"),
    }
    Ok(())
}

/// Send SIGTERM to the network daemon via the stored PID file.
pub fn network_stop() -> Result<()> {
    let global_dir = runtime::project::get_global_dir()?;
    let pid_file = global_dir.join("network.pid");
    let pid_str = std::fs::read_to_string(&pid_file)
        .map_err(|_| anyhow!("ship network is not running (no pid file)"))?;
    let pid = pid_str.trim().to_string();
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .args(["-TERM", &pid])
            .status()
            .map_err(|e| anyhow!("failed to send SIGTERM to pid {pid}: {e}"))?;
        println!("ship network: sent SIGTERM to pid {pid}");
    }
    #[cfg(not(unix))]
    anyhow::bail!("ship network stop is not supported on this platform");
    Ok(())
}

// ── REST API ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct MeshBroadcastBody {
    event_type: String,
    from_agent_id: String,
    payload: serde_json::Value,
}

async fn api_mesh_broadcast(
    State(kernel): State<Arc<Mutex<runtime::events::KernelRouter>>>,
    Json(body): Json<MeshBroadcastBody>,
) -> StatusCode {
    let envelope = match runtime::events::EventEnvelope::new(
        &body.event_type,
        &body.from_agent_id,
        &body.payload,
    )
    .map(|e| e.with_actor_id(&body.from_agent_id))
    {
        Ok(e) => e,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    let ctx = runtime::events::EmitContext {
        caller_kind: runtime::events::CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    match kernel.lock().await.route(envelope, &ctx).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn write_port_file(global_dir: &Path, port: u16) -> Result<()> {
    std::fs::write(global_dir.join("network.port"), port.to_string())
        .map_err(|e| anyhow!("failed to write network.port: {e}"))
}

fn write_pid_file(global_dir: &Path) -> Result<()> {
    let pid = std::process::id();
    std::fs::write(global_dir.join("network.pid"), pid.to_string())
        .map_err(|e| anyhow!("failed to write network.pid: {e}"))
}

/// Wait for SIGINT or SIGTERM (UNIX) or just SIGINT (other platforms).
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut sigterm) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = sigterm.recv() => {}
                }
            }
            Err(e) => {
                // SIGTERM handler unavailable — fall back to SIGINT only
                tracing::warn!("could not register SIGTERM handler: {e}; SIGINT only");
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}

async fn cors_middleware(req: Request<Body>, next: Next) -> Response {
    use axum::http::{HeaderValue, Method};
    if req.method() == Method::OPTIONS {
        let mut res = Response::new(Body::empty());
        let h = res.headers_mut();
        h.insert("access-control-allow-origin", HeaderValue::from_static("*"));
        h.insert(
            "access-control-allow-methods",
            HeaderValue::from_static("GET, POST, DELETE, OPTIONS"),
        );
        h.insert(
            "access-control-allow-headers",
            HeaderValue::from_static("content-type, authorization, accept, mcp-session-id"),
        );
        h.insert(
            "access-control-expose-headers",
            HeaderValue::from_static("mcp-session-id"),
        );
        return res;
    }
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    h.insert("access-control-allow-origin", HeaderValue::from_static("*"));
    h.insert(
        "access-control-expose-headers",
        HeaderValue::from_static("mcp-session-id"),
    );
    res
}
