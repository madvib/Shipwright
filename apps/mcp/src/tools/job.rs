use std::path::Path;

use crate::requests::{AppendJobLogRequest, CreateJobRequest, ListJobsRequest, UpdateJobRequest};

pub fn create_job(_project_dir: &Path, req: CreateJobRequest) -> String {
    let mut payload = serde_json::Map::new();
    payload.insert(
        "description".to_string(),
        serde_json::Value::String(req.description),
    );
    if let Some(ref rw) = req.requesting_workspace {
        payload.insert(
            "requesting_workspace".to_string(),
            serde_json::Value::String(rw.clone()),
        );
    }
    if let Some(ref cap_id) = req.capability_id {
        payload.insert(
            "capability_id".to_string(),
            serde_json::Value::String(cap_id.clone()),
        );
    }
    if let Some(ref scope) = req.scope {
        payload.insert("scope".to_string(), serde_json::json!(scope));
    }
    if let Some(ref criteria) = req.acceptance_criteria {
        payload.insert(
            "acceptance_criteria".to_string(),
            serde_json::json!(criteria),
        );
    }
    if let Some(ref sym) = req.symlink_name {
        payload.insert(
            "symlink_name".to_string(),
            serde_json::Value::String(sym.clone()),
        );
    }
    if let Some(ref fs) = req.file_scope {
        payload.insert("file_scope".to_string(), serde_json::json!(fs));
    }
    let capability_id = req.capability_id.clone();
    match runtime::db::jobs::create_job(
        &req.kind,
        req.branch.as_deref(),
        Some(serde_json::Value::Object(payload)),
        req.requesting_workspace.as_deref(),
        req.assigned_to.as_deref(),
        req.priority.unwrap_or(0),
        req.blocked_by.as_deref(),
        req.touched_files.unwrap_or_default(),
        req.file_scope.unwrap_or_default(),
    ) {
        Ok(job) => {
            if let Some(cap_id) = capability_id {
                let _ = runtime::db::jobs::update_job(
                    &job.id,
                    runtime::db::jobs::JobPatch {
                        capability_id: Some(cap_id),
                        ..Default::default()
                    },
                );
            }
            format!("Created job {} (kind={}, status=pending)", job.id, job.kind)
        }
        Err(e) => format!("Error creating job: {}", e),
    }
}

pub fn update_job(_project_dir: &Path, req: UpdateJobRequest) -> String {
    let patch = runtime::db::jobs::JobPatch {
        status: req.status.clone(),
        assigned_to: req.assigned_to,
        priority: req.priority,
        blocked_by: req.blocked_by,
        touched_files: req.touched_files,
        file_scope: None,
        capability_id: None,
    };
    match runtime::db::jobs::update_job(&req.id, patch) {
        Ok(()) => {
            let status_msg = req.status.as_deref().unwrap_or("(unchanged)");
            format!("Job {} updated (status={})", req.id, status_msg)
        }
        Err(e) => format!("Error updating job: {}", e),
    }
}

pub fn list_jobs(_project_dir: &Path, req: ListJobsRequest) -> String {
    match runtime::db::jobs::list_jobs(req.branch.as_deref(), req.status.as_deref()) {
        Ok(jobs) if jobs.is_empty() => "No jobs found.".to_string(),
        Ok(jobs) => {
            let mut out = String::from("Jobs:\n");
            for j in &jobs {
                out.push_str(&format!(
                    "- {} [{}] kind={} priority={} created={}\n",
                    j.id, j.status, j.kind, j.priority, j.created_at
                ));
                if let Some(ref branch) = j.branch {
                    out.push_str(&format!("  branch={}\n", branch));
                }
                if let Some(ref a) = j.assigned_to {
                    out.push_str(&format!("  assigned_to={}\n", a));
                }
                if let Some(ref b) = j.blocked_by {
                    out.push_str(&format!("  blocked_by={}\n", b));
                }
                if !j.touched_files.is_empty() {
                    out.push_str(&format!("  files={}\n", j.touched_files.join(", ")));
                }
            }
            out
        }
        Err(e) => format!("Error listing jobs: {}", e),
    }
}

pub fn append_job_log(_project_dir: &Path, req: AppendJobLogRequest) -> String {
    if let Some(path) = req.message.strip_prefix("touched: ") {
        let path = path.trim();
        if !path.is_empty()
            && let Err(e) = runtime::db::jobs::append_touched_file(&req.job_id, path)
        {
            return format!("Error recording touched file: {}", e);
        }
    }
    let message = if let Some(ref level) = req.level {
        format!("[{}] {}", level.to_ascii_uppercase(), req.message)
    } else {
        req.message.clone()
    };
    use runtime::events::store::{EventStore, SqliteEventStore};
    use runtime::events::types::{ProjectLog, event_types};
    match SqliteEventStore::new().and_then(|store| {
        let envelope = runtime::EventEnvelope::new(
            event_types::PROJECT_LOG,
            &req.job_id,
            &ProjectLog {
                action: "job.log".to_string(),
                details: message,
            },
        )?;
        store.append(&envelope)?;
        Ok(())
    }) {
        Ok(()) => format!("Log entry appended to job {}", req.job_id),
        Err(e) => format!("Error appending job log: {}", e),
    }
}

pub fn claim_file(_project_dir: &Path, job_id: &str, path: &str) -> String {
    match runtime::db::jobs::claim_file(job_id, path) {
        Ok(true) => format!("Claimed {} for job {}", path, job_id),
        Ok(false) => {
            let owner = runtime::db::jobs::get_file_owner(path)
                .unwrap_or(None)
                .unwrap_or_else(|| "unknown".to_string());
            format!("Conflict: {} is already owned by job {}", path, owner)
        }
        Err(e) => format!("Error claiming file: {}", e),
    }
}

pub fn get_file_owner(_project_dir: &Path, path: &str) -> String {
    match runtime::db::jobs::get_file_owner(path) {
        Ok(None) => format!("{}: unclaimed", path),
        Ok(Some(job_id)) => {
            let detail = runtime::db::jobs::get_job(&job_id)
                .ok()
                .flatten()
                .map(|j| format!(" (kind={}, status={})", j.kind, j.status))
                .unwrap_or_default();
            format!("{}: owned by job {}{}", path, job_id, detail)
        }
        Err(e) => format!("Error: {}", e),
    }
}
