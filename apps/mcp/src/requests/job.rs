use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct CreateJobRequest {
    /// Short identifier for the job (e.g. "auth-tests"). Must be unique and slug-safe.
    pub slug: String,
    /// Agent profile ID assigned to this job (e.g. "rust-runtime").
    pub agent: String,
    /// Git branch name for the job worktree (e.g. "job/auth-tests").
    pub branch: String,
    /// Relative path to the job spec file (e.g. ".ship-session/job-spec.md").
    pub spec_path: String,
    /// Optional reference to a plan that originated this job.
    pub plan_id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateJobRequest {
    /// Job ID returned by create_job.
    pub job_id: String,
    /// Target status: dispatched | gate_requested | gate_passed | gate_failed | blocked | merged | failed
    pub status: String,
    /// Worktree path — required when status is "dispatched".
    pub worktree: Option<String>,
    /// Blocker description — required when status is "blocked".
    pub blocker: Option<String>,
    /// Error message — required when status is "gate_failed" or "failed".
    pub error: Option<String>,
    /// OS process ID — optional, used with "dispatched".
    pub pid: Option<u32>,
    /// Gate agent ID — required when status is "gate_requested".
    pub gate_agent: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListJobsRequest {
    /// Optional status filter. Omit to return all jobs.
    pub status: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetJobRequest {
    /// Job ID to retrieve.
    pub job_id: String,
}
