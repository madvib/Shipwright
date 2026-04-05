//! Job dispatch subscriber — reacts to job lifecycle events from the kernel.
//!
//! When `job.created` flows through the kernel, prepares isolation and spawns
//! the agent via injected trait objects. When `job.update` arrives, routes the
//! message to the agent's mailbox via mesh. When `job.completed` or
//! `job.merged` arrives, cleans up resources.

use runtime::events::job::JobCreatedPayload;
use runtime::events::{ActorConfig, CallerKind, EmitContext, EventEnvelope};
use runtime::projections::job::load_jobs;
use runtime::services::dispatch_ports::{IsolationStrategy, JobContext, JobExecutor};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::adapters;
pub(crate) use super::dispatch_handlers::dispatch_unblocked_jobs;
use super::dispatch_handlers::{handle_job_cleanup, handle_job_update};
use super::dispatch_pipeline::{emit_job_dispatched, enrich_pipeline_from_spec};
use super::helpers::worktrees_dir;

/// Bundles the injected isolation and execution strategies for dispatch.
pub(crate) struct DispatchContext {
    pub isolation: Arc<dyn IsolationStrategy>,
    pub executor: Arc<dyn JobExecutor>,
}

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

    let dctx = Arc::new(DispatchContext {
        isolation: Arc::new(adapters::GitWorktreeIsolation),
        executor: Arc::new(adapters::TmuxExecutor),
    });

    let kr = kernel.clone();
    tokio::spawn(async move {
        let mut mb = mailbox;
        let mut pending: HashMap<String, JobCreatedPayload> = HashMap::new();
        while let Some(envelope) = mb.recv().await {
            handle_job_event(&kr, &envelope, &mut pending, &dctx).await;
        }
        tracing::info!("job-dispatch: mailbox closed");
    });
}

pub(crate) async fn handle_job_event(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    pending: &mut HashMap<String, JobCreatedPayload>,
    dctx: &DispatchContext,
) {
    match envelope.event_type.as_str() {
        "job.created" => handle_job_created(kernel, envelope, pending, dctx).await,
        "job.update" => handle_job_update(kernel, envelope).await,
        "job.completed" => {
            // Check if this is a pipeline job with more phases
            if !try_advance_pipeline(kernel, envelope, dctx).await {
                // No more phases — clean up and unblock dependents
                handle_job_cleanup(envelope).await;
                dispatch_unblocked_jobs(kernel, envelope, pending, dctx).await;
            }
        }
        "job.merged" => {
            handle_job_cleanup(envelope).await;
            dispatch_unblocked_jobs(kernel, envelope, pending, dctx).await;
        }
        _ => {}
    }
}

pub(crate) async fn handle_job_created(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    pending: &mut HashMap<String, JobCreatedPayload>,
    dctx: &DispatchContext,
) {
    let payload: JobCreatedPayload = match serde_json::from_str(&envelope.payload_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("job-dispatch: malformed job.created payload: {e}");
            return;
        }
    };

    // DAG-aware: defer if dependencies are unresolved
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

    // Enrich payload with pipeline from spec frontmatter if not already set
    let payload = enrich_pipeline_from_spec(payload);

    dispatch_job_internal(kernel, &payload, dctx).await;
}

pub(crate) async fn dispatch_job_internal(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    payload: &JobCreatedPayload,
    dctx: &DispatchContext,
) {
    let slug = &payload.slug;
    let branch = &payload.branch;

    // Determine active agent — pipeline phase 0 overrides the top-level agent
    let agent = if let Some(ref pipeline) = payload.pipeline {
        pipeline.first().map(|p| p.agent.as_str()).unwrap_or(payload.agent.as_str())
    } else {
        payload.agent.as_str()
    };

    tracing::info!(slug, branch, agent, "job-dispatch: dispatching job");

    let job_ctx = JobContext {
        job_id: payload.job_id.clone(),
        slug: slug.clone(),
        agent: agent.to_string(),
        branch: branch.clone(),
        spec_path: payload.spec_path.clone(),
        work_dir: worktrees_dir().join(slug),
        env: [("SHIP_MESH_ID".to_string(), slug.to_string())].into(),
        model: payload.model.clone(),
        provider: payload.provider.clone(),
    };

    // 1. Prepare isolation (worktree + spec + agent config)
    let worktree_path = match dctx.isolation.prepare(&job_ctx).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: isolation prepare failed: {e}");
            return;
        }
    };

    // 2. Spawn agent process
    let job_ctx_with_dir = JobContext { work_dir: worktree_path.clone(), ..job_ctx };
    if let Err(e) = dctx.executor.spawn(&job_ctx_with_dir).await {
        tracing::error!(slug, "job-dispatch: executor spawn failed: {e}");
        return;
    }

    // 3. Emit job.dispatched
    emit_job_dispatched(kernel, payload, &worktree_path, slug).await;
}

/// Check if a completed job has a pipeline with more phases.
/// If so, dispatch the next phase in the same worktree. Returns true if advanced.
pub(crate) async fn try_advance_pipeline(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    dctx: &DispatchContext,
) -> bool {
    let payload: serde_json::Value = match serde_json::from_str(&envelope.payload_json) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let slug = match payload["slug"].as_str() {
        Some(s) => s.to_string(),
        None => return false,
    };

    // Load current job state to check pipeline
    let jobs = match load_jobs() {
        Ok(j) => j,
        Err(_) => return false,
    };

    let job = match jobs.values().find(|j| j.slug == slug) {
        Some(j) => j.clone(),
        None => return false,
    };

    let pipeline = match &job.pipeline {
        Some(p) if p.len() > 1 => p,
        _ => return false,
    };

    // Determine current phase (default 0 if not tracked)
    let current = job.current_phase.unwrap_or(0);
    let next = current + 1;

    if next >= pipeline.len() {
        tracing::info!(slug, "job-dispatch: pipeline complete (all {} phases done)", pipeline.len());
        return false; // All phases done — let normal completion flow handle it
    }

    let next_phase = &pipeline[next];
    tracing::info!(
        slug,
        phase = next,
        agent = next_phase.agent,
        goal = next_phase.goal,
        "job-dispatch: advancing pipeline to next phase"
    );

    let worktree_path = worktrees_dir().join(&slug);
    if !worktree_path.exists() {
        tracing::error!(slug, "job-dispatch: worktree missing for pipeline advance");
        return false;
    }

    // Rewrite the spec with next phase context
    adapters::write_phase_spec(&worktree_path, &job.spec_path, &job.pipeline, Some(next));

    // Spawn the next phase via executor
    let job_ctx = JobContext {
        job_id: job.job_id.clone(),
        slug: slug.clone(),
        agent: next_phase.agent.clone(),
        branch: job.branch.clone(),
        spec_path: job.spec_path.clone(),
        work_dir: worktree_path.clone(),
        env: [("SHIP_MESH_ID".to_string(), slug.clone())].into(),
        model: None,
        provider: None,
    };
    if let Err(e) = dctx.executor.spawn(&job_ctx).await {
        tracing::error!(slug, "job-dispatch: executor spawn failed for phase: {e}");
        return false;
    }

    // Emit job.dispatched for the new phase
    // (Re-use the existing payload but the worktree stays the same)
    let dispatched_payload = runtime::events::job::JobDispatchedPayload {
        job_id: job.job_id.clone(),
        worktree: worktree_path.to_string_lossy().to_string(),
        pid: None,
    };
    let dispatched_envelope = match EventEnvelope::new(
        runtime::events::job::event_types::JOB_DISPATCHED,
        &job.job_id,
        &dispatched_payload,
    ) {
        Ok(e) => e,
        Err(_) => return true,
    };
    let ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    let _ = kernel.lock().await.route(dispatched_envelope, &ctx).await;

    true
}

#[cfg(test)]
#[path = "tests_job_dispatch.rs"]
mod tests;
