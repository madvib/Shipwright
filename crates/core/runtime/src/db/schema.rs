//! Unified platform schema — single DDL, no migration versioning.
//!
//! Every table uses `CREATE TABLE IF NOT EXISTS` so the schema is
//! idempotent: run it on every connection and new tables appear
//! automatically.  Schema = code.

pub const SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ─── Key-value state ────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS kv_state (
  namespace  TEXT NOT NULL,
  key        TEXT NOT NULL,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(namespace, key)
);

-- ─── Workspace ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS workspace (
  branch             TEXT PRIMARY KEY,
  id                 TEXT,
  workspace_type     TEXT NOT NULL DEFAULT 'feature',
  status             TEXT NOT NULL DEFAULT 'active',
  environment_id     TEXT,
  feature_id         TEXT,
  target_id          TEXT,
  active_agent       TEXT,
  active_preset      TEXT,
  providers_json     TEXT NOT NULL DEFAULT '[]',
  mcp_servers_json   TEXT NOT NULL DEFAULT '[]',
  skills_json        TEXT NOT NULL DEFAULT '[]',
  plugins_json       TEXT NOT NULL DEFAULT '[]',
  resolved_at        TEXT,
  is_worktree        INTEGER NOT NULL DEFAULT 0,
  worktree_path      TEXT,
  last_activated_at  TEXT,
  context_hash       TEXT,
  config_generation  INTEGER NOT NULL DEFAULT 0,
  compiled_at        TEXT,
  compile_error      TEXT,
  created_at         TEXT,
  updated_at         TEXT
);
CREATE INDEX IF NOT EXISTS workspace_status_idx ON workspace(status);

CREATE TABLE IF NOT EXISTS workspace_session (
  id                        TEXT PRIMARY KEY,
  workspace_id              TEXT NOT NULL,
  workspace_branch          TEXT NOT NULL,
  status                    TEXT NOT NULL DEFAULT 'active',
  started_at                TEXT NOT NULL,
  ended_at                  TEXT,
  agent_id                  TEXT,
  preset_id                 TEXT,
  primary_provider          TEXT,
  goal                      TEXT,
  summary                   TEXT,
  updated_workspace_ids_json  TEXT NOT NULL DEFAULT '[]',
  compiled_at               TEXT,
  compile_error             TEXT,
  config_generation_at_start INTEGER,
  created_at                TEXT NOT NULL,
  updated_at                TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS workspace_session_workspace_idx
  ON workspace_session(workspace_id, started_at DESC);
CREATE INDEX IF NOT EXISTS workspace_session_status_idx
  ON workspace_session(status, started_at DESC);

CREATE TABLE IF NOT EXISTS workspace_session_record (
  id                       TEXT PRIMARY KEY,
  session_id               TEXT NOT NULL UNIQUE,
  workspace_id             TEXT NOT NULL,
  workspace_branch         TEXT NOT NULL,
  summary                  TEXT,
  updated_workspace_ids_json TEXT NOT NULL DEFAULT '[]',
  created_at               TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS workspace_session_record_workspace_idx
  ON workspace_session_record(workspace_id, created_at DESC);

-- ─── Branch config ──────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS branch_config (
  branch       TEXT PRIMARY KEY,
  preset_id    TEXT NOT NULL,
  workspace_id TEXT,
  plugins_json TEXT NOT NULL DEFAULT '[]',
  compiled_at  TEXT NOT NULL,
  updated_at   TEXT NOT NULL
);

-- ─── Branch context (entity links) ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS branch_context (
  branch      TEXT PRIMARY KEY,
  link_type   TEXT NOT NULL,
  link_id     TEXT NOT NULL,
  last_synced TEXT NOT NULL
);

-- ─── Jobs ───────────────────────────────────────────────────────────────────
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

CREATE TABLE IF NOT EXISTS job_log (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  job_id     TEXT,
  branch     TEXT,
  message    TEXT NOT NULL,
  actor      TEXT,
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS job_log_branch_idx ON job_log(branch, created_at DESC);

CREATE TABLE IF NOT EXISTS job_file (
  path       TEXT PRIMARY KEY,
  job_id     TEXT NOT NULL,
  claimed_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS job_file_job_idx ON job_file(job_id);

-- ─── Notes ──────────────────────────────────────────────────────────────────
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

-- ─── ADRs ───────────────────────────────────────────────────────────────────
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

-- ─── Targets & capabilities ─────────────────────────────────────────────────
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

-- ─── Agent runtime ──────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS agent_runtime_settings (
  id              INTEGER PRIMARY KEY CHECK(id = 1),
  active_agent    TEXT,
  providers_json  TEXT NOT NULL DEFAULT '[]',
  hooks_json      TEXT NOT NULL DEFAULT '[]',
  statuses_json   TEXT NOT NULL DEFAULT '[]',
  ai_json         TEXT,
  git_json        TEXT NOT NULL DEFAULT '{}',
  namespaces_json TEXT NOT NULL DEFAULT '[]',
  updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS agent_artifact_registry (
  uuid         TEXT PRIMARY KEY,
  kind         TEXT NOT NULL,
  external_id  TEXT NOT NULL,
  name         TEXT NOT NULL,
  source_path  TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  updated_at   TEXT NOT NULL,
  UNIQUE(kind, external_id)
);
CREATE INDEX IF NOT EXISTS agent_artifact_kind_idx
  ON agent_artifact_registry(kind);

CREATE TABLE IF NOT EXISTS agent_mode (
  id                 TEXT PRIMARY KEY,
  name               TEXT NOT NULL,
  description        TEXT,
  active_tools_json  TEXT NOT NULL DEFAULT '[]',
  mcp_refs_json      TEXT NOT NULL DEFAULT '[]',
  skill_refs_json    TEXT NOT NULL DEFAULT '[]',
  rule_refs_json     TEXT NOT NULL DEFAULT '[]',
  prompt_id          TEXT,
  hooks_json         TEXT NOT NULL DEFAULT '[]',
  permissions_json   TEXT NOT NULL DEFAULT '{}',
  target_agents_json TEXT NOT NULL DEFAULT '[]',
  updated_at         TEXT NOT NULL
);

-- ─── Managed MCP state ──────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS managed_mcp_state (
  provider         TEXT PRIMARY KEY,
  server_ids_json  TEXT NOT NULL DEFAULT '[]',
  last_mode        TEXT,
  updated_at       TEXT NOT NULL
);

-- ─── Runtime primitives (environments, processes, git workspaces) ───────────
CREATE TABLE IF NOT EXISTS environment (
  id               TEXT PRIMARY KEY,
  name             TEXT,
  tools_json       TEXT NOT NULL DEFAULT '[]',
  rules_json       TEXT NOT NULL DEFAULT '[]',
  permissions_json TEXT NOT NULL DEFAULT '{}',
  providers_json   TEXT NOT NULL DEFAULT '[]',
  hooks_json       TEXT NOT NULL DEFAULT '{}',
  mcp_servers_json TEXT NOT NULL DEFAULT '[]',
  created_at       TEXT NOT NULL,
  updated_at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS runtime_process (
  id           TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  status       TEXT NOT NULL,
  provider     TEXT,
  capability   TEXT,
  started_at   TEXT NOT NULL,
  ended_at     TEXT,
  error        TEXT
);
CREATE INDEX IF NOT EXISTS runtime_process_workspace_idx
  ON runtime_process(workspace_id, started_at DESC);

"#;
