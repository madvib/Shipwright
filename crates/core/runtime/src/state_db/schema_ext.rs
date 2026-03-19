pub(super) const PROJECT_SCHEMA_ADRS: &str = r#"
CREATE TABLE IF NOT EXISTS adr (
  id              TEXT PRIMARY KEY,
  title           TEXT NOT NULL,
  status          TEXT NOT NULL DEFAULT 'proposed',
  date            TEXT NOT NULL,
  context         TEXT NOT NULL DEFAULT '',
  decision        TEXT NOT NULL DEFAULT '',
  tags_json       TEXT NOT NULL DEFAULT '[]',
  spec_id         TEXT,
  supersedes_id   TEXT,
  created_at      TEXT NOT NULL,
  updated_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS adr_status_idx ON adr(status);

CREATE TABLE IF NOT EXISTS adr_option (
  id                TEXT PRIMARY KEY,
  adr_id            TEXT NOT NULL REFERENCES adr(id) ON DELETE CASCADE,
  title             TEXT NOT NULL,
  arguments_for     TEXT NOT NULL DEFAULT '',
  arguments_against TEXT NOT NULL DEFAULT '',
  ord               INTEGER NOT NULL DEFAULT 0
);
"#;

pub(super) const PROJECT_SCHEMA_NOTES: &str = r#"
CREATE TABLE IF NOT EXISTS note (
  id              TEXT PRIMARY KEY,
  title           TEXT NOT NULL,
  content         TEXT NOT NULL DEFAULT '',
  tags_json       TEXT NOT NULL DEFAULT '[]',
  scope           TEXT NOT NULL DEFAULT 'project',
  created_at      TEXT NOT NULL,
  updated_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS note_scope_idx ON note(scope);
"#;

pub(super) const PROJECT_SCHEMA_FEATURES_RELEASES: &str = r#"
CREATE TABLE IF NOT EXISTS feature (
  id              TEXT PRIMARY KEY,
  title           TEXT NOT NULL,
  description     TEXT,
  status          TEXT NOT NULL DEFAULT 'planned',
  release_id      TEXT,
  active_target_id TEXT,
  spec_id         TEXT,
  branch          TEXT,
  agent_json      TEXT,
  tags_json       TEXT NOT NULL DEFAULT '[]',
  created_at      TEXT NOT NULL,
  updated_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS feature_status_idx ON feature(status);
CREATE INDEX IF NOT EXISTS feature_release_idx ON feature(release_id);
CREATE INDEX IF NOT EXISTS feature_active_target_idx ON feature(active_target_id);

CREATE TABLE IF NOT EXISTS feature_todo (
  id              TEXT PRIMARY KEY,
  feature_id      TEXT NOT NULL REFERENCES feature(id) ON DELETE CASCADE,
  text            TEXT NOT NULL,
  completed       INTEGER NOT NULL DEFAULT 0,
  ord             INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS feature_criterion (
  id              TEXT PRIMARY KEY,
  feature_id      TEXT NOT NULL REFERENCES feature(id) ON DELETE CASCADE,
  text            TEXT NOT NULL,
  met             INTEGER NOT NULL DEFAULT 0,
  ord             INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS release (
  id              TEXT PRIMARY KEY,
  version         TEXT NOT NULL,
  status          TEXT NOT NULL DEFAULT 'planned',
  target_date     TEXT,
  supported       INTEGER,
  body            TEXT NOT NULL DEFAULT '',
  created_at      TEXT NOT NULL,
  updated_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS release_status_idx ON release(status);

CREATE TABLE IF NOT EXISTS release_breaking_change (
  id              TEXT PRIMARY KEY,
  release_id      TEXT NOT NULL REFERENCES release(id) ON DELETE CASCADE,
  text            TEXT NOT NULL,
  ord             INTEGER NOT NULL DEFAULT 0
);
"#;

pub(super) const PROJECT_SCHEMA_FEATURE_DOCS: &str = r#"
CREATE TABLE IF NOT EXISTS feature_doc (
  feature_id         TEXT PRIMARY KEY REFERENCES feature(id) ON DELETE CASCADE,
  status             TEXT NOT NULL DEFAULT 'not-started',
  content            TEXT NOT NULL DEFAULT '',
  revision           INTEGER NOT NULL DEFAULT 1,
  last_verified_at   TEXT,
  created_at         TEXT NOT NULL,
  updated_at         TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS feature_doc_revision (
  id              TEXT PRIMARY KEY,
  feature_id      TEXT NOT NULL REFERENCES feature(id) ON DELETE CASCADE,
  revision        INTEGER NOT NULL,
  status          TEXT NOT NULL,
  content         TEXT NOT NULL,
  actor           TEXT NOT NULL DEFAULT 'ship',
  created_at      TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS feature_doc_revision_feature_idx
  ON feature_doc_revision(feature_id, revision);
"#;

pub(super) const PROJECT_SCHEMA_SPECS: &str = r#"
CREATE TABLE IF NOT EXISTS spec (
  id              TEXT PRIMARY KEY,
  title           TEXT NOT NULL,
  body            TEXT NOT NULL DEFAULT '',
  status          TEXT NOT NULL DEFAULT 'draft',
  author          TEXT,
  branch          TEXT,
  workspace_id    TEXT,
  feature_id      TEXT,
  release_id      TEXT,
  tags_json       TEXT NOT NULL DEFAULT '[]',
  created_at      TEXT NOT NULL,
  updated_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS spec_status_idx ON spec(status);
CREATE INDEX IF NOT EXISTS spec_workspace_idx ON spec(workspace_id);
"#;

pub(super) const SCHEMA_MIGRATION_META: &str = r#"
CREATE TABLE IF NOT EXISTS migration_meta (
  entity_type TEXT PRIMARY KEY,
  migrated_at TEXT NOT NULL,
  file_count  INTEGER NOT NULL DEFAULT 0
);
"#;

pub(super) const PROJECT_SCHEMA_EVENTS: &str = r#"
CREATE TABLE IF NOT EXISTS event_log (
  seq         INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp   TEXT NOT NULL,
  actor       TEXT NOT NULL,
  entity      TEXT NOT NULL,
  action      TEXT NOT NULL,
  subject     TEXT NOT NULL,
  details     TEXT
);
CREATE INDEX IF NOT EXISTS event_log_timestamp_idx ON event_log(timestamp);
CREATE INDEX IF NOT EXISTS event_log_lookup_idx
  ON event_log(timestamp, actor, entity, action, subject);
"#;
