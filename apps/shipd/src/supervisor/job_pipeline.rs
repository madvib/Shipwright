//! Pipeline phase management and ancillary job event handlers.

use runtime::events::job::{JobCreatedPayload, PipelinePhase};
use std::collections::HashMap;
use runtime::events::{CallerKind, EmitContext, EventEnvelope};
use runtime::projections::job::load_jobs;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::helpers::{
    compile_agent_config, delete_branch, kill_tmux_session, remove_worktree, worktrees_dir,
};
use super::job_dispatch::{dispatch_job, spawn_agent_with_mesh_id};

/// Enrich job payload with pipeline from spec frontmatter if not already set.
pub fn enrich_pipeline_from_spec(mut payload: JobCreatedPayload) -> JobCreatedPayload {
    if payload.pipeline.is_some() {
        return payload;
    }

    let spec_content = read_spec_content(&payload.spec_path);
    if let Some(pipeline) = parse_pipeline_from_frontmatter(&spec_content) {
        if !pipeline.is_empty() {
            tracing::info!(
                slug = payload.slug,
                phases = pipeline.len(),
                "job-dispatch: pipeline found in spec frontmatter"
            );
            if let Some(first) = pipeline.first() {
                payload.agent = first.agent.clone();
            }
            payload.pipeline = Some(pipeline);
        }
    }
    payload
}

pub fn read_spec_content(spec_path: &str) -> String {
    let src = std::path::Path::new(spec_path);
    if src.exists() {
        return std::fs::read_to_string(src).unwrap_or_default();
    }
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

pub fn parse_pipeline_from_frontmatter(content: &str) -> Option<Vec<PipelinePhase>> {
    let content = content.trim();
    if !content.starts_with("---") { return None; }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let yaml_str = &rest[..end];

    let fm: serde_json::Value = serde_yaml::from_str(yaml_str).ok()?;
    serde_json::from_value(fm.get("pipeline")?.clone()).ok()
}

pub fn write_phase_spec(
    worktree_path: &std::path::Path,
    spec_path: &str,
    pipeline: &Option<Vec<PipelinePhase>>,
    phase_idx: Option<usize>,
) {
    let dst_spec = worktree_path.join(".ship-session").join("job-spec.md");
    let mut content = read_spec_content(spec_path);

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

/// Advance to the next pipeline phase if one exists. Returns true if advanced.
pub async fn try_advance_pipeline(
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

    let current = job.current_phase.unwrap_or(0);
    let next = current + 1;

    if next >= pipeline.len() {
        tracing::info!(slug, "job-dispatch: pipeline complete (all {} phases done)", pipeline.len());
        return false;
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

    write_phase_spec(&worktree_path, &job.spec_path, &job.pipeline, Some(next));

    if let Err(e) = compile_agent_config(&worktree_path, &next_phase.agent) {
        tracing::error!(slug, agent = next_phase.agent, "job-dispatch: ship use failed for phase: {e}");
        return false;
    }

    let tmux_session = format!("job-{slug}");
    spawn_agent_with_mesh_id(&tmux_session, &slug);

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

pub async fn handle_job_cleanup(envelope: &EventEnvelope) {
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

    let _ = kill_tmux_session(&tmux_session);
    let _ = remove_worktree(&worktree_path);
    let _ = delete_branch(&branch);
    tracing::info!(slug, "job-dispatch: cleanup complete");
}

pub async fn dispatch_unblocked_jobs(
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

pub async fn handle_job_update(
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
