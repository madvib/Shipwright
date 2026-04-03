//! MCP tool handlers for job lifecycle management.
//!
//! Jobs are event-sourced. These handlers emit the appropriate `job.*` event
//! and return the result. State is read back via the projection in `load_jobs`.

use runtime::events::job::{
    JobBlockedPayload, JobCreatedPayload, JobDispatchedPayload, JobFailedPayload,
    JobGateFailedPayload, JobGatePassedPayload, JobGateRequestedPayload, JobMergedPayload,
    event_types,
};
use runtime::events::{EventEnvelope, EventStore, SqliteEventStore};
use runtime::projections::job::{JobStatus, load_jobs};

use crate::requests::{CreateJobRequest, GetJobRequest, ListJobsRequest, UpdateJobRequest};

/// Emit `job.created` and return the new job_id as a JSON string.
///
/// Persists to the event store AND routes through the KernelRouter so
/// daemon subscribers (job-dispatch) see the event.
pub async fn create_job(req: CreateJobRequest) -> String {
    let job_id = runtime::gen_ulid();
    let payload = JobCreatedPayload {
        job_id: job_id.clone(),
        slug: req.slug,
        agent: req.agent,
        branch: req.branch,
        spec_path: req.spec_path,
        plan_id: req.plan_id,
    };
    let envelope = match EventEnvelope::new(event_types::JOB_CREATED, &job_id, &payload) {
        Ok(e) => e,
        Err(e) => return format!("Error building event: {e}"),
    };
    let store = match SqliteEventStore::new() {
        Ok(s) => s,
        Err(e) => return format!("Error opening event store: {e}"),
    };
    if let Err(e) = store.append(&envelope) {
        return format!("Error persisting job.created: {e}");
    }

    // Route through kernel for subscriber delivery (daemon job-dispatch).
    if let Some(kr) = runtime::events::kernel_router() {
        let ctx = runtime::events::EmitContext {
            caller_kind: runtime::events::CallerKind::Mcp,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        };
        if let Err(e) = kr.lock().await.route(envelope, &ctx).await {
            tracing::warn!("job.created kernel routing failed: {e}");
        }
    }

    serde_json::json!({"job_id": job_id}).to_string()
}

/// Emit the appropriate event for the requested status transition.
pub fn update_job(req: UpdateJobRequest) -> String {
    let store = match SqliteEventStore::new() {
        Ok(s) => s,
        Err(e) => return format!("Error opening event store: {e}"),
    };

    let envelope_result = match req.status.as_str() {
        "dispatched" => {
            let worktree = match req.worktree {
                Some(w) => w,
                None => return "Error: worktree is required for status 'dispatched'".to_string(),
            };
            EventEnvelope::new(
                event_types::JOB_DISPATCHED,
                &req.job_id,
                &JobDispatchedPayload {
                    job_id: req.job_id.clone(),
                    worktree,
                    pid: req.pid,
                },
            )
        }
        "gate_requested" => {
            let gate_agent = match req.gate_agent {
                Some(g) => g,
                None => return "Error: gate_agent is required for status 'gate_requested'".to_string(),
            };
            EventEnvelope::new(
                event_types::JOB_GATE_REQUESTED,
                &req.job_id,
                &JobGateRequestedPayload {
                    job_id: req.job_id.clone(),
                    gate_agent,
                },
            )
        }
        "gate_passed" => EventEnvelope::new(
            event_types::JOB_GATE_PASSED,
            &req.job_id,
            &JobGatePassedPayload {
                job_id: req.job_id.clone(),
            },
        ),
        "gate_failed" => {
            let reason = match req.error {
                Some(r) => r,
                None => return "Error: error is required for status 'gate_failed'".to_string(),
            };
            EventEnvelope::new(
                event_types::JOB_GATE_FAILED,
                &req.job_id,
                &JobGateFailedPayload {
                    job_id: req.job_id.clone(),
                    reason,
                },
            )
        }
        "blocked" => {
            let blocker = match req.blocker {
                Some(b) => b,
                None => return "Error: blocker is required for status 'blocked'".to_string(),
            };
            EventEnvelope::new(
                event_types::JOB_BLOCKED,
                &req.job_id,
                &JobBlockedPayload {
                    job_id: req.job_id.clone(),
                    blocker,
                    needs_human: true,
                },
            )
        }
        "merged" => EventEnvelope::new(
            event_types::JOB_MERGED,
            &req.job_id,
            &JobMergedPayload {
                job_id: req.job_id.clone(),
            },
        ),
        "failed" => {
            let error = match req.error {
                Some(e) => e,
                None => return "Error: error is required for status 'failed'".to_string(),
            };
            EventEnvelope::new(
                event_types::JOB_FAILED,
                &req.job_id,
                &JobFailedPayload {
                    job_id: req.job_id.clone(),
                    error,
                },
            )
        }
        other => {
            return format!(
                "Error: unknown status '{other}'. Valid values: \
                dispatched, gate_requested, gate_passed, gate_failed, blocked, merged, failed"
            );
        }
    };

    let envelope = match envelope_result {
        Ok(e) => e,
        Err(e) => return format!("Error building event: {e}"),
    };
    if let Err(e) = store.append(&envelope) {
        return format!("Error persisting event: {e}");
    }
    format!("Job {} updated to status '{}'", req.job_id, req.status)
}

/// Return all jobs, optionally filtered by status.
pub fn list_jobs(req: ListJobsRequest) -> String {
    let jobs = match load_jobs() {
        Ok(j) => j,
        Err(e) => return format!("Error loading jobs: {e}"),
    };

    let mut records: Vec<_> = jobs.into_values().collect();

    if let Some(status_filter) = req.status {
        let target = match status_filter.as_str() {
            "pending" => Some(JobStatus::Pending),
            "dispatched" => Some(JobStatus::Dispatched),
            "gate_pending" => Some(JobStatus::GatePending),
            "blocked" => Some(JobStatus::Blocked),
            "merged" => Some(JobStatus::Merged),
            "failed" => Some(JobStatus::Failed),
            other => {
                return format!(
                    "Error: unknown status filter '{other}'. Valid values: \
                    pending, dispatched, gate_pending, blocked, merged, failed"
                );
            }
        };
        if let Some(t) = target {
            records.retain(|r| r.status == t);
        }
    }

    records.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    match serde_json::to_string_pretty(&records) {
        Ok(json) => json,
        Err(e) => format!("Error serializing jobs: {e}"),
    }
}

/// Return a single job record by job_id.
pub fn get_job(req: GetJobRequest) -> String {
    let jobs = match load_jobs() {
        Ok(j) => j,
        Err(e) => return format!("Error loading jobs: {e}"),
    };

    match jobs.get(&req.job_id) {
        Some(record) => match serde_json::to_string_pretty(record) {
            Ok(json) => json,
            Err(e) => format!("Error serializing job: {e}"),
        },
        None => format!("Error: job '{}' not found", req.job_id),
    }
}
