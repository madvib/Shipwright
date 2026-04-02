//! Project REST API — exposes workspace-scoped file, git, and event operations.
//!
//! These endpoints replace the MCP bridge that Studio previously used for
//! filesystem, git, and event operations on a single local project.
//! Routes are mounted under `/api` in lib.rs.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::path::PathBuf;

use crate::rest_api::{ApiState, MeshResponse};

// ---- Helpers ----

/// Resolve a workspace branch to its worktree path. Returns 404 if not found.
fn resolve_worktree(id: &str) -> Result<PathBuf, (StatusCode, Json<MeshResponse>)> {
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

fn ok_response(data: serde_json::Value) -> (StatusCode, Json<MeshResponse>) {
    (StatusCode::OK, Json(MeshResponse { ok: true, data }))
}

fn err_response(
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

fn file_type_from_ext(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "html",
        Some("md") => "markdown",
        Some("json") => "json",
        _ => "text",
    }
}

// ---- Session file endpoints ----

/// GET /api/workspaces/{id}/session-files
pub async fn list_session_files(
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(&id)?;
    let session_dir = worktree.join(".ship-session");

    if !session_dir.exists() {
        return Ok(ok_response(serde_json::json!({ "files": [] })));
    }

    let mut files = Vec::new();
    let entries = std::fs::read_dir(&session_dir).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to read session dir: {e}"),
        )
    })?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let file_type = file_type_from_ext(&path);
        files.push(serde_json::json!({
            "name": name,
            "path": name,
            "size": size,
            "type": file_type,
        }));
    }

    Ok(ok_response(serde_json::json!({ "files": files })))
}

/// GET /api/workspaces/{id}/session-files/*path
pub async fn read_session_file(
    Path((id, file_path)): Path<(String, String)>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(&id)?;
    let target = worktree.join(".ship-session").join(&file_path);

    // Prevent path traversal
    if !target.starts_with(worktree.join(".ship-session")) {
        return Err(err_response(StatusCode::BAD_REQUEST, "invalid path"));
    }

    let content = std::fs::read_to_string(&target).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            err_response(StatusCode::NOT_FOUND, "file not found")
        } else {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("failed to read file: {e}"),
            )
        }
    })?;

    Ok(ok_response(serde_json::json!({ "content": content })))
}

#[derive(Deserialize)]
pub struct WriteFileReq {
    pub content: String,
}

/// PUT /api/workspaces/{id}/session-files/*path
pub async fn write_session_file(
    Path((id, file_path)): Path<(String, String)>,
    Json(body): Json<WriteFileReq>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(&id)?;
    let session_dir = worktree.join(".ship-session");
    let target = session_dir.join(&file_path);

    // Prevent path traversal
    if !target.starts_with(&session_dir) {
        return Err(err_response(StatusCode::BAD_REQUEST, "invalid path"));
    }

    // Ensure parent directories exist
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("failed to create directory: {e}"),
            )
        })?;
    }

    std::fs::write(&target, &body.content).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to write file: {e}"),
        )
    })?;

    Ok(ok_response(serde_json::json!({ "path": file_path })))
}

/// DELETE /api/workspaces/{id}/session-files/*path
pub async fn delete_session_file(
    Path((id, file_path)): Path<(String, String)>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(&id)?;
    let target = worktree.join(".ship-session").join(&file_path);

    // Prevent path traversal
    if !target.starts_with(worktree.join(".ship-session")) {
        return Err(err_response(StatusCode::BAD_REQUEST, "invalid path"));
    }

    std::fs::remove_file(&target).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            err_response(StatusCode::NOT_FOUND, "file not found")
        } else {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("failed to delete file: {e}"),
            )
        }
    })?;

    Ok(ok_response(serde_json::json!({})))
}

// ---- Git endpoints ----

fn run_git(worktree: &std::path::Path, args: &[&str]) -> Result<String, (StatusCode, Json<MeshResponse>)> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(worktree)
        .output()
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("failed to run git: {e}"),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("git error: {stderr}"),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// GET /api/workspaces/{id}/git/status
pub async fn git_status(
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(&id)?;
    let output = run_git(&worktree, &["status"])?;
    Ok(ok_response(serde_json::json!({ "output": output })))
}

#[derive(Deserialize)]
pub struct DiffQuery {
    pub range: Option<String>,
}

/// GET /api/workspaces/{id}/git/diff
pub async fn git_diff(
    Path(id): Path<String>,
    Query(params): Query<DiffQuery>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(&id)?;
    let output = match &params.range {
        Some(range) => run_git(&worktree, &["diff", range])?,
        None => run_git(&worktree, &["diff"])?,
    };
    Ok(ok_response(serde_json::json!({ "output": output })))
}

#[derive(Deserialize)]
pub struct LogQuery {
    pub limit: Option<u32>,
}

/// GET /api/workspaces/{id}/git/log
pub async fn git_log(
    Path(id): Path<String>,
    Query(params): Query<LogQuery>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(&id)?;
    let limit = params.limit.unwrap_or(20);
    let limit_arg = format!("-{limit}");
    let output = run_git(
        &worktree,
        &["log", "--format=%H%x1f%s%x1f%an%x1f%ad", "--date=short", &limit_arg],
    )?;

    let commits: Vec<serde_json::Value> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\x1f').collect();
            if parts.len() >= 4 {
                Some(serde_json::json!({
                    "hash": parts[0],
                    "subject": parts[1],
                    "author": parts[2],
                    "date": parts[3],
                }))
            } else {
                None
            }
        })
        .collect();

    Ok(ok_response(serde_json::json!({
        "output": output,
        "commits": commits,
    })))
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
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string(),
            )
        })?;

    Ok(ok_response(serde_json::json!({})))
}

// ---- Workspace activate endpoint ----

/// POST /api/workspaces/{id}/activate
pub async fn activate_workspace(
    Path(id): Path<String>,
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
mod tests {
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
        assert_eq!(file_type_from_ext(std::path::Path::new("a.html")), "html");
        assert_eq!(file_type_from_ext(std::path::Path::new("b.md")), "markdown");
        assert_eq!(file_type_from_ext(std::path::Path::new("c.json")), "json");
        assert_eq!(file_type_from_ext(std::path::Path::new("d.txt")), "text");
        assert_eq!(file_type_from_ext(std::path::Path::new("e")), "text");
    }
}
