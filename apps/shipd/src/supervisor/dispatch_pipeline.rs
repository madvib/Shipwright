//! Pipeline enrichment and spec parsing for job dispatch.

use runtime::events::job::{JobCreatedPayload, PipelinePhase};
use runtime::events::{CallerKind, EmitContext, EventEnvelope};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Parse the spec file's YAML frontmatter and extract the pipeline field.
/// Returns the payload with pipeline set if found in the spec.
pub(crate) fn enrich_pipeline_from_spec(
    mut payload: JobCreatedPayload,
) -> JobCreatedPayload {
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

/// Read spec content, trying absolute path then relative to project root.
fn read_spec_content(spec_path: &str) -> String {
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

/// Extract pipeline phases from YAML frontmatter.
fn parse_pipeline_from_frontmatter(content: &str) -> Option<Vec<PipelinePhase>> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let yaml_str = &rest[..end];

    let fm: serde_json::Value = serde_yaml::from_str(yaml_str).ok()?;
    let pipeline_val = fm.get("pipeline")?;
    serde_json::from_value(pipeline_val.clone()).ok()
}

/// Emit a `job.dispatched` event, persisting it and routing through the kernel.
pub(crate) async fn emit_job_dispatched(
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
