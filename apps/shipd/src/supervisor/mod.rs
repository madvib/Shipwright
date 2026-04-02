//! Supervisor API — workspace lifecycle management for shipd.
//!
//! Route: POST /api/supervisor/workspaces/:id/start

pub mod helpers;
pub mod terminal_launcher;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use runtime::events::{CallerKind, EmitContext, EventEnvelope};
use serde::{Deserialize, Serialize};

use crate::rest_api::ApiState;
use helpers::{
    compile_agent_config, create_worktree, ensure_tmux_session, send_agent_command, worktrees_dir,
};

#[derive(Deserialize)]
pub struct StartWorkspaceRequest {
    pub agent_id: String,
    pub base_branch: Option<String>,
    pub open_terminal: Option<bool>,
}

#[derive(Serialize)]
pub struct StartWorkspaceResponse {
    pub ok: bool,
    pub worktree_path: String,
    pub tmux_session: String,
    pub terminal_launched: bool,
    pub terminal_strategy: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": msg.into() })))
}

/// POST /api/supervisor/workspaces/:id/start
pub async fn start_workspace(
    Path(workspace_id): Path<String>,
    State(state): State<ApiState>,
    Json(req): Json<StartWorkspaceRequest>,
) -> impl IntoResponse {
    let ship_dir = match runtime::project::get_global_dir() {
        Ok(p) => p,
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Step 1: Look up workspace by id.
    let workspace = match runtime::workspace::get_workspace_by_id(&ship_dir, &workspace_id) {
        Ok(Some(w)) => w,
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, format!("workspace '{workspace_id}' not found"))
                .into_response()
        }
        Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let branch = workspace.branch.clone();
    let worktree_path = worktrees_dir().join(&branch);
    let tmux_session = workspace_id.clone();

    // Step 2: Create worktree — idempotent.
    if let Err(e) = create_worktree(&worktree_path, &branch, req.base_branch.as_deref()) {
        return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // Step 3: Run `ship use <agent_id>` in worktree.
    if let Err(e) = compile_agent_config(&worktree_path, &req.agent_id) {
        return err(
            StatusCode::BAD_REQUEST,
            format!("ship use {} failed: {e}", req.agent_id),
        )
        .into_response();
    }

    // Step 4: Create tmux session — idempotent.
    if let Err(e) = ensure_tmux_session(&tmux_session, &worktree_path) {
        return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // Step 5: Spawn agent command in the tmux window.
    send_agent_command(&tmux_session, &workspace.providers);

    // Step 6: Register agent on mesh.
    register_on_mesh(&state, &req.agent_id).await;

    // Step 7: Write tmux_session_name and worktree_path to workspace record.
    if let Err(e) =
        runtime::workspace::set_workspace_started(&ship_dir, &branch, &worktree_path, &tmux_session)
    {
        return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // Step 8: Open terminal if requested.
    let (terminal_strategy, terminal_launched) = if req.open_terminal.unwrap_or(false) {
        terminal_launcher::launch(&tmux_session)
    } else {
        ("manual".to_string(), false)
    };

    (
        StatusCode::OK,
        Json(StartWorkspaceResponse {
            ok: true,
            worktree_path: worktree_path.to_string_lossy().into_owned(),
            tmux_session,
            terminal_launched,
            terminal_strategy,
        }),
    )
        .into_response()
}

// ── Mesh ──────────────────────────────────────────────────────────────────────

async fn register_on_mesh(state: &ApiState, agent_id: &str) {
    let envelope = match EventEnvelope::new(
        "mesh.register",
        agent_id,
        &serde_json::json!({ "agent_id": agent_id, "capabilities": ["workspace"] }),
    ) {
        Ok(e) => e.with_actor_id(agent_id),
        Err(e) => {
            tracing::warn!(agent_id, "mesh.register envelope error: {e}");
            return;
        }
    };

    let ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };

    if let Err(e) = state.kernel.lock().await.route(envelope, &ctx).await {
        tracing::warn!(agent_id, "mesh.register routing error: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::{provider_cli, worktrees_dir};
    use std::path::PathBuf;

    #[test]
    fn provider_cli_claude_code() {
        let cli = provider_cli(&["claude-code".to_string()]);
        assert!(cli.unwrap().contains("claude"));
    }

    #[test]
    fn provider_cli_codex() {
        let cli = provider_cli(&["codex".to_string()]);
        assert_eq!(cli.unwrap(), "codex");
    }

    #[test]
    fn provider_cli_unknown_returns_none() {
        let cli = provider_cli(&["cursor".to_string()]);
        assert!(cli.is_none());
    }

    #[test]
    fn worktrees_dir_uses_env_override() {
        // Safety: single-threaded test; no other threads read SHIP_WORKTREE_DIR.
        unsafe {
            std::env::set_var("SHIP_WORKTREE_DIR", "/tmp/test-worktrees");
        }
        let dir = worktrees_dir();
        unsafe {
            std::env::remove_var("SHIP_WORKTREE_DIR");
        }
        assert_eq!(dir, PathBuf::from("/tmp/test-worktrees"));
    }
}
