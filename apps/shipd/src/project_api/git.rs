//! Git endpoints — status, diff, log for workspace worktrees.

use axum::{Json, extract::Path, extract::Query, http::StatusCode};
use serde::Deserialize;

use crate::rest_api::MeshResponse;

use super::{err_response, ok_response, resolve_worktree};

fn run_git(
    worktree: &std::path::Path,
    args: &[&str],
) -> Result<String, (StatusCode, Json<MeshResponse>)> {
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
        &[
            "log",
            "--format=%H%x1f%s%x1f%an%x1f%ad",
            "--date=short",
            &limit_arg,
        ],
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
