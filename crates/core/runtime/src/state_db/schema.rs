pub(super) const PROJECT_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
  version TEXT PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS kv_state (
  namespace TEXT NOT NULL,
  key TEXT NOT NULL,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(namespace, key)
);

CREATE TABLE IF NOT EXISTS migration_audit (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  scope TEXT NOT NULL,
  source_path TEXT NOT NULL,
  target_path TEXT NOT NULL,
  action TEXT NOT NULL,
  created_at TEXT NOT NULL
);
"#;

pub(super) const GLOBAL_SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
  version TEXT PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS global_state (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
"#;

pub(super) const PROJECT_SCHEMA_OPERATIONAL: &str = r#"
CREATE TABLE IF NOT EXISTS managed_mcp_state (
  provider TEXT PRIMARY KEY,
  server_ids_json TEXT NOT NULL DEFAULT '[]',
  last_mode TEXT,
  updated_at TEXT NOT NULL
);

-- branch_context keeps branch -> linked-entity mappings used by git/workspace sync.
-- Legacy databases used doc_type/doc_id; compatibility migration backfills link_*.
CREATE TABLE IF NOT EXISTS branch_context (
  branch TEXT PRIMARY KEY,
  link_type TEXT NOT NULL,
  link_id TEXT NOT NULL,
  last_synced TEXT NOT NULL
);
"#;

pub(super) const PROJECT_SCHEMA_WORKSPACE: &str = r#"
CREATE TABLE IF NOT EXISTS workspace (
  branch         TEXT PRIMARY KEY,
  feature_id     TEXT,
  target_id      TEXT,
  active_agent   TEXT,
  providers_json TEXT NOT NULL DEFAULT '[]',
  resolved_at    TEXT NOT NULL,
  is_worktree    INTEGER NOT NULL DEFAULT 0,
  worktree_path  TEXT
);
"#;

pub(super) const PROJECT_SCHEMA_WORKSPACE_V2: &str = r#"
ALTER TABLE workspace ADD COLUMN id TEXT;
ALTER TABLE workspace ADD COLUMN workspace_type TEXT NOT NULL DEFAULT 'feature';
ALTER TABLE workspace ADD COLUMN status TEXT NOT NULL DEFAULT 'active';
ALTER TABLE workspace ADD COLUMN environment_id TEXT;
ALTER TABLE workspace ADD COLUMN last_activated_at TEXT;
ALTER TABLE workspace ADD COLUMN context_hash TEXT;

UPDATE workspace
SET id = branch
WHERE id IS NULL OR id = '';

UPDATE workspace
SET status = 'active'
WHERE status IS NULL OR status = '';
"#;

pub(super) const PROJECT_SCHEMA_WORKSPACE_SESSION: &str = r#"
CREATE TABLE IF NOT EXISTS workspace_session (
  id                        TEXT PRIMARY KEY,
  workspace_id              TEXT NOT NULL,
  workspace_branch          TEXT NOT NULL,
  status                    TEXT NOT NULL DEFAULT 'active',
  started_at                TEXT NOT NULL,
  ended_at                  TEXT,
  agent_id                  TEXT,
  primary_provider          TEXT,
  goal                      TEXT,
  summary                   TEXT,
  updated_feature_ids_json  TEXT NOT NULL DEFAULT '[]',
  compiled_at               TEXT,
  compile_error             TEXT,
  created_at                TEXT NOT NULL,
  updated_at                TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS workspace_session_workspace_idx
  ON workspace_session(workspace_id, started_at DESC);

CREATE INDEX IF NOT EXISTS workspace_session_status_idx
  ON workspace_session(status, started_at DESC);
"#;

pub(super) const PROJECT_SCHEMA_WORKSPACE_COMPILE_STATE: &str = r#"
ALTER TABLE workspace ADD COLUMN config_generation INTEGER NOT NULL DEFAULT 0;
ALTER TABLE workspace ADD COLUMN compiled_at TEXT;
ALTER TABLE workspace ADD COLUMN compile_error TEXT;
"#;

pub(super) const PROJECT_SCHEMA_RUNTIME_PRIMITIVES_V3: &str = r#"
CREATE TABLE IF NOT EXISTS environment (
  id            TEXT PRIMARY KEY,
  name          TEXT,
  tools_json    TEXT NOT NULL DEFAULT '[]',
  rules_json    TEXT NOT NULL DEFAULT '[]',
  permissions_json TEXT NOT NULL DEFAULT '{}',
  providers_json TEXT NOT NULL DEFAULT '[]',
  hooks_json    TEXT NOT NULL DEFAULT '{}',
  mcp_servers_json TEXT NOT NULL DEFAULT '[]',
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS runtime_process (
  id            TEXT PRIMARY KEY,
  workspace_id  TEXT NOT NULL,
  status        TEXT NOT NULL,
  provider      TEXT,
  capability    TEXT,
  started_at    TEXT NOT NULL,
  ended_at      TEXT,
  error         TEXT
);
CREATE INDEX IF NOT EXISTS runtime_process_workspace_idx
  ON runtime_process(workspace_id, started_at DESC);

CREATE TABLE IF NOT EXISTS git_workspace (
  workspace_id      TEXT PRIMARY KEY,
  branch            TEXT NOT NULL UNIQUE,
  worktree_path     TEXT,
  feature_id        TEXT,
  release_id        TEXT,
  compile_generation INTEGER NOT NULL DEFAULT 0,
  compiled_at       TEXT,
  compile_error     TEXT,
  context_hash      TEXT
);
CREATE INDEX IF NOT EXISTS git_workspace_feature_idx
  ON git_workspace(feature_id);
"#;

pub(super) const PROJECT_SCHEMA_AGENT_RUNTIME_SETTINGS: &str = r#"
CREATE TABLE IF NOT EXISTS agent_runtime_settings (
  id             INTEGER PRIMARY KEY CHECK(id = 1),
  active_agent   TEXT,
  providers_json TEXT NOT NULL DEFAULT '[]',
  hooks_json     TEXT NOT NULL DEFAULT '[]',
  statuses_json  TEXT NOT NULL DEFAULT '[]',
  ai_json        TEXT,
  git_json       TEXT NOT NULL DEFAULT '{}',
  namespaces_json TEXT NOT NULL DEFAULT '[]',
  updated_at     TEXT NOT NULL
);
"#;

pub(super) const PROJECT_SCHEMA_AGENT_CATALOG: &str = r#"
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
  id                TEXT PRIMARY KEY,
  name              TEXT NOT NULL,
  description       TEXT,
  active_tools_json TEXT NOT NULL DEFAULT '[]',
  mcp_refs_json     TEXT NOT NULL DEFAULT '[]',
  skill_refs_json   TEXT NOT NULL DEFAULT '[]',
  rule_refs_json    TEXT NOT NULL DEFAULT '[]',
  prompt_id         TEXT,
  hooks_json        TEXT NOT NULL DEFAULT '[]',
  permissions_json  TEXT NOT NULL DEFAULT '{}',
  target_agents_json TEXT NOT NULL DEFAULT '[]',
  updated_at        TEXT NOT NULL
);
"#;
