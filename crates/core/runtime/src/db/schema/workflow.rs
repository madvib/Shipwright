//! Schema DDL for workflow tables — the opinionated planning layer.
//!
//! These tables power Ship's target/capability/job workflow. They are
//! the canonical example but designed to be separable — teams may
//! eventually customize or replace this layer.

/// Target: a named goal. Two kinds:
///   - milestone: time-bounded (v0.1, v0.2), has due_date
///   - surface: evergreen capability domain (Compiler, Studio, Registry)
pub const TARGET: &str = r#"
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
CREATE TABLE IF NOT EXISTS capability (
  id                  TEXT PRIMARY KEY,
  target_id           TEXT NOT NULL,
  title               TEXT NOT NULL,
  status              TEXT NOT NULL DEFAULT 'aspirational',
  evidence            TEXT,
  milestone_id        TEXT,
  phase               TEXT,
  acceptance_criteria TEXT NOT NULL DEFAULT '[]',
  assigned_to         TEXT,
  priority            INTEGER NOT NULL DEFAULT 0,
  file_scope          TEXT,
  created_at          TEXT NOT NULL,
  updated_at          TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS capability_target_idx ON capability(target_id, status);
CREATE INDEX IF NOT EXISTS capability_milestone_idx ON capability(milestone_id, status);
CREATE INDEX IF NOT EXISTS capability_phase_idx ON capability(target_id, phase, status);
CREATE INDEX IF NOT EXISTS capability_assignment_idx ON capability(assigned_to, status);
"#;

/// Job: a unit of work in the agent queue.
/// Status lifecycle: pending -> running -> complete | failed | done.
pub const JOB: &str = r#"
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

/// Job file ownership: exclusive file claims per job.
pub const JOB_FILE: &str = r#"
CREATE TABLE IF NOT EXISTS job_file (
  path       TEXT PRIMARY KEY,
  job_id     TEXT NOT NULL,
  claimed_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS job_file_job_idx ON job_file(job_id);
"#;

/// File claim: batch-atomic file-path claims for concurrent agent coordination.
pub const FILE_CLAIM: &str = r#"
CREATE TABLE IF NOT EXISTS file_claim (
  path         TEXT PRIMARY KEY,
  job_id       TEXT NOT NULL,
  workspace_id TEXT,
  claimed_at   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_file_claim_job ON file_claim(job_id);
"#;

/// Note: human-facing scratchpad documents.
pub const NOTE: &str = r#"
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

/// ADR: architecture decision record. Supersession model.
/// Status: proposed -> accepted | deprecated | superseded.
pub const ADR: &str = r#"
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
