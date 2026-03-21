//! Schema DDL for jobs, notes, ADRs, targets, and capabilities.

/// Job: a unit of work in the agent queue.
/// Status lifecycle: pending -> running -> complete | failed | done.
/// Links to capabilities via capability_id. File ownership tracked in job_file.
pub const JOB: &str = r#"
-- job: queued unit of work for agent coordination.
-- Status: pending -> running -> complete | failed | done.
CREATE TABLE IF NOT EXISTS job (
  id            TEXT PRIMARY KEY,
  kind          TEXT NOT NULL,
  status        TEXT NOT NULL DEFAULT 'pending',
  branch        TEXT,
  payload_json  TEXT NOT NULL DEFAULT '{}',
  created_by    TEXT,
  claimed_by    TEXT,
  touched_files TEXT NOT NULL DEFAULT '[]',
  assigned_to   TEXT,
  priority      INTEGER NOT NULL DEFAULT 0,
  blocked_by    TEXT,
  file_scope    TEXT NOT NULL DEFAULT '[]',
  capability_id TEXT,
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS job_status_idx ON job(status, created_at DESC);
CREATE INDEX IF NOT EXISTS job_branch_idx ON job(branch, status);
"#;

/// DEPRECATED: job_log is superseded by event_log with entity_type='job'.
///
/// New code MUST use event_log (via `db::events::insert_event`).
/// This table is retained because:
/// - `db::jobs::append_log` and `db::jobs::list_logs` still write/read it
/// - `db::events::migrate_job_log_to_events` reads it for migration
/// - `apps/web` queries it directly in the dev UI
///
/// Removal criteria: all callers migrated to event_log, no reads in codebase,
/// migration function confirmed run on all deployed instances.
/// When ready to remove, also delete: JobLogEntry, L_COLS, row_to_log,
/// append_log, list_logs from db::jobs.
pub const JOB_LOG_DEPRECATED: &str = r#"
-- DEPRECATED: job_log is superseded by event_log with entity_type='job'.
-- Retained for backward compat. See removal criteria in schema/work.rs.
CREATE TABLE IF NOT EXISTS job_log (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  job_id     TEXT,
  branch     TEXT,
  message    TEXT NOT NULL,
  actor      TEXT,
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS job_log_branch_idx ON job_log(branch, created_at DESC);
"#;

/// Job file ownership: exclusive file claims per job.
/// Released automatically when a job reaches a terminal status.
pub const JOB_FILE: &str = r#"
-- job_file: exclusive file-path claims per job. Released on job completion.
CREATE TABLE IF NOT EXISTS job_file (
  path       TEXT PRIMARY KEY,
  job_id     TEXT NOT NULL,
  claimed_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS job_file_job_idx ON job_file(job_id);
"#;

/// File claim: batch-atomic file-path claims for concurrent agent coordination.
/// Unlike job_file (single-file, first-wins), this provides batch atomicity:
/// all paths claimed or none. Includes optional workspace_id for cross-workspace
/// tracking. See `db::file_claims` for the query module.
pub const FILE_CLAIM: &str = r#"
-- file_claim: batch-atomic file claims with workspace tracking.
-- See db::file_claims module. Separate from job_file (different semantics).
CREATE TABLE IF NOT EXISTS file_claim (
  path         TEXT PRIMARY KEY,
  job_id       TEXT NOT NULL,
  workspace_id TEXT,
  claimed_at   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_file_claim_job ON file_claim(job_id);
"#;

/// Note: human-facing scratchpad documents. Not for agent coordination.
/// Optionally scoped to a branch.
pub const NOTE: &str = r#"
-- note: human-facing scratchpad. Not for agent plans or coordination.
CREATE TABLE IF NOT EXISTS note (
  id         TEXT PRIMARY KEY,
  title      TEXT NOT NULL,
  content    TEXT NOT NULL DEFAULT '',
  tags_json  TEXT NOT NULL DEFAULT '[]',
  branch     TEXT,
  synced_at  TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS note_branch_idx ON note(branch, updated_at DESC);
"#;

/// ADR: architecture decision record. Human-driven, supersession model.
/// Status: proposed -> accepted | deprecated | superseded.
pub const ADR: &str = r#"
-- adr: architecture decision record. Supersession via supersedes_id.
CREATE TABLE IF NOT EXISTS adr (
  id            TEXT PRIMARY KEY,
  title         TEXT NOT NULL,
  status        TEXT NOT NULL DEFAULT 'proposed',
  date          TEXT NOT NULL,
  context       TEXT NOT NULL DEFAULT '',
  decision      TEXT NOT NULL DEFAULT '',
  tags_json     TEXT NOT NULL DEFAULT '[]',
  supersedes_id TEXT,
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_adr_status ON adr(status);
"#;

/// Target: a named goal. Two kinds:
/// - milestone: time-bounded (v0.1, v0.2), has due_date
/// - surface: evergreen capability domain (Compiler, Studio, Registry)
/// Carries body_markdown as a living document.
pub const TARGET: &str = r#"
-- target: named goal. kind='milestone' (time-bounded) or 'surface' (evergreen).
CREATE TABLE IF NOT EXISTS target (
  id              TEXT PRIMARY KEY,
  kind            TEXT NOT NULL,
  title           TEXT NOT NULL,
  description     TEXT,
  status          TEXT NOT NULL DEFAULT 'active',
  goal            TEXT,
  phase           TEXT,
  due_date        TEXT,
  body_markdown   TEXT,
  file_scope_json TEXT,
  created_at      TEXT NOT NULL,
  updated_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS target_kind_idx ON target(kind, status);
CREATE INDEX IF NOT EXISTS target_phase_status_idx ON target(phase, status);
"#;

/// Capability: a specific thing that must be true. Belongs to one surface
/// (target_id), optionally scheduled into one milestone (milestone_id).
/// Status lifecycle: aspirational -> in_progress -> actual.
/// Evidence required to mark actual.
pub const CAPABILITY: &str = r#"
-- capability: concrete requirement under a target.
-- Status: aspirational -> in_progress -> actual (evidence required).
CREATE TABLE IF NOT EXISTS capability (
  id                  TEXT PRIMARY KEY,
  target_id           TEXT NOT NULL,
  title               TEXT NOT NULL,
  status              TEXT NOT NULL DEFAULT 'aspirational',
  evidence            TEXT,
  milestone_id        TEXT,
  phase               TEXT,
  acceptance_criteria TEXT NOT NULL DEFAULT '[]',
  preset_hint         TEXT,
  file_scope          TEXT,
  assigned_to         TEXT,
  priority            INTEGER NOT NULL DEFAULT 0,
  surface             TEXT,
  tier                TEXT,
  test_refs           TEXT NOT NULL DEFAULT '[]',
  related_files       TEXT NOT NULL DEFAULT '[]',
  last_job_id         TEXT,
  created_at          TEXT NOT NULL,
  updated_at          TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS capability_target_idx ON capability(target_id, status);
CREATE INDEX IF NOT EXISTS capability_milestone_idx ON capability(milestone_id, status);
CREATE INDEX IF NOT EXISTS capability_phase_idx ON capability(target_id, phase, status);
CREATE INDEX IF NOT EXISTS capability_assignment_idx ON capability(assigned_to, status);
CREATE INDEX IF NOT EXISTS capability_preset_idx ON capability(preset_hint);
"#;
