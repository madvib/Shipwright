//! Schema DDL for state and workspace tables.

/// Key-value state: generic namespaced store for runtime flags, cache keys,
/// and any transient data that does not warrant its own table.
/// Primary key is (namespace, key).
pub const KV_STATE: &str = r#"
-- kv_state: generic namespaced key-value store for runtime flags and caches.
-- PK: (namespace, key). Values are JSON strings.
CREATE TABLE IF NOT EXISTS kv_state (
  namespace  TEXT NOT NULL,
  key        TEXT NOT NULL,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(namespace, key)
);
"#;

/// Workspace: the primary record for a git-branch-based unit of work.
/// Branch is the natural key. Tracks agent assignment, provider config,
/// compilation state, and worktree metadata.
///
/// Two query modules operate on this table with different column subsets:
/// - `db::workspace` (struct-based, MCP/studio access)
/// - `db::workspace_state` (tuple-based, internal lifecycle)
pub const WORKSPACE: &str = r#"
-- workspace: branch-keyed unit of work. Tracks agent, provider, and compile state.
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
"#;

/// Workspace session: a heartbeat-scoped work interval within a workspace.
/// One active session per workspace at a time. Immutable once ended.
pub const WORKSPACE_SESSION: &str = r#"
-- workspace_session: time-bounded work interval within a workspace.
-- One active per workspace. Becomes immutable after status = 'ended'.
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
/// One-to-one with workspace_session (UNIQUE on session_id).
pub const WORKSPACE_SESSION_RECORD: &str = r#"
-- workspace_session_record: immutable end-of-session snapshot.
-- Keyed by session_id (unique). Created by end_session.
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

/// Branch config: compiled preset and plugin state per branch.
/// Written during `ship use` compilation.
pub const BRANCH_CONFIG: &str = r#"
-- branch_config: compiled preset/plugin state per branch, written by `ship use`.
CREATE TABLE IF NOT EXISTS branch_config (
  branch       TEXT PRIMARY KEY,
  preset_id    TEXT NOT NULL,
  workspace_id TEXT,
  plugins_json TEXT NOT NULL DEFAULT '[]',
  compiled_at  TEXT NOT NULL,
  updated_at   TEXT NOT NULL
);
"#;

/// Branch context: links a branch to an external entity (e.g. a target or capability).
/// Used for branch-scoped navigation and context injection.
pub const BRANCH_CONTEXT: &str = r#"
-- branch_context: branch-to-entity link (e.g. target, capability).
CREATE TABLE IF NOT EXISTS branch_context (
  branch      TEXT PRIMARY KEY,
  link_type   TEXT NOT NULL,
  link_id     TEXT NOT NULL,
  last_synced TEXT NOT NULL
);
"#;
