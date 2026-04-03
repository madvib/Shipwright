//! Schema DDL for platform tables — the portable runtime layer.
//!
//! These tables exist in every Ship installation regardless of workflow.
//! They power the compilation pipeline, session lifecycle, agent config,
//! and audit trail.
//!
//! Canonical DDL is in `migrations/0001_initial.sql`. These constants
//! are retained as code-level documentation.

/// Key-value state: generic namespaced store for runtime flags, cache keys,
/// and any transient data that does not warrant its own table.
pub const KV_STATE: &str = r#"
CREATE TABLE IF NOT EXISTS kv_state (
  namespace  TEXT NOT NULL,
  key        TEXT NOT NULL,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(namespace, key)
);
"#;

/// Workspace: branch-keyed unit of work. Tracks agent assignment, provider
/// config, compilation state, and worktree metadata.
pub const WORKSPACE: &str = r#"
CREATE TABLE IF NOT EXISTS workspace (
  branch             TEXT PRIMARY KEY,
  id                 TEXT,
  workspace_type     TEXT NOT NULL DEFAULT 'feature',
  status             TEXT NOT NULL DEFAULT 'active',
  active_agent       TEXT,
  active_preset      TEXT,
  providers_json     TEXT NOT NULL DEFAULT '[]',
  mcp_servers_json   TEXT NOT NULL DEFAULT '[]',
  skills_json        TEXT NOT NULL DEFAULT '[]',
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
"#;

/// Workspace session: a heartbeat-scoped work interval within a workspace.
/// One active session per workspace at a time. Immutable once ended.
pub const WORKSPACE_SESSION: &str = r#"
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
"#;

/// Workspace session record: immutable snapshot created when a session ends.
pub const WORKSPACE_SESSION_RECORD: &str = r#"
CREATE TABLE IF NOT EXISTS workspace_session_record (
  id                       TEXT PRIMARY KEY,
  session_id               TEXT NOT NULL UNIQUE,
  workspace_id             TEXT NOT NULL,
  workspace_branch         TEXT NOT NULL,
  summary                  TEXT,
  updated_workspace_ids_json TEXT NOT NULL DEFAULT '[]',
  duration_secs            INTEGER,
  provider                 TEXT,
  model                    TEXT,
  agent_id                 TEXT,
  files_changed            INTEGER,
  gate_result              TEXT,
  created_at               TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS workspace_session_record_workspace_idx
  ON workspace_session_record(workspace_id, created_at DESC);
"#;

/// Branch context: links a branch to an external entity (e.g. a target or capability).
pub const BRANCH_CONTEXT: &str = r#"
CREATE TABLE IF NOT EXISTS branch_context (
  branch      TEXT PRIMARY KEY,
  link_type   TEXT NOT NULL,
  link_id     TEXT NOT NULL,
  last_synced TEXT NOT NULL
);
"#;

/// Event log: append-only state change record. Never update or delete events.
/// Context columns enable scoped queries without joins.
pub const EVENT_LOG: &str = r#"
CREATE TABLE IF NOT EXISTS event_log (
    id             TEXT PRIMARY KEY NOT NULL,
    actor          TEXT NOT NULL DEFAULT 'ship',
    entity_type    TEXT NOT NULL,
    entity_id      TEXT,
    action         TEXT NOT NULL,
    detail         TEXT,
    workspace_id   TEXT,
    session_id     TEXT,
    job_id         TEXT,
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    version        INTEGER,
    correlation_id TEXT,
    causation_id   TEXT,
    synced_at      TEXT
);
CREATE INDEX IF NOT EXISTS idx_event_workspace ON event_log(workspace_id);
CREATE INDEX IF NOT EXISTS idx_event_session ON event_log(session_id);
CREATE INDEX IF NOT EXISTS idx_event_job ON event_log(job_id);
CREATE INDEX IF NOT EXISTS idx_event_entity ON event_log(entity_type, entity_id);
"#;

/// Agent artifact registry: content-addressed registry of compiled artifacts.
pub const AGENT_ARTIFACT_REGISTRY: &str = r#"
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
"#;

/// Managed MCP state: tracks which MCP server processes Ship manages
/// per provider, and the last agent config that was applied.
pub const MANAGED_MCP_STATE: &str = r#"
CREATE TABLE IF NOT EXISTS managed_mcp_state (
  provider         TEXT PRIMARY KEY,
  server_ids_json  TEXT NOT NULL DEFAULT '[]',
  last_mode        TEXT,
  updated_at       TEXT NOT NULL
);
"#;
