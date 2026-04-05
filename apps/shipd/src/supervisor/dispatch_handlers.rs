//! Secondary event handlers for job dispatch — update forwarding, cleanup, DAG unblocking.

use runtime::events::job::JobCreatedPayload;
use runtime::events::{CallerKind, EmitContext, EventEnvelope};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::helpers::{delete_branch, kill_tmux_session, remove_worktree, worktrees_dir};
use super::job_dispatch::DispatchContext;

/// Forward `job.update` messages to the agent via mesh routing.
pub(crate) async fn handle_job_update(
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

/// Clean up worktree, tmux session, and branch for a completed/merged job.
pub(crate) async fn handle_job_cleanup(envelope: &EventEnvelope) {
    let payload: serde_json::Value = match serde_json::from_str(&envelope.payload_json) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("job-dispatch: malformed cleanup payload: {e}");
            return;
        }
    };

    let job_id = payload["job_id"].as_str().unwrap_or(&envelope.entity_id);
    let slug = payload["slug"]
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| job_id.to_string());

    let tmux_session = format!("job-{slug}");
    let worktree_path = worktrees_dir().join(&slug);
    let branch = format!("job/{slug}");

    tracing::info!(slug, "job-dispatch: cleaning up job resources");

    if let Err(e) = kill_tmux_session(&tmux_session) {
        tracing::warn!(slug, "job-dispatch: tmux cleanup failed: {e}");
    }
    if let Err(e) = remove_worktree(&worktree_path) {
        tracing::warn!(slug, "job-dispatch: worktree cleanup failed: {e}");
    }
    if let Err(e) = delete_branch(&branch) {
        tracing::warn!(slug, "job-dispatch: branch cleanup failed: {e}");
    }

    tracing::info!(slug, "job-dispatch: cleanup complete");
}

/// After a job completes/merges, check if any pending jobs had it as a dependency.
/// Dispatch any that become fully unblocked.
pub(crate) async fn dispatch_unblocked_jobs(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    pending: &mut HashMap<String, JobCreatedPayload>,
    dctx: &DispatchContext,
) {
    let payload: serde_json::Value = match serde_json::from_str(&envelope.payload_json) {
        Ok(v) => v,
        Err(_) => return,
    };

    let completed_slug = payload["slug"]
        .as_str()
        .or_else(|| payload["job_id"].as_str())
        .unwrap_or(&envelope.entity_id);

    let mut ready: Vec<String> = Vec::new();
    for (slug, job) in pending.iter_mut() {
        if let Some(deps) = &mut job.depends_on {
            deps.retain(|d| d != completed_slug);
            if deps.is_empty() {
                ready.push(slug.clone());
            }
        }
    }

    for slug in ready {
        if let Some(mut job) = pending.remove(&slug) {
            tracing::info!(slug, "job-dispatch: dependencies cleared, dispatching deferred job");
            job.depends_on = None;
            super::job_dispatch::dispatch_job_internal(kernel, &job, dctx).await;
        }
    }
}
