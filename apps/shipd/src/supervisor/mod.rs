//! Supervisor API — workspace lifecycle management for shipd.
//!
//! Route: POST /api/supervisor/workspaces/:id/start

pub mod helpers;
pub mod job_dispatch;
pub mod job_pipeline;
pub mod terminal_launcher;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use runtime::events::{ActorConfig, CallerKind, EmitContext, EventEnvelope};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    send_agent_command(&tmux_session, &[]);

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

// ── Workspace event subscription ─────────────────────────────────────────────

/// Subscribe to `workspace.*` kernel events and upsert workspace DB records.
///
/// Spawns an actor in the KernelRouter subscribed to the `workspace.` namespace.
/// Runs in a background task for the daemon lifetime — never panics, never blocks
/// the receive loop.
pub async fn subscribe_workspace_events(
    kernel: Arc<Mutex<runtime::events::KernelRouter>>,
    ship_dir: PathBuf,
) {
    let actor_id = "service.workspace-sync".to_string();
    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["workspace.".to_string()],
    };

    let mailbox = {
        let mut k = kernel.lock().await;
        match k.spawn_actor(&actor_id, config) {
            Ok((_store, mb)) => mb,
            Err(e) => {
                tracing::warn!("workspace-sync: failed to spawn actor: {e}");
                return;
            }
        }
    };

    tokio::spawn(async move {
        let mut mb = mailbox;
        while let Some(envelope) = mb.recv().await {
            handle_workspace_event(&ship_dir, &envelope);
        }
        tracing::info!("workspace-sync: mailbox closed");
    });
}

fn handle_workspace_event(ship_dir: &PathBuf, envelope: &EventEnvelope) {
    match envelope.event_type.as_str() {
        "workspace.activated" => {
            let branch = &envelope.entity_id;
            let payload: serde_json::Value =
                match serde_json::from_str(&envelope.payload_json) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!("workspace-sync: malformed workspace.activated payload: {e}");
                        return;
                    }
                };
            let worktree_path = payload
                .get("worktree_path")
                .and_then(|v| v.as_str())
                .map(str::to_string);

            // Ensure workspace record exists.
            match runtime::workspace::get_workspace(ship_dir, branch) {
                Ok(None) => {
                    let req = runtime::workspace::CreateWorkspaceRequest {
                        branch: branch.clone(),
                        is_worktree: Some(true),
                        worktree_path: worktree_path.clone(),
                        ..Default::default()
                    };
                    if let Err(e) = runtime::workspace::create_workspace(ship_dir, req) {
                        tracing::warn!(
                            branch,
                            "workspace-sync: create_workspace failed: {e}"
                        );
                        return;
                    }
                }
                Ok(Some(_)) => {}
                Err(e) => {
                    tracing::warn!(branch, "workspace-sync: get_workspace failed: {e}");
                    return;
                }
            }

            // Write worktree_path if available.
            if let Some(ref path_str) = worktree_path {
                let path = std::path::Path::new(path_str);
                // tmux session name is not in workspace.activated — use branch as placeholder.
                if let Err(e) =
                    runtime::workspace::set_workspace_started(ship_dir, branch, path, branch)
                {
                    tracing::warn!(
                        branch,
                        "workspace-sync: set_workspace_started failed: {e}"
                    );
                }
            }
        }

        "workspace.session.started" => {
            let payload: serde_json::Value =
                match serde_json::from_str(&envelope.payload_json) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!(
                            "workspace-sync: malformed workspace.session.started payload: {e}"
                        );
                        return;
                    }
                };
            let workspace_id = match payload.get("workspace_id").and_then(|v| v.as_str()) {
                Some(id) => id.to_string(),
                None => {
                    tracing::warn!("workspace-sync: workspace.session.started missing workspace_id");
                    return;
                }
            };
            let tmux_session = payload
                .get("tmux_session")
                .and_then(|v| v.as_str())
                .map(str::to_string);

            // workspace_id is the branch key for set_workspace_tmux_session.
            if let Err(e) = runtime::workspace::set_workspace_tmux_session(
                ship_dir,
                &workspace_id,
                tmux_session.as_deref(),
            ) {
                tracing::warn!(
                    workspace_id,
                    "workspace-sync: set_workspace_tmux_session failed: {e}"
                );
            }
        }

        _ => {}
    }
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
    use super::handle_workspace_event;
    use super::helpers::{provider_cli, worktrees_dir};
    use runtime::events::EventEnvelope;
    use std::path::PathBuf;
    use tempfile::TempDir;

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

    fn make_envelope(event_type: &str, entity_id: &str, payload: serde_json::Value) -> EventEnvelope {
        EventEnvelope::new(event_type, entity_id, &payload).unwrap()
    }

    /// Sets up an isolated ship dir for tests and ensures the DB schema exists.
    fn setup_ship_dir() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let ship_dir = runtime::project::init_project(dir.path().to_path_buf()).unwrap();
        (dir, ship_dir)
    }

    #[test]
    fn handle_workspace_activated_creates_workspace() {
        let (_dir, ship_dir) = setup_ship_dir();

        let envelope = make_envelope(
            "workspace.activated",
            "my-branch",
            serde_json::json!({ "agent_id": null, "providers": [] }),
        );
        handle_workspace_event(&ship_dir, &envelope);

        let ws = runtime::workspace::get_workspace(&ship_dir, "my-branch").unwrap();
        assert!(ws.is_some(), "workspace should be created on workspace.activated");
    }

    #[test]
    fn handle_workspace_activated_idempotent() {
        let (_dir, ship_dir) = setup_ship_dir();

        let envelope = make_envelope(
            "workspace.activated",
            "idempotent-branch",
            serde_json::json!({ "agent_id": null, "providers": [] }),
        );
        handle_workspace_event(&ship_dir, &envelope);
        handle_workspace_event(&ship_dir, &envelope);

        let ws = runtime::workspace::list_workspaces(&ship_dir).unwrap();
        assert_eq!(ws.iter().filter(|w| w.branch == "idempotent-branch").count(), 1);
    }

    #[test]
    fn handle_workspace_activated_malformed_payload_does_not_panic() {
        let (_dir, ship_dir) = setup_ship_dir();

        let mut envelope = make_envelope(
            "workspace.activated",
            "bad-branch",
            serde_json::json!({}),
        );
        envelope.payload_json = "not json {{{".to_string();
        // Must not panic.
        handle_workspace_event(&ship_dir, &envelope);
    }

    #[test]
    fn handle_unknown_event_type_does_nothing() {
        let (_dir, ship_dir) = setup_ship_dir();
        let envelope = make_envelope("workspace.compiled", "branch", serde_json::json!({}));
        // Must not panic or error.
        handle_workspace_event(&ship_dir, &envelope);
    }
}
