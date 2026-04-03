//! Job dispatch subscriber — reacts to job lifecycle events from the kernel.
//!
//! When `job.created` flows through the kernel, creates a worktree, copies the
//! job spec, runs `ship use`, and spawns a terminal. When `job.update` arrives,
//! routes the message to the agent's mailbox via mesh.

use runtime::events::job::JobCreatedPayload;
use runtime::events::{ActorConfig, CallerKind, EmitContext, EventEnvelope};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::helpers::{compile_agent_config, create_worktree, ensure_tmux_session, worktrees_dir};
use super::terminal_launcher;

/// Subscribe to `job.*` kernel events and dispatch jobs to worktrees.
///
/// Spawns a `service.job-dispatch` actor subscribed to `job.` namespace.
/// Runs in a background task for the daemon lifetime.
pub async fn subscribe_job_events(
    kernel: Arc<Mutex<runtime::events::KernelRouter>>,
    _ship_dir: PathBuf,
) {
    let actor_id = "service.job-dispatch".to_string();
    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec!["job.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string()],
    };

    let mailbox = {
        let mut k = kernel.lock().await;
        match k.spawn_actor(&actor_id, config) {
            Ok((_store, mb)) => mb,
            Err(e) => {
                tracing::warn!("job-dispatch: failed to spawn actor: {e}");
                return;
            }
        }
    };

    let kr = kernel.clone();
    tokio::spawn(async move {
        let mut mb = mailbox;
        while let Some(envelope) = mb.recv().await {
            handle_job_event(&kr, &envelope).await;
        }
        tracing::info!("job-dispatch: mailbox closed");
    });
}

async fn handle_job_event(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
) {
    match envelope.event_type.as_str() {
        "job.created" => handle_job_created(kernel, envelope).await,
        "job.update" => handle_job_update(kernel, envelope).await,
        _ => {}
    }
}

async fn handle_job_created(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
) {
    let payload: JobCreatedPayload = match serde_json::from_str(&envelope.payload_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("job-dispatch: malformed job.created payload: {e}");
            return;
        }
    };

    let slug = &payload.slug;
    let branch = &payload.branch;
    let agent = &payload.agent;
    let spec_path = &payload.spec_path;
    tracing::info!(slug, branch, agent, "job-dispatch: handling job.created");

    // 1. Create git worktree
    let worktree_path = worktrees_dir().join(slug);
    if let Err(e) = create_worktree(&worktree_path, branch, None) {
        tracing::error!(slug, "job-dispatch: worktree creation failed: {e}");
        return;
    }

    // 2. Copy spec file into worktree as .ship-session/job-spec.md
    let session_dir = worktree_path.join(".ship-session");
    if let Err(e) = std::fs::create_dir_all(&session_dir) {
        tracing::error!(slug, "job-dispatch: failed to create .ship-session: {e}");
        return;
    }
    let src_spec = std::path::Path::new(spec_path);
    let dst_spec = session_dir.join("job-spec.md");
    if src_spec.exists() {
        if let Err(e) = std::fs::copy(src_spec, &dst_spec) {
            tracing::error!(slug, "job-dispatch: spec copy failed: {e}");
            return;
        }
    } else {
        tracing::warn!(slug, spec_path, "job-dispatch: spec file not found, skipping copy");
    }

    // 3. Run `ship use {agent}` in the worktree
    if let Err(e) = compile_agent_config(&worktree_path, agent) {
        tracing::error!(slug, agent, "job-dispatch: ship use failed: {e}");
        return;
    }

    // 4. Create tmux session and spawn terminal
    let tmux_session = format!("job-{slug}");
    if let Err(e) = ensure_tmux_session(&tmux_session, &worktree_path) {
        tracing::error!(slug, "job-dispatch: tmux session creation failed: {e}");
        return;
    }

    // 5. Send agent command with SHIP_MESH_ID in environment
    spawn_agent_with_mesh_id(&tmux_session, slug);

    // 6. Launch terminal (respects SHIP_DEFAULT_TERMINAL)
    let (strategy, launched) = terminal_launcher::launch(&tmux_session);
    tracing::info!(
        slug,
        strategy,
        launched,
        "job-dispatch: terminal launch attempted"
    );

    // 7. Emit job.dispatched
    let dispatched_payload = runtime::events::job::JobDispatchedPayload {
        job_id: payload.job_id.clone(),
        worktree: worktree_path.to_string_lossy().into_owned(),
        pid: None,
    };
    let dispatched_envelope = match EventEnvelope::new(
        runtime::events::job::event_types::JOB_DISPATCHED,
        &payload.job_id,
        &dispatched_payload,
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(slug, "job-dispatch: failed to build job.dispatched event: {e}");
            return;
        }
    };

    // Persist to event store
    if let Ok(store) = runtime::events::SqliteEventStore::new() {
        if let Err(e) = runtime::events::EventStore::append(&store, &dispatched_envelope) {
            tracing::warn!(slug, "job-dispatch: failed to persist job.dispatched: {e}");
        }
    }

    // Route through kernel
    let ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.route(dispatched_envelope, &ctx).await {
        tracing::warn!(slug, "job-dispatch: job.dispatched routing failed: {e}");
    }

    tracing::info!(slug, "job-dispatch: job dispatched successfully");
}

/// Send agent CLI command into the tmux session with SHIP_MESH_ID set.
fn spawn_agent_with_mesh_id(tmux_session: &str, slug: &str) {
    let cmd = format!(
        "SHIP_MESH_ID={slug} claude --dangerously-skip-permissions --dangerously-load-development-channels"
    );
    let result = std::process::Command::new("tmux")
        .args(["send-keys", "-t", tmux_session, &cmd, "Enter"])
        .status();
    if let Err(e) = result {
        tracing::warn!(session = tmux_session, "job-dispatch: tmux send-keys failed: {e}");
    }
}

async fn handle_job_update(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
) {
    let entity_id = &envelope.entity_id;
    let payload: serde_json::Value = match serde_json::from_str(&envelope.payload_json) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("job-dispatch: malformed job.update payload: {e}");
            return;
        }
    };

    // Forward to the agent's mailbox via mesh.send through the kernel.
    let slug = payload
        .get("slug")
        .and_then(|v| v.as_str())
        .unwrap_or(entity_id);

    let mesh_envelope = match EventEnvelope::new(
        "mesh.send",
        "service.job-dispatch",
        &serde_json::json!({
            "from": "service.job-dispatch",
            "to": slug,
            "body": {
                "type": "job.update",
                "job_id": entity_id,
                "payload": payload,
            }
        }),
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(entity_id, "job-dispatch: failed to build mesh.send: {e}");
            return;
        }
    };

    let ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    match kernel.lock().await.route(mesh_envelope, &ctx).await {
        Ok(_) => tracing::info!(entity_id, "job-dispatch: forwarded job.update to agent"),
        Err(e) => tracing::warn!(entity_id, "job-dispatch: mesh forward failed: {e}"),
    }
}
