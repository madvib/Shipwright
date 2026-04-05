//! Runtime read API — exposes live platform state for Studio.
//!
//! Routes are mounted under `/api/runtime` in lib.rs.
//! All handlers are read-only and use the existing `ApiState`.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum::response::sse::{Event, KeepAlive, Sse};
use runtime::events::{ActorConfig, CallerKind, EmitContext, EventEnvelope};
use runtime::events::job::{JobCreatedPayload, event_types};
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

// ---- Views ----

/// GET /api/runtime/views — list available views from .ship/views/.
pub async fn list_views() -> impl IntoResponse {
    let project_dir = match runtime::project::get_project_dir(None) {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(MeshResponse { ok: true, data: serde_json::json!({ "views": [] }) }),
            );
        }
    };

    let views_dir = project_dir.join("views");
    let mut views = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&views_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().to_string();
                let index = entry.path().join("index.html");
                if index.exists() {
                    views.push(serde_json::json!({ "name": name }));
                }
            }
        }
    }

    (
        StatusCode::OK,
        Json(MeshResponse { ok: true, data: serde_json::json!({ "views": views }) }),
    )
}

/// GET /api/runtime/view/{name} — serve a view's index.html from .ship/views/{name}/.
///
/// Reads the view HTML and inlines the ship-sdk script so the view can
/// communicate with Studio via postMessage.
pub async fn serve_view(
    Path(name): Path<String>,
) -> impl IntoResponse {
    let project_dir = match runtime::project::get_project_dir(None) {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse { ok: false, data: serde_json::json!("no project root found") }),
            );
        }
    };

    // Reject path traversal attempts
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return (
            StatusCode::BAD_REQUEST,
            Json(MeshResponse { ok: false, data: serde_json::json!("invalid view name") }),
        );
    }

    // get_project_dir returns the .ship/ directory
    let view_path = project_dir.join("views").join(&name).join("index.html");
    let sdk_path = project_dir.join("views/ship-sdk.js");

    let html = match std::fs::read_to_string(&view_path) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(MeshResponse { ok: false, data: serde_json::json!(format!("view '{name}' not found")) }),
            );
        }
    };

    // Inline the SDK script so the view doesn't need to resolve relative paths
    let html = if let Ok(sdk) = std::fs::read_to_string(&sdk_path) {
        html.replace(
            r#"<script src="../ship-sdk.js"></script>"#,
            &format!("<script>\n{sdk}\n</script>"),
        )
    } else {
        html
    };

    (
        StatusCode::OK,
        Json(MeshResponse { ok: true, data: serde_json::json!({ "html": html }) }),
    )
}

// ---- Job request types ----

#[derive(Deserialize)]
pub struct CreateJobRequest {
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
    pub depends_on: Option<Vec<String>>,
}

/// GET /api/runtime/jobs
pub async fn list_jobs(State(_state): State<ApiState>) -> impl IntoResponse {
    match runtime::projections::job::load_jobs() {
        Ok(jobs) => {
            let data: Vec<_> = jobs.into_values().collect();
            (
                StatusCode::OK,
                Json(MeshResponse {
                    ok: true,
                    data: serde_json::json!({ "jobs": data }),
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

/// POST /api/runtime/jobs — emit job.created event through the kernel.
///
/// The kernel persists the event and routes it to subscribers.
/// The job_dispatch service handles all heavy lifting (worktree creation,
/// dependency resolution, agent spawning).
pub async fn create_job(
    State(state): State<ApiState>,
    Json(req): Json<CreateJobRequest>,
) -> impl IntoResponse {
    let job_id = runtime::gen_ulid();
    let payload = JobCreatedPayload {
        job_id: job_id.clone(),
        slug: req.slug,
        agent: req.agent,
        branch: req.branch,
        spec_path: req.spec_path,
        plan_id: None,
        model: None,
        provider: None,
        depends_on: req.depends_on,
    };

    let envelope = match EventEnvelope::new(event_types::JOB_CREATED, &job_id, &payload)
        .map(|e| e.with_actor_id("studio"))
    {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(e.to_string()),
                }),
            );
        }
    };

    let ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };

    match state.kernel.lock().await.route(envelope, &ctx).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({ "job_id": &job_id }),
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
