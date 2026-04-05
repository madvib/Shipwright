//! Actor spawning and event routing endpoints.
//!
//! These generalize mesh operations so the MCP process can delegate
//! actor lifecycle and event routing to the daemon's shared kernel.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use runtime::events::{CallerKind, EmitContext, EventEnvelope};
use serde::Deserialize;

use crate::rest_api::{ApiState, MeshResponse};

fn emit_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    }
}

#[derive(Deserialize)]
pub struct ActorSpawnReq {
    pub actor_id: String,
    pub config: runtime::events::ActorConfig,
    /// Optional mesh capabilities for auto-registration.
    pub capabilities: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct EventRouteReq {
    pub envelope: runtime::events::EventEnvelope,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
}

pub async fn actor_spawn(
    State(state): State<ApiState>,
    Json(req): Json<ActorSpawnReq>,
) -> impl IntoResponse {
    {
        let mut kernel = state.kernel.lock().await;
        let _ = kernel.stop_actor(&req.actor_id);
        match kernel.spawn_actor(&req.actor_id, req.config) {
            Ok((_store, mailbox)) => {
                state.agent_mailboxes.lock().await
                    .insert(req.actor_id.clone(), mailbox);
            }
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse {
                ok: false, data: serde_json::json!(e.to_string()),
            })),
        }
    }

    if let Some(caps) = req.capabilities {
        let envelope = match EventEnvelope::new(
            "mesh.register",
            &req.actor_id,
            &serde_json::json!({ "agent_id": &req.actor_id, "capabilities": caps }),
        )
        .map(|e| e.with_actor_id(&req.actor_id))
        {
            Ok(e) => e,
            Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse {
                ok: false, data: serde_json::json!(e.to_string()),
            })),
        };
        if let Err(e) = state.kernel.lock().await.route(envelope, &emit_ctx()).await {
            tracing::warn!(actor_id = %req.actor_id, "mesh.register routing failed: {e}");
        }
    }

    (StatusCode::OK, Json(MeshResponse {
        ok: true, data: serde_json::json!(format!("spawned: {}", req.actor_id)),
    }))
}

pub async fn event_route(
    State(state): State<ApiState>,
    Json(req): Json<EventRouteReq>,
) -> impl IntoResponse {
    let ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        workspace_id: req.workspace_id,
        session_id: req.session_id,
    };
    match state.kernel.lock().await.route(req.envelope, &ctx).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!("routed") })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    }
}
