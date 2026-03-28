use serde::{Deserialize, Serialize};

// ── Workspace aggregate ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceActivated {
    pub agent_id: Option<String>,
    pub providers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceCompiled {
    pub config_generation: u32,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceCompileFailed {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceArchived {}

// ── Session aggregate ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStarted {
    pub goal: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionProgress {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionEnded {
    pub summary: Option<String>,
    pub duration_secs: Option<u64>,
    pub gate_result: Option<String>,
}

// ── Skill aggregate ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillStarted {
    pub skill_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillCompleted {
    pub skill_id: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillFailed {
    pub skill_id: String,
    pub error: String,
}

// ── Actor aggregate (v0.2.0 kernel) ──────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorCreated {
    pub kind: String,
    pub environment_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorWoke {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorSlept {
    pub idle_secs: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorStopped {
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorCrashed {
    pub error: String,
    pub restart_count: u32,
}

// ── Gate aggregate ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct GatePassed {
    pub evidence: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GateFailed {
    pub evidence: String,
}

// ── Job aggregate ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct JobCreated {
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobClaimed {
    pub agent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobCompleted {
    pub summary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobFailed {
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobDispatched {
    pub agent_id: Option<String>,
}

// ── Config / Log aggregates ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigChanged {
    pub subject: String,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectLog {
    pub action: String,
    pub details: String,
}

// ── Event type constants ──────────────────────────────────────────────────────

pub mod event_types {
    pub const WORKSPACE_ACTIVATED: &str = "workspace.activated";
    pub const WORKSPACE_COMPILED: &str = "workspace.compiled";
    pub const WORKSPACE_COMPILE_FAILED: &str = "workspace.compile_failed";
    pub const WORKSPACE_ARCHIVED: &str = "workspace.archived";
    pub const SESSION_STARTED: &str = "session.started";
    pub const SESSION_PROGRESS: &str = "session.progress";
    pub const SESSION_ENDED: &str = "session.ended";
    pub const ACTOR_CREATED: &str = "actor.created";
    pub const ACTOR_WOKE: &str = "actor.woke";
    pub const ACTOR_SLEPT: &str = "actor.slept";
    pub const ACTOR_STOPPED: &str = "actor.stopped";
    pub const ACTOR_CRASHED: &str = "actor.crashed";
    pub const GATE_PASSED: &str = "gate.passed";
    pub const GATE_FAILED: &str = "gate.failed";
    pub const JOB_CREATED: &str = "job.created";
    pub const JOB_CLAIMED: &str = "job.claimed";
    pub const JOB_COMPLETED: &str = "job.completed";
    pub const JOB_FAILED: &str = "job.failed";
    pub const JOB_DISPATCHED: &str = "job.dispatched";
    pub const CONFIG_CHANGED: &str = "config.changed";
    pub const PROJECT_LOG: &str = "project.log";

    pub const SKILL_STARTED: &str = "skill.started";
    pub const SKILL_COMPLETED: &str = "skill.completed";
    pub const SKILL_FAILED: &str = "skill.failed";

    pub const ALL: &[&str] = &[
        WORKSPACE_ACTIVATED,
        WORKSPACE_COMPILED,
        WORKSPACE_COMPILE_FAILED,
        WORKSPACE_ARCHIVED,
        SESSION_STARTED,
        SESSION_PROGRESS,
        SESSION_ENDED,
        ACTOR_CREATED,
        ACTOR_WOKE,
        ACTOR_SLEPT,
        ACTOR_STOPPED,
        ACTOR_CRASHED,
        GATE_PASSED,
        GATE_FAILED,
        JOB_CREATED,
        JOB_CLAIMED,
        JOB_COMPLETED,
        JOB_FAILED,
        JOB_DISPATCHED,
        CONFIG_CHANGED,
        PROJECT_LOG,
        SKILL_STARTED,
        SKILL_COMPLETED,
        SKILL_FAILED,
    ];
}

#[cfg(test)]
mod tests {
    use super::event_types::*;

    #[test]
    fn event_type_constants_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for &t in ALL {
            assert!(seen.insert(t), "duplicate event type constant: {t}");
        }
    }

    #[test]
    fn all_constants_have_expected_count() {
        assert_eq!(ALL.len(), 24, "exactly 24 event type constants required");
    }
}
