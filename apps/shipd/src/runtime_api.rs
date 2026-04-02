//! Runtime read API — exposes live platform state for Studio.
//!
//! Routes are mounted under `/api/runtime` in lib.rs.
//! All handlers are read-only and use the existing `ApiState`.

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum::response::sse::{Event, KeepAlive, Sse};
use runtime::events::ActorConfig;
use serde::Deserialize;

use crate::rest_api::{ApiState, MeshResponse};

// ---- Query params ----

#[derive(Deserialize)]
pub struct SessionsQuery {
    pub workspace_id: Option<String>,
}

// ---- Handlers ----

/// GET /api/runtime/workspaces
pub async fn list_workspaces(State(_state): State<ApiState>) -> impl IntoResponse {
    let ship_dir = match runtime::project::get_global_dir() {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(e.to_string()),
                }),
            );
        }
    };
    match runtime::workspace::list_workspaces(&ship_dir) {
        Ok(ws) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({ "workspaces": ws }),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e.to_string()),
            }),
        ),
    }
}

/// GET /api/runtime/sessions[?workspace_id=<id>]
pub async fn list_sessions(
    State(_state): State<ApiState>,
    Query(params): Query<SessionsQuery>,
) -> impl IntoResponse {
    let workspace_id = params.workspace_id.as_deref();
    match runtime::db::session::list_workspace_sessions_db(workspace_id, 100) {
        Ok(sessions) => {
            let data: Vec<serde_json::Value> = sessions
                .into_iter()
                .map(|s| {
                    serde_json::json!({
                        "id": s.id,
                        "workspace_id": s.workspace_id,
                        "workspace_branch": s.workspace_branch,
                        "status": s.status,
                        "started_at": s.started_at,
                        "ended_at": s.ended_at,
                        "agent_id": s.agent_id,
                        "primary_provider": s.primary_provider,
                        "goal": s.goal,
                        "summary": s.summary,
                        "tool_call_count": s.tool_call_count,
                        "created_at": s.created_at,
                        "updated_at": s.updated_at,
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(MeshResponse {
                    ok: true,
                    data: serde_json::json!({ "sessions": data }),
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e.to_string()),
            }),
        ),
    }
}

/// GET /api/runtime/agents
pub async fn list_agents(State(state): State<ApiState>) -> impl IntoResponse {
    let registry = state.mesh_registry.read().await;
    let agents: Vec<serde_json::Value> = registry
        .values()
        .map(|e| {
            serde_json::json!({
                "agent_id": e.agent_id,
                "label": e.label,
                "capabilities": e.capabilities,
                "status": e.status,
            })
        })
        .collect();
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: serde_json::json!({ "agents": agents }),
        }),
    )
}

/// Stops an actor when dropped — used to clean up the SSE stream actor.
struct ActorGuard {
    actor_id: String,
    kernel: crate::rest_api::SharedKernel,
}

impl Drop for ActorGuard {
    fn drop(&mut self) {
        let actor_id = self.actor_id.clone();
        let kernel = self.kernel.clone();
        tokio::spawn(async move {
            let _ = kernel.lock().await.stop_actor(&actor_id);
        });
    }
}

/// GET /api/runtime/events — SSE stream of all kernel events.
pub async fn event_stream(
    State(state): State<ApiState>,
) -> Result<Sse<impl futures::stream::Stream<Item = Result<Event, std::convert::Infallible>>>, (StatusCode, String)> {
    let actor_id = format!(
        "studio-events-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );

    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["".to_string()],
    };

    let mailbox = {
        let mut kernel = state.kernel.lock().await;
        kernel.spawn_actor(&actor_id, config).map(|(_store, mb)| mb).map_err(|e| {
            tracing::warn!("event_stream: actor spawn failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
    };

    let guard = ActorGuard {
        actor_id,
        kernel: state.kernel.clone(),
    };

    let stream = async_stream::stream! {
        let _guard = guard;
        let mut mb = mailbox;
        while let Some(event) = mb.recv().await {
            let json = match serde_json::to_string(&event) {
                Ok(j) => j,
                Err(_) => continue,
            };
            yield Ok::<_, std::convert::Infallible>(
                Event::default().event("ship.event").data(json)
            );
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::{Query, State};
    use axum::response::IntoResponse;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::Mutex;

    fn make_api_state(dir: &TempDir) -> ApiState {
        let ship_dir = dir.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        let kernel = runtime::events::init_kernel_router(ship_dir).unwrap();
        let mesh_registry = Arc::new(tokio::sync::RwLock::new(
            std::collections::HashMap::new(),
        ));
        ApiState {
            kernel,
            mesh_registry,
            agent_mailboxes: Arc::new(Mutex::new(std::collections::HashMap::new())),
            pty_connections: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    async fn collect_body(response: axum::response::Response) -> serde_json::Value {
        use http_body_util::BodyExt;
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_workspaces_returns_ok_with_array() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let response = list_workspaces(State(state)).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert!(val["data"]["workspaces"].is_array(), "expected workspaces array");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_sessions_returns_ok_no_workspace_id() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let query = Query(SessionsQuery { workspace_id: None });
        let response = list_sessions(State(state), query).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert!(val["data"]["sessions"].is_array(), "expected sessions array");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_sessions_returns_ok_with_workspace_id() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let query = Query(SessionsQuery {
            workspace_id: Some("nonexistent-workspace".to_string()),
        });
        let response = list_sessions(State(state), query).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert!(val["data"]["sessions"].is_array());
        assert_eq!(val["data"]["sessions"].as_array().unwrap().len(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_agents_returns_ok_empty_registry() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let response = list_agents(State(state)).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let val = collect_body(response).await;
        assert_eq!(val["ok"], serde_json::json!(true));
        assert!(val["data"]["agents"].is_array());
        assert_eq!(val["data"]["agents"].as_array().unwrap().len(), 0);
    }
}
