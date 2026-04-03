//! Session file endpoints — list, read, write, delete files in .ship-session/.

use axum::{Json, extract::Path, http::StatusCode};
use serde::Deserialize;

use crate::rest_api::MeshResponse;

use super::{err_response, ok_response, resolve_worktree};

pub(crate) fn file_type_from_ext(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "html",
        Some("md") => "markdown",
        Some("json") => "json",
        _ => "text",
    }
}

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
