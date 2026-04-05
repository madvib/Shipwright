//! Ship daemon (shipd) library.
//!
//! Exposes `run_network` for starting the HTTP/SSE MCP daemon and
//! `network_status` / `network_stop` for CLI subcommands.

pub mod actor_api;
mod connections;
mod handler;
pub mod human_webhook;
pub mod project_api;
pub mod pty_handler;
pub mod rest_api;
pub mod runtime_api;
mod server;
pub mod supervisor;

pub use server::NetworkServer;

use anyhow::{Result, anyhow};
use axum::{
    Router,
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use std::path::Path;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Start the network daemon and block until shutdown (SIGINT/SIGTERM).
pub async fn run_network(host: String, port: u16) -> Result<()> {
    let global_dir = runtime::project::get_global_dir()?;
    let kernel = runtime::events::init_kernel_router(global_dir.clone())
        .map_err(|e| anyhow!("failed to initialize KernelRouter: {e}"))?;

    let mesh_registry = connections::spawn_mesh_service(&kernel).await?;
    supervisor::subscribe_workspace_events(kernel.clone(), global_dir.clone()).await;
    supervisor::job_dispatch::subscribe_job_events(kernel.clone(), global_dir.clone()).await;
    spawn_human_gateway(&kernel).await;

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

    let api_state = rest_api::ApiState {
        kernel: kernel.clone(),
        mesh_registry: mesh_registry.clone(),
        agent_mailboxes: std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
        pty_connections: std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
    };
    let webhook_routes = axum::Router::new()
        .route(
            "/webhook/telegram",
            axum::routing::post(human_webhook::telegram_webhook),
        )
        .with_state(kernel.clone());

    let hook_routes = axum::Router::new()
        .route(
            "/hooks/checkout",
            axum::routing::post(rest_api::hooks_checkout),
        );

    let api_routes = axum::Router::new()
        .route("/mesh/register", axum::routing::post(rest_api::mesh_register))
        .route("/mesh/send", axum::routing::post(rest_api::mesh_send))
        .route("/mesh/broadcast", axum::routing::post(rest_api::mesh_broadcast))
        .route("/mesh/discover", axum::routing::get(rest_api::mesh_discover))
        .route("/mesh/status", axum::routing::post(rest_api::mesh_status_update))
        .route("/mesh/events/{agent_id}", axum::routing::get(rest_api::mesh_events))
        .route("/actor/spawn", axum::routing::post(actor_api::actor_spawn))
        .route("/events/route", axum::routing::post(actor_api::event_route))
        .with_state(api_state.clone());

    let runtime_routes = axum::Router::new()
        .route("/workspaces", axum::routing::get(runtime_api::list_workspaces))
        .route("/sessions", axum::routing::get(runtime_api::list_sessions))
        .route("/agents", axum::routing::get(runtime_api::list_agents))
        .route("/events", axum::routing::get(runtime_api::event_stream))
        .route(
            "/workspaces/{id}/pty",
            axum::routing::get(pty_handler::workspace_pty),
        )
        .with_state(api_state.clone());

    let supervisor_routes = axum::Router::new()
        .route(
            "/supervisor/workspaces/{id}/start",
            axum::routing::post(supervisor::start_workspace),
        )
        .with_state(api_state.clone());

    let project_routes = axum::Router::new()
        .route(
            "/workspaces/{id}/session-files",
            axum::routing::get(project_api::list_session_files),
        )
        .route(
            "/workspaces/{id}/session-files/{*path}",
            axum::routing::get(project_api::read_session_file)
                .put(project_api::write_session_file)
                .delete(project_api::delete_session_file),
        )
        .route(
            "/workspaces/{id}/git/status",
            axum::routing::get(project_api::git_status),
        )
        .route(
            "/workspaces/{id}/git/diff",
            axum::routing::get(project_api::git_diff),
        )
        .route(
            "/workspaces/{id}/git/log",
            axum::routing::get(project_api::git_log),
        )
        .route(
            "/workspaces/{id}/agents",
            axum::routing::get(project_api::list_agents),
        )
        .route(
            "/workspaces/{id}/skills",
            axum::routing::get(project_api::list_skills),
        )
        .route(
            "/workspaces/{id}",
            axum::routing::delete(project_api::delete_workspace),
        )
        .route(
            "/workspaces/{id}/activate",
            axum::routing::post(project_api::activate_workspace),
        )
        .route(
            "/events/emit",
            axum::routing::post(project_api::emit_event),
        )
        .with_state(api_state);

    let app = Router::new()
        .nest("/api", api_routes)
        .nest("/api", hook_routes)
        .nest("/api", webhook_routes)
        .nest("/api/runtime", runtime_routes)
        .nest("/api", supervisor_routes)
        .nest("/api", project_routes)
        .nest_service("/mcp", service)
        .layer(axum::middleware::from_fn(cors_middleware));

    let addr = format!("{host}:{port}");
    eprintln!("shipd: listening on http://{addr}/mcp");
    tracing::info!("shipd: listening on http://{addr}/mcp");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow!("failed to bind {addr}: {e}"))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async move { ct.cancelled_owned().await })
        .await
        .map_err(|e| anyhow!("server error: {e}"))?;

    let _ = std::fs::remove_file(global_dir.join("network.port"));
    let _ = std::fs::remove_file(global_dir.join("network.pid"));
    tracing::info!("shipd: shutdown complete");
    Ok(())
}

/// Check whether the network daemon is running by inspecting ~/.ship/network.port.
pub fn network_status() -> Result<()> {
    let global_dir = runtime::project::get_global_dir()?;
    match std::fs::read_to_string(global_dir.join("network.port")) {
        Ok(port) => println!("shipd: running on port {}", port.trim()),
        Err(_) => println!("shipd: not running"),
    }
    Ok(())
}

/// Send SIGTERM to the network daemon via the stored PID file.
pub fn network_stop() -> Result<()> {
    let global_dir = runtime::project::get_global_dir()?;
    let pid_file = global_dir.join("network.pid");
    let pid_str = std::fs::read_to_string(&pid_file)
        .map_err(|_| anyhow!("shipd is not running (no pid file)"))?;
    let pid = pid_str.trim().to_string();
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .args(["-TERM", &pid])
            .status()
            .map_err(|e| anyhow!("failed to send SIGTERM to pid {pid}: {e}"))?;
        println!("shipd: sent SIGTERM to pid {pid}");
    }
    #[cfg(not(unix))]
    anyhow::bail!("shipd stop is not supported on this platform");
    Ok(())
}

/// Spawn `HumanGatewayService` if Telegram env vars are present.
/// Silently skips if `SHIP_TELEGRAM_TOKEN` or `SHIP_TELEGRAM_CHAT_ID` is absent.
async fn spawn_human_gateway(
    kernel: &Arc<tokio::sync::Mutex<runtime::events::KernelRouter>>,
) {
    use runtime::services::human_gateway::{HumanGatewayService, TelegramAdapter};
    use runtime::services::spawn_service;
    use runtime::events::ActorConfig;
    use std::sync::Arc as StdArc;

    let adapter = match TelegramAdapter::from_env() {
        Some(a) => a,
        None => {
            tracing::info!("shipd: SHIP_TELEGRAM_TOKEN/CHAT_ID not set — human gateway disabled");
            return;
        }
    };

    let (outbox_tx, mut outbox_rx) =
        tokio::sync::mpsc::unbounded_channel::<runtime::events::EventEnvelope>();

    let config = ActorConfig {
        namespace: "service.human-gateway".to_string(),
        write_namespaces: vec!["human.".to_string()],
        read_namespaces: vec!["job.".to_string()],
        subscribe_namespaces: vec!["job.".to_string()],
    };

    let handler: Box<dyn runtime::services::ServiceHandler> = Box::new(
        HumanGatewayService::new(StdArc::new(adapter), outbox_tx),
    );

    match spawn_service(&mut *kernel.lock().await, "service.human-gateway", config, handler) {
        Ok(_) => tracing::info!("shipd: human gateway service started (telegram)"),
        Err(e) => {
            tracing::warn!("shipd: failed to spawn human gateway: {e}");
            return;
        }
    }

    // Drain outbox → kernel (currently unused, reserved for future adapter replies)
    let kr = kernel.clone();
    tokio::spawn(async move {
        let ctx = runtime::events::EmitContext {
            caller_kind: runtime::events::CallerKind::Mcp,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        };
        while let Some(event) = outbox_rx.recv().await {
            if let Err(e) = kr.lock().await.route(event, &ctx).await {
                tracing::warn!("human-gateway outbox routing error: {e}");
            }
        }
    });
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

/// Allowed origins for CORS. Localhost ports (dev) and getship.dev (production).
/// Wildcard is intentionally excluded — only known clients may connect.
const ALLOWED_ORIGINS: &[&str] = &[
    "https://getship.dev",
    "http://localhost:3000",
    "http://localhost:5173",
    "http://localhost:4321",
];

async fn cors_middleware(req: Request<Body>, next: Next) -> Response {
    use axum::http::{HeaderValue, Method};

    let origin = req
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let allowed = ALLOWED_ORIGINS.contains(&origin)
        || origin.starts_with("http://localhost:")
        || origin.starts_with("http://127.0.0.1:");

    if !allowed && !origin.is_empty() {
        tracing::warn!("shipd: rejected CORS request from origin: {origin}");
    }

    let origin_header = if allowed && !origin.is_empty() {
        origin.to_string()
    } else {
        // Non-browser clients (curl, CLI) send no Origin — allow through without ACAO header.
        // Browsers with a disallowed origin are rejected by the browser itself.
        String::new()
    };

    if req.method() == Method::OPTIONS {
        let mut res = Response::new(Body::empty());
        let h = res.headers_mut();
        if !origin_header.is_empty() {
            h.insert(
                "access-control-allow-origin",
                HeaderValue::from_str(&origin_header).unwrap(),
            );
        }
        h.insert(
            "access-control-allow-methods",
            HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
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
    if !origin_header.is_empty() {
        h.insert(
            "access-control-allow-origin",
            HeaderValue::from_str(&origin_header).unwrap(),
        );
    }
    h.insert(
        "access-control-expose-headers",
        HeaderValue::from_static("mcp-session-id"),
    );
    res
}
