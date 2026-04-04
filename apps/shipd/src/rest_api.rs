//! Plain REST API for ship-mcp relay clients.
//!
//! These endpoints bypass the MCP protocol — no handshake needed.
//! ship-mcp calls these to forward mesh operations to the shared kernel.

use axum::{Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::Stream;
use runtime::events::{CallerKind, EmitContext, EventEnvelope};
use runtime::services::mesh::SharedMeshRegistry;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type SharedKernel = Arc<Mutex<runtime::events::KernelRouter>>;

/// Stashed mailboxes for remote agents, taken once by the SSE endpoint.
pub type AgentMailboxes = Arc<Mutex<std::collections::HashMap<String, runtime::events::Mailbox>>>;

/// Active PTY connection counts, keyed by workspace id.
pub type PtyConnections = Arc<Mutex<std::collections::HashMap<String, usize>>>;

/// Combined state for REST API routes.
#[derive(Clone)]
pub struct ApiState {
    pub kernel: SharedKernel,
    pub mesh_registry: SharedMeshRegistry,
    pub agent_mailboxes: AgentMailboxes,
    pub pty_connections: PtyConnections,
}

// ---- Request / Response types ----

#[derive(Deserialize)]
pub struct MeshRegisterReq {
    pub agent_id: String,
    pub capabilities: Vec<String>,
}

#[derive(Deserialize)]
pub struct MeshSendReq {
    pub from: String,
    pub to: String,
    pub body: serde_json::Value,
}

#[derive(Deserialize)]
pub struct MeshBroadcastReq {
    pub from: String,
    pub body: serde_json::Value,
    pub capability_filter: Option<String>,
}

#[derive(Deserialize)]
pub struct MeshStatusReq {
    pub agent_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct MeshResponse {
    pub ok: bool,
    pub data: serde_json::Value,
}

// ---- Handlers ----

fn emit_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    }
}

pub async fn mesh_register(
    State(state): State<ApiState>,
    Json(req): Json<MeshRegisterReq>,
) -> impl IntoResponse {
    // Spawn (or re-spawn) an actor in the kernel for this remote agent
    // so messages with target_actor_id can be delivered to its mailbox.
    {
        let mut kernel = state.kernel.lock().await;
        // Stop existing actor so we get a fresh mailbox for the SSE endpoint.
        let _ = kernel.stop_actor(&req.agent_id);
        let config = runtime::events::ActorConfig {
            namespace: req.agent_id.clone(),
            write_namespaces: vec!["mesh.".to_string()],
            read_namespaces: vec!["mesh.".to_string()],
            subscribe_namespaces: vec!["mesh.".to_string()],
        };
        match kernel.spawn_actor(&req.agent_id, config) {
            Ok((_store, mailbox)) => {
                state.agent_mailboxes.lock().await
                    .insert(req.agent_id.clone(), mailbox);
            }
            Err(e) => tracing::warn!(agent_id = %req.agent_id, "actor spawn failed: {e}"),
        }
    }

    let envelope = match EventEnvelope::new(
        "mesh.register",
        &req.agent_id,
        &serde_json::json!({ "agent_id": &req.agent_id, "capabilities": req.capabilities }),
    )
    .map(|e| e.with_actor_id(&req.agent_id))
    {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    };

    match state.kernel.lock().await.route(envelope, &emit_ctx()).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!(format!("registered: {}", req.agent_id)) })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    }
}

pub async fn mesh_send(
    State(state): State<ApiState>,
    Json(req): Json<MeshSendReq>,
) -> impl IntoResponse {
    // Route through MeshService via mesh.send event type so it validates
    // the target and sets target_actor_id for directed delivery.
    let envelope = match EventEnvelope::new(
        "mesh.send",
        &req.to,
        &serde_json::json!({ "from": &req.from, "to": &req.to, "body": req.body }),
    )
    .map(|e| e.with_actor_id(&req.from))
    {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    };

    match state.kernel.lock().await.route(envelope, &emit_ctx()).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!("sent") })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    }
}

pub async fn mesh_broadcast(
    State(state): State<ApiState>,
    Json(req): Json<MeshBroadcastReq>,
) -> impl IntoResponse {
    let mut payload = serde_json::json!({ "from": &req.from, "body": req.body });
    if let Some(ref cap) = req.capability_filter {
        payload["capability_filter"] = serde_json::json!(cap);
    }
    let envelope = match EventEnvelope::new("mesh.broadcast", &req.from, &payload)
        .map(|e| e.with_actor_id(&req.from))
    {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    };

    match state.kernel.lock().await.route(envelope, &emit_ctx()).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!("broadcast sent") })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    }
}

pub async fn mesh_discover(
    State(state): State<ApiState>,
) -> impl IntoResponse {
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
    (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!({ "agents": agents }) }))
}

pub async fn mesh_status_update(
    State(state): State<ApiState>,
    Json(req): Json<MeshStatusReq>,
) -> impl IntoResponse {
    let envelope = match EventEnvelope::new(
        "mesh.status",
        &req.agent_id,
        &serde_json::json!({ "agent_id": &req.agent_id, "status": req.status }),
    )
    .map(|e| e.with_actor_id(&req.agent_id))
    {
        Ok(e) => e,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    };

    match state.kernel.lock().await.route(envelope, &emit_ctx()).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!("status updated") })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    }
}

// ---- Checkout hook ----

#[derive(Deserialize)]
pub struct CheckoutHookReq {
    pub branch: String,
    pub path: String,
}

pub async fn hooks_checkout(
    Json(req): Json<CheckoutHookReq>,
) -> impl IntoResponse {
    let ship_dir_result = runtime::project::get_project_dir(Some(std::path::PathBuf::from(&req.path)));
    let ship_dir = match ship_dir_result {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(format!("no .ship/ found: {e}")),
                }),
            );
        }
    };

    let connection_path = std::path::Path::new(&req.path);
    match runtime::reconcile_workspace(&ship_dir, &req.branch, connection_path) {
        Ok(Some(ws)) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!({
                    "branch": ws.branch,
                    "is_worktree": ws.is_worktree,
                    "worktree_path": ws.worktree_path,
                }),
            }),
        ),
        Ok(None) => (
            StatusCode::OK,
            Json(MeshResponse {
                ok: true,
                data: serde_json::json!("no workspace for branch"),
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

/// SSE endpoint — drains an agent's mailbox as server-sent events.
/// The agent must register first (which spawns the actor + stashes the mailbox).
/// Only one SSE connection per agent is supported (mailbox is taken, not cloned).
pub async fn mesh_events(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
) -> impl IntoResponse {
    let mailbox = state.agent_mailboxes.lock().await.remove(&agent_id);
    let Some(mut mailbox) = mailbox else {
        return Err((
            StatusCode::NOT_FOUND,
            format!("no mailbox for agent '{agent_id}' — register first"),
        ));
    };

    let stream = async_stream::stream! {
        while let Some(event) = mailbox.recv().await {
            let json = match serde_json::to_string(&event) {
                Ok(j) => j,
                Err(_) => continue,
            };
            yield Ok::<_, std::convert::Infallible>(
                Event::default()
                    .event("mesh.event")
                    .data(json)
            );
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
