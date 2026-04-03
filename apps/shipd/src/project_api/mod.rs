//! Project REST API — exposes workspace-scoped file, git, event, and config operations.
//!
//! These endpoints replace the MCP bridge that Studio previously used for
//! filesystem, git, and event operations on a single local project.
//! Routes are mounted under `/api` in lib.rs.

mod agents_skills;
mod git;
mod session_files;

pub use agents_skills::{list_agents, list_skills};
pub use git::{git_diff, git_log, git_status};
pub use session_files::{
    delete_session_file, list_session_files, read_session_file, write_session_file,
};

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use std::path::PathBuf;

use crate::rest_api::{ApiState, MeshResponse};

// ---- Shared helpers ----

/// Resolve a workspace branch to its worktree path. Returns 404 if not found.
pub(crate) fn resolve_worktree(id: &str) -> Result<PathBuf, (StatusCode, Json<MeshResponse>)> {
    let ship_dir = runtime::project::get_global_dir().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!(e.to_string()),
            }),
        )
    })?;

    let ws = runtime::workspace::get_workspace(&ship_dir, id)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!(e.to_string()),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(MeshResponse {
                    ok: false,
                    data: serde_json::json!("workspace not found"),
                }),
            )
        })?;

    let worktree = ws.worktree_path.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(MeshResponse {
                ok: false,
                data: serde_json::json!("workspace not found"),
            }),
        )
    })?;

    Ok(PathBuf::from(worktree))
}

pub(crate) fn ok_response(data: serde_json::Value) -> (StatusCode, Json<MeshResponse>) {
    (StatusCode::OK, Json(MeshResponse { ok: true, data }))
}

pub(crate) fn err_response(
    status: StatusCode,
    msg: &str,
) -> (StatusCode, Json<MeshResponse>) {
    (
        status,
        Json(MeshResponse {
            ok: false,
            data: serde_json::json!(msg),
        }),
    )
}

// ---- Event emit endpoint ----

#[derive(Deserialize)]
pub struct EmitEventReq {
    pub event_type: String,
    pub entity_id: String,
    pub workspace_id: Option<String>,
    pub payload: serde_json::Value,
}

/// POST /api/events/emit
pub async fn emit_event(
    State(state): State<ApiState>,
    Json(body): Json<EmitEventReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let mut envelope =
        runtime::events::EventEnvelope::new(&body.event_type, &body.entity_id, &body.payload)
            .map_err(|e| err_response(StatusCode::BAD_REQUEST, &e.to_string()))?;

    envelope = envelope.with_actor_id("studio");
    if let Some(ref ws_id) = body.workspace_id {
        envelope = envelope.with_context(Some(ws_id), None);
    }

    let ctx = runtime::events::EmitContext {
        caller_kind: runtime::events::CallerKind::Mcp,
        skill_id: None,
        workspace_id: body.workspace_id.clone(),
        session_id: None,
    };

    state
        .kernel
        .lock()
        .await
        .route(envelope, &ctx)
        .await
        .map_err(|e| {
            err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        })?;

    Ok(ok_response(serde_json::json!({})))
}

// ---- Workspace delete endpoint ----

/// DELETE /api/workspaces/{id}
pub async fn delete_workspace(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let ship_dir = runtime::project::get_global_dir().map_err(|e| {
        err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    })?;

    runtime::workspace::delete_workspace(&ship_dir, &id).map_err(|e| {
        err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    })?;

    Ok(ok_response(serde_json::json!({ "branch": id })))
}

// ---- Workspace activate endpoint ----

/// POST /api/workspaces/{id}/activate
pub async fn activate_workspace(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let ship_dir = runtime::project::get_global_dir().map_err(|e| {
        err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    })?;

    runtime::workspace::activate_workspace(&ship_dir, &id).map_err(|e| {
        err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    })?;

    Ok(ok_response(serde_json::json!({ "branch": id })))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_inline {
    use super::*;
    use axum::extract::Path;

    #[tokio::test(flavor = "multi_thread")]
    async fn nonexistent_workspace_returns_404() {
        let response = git_status(Path("nonexistent-branch-xyz".to_string()))
            .await
            .unwrap_err();
        let (status, body) = response;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body.0.ok, false);
    }

    #[test]
    fn file_type_detection() {
        assert_eq!(
            session_files::file_type_from_ext(std::path::Path::new("a.html")),
            "html"
        );
        assert_eq!(
            session_files::file_type_from_ext(std::path::Path::new("b.md")),
            "markdown"
        );
        assert_eq!(
            session_files::file_type_from_ext(std::path::Path::new("c.json")),
            "json"
        );
        assert_eq!(
            session_files::file_type_from_ext(std::path::Path::new("d.txt")),
            "text"
        );
        assert_eq!(
            session_files::file_type_from_ext(std::path::Path::new("e")),
            "text"
        );
    }
}
