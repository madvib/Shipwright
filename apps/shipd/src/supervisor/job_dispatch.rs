//! Job dispatch subscriber — reacts to job lifecycle events from the kernel.
//!
//! When `job.created` flows through the kernel, creates a worktree, copies the
//! job spec, runs `ship use`, and spawns a terminal. When `job.update` arrives,
//! routes the message to the agent's mailbox via mesh. When `job.completed` or
//! `job.merged` arrives, cleans up tmux/worktree/branch resources.

use runtime::events::job::JobCreatedPayload;
use runtime::events::{ActorConfig, CallerKind, EmitContext, EventEnvelope};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::helpers::{
    compile_agent_config, create_worktree, ensure_tmux_session, worktrees_dir,
};
use super::job_pipeline::{
    dispatch_unblocked_jobs, enrich_pipeline_from_spec, handle_job_cleanup, handle_job_update,
    try_advance_pipeline, write_phase_spec,
};
use super::terminal_launcher;

/// Subscribe to `job.*` kernel events and dispatch jobs to worktrees.
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
        let mut pending: HashMap<String, JobCreatedPayload> = HashMap::new();
        while let Some(envelope) = mb.recv().await {
            handle_job_event(&kr, &envelope, &mut pending).await;
        }
        tracing::info!("job-dispatch: mailbox closed");
    });
}

async fn handle_job_event(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    pending: &mut HashMap<String, JobCreatedPayload>,
) {
    match envelope.event_type.as_str() {
        "job.created" => handle_job_created(kernel, envelope, pending).await,
        "job.update" => handle_job_update(kernel, envelope).await,
        "job.completed" => {
            if !try_advance_pipeline(kernel, envelope).await {
                handle_job_cleanup(envelope).await;
                dispatch_unblocked_jobs(kernel, envelope, pending).await;
            }
        }
        "job.merged" => {
            handle_job_cleanup(envelope).await;
            dispatch_unblocked_jobs(kernel, envelope, pending).await;
        }
        _ => {}
    }
}

async fn handle_job_created(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    pending: &mut HashMap<String, JobCreatedPayload>,
) {
    let payload: JobCreatedPayload = match serde_json::from_str(&envelope.payload_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("job-dispatch: malformed job.created payload: {e}");
            return;
        }
    };

    if let Some(deps) = &payload.depends_on {
        if !deps.is_empty() {
            tracing::info!(
                slug = payload.slug,
                deps = ?deps,
                "job-dispatch: deferring job until dependencies complete"
            );
            pending.insert(payload.slug.clone(), payload);
            return;
        }
    }

    let payload = enrich_pipeline_from_spec(payload);
    dispatch_job(kernel, &payload).await;
}

pub(super) async fn dispatch_job(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    payload: &JobCreatedPayload,
) {
    let slug = &payload.slug;
    let branch = &payload.branch;

    let (agent, phase_idx) = if let Some(ref pipeline) = payload.pipeline {
        if let Some(phase) = pipeline.first() {
            (phase.agent.as_str(), Some(0usize))
        } else {
            (payload.agent.as_str(), None)
        }
    } else {
        (payload.agent.as_str(), None)
    };

    tracing::info!(slug, branch, agent, phase = ?phase_idx, "job-dispatch: dispatching job");

    let worktree_path = worktrees_dir().join(slug);
    if let Err(e) = create_worktree(&worktree_path, branch, None) {
        tracing::error!(slug, "job-dispatch: worktree creation failed: {e}");
        return;
    }

    let session_dir = worktree_path.join(".ship-session");
    if let Err(e) = std::fs::create_dir_all(&session_dir) {
        tracing::error!(slug, "job-dispatch: failed to create .ship-session: {e}");
        return;
    }
    write_phase_spec(&worktree_path, &payload.spec_path, &payload.pipeline, phase_idx);

    if let Err(e) = compile_agent_config(&worktree_path, agent) {
        tracing::error!(slug, agent, "job-dispatch: ship use failed: {e}");
        return;
    }

    let tmux_session = format!("job-{slug}");
    if let Err(e) = ensure_tmux_session(&tmux_session, &worktree_path) {
        tracing::error!(slug, "job-dispatch: tmux session creation failed: {e}");
        return;
    }

    spawn_agent_with_mesh_id(&tmux_session, slug);

    let (strategy, launched) = terminal_launcher::launch(&tmux_session);
    tracing::info!(slug, strategy, launched, "job-dispatch: terminal launch attempted");

    emit_job_dispatched(kernel, payload, &worktree_path, slug).await;
}

async fn emit_job_dispatched(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    payload: &JobCreatedPayload,
    worktree_path: &std::path::Path,
    slug: &str,
) {
    let dispatched = runtime::events::job::JobDispatchedPayload {
        job_id: payload.job_id.clone(),
        worktree: worktree_path.to_string_lossy().into_owned(),
        pid: None,
    };
    let envelope = match EventEnvelope::new(
        runtime::events::job::event_types::JOB_DISPATCHED,
        &payload.job_id,
        &dispatched,
    ) {
        Ok(e) => e.with_actor_id("service.job-dispatch"),
        Err(e) => {
            tracing::warn!(slug, "job-dispatch: failed to build job.dispatched: {e}");
            return;
        }
    };

    if let Ok(store) = runtime::events::SqliteEventStore::new() {
        if let Err(e) = runtime::events::EventStore::append(&store, &envelope) {
            tracing::warn!(slug, "job-dispatch: failed to persist job.dispatched: {e}");
        }
    }

    let ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.route(envelope, &ctx).await {
        tracing::warn!(slug, "job-dispatch: job.dispatched routing failed: {e}");
    }

    tracing::info!(slug, "job-dispatch: job dispatched successfully");
}

/// Send agent CLI command into the tmux session with SHIP_MESH_ID set.
pub(super) fn spawn_agent_with_mesh_id(tmux_session: &str, slug: &str) {
    let cmd = format!(
        "SHIP_MESH_ID={slug} claude --dangerously-skip-permissions --dangerously-load-development-channels server:ship"
    );
    let result = std::process::Command::new("tmux")
        .args(["send-keys", "-t", tmux_session, &cmd, "Enter"])
        .status();
    if let Err(e) = result {
        tracing::warn!(session = tmux_session, "job-dispatch: tmux send-keys failed: {e}");
        return;
    }

    let session = tmux_session.to_string();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let _ = std::process::Command::new("tmux")
            .args(["send-keys", "-t", &session, "Enter"])
            .status();
        std::thread::sleep(std::time::Duration::from_secs(5));
        let _ = std::process::Command::new("tmux")
            .args(["send-keys", "-t", &session, "go", "Enter"])
            .status();
    });
}

#[cfg(test)]
#[path = "tests_job_dispatch.rs"]
mod tests;
