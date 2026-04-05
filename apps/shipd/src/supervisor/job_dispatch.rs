//! Job dispatch subscriber — reacts to job lifecycle events from the kernel.
//!
//! When `job.created` flows through the kernel, creates a worktree, copies the
//! job spec, runs `ship use`, and spawns a terminal. When `job.update` arrives,
//! routes the message to the agent's mailbox via mesh. When `job.completed` or
//! `job.merged` arrives, cleans up tmux/worktree/branch resources.

use runtime::events::job::{JobCreatedPayload, PipelinePhase};
use runtime::events::{ActorConfig, CallerKind, EmitContext, EventEnvelope};
use runtime::projections::job::load_jobs;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::helpers::{
    compile_agent_config, create_worktree, delete_branch, ensure_tmux_session, kill_tmux_session,
    remove_worktree, worktrees_dir,
};
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
            // Check if this is a pipeline job with more phases
            if !try_advance_pipeline(kernel, envelope).await {
                // No more phases — clean up and unblock dependents
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

    dispatch_job(kernel, &payload).await;
}

/// Parse the spec file's YAML frontmatter and extract the pipeline field.
/// Returns the payload with pipeline set if found in the spec.
fn enrich_pipeline_from_spec(mut payload: JobCreatedPayload) -> JobCreatedPayload {
    if payload.pipeline.is_some() {
        return payload; // Already set by caller
    }

    let spec_content = read_spec_content(&payload.spec_path);
    if let Some(pipeline) = parse_pipeline_from_frontmatter(&spec_content) {
        if !pipeline.is_empty() {
            tracing::info!(
                slug = payload.slug,
                phases = pipeline.len(),
                "job-dispatch: pipeline found in spec frontmatter"
            );
            // Override agent with phase 0 agent
            if let Some(first) = pipeline.first() {
                payload.agent = first.agent.clone();
            }
            payload.pipeline = Some(pipeline);
        }
    }
    payload
}

/// Read spec content, trying absolute path then relative to project root.
fn read_spec_content(spec_path: &str) -> String {
    let src = std::path::Path::new(spec_path);
    if src.exists() {
        return std::fs::read_to_string(src).unwrap_or_default();
    }
    // Try relative to project root via get_project_dir
    if let Ok(ship_dir) = runtime::project::get_global_dir() {
        if let Some(root) = ship_dir.parent() {
            let alt = root.join(spec_path);
            if alt.exists() {
                return std::fs::read_to_string(&alt).unwrap_or_default();
            }
        }
    }
    String::new()
}

/// Extract pipeline phases from YAML frontmatter.
///
/// Expects:
/// ```yaml
/// pipeline:
///   - agent: test-writer
///     goal: Write failing tests
///   - agent: rust-runtime
///     goal: Make tests pass
/// ```
fn parse_pipeline_from_frontmatter(content: &str) -> Option<Vec<PipelinePhase>> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let yaml_str = &rest[..end];

    // Find the pipeline section and parse it
    // Use serde_yaml for proper parsing
    let fm: serde_json::Value = serde_yaml::from_str(yaml_str).ok()?;
    let pipeline_val = fm.get("pipeline")?;
    serde_json::from_value(pipeline_val.clone()).ok()
}

async fn dispatch_job(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    payload: &JobCreatedPayload,
) {
    let slug = &payload.slug;
    let branch = &payload.branch;
    let spec_path = &payload.spec_path;

    // Determine active agent — pipeline phase 0 overrides the top-level agent
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

    // 1. Create git worktree
    let worktree_path = worktrees_dir().join(slug);
    if let Err(e) = create_worktree(&worktree_path, branch, None) {
        tracing::error!(slug, "job-dispatch: worktree creation failed: {e}");
        return;
    }

    // 2. Copy spec + inject phase context
    let session_dir = worktree_path.join(".ship-session");
    if let Err(e) = std::fs::create_dir_all(&session_dir) {
        tracing::error!(slug, "job-dispatch: failed to create .ship-session: {e}");
        return;
    }
    write_phase_spec(&worktree_path, spec_path, &payload.pipeline, phase_idx);

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
    tracing::info!(slug, strategy, launched, "job-dispatch: terminal launch attempted");

    // 7. Emit job.dispatched
    emit_job_dispatched(kernel, payload, &worktree_path, slug).await;
}

async fn emit_job_dispatched(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    payload: &JobCreatedPayload,
    worktree_path: &std::path::Path,
    slug: &str,
) {
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

    if let Ok(store) = runtime::events::SqliteEventStore::new() {
        if let Err(e) = runtime::events::EventStore::append(&store, &dispatched_envelope) {
            tracing::warn!(slug, "job-dispatch: failed to persist job.dispatched: {e}");
        }
    }

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
/// Sends "1" + Enter after a delay to accept the development channels warning.
fn spawn_agent_with_mesh_id(tmux_session: &str, slug: &str) {
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

    // Accept the development channels warning prompt.
    // Send "1" + Enter at 5s and 8s to handle variable startup times.
    let session = tmux_session.to_string();
    std::thread::spawn(move || {
        for delay in [5, 3] {
            std::thread::sleep(std::time::Duration::from_secs(delay));
            let _ = std::process::Command::new("tmux")
                .args(["send-keys", "-t", &session, "1", "Enter"])
                .status();
        }
    });
}

/// Write the job spec to the worktree, optionally injecting pipeline phase context.
fn write_phase_spec(
    worktree_path: &std::path::Path,
    spec_path: &str,
    pipeline: &Option<Vec<PipelinePhase>>,
    phase_idx: Option<usize>,
) {
    let dst_spec = worktree_path.join(".ship-session").join("job-spec.md");

    // Read the original spec
    let src = std::path::Path::new(spec_path);
    let mut content = if src.exists() {
        std::fs::read_to_string(src).unwrap_or_default()
    } else {
        // Try relative to worktree parent (main repo)
        let parent = worktree_path.parent().and_then(|p| p.parent());
        if let Some(root) = parent {
            let alt = root.join(spec_path);
            std::fs::read_to_string(&alt).unwrap_or_default()
        } else {
            String::new()
        }
    };

    // Inject phase context if this is a pipeline job
    if let (Some(pipeline), Some(idx)) = (pipeline, phase_idx) {
        let total = pipeline.len();
        let current = &pipeline[idx];
        let mut phase_header = format!(
            "## Current Phase\n\nPhase {} of {}: {}\nAgent: {}\nGoal: {}\n",
            idx + 1, total, current.goal, current.agent, current.goal,
        );

        if idx > 0 {
            phase_header.push_str("\nPrior phases completed:\n");
            for (i, p) in pipeline[..idx].iter().enumerate() {
                phase_header.push_str(&format!("- Phase {} ({}): {}\n", i + 1, p.agent, p.goal));
            }
        }

        phase_header.push('\n');

        // Insert after frontmatter (after second ---)
        if let Some(end) = content.find("---\n").and_then(|first| {
            content[first + 4..].find("---\n").map(|second| first + 4 + second + 4)
        }) {
            content.insert_str(end, &phase_header);
        } else {
            content = format!("{phase_header}\n{content}");
        }
    }

    if let Err(e) = std::fs::write(&dst_spec, &content) {
        tracing::error!("job-dispatch: failed to write job-spec.md: {e}");
    }
}

/// Check if a completed job has a pipeline with more phases.
/// If so, dispatch the next phase in the same worktree. Returns true if advanced.
async fn try_advance_pipeline(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
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
    write_phase_spec(&worktree_path, &job.spec_path, &job.pipeline, Some(next));

    // Skip external process calls in test context
    if !cfg!(test) {
        // Re-run ship use with the new agent
        if let Err(e) = compile_agent_config(&worktree_path, &next_phase.agent) {
            tracing::error!(slug, agent = next_phase.agent, "job-dispatch: ship use failed for phase: {e}");
            return false;
        }

        // Re-dispatch in the existing tmux session
        let tmux_session = format!("job-{slug}");
        spawn_agent_with_mesh_id(&tmux_session, &slug);
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

/// Clean up worktree, tmux session, and branch for a completed/merged job.
async fn handle_job_cleanup(envelope: &EventEnvelope) {
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
async fn dispatch_unblocked_jobs(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    pending: &mut HashMap<String, JobCreatedPayload>,
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
            dispatch_job(kernel, &job).await;
        }
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

#[cfg(test)]
#[path = "tests_job_dispatch.rs"]
mod tests;
