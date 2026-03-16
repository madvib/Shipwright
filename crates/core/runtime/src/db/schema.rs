//! Clean platform schema — no workflow-layer tables.

const FOUNDATION: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
  version    TEXT PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS kv_state (
  namespace  TEXT NOT NULL,
  key        TEXT NOT NULL,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(namespace, key)
);
"#;

const EVENT_LOG: &str = r#"
CREATE TABLE IF NOT EXISTS event_log (
  seq       INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp TEXT NOT NULL,
  actor     TEXT NOT NULL,
  entity    TEXT NOT NULL,
  action    TEXT NOT NULL,
  subject   TEXT NOT NULL,
  details   TEXT
);
CREATE INDEX IF NOT EXISTS event_log_timestamp_idx ON event_log(timestamp);
CREATE INDEX IF NOT EXISTS event_log_lookup_idx
  ON event_log(timestamp, actor, entity, action, subject);
"#;

const WORKSPACE: &str = r#"
CREATE TABLE IF NOT EXISTS workspace (
  id               TEXT PRIMARY KEY,
  branch           TEXT NOT NULL UNIQUE,
  worktree_path    TEXT,
  workspace_type   TEXT NOT NULL DEFAULT 'declarative',
  status           TEXT NOT NULL DEFAULT 'active',
  active_preset    TEXT,
  providers_json   TEXT NOT NULL DEFAULT '[]',
  skills_json      TEXT NOT NULL DEFAULT '[]',
  mcp_servers_json TEXT NOT NULL DEFAULT '[]',
  plugins_json     TEXT NOT NULL DEFAULT '[]',
  compiled_at      TEXT,
  compile_error    TEXT,
  created_at       TEXT NOT NULL,
  updated_at       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS workspace_status_idx ON workspace(status);

CREATE TABLE IF NOT EXISTS workspace_session (
  id               TEXT PRIMARY KEY,
  workspace_id     TEXT NOT NULL REFERENCES workspace(id) ON DELETE CASCADE,
  branch           TEXT NOT NULL,
  status           TEXT NOT NULL DEFAULT 'active',
  preset_id        TEXT,
  primary_provider TEXT,
  goal             TEXT,
  summary          TEXT,
  started_at       TEXT NOT NULL,
  ended_at         TEXT,
  created_at       TEXT NOT NULL,
  updated_at       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS workspace_session_workspace_idx
  ON workspace_session(workspace_id, started_at DESC);
CREATE INDEX IF NOT EXISTS workspace_session_status_idx
  ON workspace_session(status, started_at DESC);
"#;

const BRANCH_CONFIG: &str = r#"
CREATE TABLE IF NOT EXISTS branch_config (
  branch       TEXT PRIMARY KEY,
  preset_id    TEXT NOT NULL,
  workspace_id TEXT REFERENCES workspace(id) ON DELETE SET NULL,
  plugins_json TEXT NOT NULL DEFAULT '[]',
  compiled_at  TEXT NOT NULL,
  updated_at   TEXT NOT NULL
);
"#;

const JOBS: &str = r#"
CREATE TABLE IF NOT EXISTS job (
  id           TEXT PRIMARY KEY,
  kind         TEXT NOT NULL,
  status       TEXT NOT NULL DEFAULT 'pending',
  branch       TEXT,
  payload_json TEXT NOT NULL DEFAULT '{}',
  created_by   TEXT,
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS job_status_idx ON job(status, created_at DESC);
CREATE INDEX IF NOT EXISTS job_branch_idx  ON job(branch, status);

CREATE TABLE IF NOT EXISTS job_log (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  job_id     TEXT REFERENCES job(id) ON DELETE SET NULL,
  branch     TEXT,
  message    TEXT NOT NULL,
  actor      TEXT,
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS job_log_branch_idx ON job_log(branch, created_at DESC);
"#;

const NOTES: &str = r#"
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

const ADRS: &str = r#"
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

pub const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_foundation", FOUNDATION),
    ("0002_event_log", EVENT_LOG),
    ("0003_workspace", WORKSPACE),
    ("0004_branch_config", BRANCH_CONFIG),
    ("0005_jobs", JOBS),
    ("0006_notes", NOTES),
    ("0007_adrs", ADRS),
    ("0008_job_claimed_by", "ALTER TABLE job ADD COLUMN claimed_by TEXT;"),
];
