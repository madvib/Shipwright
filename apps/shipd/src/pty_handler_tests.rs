use super::{workspace_pty, MAX_PTY_CONNECTIONS};
use crate::rest_api::ApiState;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

fn make_api_state() -> ApiState {
    let kernel = {
        let tmp = std::env::temp_dir().join(format!("ship-pty-test-{}", std::process::id()));
        let ship_dir = tmp.join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        unsafe { std::env::set_var("SHIP_GLOBAL_DIR", &ship_dir) };
        runtime::events::init_kernel_router(ship_dir).unwrap()
    };
    let mesh_registry = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    ApiState {
        kernel,
        mesh_registry,
        agent_mailboxes: Arc::new(Mutex::new(HashMap::new())),
        pty_connections: Arc::new(Mutex::new(HashMap::new())),
    }
}

/// Set up the global DB via init_project on a temp project dir.
fn setup_db() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    unsafe { std::env::set_var("SHIP_GLOBAL_DIR", &ship_dir) };
    runtime::project::init_project(tmp.path().to_path_buf()).unwrap();
    runtime::db::ensure_db().unwrap();
    tmp
}

#[tokio::test(flavor = "multi_thread")]
async fn pty_returns_404_for_unknown_workspace() {
    use http_body_util::BodyExt;
    let _tmp = setup_db();
    let state = make_api_state();

    let req = axum::http::Request::builder()
        .uri("/api/runtime/workspaces/nonexistent/pty")
        .header("host", "localhost")
        .header("connection", "upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .body(axum::body::Body::empty())
        .unwrap();

    let app = axum::Router::new()
        .route(
            "/api/runtime/workspaces/{id}/pty",
            axum::routing::get(workspace_pty),
        )
        .with_state(state);

    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let val: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(val["error"], "workspace not found");
}

#[tokio::test(flavor = "multi_thread")]
async fn pty_returns_404_when_no_tmux_session() {
    use http_body_util::BodyExt;
    let _tmp = setup_db();
    let state = make_api_state();

    let ship_dir = runtime::project::get_global_dir().unwrap();
    runtime::workspace::create_workspace(
        &ship_dir,
        runtime::workspace::CreateWorkspaceRequest {
            branch: "feat/pty-no-tmux".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let ws = runtime::workspace::get_workspace(&ship_dir, "feat/pty-no-tmux")
        .unwrap()
        .unwrap();
    let ws_id = ws.id.clone();

    let req = axum::http::Request::builder()
        .uri(format!("/api/runtime/workspaces/{ws_id}/pty"))
        .header("host", "localhost")
        .header("connection", "upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .body(axum::body::Body::empty())
        .unwrap();

    let app = axum::Router::new()
        .route(
            "/api/runtime/workspaces/{id}/pty",
            axum::routing::get(workspace_pty),
        )
        .with_state(state);

    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let val: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(val["error"], "no tmux session for this workspace");
}

#[tokio::test(flavor = "multi_thread")]
async fn pty_returns_429_when_over_connection_limit() {
    use http_body_util::BodyExt;
    let _tmp = setup_db();
    let state = make_api_state();

    let ship_dir = runtime::project::get_global_dir().unwrap();
    runtime::workspace::create_workspace(
        &ship_dir,
        runtime::workspace::CreateWorkspaceRequest {
            branch: "feat/pty-limit".to_string(),
            ..Default::default()
        },
    )
    .unwrap();

    let ws = runtime::workspace::get_workspace(&ship_dir, "feat/pty-limit")
        .unwrap()
        .unwrap();
    let ws_id = ws.id.clone();

    runtime::workspace::set_workspace_tmux_session(
        &ship_dir,
        "feat/pty-limit",
        Some("test-tmux-session"),
    )
    .unwrap();

    // Pre-fill the connection counter to the limit.
    {
        let mut conns = state.pty_connections.lock().await;
        conns.insert(ws_id.clone(), MAX_PTY_CONNECTIONS);
    }

    let req = axum::http::Request::builder()
        .uri(format!("/api/runtime/workspaces/{ws_id}/pty"))
        .header("host", "localhost")
        .header("connection", "upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .body(axum::body::Body::empty())
        .unwrap();

    let app = axum::Router::new()
        .route(
            "/api/runtime/workspaces/{id}/pty",
            axum::routing::get(workspace_pty),
        )
        .with_state(state);

    let response = tower::ServiceExt::oneshot(app, req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let val: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(val["error"]
        .as_str()
        .unwrap_or("")
        .contains("too many concurrent"));
}
