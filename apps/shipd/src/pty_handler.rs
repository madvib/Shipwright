//! PTY WebSocket endpoint — bridges a tmux attach-session process to a WS client.
//!
//! Route: GET /api/runtime/workspaces/:id/pty
//! Upgrades to WebSocket, then spawns `tmux attach-session -t <name>` with stdin/stdout
//! bridged to the WS frame stream. Multiple clients may attach concurrently (tmux handles
//! multiplexing). Limit: 5 concurrent attachments per workspace.

use axum::{
    Json,
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        FromRequestParts, Path, Request, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::{Duration, timeout};

use crate::rest_api::ApiState;

/// Maximum concurrent tmux attach processes per workspace.
const MAX_PTY_CONNECTIONS: usize = 5;

/// GET /api/runtime/workspaces/:id/pty
///
/// Upgrades to WebSocket or returns 404/429 before the upgrade.
/// WebSocketUpgrade is extracted manually after validation so tests can reach
/// the 404/429 paths without a live HTTP connection (which axum requires for
/// upgrade state — without it the extractor returns 426 before the handler runs).
pub async fn workspace_pty(
    Path(workspace_id): Path<String>,
    State(state): State<ApiState>,
    request: Request,
) -> Response {
    // Resolve workspace and tmux session name.
    let ship_dir = match runtime::project::get_global_dir() {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let workspace = match runtime::workspace::get_workspace_by_id(&ship_dir, &workspace_id) {
        Ok(Some(w)) => w,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "workspace not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let session_name = match workspace.tmux_session_name.as_deref() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "no tmux session for this workspace"})),
            )
                .into_response();
        }
    };

    // Enforce connection limit.
    {
        let mut conns = state.pty_connections.lock().await;
        let count = conns.entry(workspace_id.clone()).or_insert(0);
        if *count >= MAX_PTY_CONNECTIONS {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({"error": "too many concurrent terminal connections for this workspace"})),
            )
                .into_response();
        }
        *count += 1;
    }

    let pty_connections = state.pty_connections.clone();
    let ws_id = workspace_id.clone();

    // Extract WebSocketUpgrade now that validation has passed.
    let (mut parts, body) = request.into_parts();
    let ws = match WebSocketUpgrade::from_request_parts(&mut parts, &()).await {
        Ok(ws) => ws,
        Err(e) => return e.into_response(),
    };
    drop(body);

    ws.on_upgrade(move |socket| async move {
        handle_pty_socket(socket, session_name, pty_connections, ws_id).await;
    })
    .into_response()
}

async fn handle_pty_socket(
    mut socket: WebSocket,
    session_name: String,
    pty_connections: Arc<Mutex<std::collections::HashMap<String, usize>>>,
    workspace_id: String,
) {
    let _guard = PtyConnectionGuard {
        pty_connections: pty_connections.clone(),
        workspace_id: workspace_id.clone(),
    };

    // Spawn `tmux attach-session -t <name>` with piped stdin/stdout.
    // script(1) allocates a PTY so tmux doesn't complain about not being attached to a terminal.
    let mut child = match Command::new("script")
        .args(["-q", "-c", &format!("tmux attach-session -t {session_name}"), "/dev/null"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = socket
                .send(axum::extract::ws::Message::Text(
                    json!({"error": format!("failed to spawn tmux: {e}")}).to_string().into(),
                ))
                .await;
            return;
        }
    };

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    let mut stdout_buf = vec![0u8; 4096];
    let ping_interval = Duration::from_secs(30);
    let pong_timeout = Duration::from_secs(10);
    let mut waiting_for_pong = false;

    loop {
        tokio::select! {
            // Read output from the process and forward to WS.
            n = stdout.read(&mut stdout_buf) => {
                match n {
                    Ok(0) | Err(_) => {
                        // Process exited.
                        let _ = socket
                            .send(axum::extract::ws::Message::Close(Some(axum::extract::ws::CloseFrame {
                                code: 1000,
                                reason: "session ended".into(),
                            })))
                            .await;
                        break;
                    }
                    Ok(n) => {
                        let data = stdout_buf[..n].to_vec();
                        if socket
                            .send(axum::extract::ws::Message::Binary(data.into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }

            // Read WS messages and forward to process stdin.
            msg = timeout(ping_interval, socket.recv()) => {
                match msg {
                    // Ping timeout — send a ping and wait for pong.
                    Err(_elapsed) => {
                        if waiting_for_pong {
                            // No pong received — close.
                            break;
                        }
                        waiting_for_pong = true;
                        if socket
                            .send(axum::extract::ws::Message::Ping(vec![].into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                        // Give the client pong_timeout to respond.
                        match timeout(pong_timeout, socket.recv()).await {
                            Ok(Some(Ok(axum::extract::ws::Message::Pong(_)))) => {
                                waiting_for_pong = false;
                            }
                            _ => break,
                        }
                    }
                    Ok(None) | Ok(Some(Err(_))) => break,
                    Ok(Some(Ok(msg))) => {
                        waiting_for_pong = false;
                        match msg {
                            axum::extract::ws::Message::Binary(data) => {
                                if stdin.write_all(&data).await.is_err() {
                                    break;
                                }
                            }
                            axum::extract::ws::Message::Text(text) => {
                                if stdin.write_all(text.as_bytes()).await.is_err() {
                                    break;
                                }
                            }
                            axum::extract::ws::Message::Pong(_) => {
                                waiting_for_pong = false;
                            }
                            axum::extract::ws::Message::Close(_) => break,
                            axum::extract::ws::Message::Ping(data) => {
                                let _ = socket
                                    .send(axum::extract::ws::Message::Pong(data))
                                    .await;
                            }
                        }
                    }
                }
            }
        }
    }

    // Kill the attach process (not the tmux session).
    let _ = child.kill().await;
}

/// Decrements the connection counter when dropped.
struct PtyConnectionGuard {
    pty_connections: Arc<Mutex<std::collections::HashMap<String, usize>>>,
    workspace_id: String,
}

impl Drop for PtyConnectionGuard {
    fn drop(&mut self) {
        let pty_connections = self.pty_connections.clone();
        let workspace_id = self.workspace_id.clone();
        tokio::spawn(async move {
            let mut conns = pty_connections.lock().await;
            if let Some(count) = conns.get_mut(&workspace_id) {
                if *count > 0 {
                    *count -= 1;
                }
                if *count == 0 {
                    conns.remove(&workspace_id);
                }
            }
        });
    }
}

#[cfg(test)]
#[path = "pty_handler_tests.rs"]
mod tests;
