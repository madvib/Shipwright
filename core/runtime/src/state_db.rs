use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use serde_json;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, Row, SqliteConnection};
use std::path::{Path, PathBuf};
use std::str::FromStr;

const PROJECT_SCHEMA_V1: &str = r#"
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

const GLOBAL_SCHEMA_V1: &str = r#"
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

const PROJECT_SCHEMA_OPERATIONAL: &str = r#"
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

const PROJECT_SCHEMA_WORKSPACE: &str = r#"
CREATE TABLE IF NOT EXISTS workspace (
  branch         TEXT PRIMARY KEY,
  feature_id     TEXT,
  spec_id        TEXT,
  active_mode    TEXT,
  providers_json TEXT NOT NULL DEFAULT '[]',
  resolved_at    TEXT NOT NULL,
  is_worktree    INTEGER NOT NULL DEFAULT 0,
  worktree_path  TEXT
);
"#;

const PROJECT_SCHEMA_WORKSPACE_V2: &str = r#"
ALTER TABLE workspace ADD COLUMN id TEXT;
ALTER TABLE workspace ADD COLUMN workspace_type TEXT NOT NULL DEFAULT 'feature';
ALTER TABLE workspace ADD COLUMN status TEXT NOT NULL DEFAULT 'idle';
ALTER TABLE workspace ADD COLUMN release_id TEXT;
ALTER TABLE workspace ADD COLUMN last_activated_at TEXT;
ALTER TABLE workspace ADD COLUMN context_hash TEXT;

UPDATE workspace
SET id = branch
WHERE id IS NULL OR id = '';

UPDATE workspace
SET status = 'active'
WHERE status IS NULL OR status = '';
"#;

const PROJECT_SCHEMA_WORKSPACE_SESSION: &str = r#"
CREATE TABLE IF NOT EXISTS workspace_session (
  id                        TEXT PRIMARY KEY,
  workspace_id              TEXT NOT NULL,
  workspace_branch          TEXT NOT NULL,
  status                    TEXT NOT NULL DEFAULT 'active',
  started_at                TEXT NOT NULL,
  ended_at                  TEXT,
  mode_id                   TEXT,
  primary_provider          TEXT,
  goal                      TEXT,
  summary                   TEXT,
  updated_feature_ids_json  TEXT NOT NULL DEFAULT '[]',
  updated_spec_ids_json     TEXT NOT NULL DEFAULT '[]',
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

const PROJECT_SCHEMA_WORKSPACE_COMPILE_STATE: &str = r#"
ALTER TABLE workspace ADD COLUMN config_generation INTEGER NOT NULL DEFAULT 0;
ALTER TABLE workspace ADD COLUMN compiled_at TEXT;
ALTER TABLE workspace ADD COLUMN compile_error TEXT;
"#;

const PROJECT_SCHEMA_AGENT_RUNTIME_SETTINGS: &str = r#"
CREATE TABLE IF NOT EXISTS agent_runtime_settings (
  id             INTEGER PRIMARY KEY CHECK(id = 1),
  active_mode    TEXT,
  providers_json TEXT NOT NULL DEFAULT '[]',
  hooks_json     TEXT NOT NULL DEFAULT '[]',
  updated_at     TEXT NOT NULL
);
"#;

const PROJECT_SCHEMA_AGENT_CATALOG: &str = r#"
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

const PROJECT_SCHEMA_ADRS: &str = r#"
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

const PROJECT_SCHEMA_NOTES: &str = r#"
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

const PROJECT_SCHEMA_FEATURES_RELEASES: &str = r#"
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

const PROJECT_SCHEMA_FEATURE_DOCS: &str = r#"
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

const PROJECT_SCHEMA_ISSUES_SPECS: &str = r#"
CREATE TABLE IF NOT EXISTS issue (
  id              TEXT PRIMARY KEY,
  title           TEXT NOT NULL,
  description     TEXT NOT NULL DEFAULT '',
  status          TEXT NOT NULL DEFAULT 'backlog',
  assignee        TEXT,
  priority        TEXT,
  release_id      TEXT,
  feature_id      TEXT,
  spec_id         TEXT,
  tags_json       TEXT NOT NULL DEFAULT '[]',
  links_json      TEXT NOT NULL DEFAULT '[]',
  created_at      TEXT NOT NULL,
  updated_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS issue_status_idx ON issue(status);

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

const SCHEMA_MIGRATION_META: &str = r#"
CREATE TABLE IF NOT EXISTS migration_meta (
  entity_type TEXT PRIMARY KEY,
  migrated_at TEXT NOT NULL,
  file_count  INTEGER NOT NULL DEFAULT 0
);
"#;

const PROJECT_SCHEMA_EVENTS: &str = r#"
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
"#;

const PROJECT_MIGRATIONS: &[(&str, &str)] = &[
    ("0001_project_schema", PROJECT_SCHEMA_V1),
    ("0002_operational_state", PROJECT_SCHEMA_OPERATIONAL),
    ("0003_workspace", PROJECT_SCHEMA_WORKSPACE),
    ("0004_adrs", PROJECT_SCHEMA_ADRS),
    ("0005_notes", PROJECT_SCHEMA_NOTES),
    ("0006_features_releases", PROJECT_SCHEMA_FEATURES_RELEASES),
    ("0007_workspace_lifecycle", PROJECT_SCHEMA_WORKSPACE_V2),
    ("0008_issues_specs", PROJECT_SCHEMA_ISSUES_SPECS),
    ("0009_migration_meta", SCHEMA_MIGRATION_META),
    ("0010_event_log", PROJECT_SCHEMA_EVENTS),
    (
        "0011_agent_runtime_settings",
        PROJECT_SCHEMA_AGENT_RUNTIME_SETTINGS,
    ),
    ("0012_agent_catalog", PROJECT_SCHEMA_AGENT_CATALOG),
    ("0013_workspace_sessions", PROJECT_SCHEMA_WORKSPACE_SESSION),
    (
        "0014_workspace_compile_state",
        PROJECT_SCHEMA_WORKSPACE_COMPILE_STATE,
    ),
    ("0015_feature_docs", PROJECT_SCHEMA_FEATURE_DOCS),
];
const GLOBAL_MIGRATIONS: &[(&str, &str)] = &[
    ("0001_global_schema", GLOBAL_SCHEMA_V1),
    ("0002_notes", PROJECT_SCHEMA_NOTES),
    ("0003_migration_meta", SCHEMA_MIGRATION_META),
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DatabaseMigrationReport {
    pub db_path: PathBuf,
    pub created: bool,
    pub applied_migrations: usize,
}

pub type FeatureBranchLinks = (String, Option<String>, Option<String>);

pub type WorkspaceDbRow = (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Vec<String>,
    String,
    bool,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
    Option<String>,
    Option<String>,
);

pub type WorkspaceDbListRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Vec<String>,
    String,
    bool,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
    Option<String>,
    Option<String>,
);

pub struct WorkspaceUpsert<'a> {
    pub branch: &'a str,
    pub workspace_id: &'a str,
    pub workspace_type: &'a str,
    pub status: &'a str,
    pub feature_id: Option<&'a str>,
    pub spec_id: Option<&'a str>,
    pub release_id: Option<&'a str>,
    pub active_mode: Option<&'a str>,
    pub providers: &'a [String],
    pub resolved_at: &'a str,
    pub is_worktree: bool,
    pub worktree_path: Option<&'a str>,
    pub last_activated_at: Option<&'a str>,
    pub context_hash: Option<&'a str>,
    pub config_generation: i64,
    pub compiled_at: Option<&'a str>,
    pub compile_error: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSessionDb {
    pub id: String,
    pub workspace_id: String,
    pub workspace_branch: String,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub mode_id: Option<String>,
    pub primary_provider: Option<String>,
    pub goal: Option<String>,
    pub summary: Option<String>,
    pub updated_feature_ids: Vec<String>,
    pub updated_spec_ids: Vec<String>,
    pub compiled_at: Option<String>,
    pub compile_error: Option<String>,
    pub config_generation_at_start: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentRuntimeSettingsDb {
    pub providers: Vec<String>,
    pub active_mode: Option<String>,
    pub hooks_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentArtifactRegistryDb {
    pub uuid: String,
    pub kind: String,
    pub external_id: String,
    pub name: String,
    pub source_path: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentModeDb {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub active_tools_json: String,
    pub mcp_refs_json: String,
    pub skill_refs_json: String,
    pub rule_refs_json: String,
    pub prompt_id: Option<String>,
    pub hooks_json: String,
    pub permissions_json: String,
    pub target_agents_json: String,
}

/// Returns `~/.ship/state/<project-slug>/ship.db` for the given ship_dir.
/// The slug is derived from the canonical project root path, making it stable
/// across sessions and safe to store alongside the global DB.
pub fn project_db_path(ship_dir: &Path) -> Result<PathBuf> {
    let slug = project_slug(ship_dir)?;
    let global_dir = ship_global_dir()?;
    ensure_global_dir_outside_project(ship_dir, &global_dir)?;
    Ok(global_dir.join("state").join(slug).join("ship.db"))
}

pub fn ensure_project_database(ship_dir: &Path) -> Result<DatabaseMigrationReport> {
    let db_path = project_db_path(ship_dir)?;
    ensure_database(&db_path, PROJECT_MIGRATIONS)
}

pub fn ensure_global_database(global_dir: &Path) -> Result<DatabaseMigrationReport> {
    let db_path = global_dir.join("ship.db");
    ensure_database(&db_path, GLOBAL_MIGRATIONS)
}

// ─── CRUD helpers ─────────────────────────────────────────────────────────────

fn open_project_db(ship_dir: &Path) -> Result<SqliteConnection> {
    ensure_project_database(ship_dir)?;
    let db_path = project_db_path(ship_dir)?;
    let db_url = sqlite_url(&db_path);
    let options = SqliteConnectOptions::from_str(&db_url)
        .with_context(|| format!("Invalid sqlite url {}", db_url))?;
    let mut connection = block_on(async { SqliteConnection::connect_with(&options).await })?;
    ensure_project_schema_compat(&mut connection)?;
    Ok(connection)
}

/// Exposed for use by module crates (e.g. `ship-module-project`).
pub fn open_project_connection(ship_dir: &Path) -> Result<SqliteConnection> {
    open_project_db(ship_dir)
}

/// Exposed for use by module crates (e.g. `ship-module-project`).
pub fn open_global_connection() -> Result<SqliteConnection> {
    let global_dir = crate::project::get_global_dir()?;
    ensure_global_database(&global_dir)?;
    let db_path = global_dir.join("ship.db");
    let db_url = sqlite_url(&db_path);
    let options = SqliteConnectOptions::from_str(&db_url)
        .with_context(|| format!("Invalid sqlite url {}", db_url))?;
    block_on(async { SqliteConnection::connect_with(&options).await })
}

/// Returns `(server_ids, last_mode)` for the given provider, or empty defaults.
pub fn get_managed_state_db(
    ship_dir: &Path,
    provider: &str,
) -> Result<(Vec<String>, Option<String>)> {
    let mut conn = open_project_db(ship_dir)?;
    let row_opt = block_on(async {
        sqlx::query("SELECT server_ids_json, last_mode FROM managed_mcp_state WHERE provider = ?")
            .bind(provider)
            .fetch_optional(&mut conn)
            .await
    })?;
    if let Some(row) = row_opt {
        use sqlx::Row;
        let ids_json: String = row.get(0);
        let last_mode: Option<String> = row.get(1);
        let ids: Vec<String> = serde_json::from_str(&ids_json).unwrap_or_default();
        Ok((ids, last_mode))
    } else {
        Ok((Vec::new(), None))
    }
}

pub fn migration_meta_complete_project(ship_dir: &Path, entity_type: &str) -> Result<bool> {
    let mut conn = open_project_db(ship_dir)?;
    migration_meta_complete(&mut conn, entity_type)
}

pub fn migration_meta_complete_global(entity_type: &str) -> Result<bool> {
    let mut conn = open_global_connection()?;
    migration_meta_complete(&mut conn, entity_type)
}

pub fn mark_migration_meta_complete_project(
    ship_dir: &Path,
    entity_type: &str,
    file_count: usize,
) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    mark_migration_meta_complete(&mut conn, entity_type, file_count)
}

pub fn mark_migration_meta_complete_global(entity_type: &str, file_count: usize) -> Result<()> {
    let mut conn = open_global_connection()?;
    mark_migration_meta_complete(&mut conn, entity_type, file_count)
}

pub fn clear_project_migration_meta(ship_dir: &Path) -> Result<usize> {
    let mut conn = open_project_db(ship_dir)?;
    clear_migration_meta(&mut conn)
}

pub fn clear_global_migration_meta() -> Result<usize> {
    let mut conn = open_global_connection()?;
    clear_migration_meta(&mut conn)
}

fn migration_meta_complete(conn: &mut SqliteConnection, entity_type: &str) -> Result<bool> {
    let row_opt = block_on(async {
        sqlx::query("SELECT entity_type FROM migration_meta WHERE entity_type = ?")
            .bind(entity_type)
            .fetch_optional(&mut *conn)
            .await
    })?;
    Ok(row_opt.is_some())
}

fn mark_migration_meta_complete(
    conn: &mut SqliteConnection,
    entity_type: &str,
    file_count: usize,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO migration_meta (entity_type, migrated_at, file_count)
             VALUES (?, ?, ?)
             ON CONFLICT(entity_type) DO UPDATE SET
               migrated_at = excluded.migrated_at,
               file_count = excluded.file_count",
        )
        .bind(entity_type)
        .bind(&now)
        .bind(file_count as i64)
        .execute(&mut *conn)
        .await
    })?;
    Ok(())
}

fn clear_migration_meta(conn: &mut SqliteConnection) -> Result<usize> {
    let result = block_on(async {
        sqlx::query("DELETE FROM migration_meta")
            .execute(&mut *conn)
            .await
    })?;
    Ok(result.rows_affected() as usize)
}

/// Persist the managed server ids and last mode for the given provider.
pub fn set_managed_state_db(
    ship_dir: &Path,
    provider: &str,
    ids: &[String],
    last_mode: Option<&str>,
) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    let ids_json = serde_json::to_string(ids)
        .with_context(|| format!("Failed to serialize server ids for provider {}", provider))?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO managed_mcp_state (provider, server_ids_json, last_mode, updated_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(provider) DO UPDATE SET
               server_ids_json = excluded.server_ids_json,
               last_mode = excluded.last_mode,
               updated_at = excluded.updated_at",
        )
        .bind(provider)
        .bind(&ids_json)
        .bind(last_mode)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn get_agent_runtime_settings_db(ship_dir: &Path) -> Result<Option<AgentRuntimeSettingsDb>> {
    let mut conn = open_project_db(ship_dir)?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT providers_json, active_mode, hooks_json
             FROM agent_runtime_settings
             WHERE id = 1",
        )
        .fetch_optional(&mut conn)
        .await
    })?;

    let Some(row) = row_opt else {
        return Ok(None);
    };

    use sqlx::Row;
    let providers_json: String = row.get(0);
    let active_mode: Option<String> = row.get(1);
    let hooks_json: String = row.get(2);
    let providers: Vec<String> = serde_json::from_str(&providers_json).unwrap_or_default();

    Ok(Some(AgentRuntimeSettingsDb {
        providers,
        active_mode,
        hooks_json,
    }))
}

pub fn set_agent_runtime_settings_db(
    ship_dir: &Path,
    providers: &[String],
    active_mode: Option<&str>,
    hooks_json: &str,
) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    let providers_json = serde_json::to_string(providers)
        .with_context(|| "Failed to serialize providers for agent runtime settings")?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO agent_runtime_settings (id, providers_json, active_mode, hooks_json, updated_at)
             VALUES (1, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               providers_json = excluded.providers_json,
               active_mode = excluded.active_mode,
               hooks_json = excluded.hooks_json,
               updated_at = excluded.updated_at",
        )
        .bind(&providers_json)
        .bind(active_mode)
        .bind(hooks_json)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn upsert_agent_artifact_registry_db(
    ship_dir: &Path,
    kind: &str,
    external_id: &str,
    name: &str,
    source_path: &str,
    content_hash: &str,
) -> Result<String> {
    let mut conn = open_project_db(ship_dir)?;
    let existing_uuid = block_on(async {
        sqlx::query(
            "SELECT uuid
             FROM agent_artifact_registry
             WHERE kind = ? AND external_id = ?",
        )
        .bind(kind)
        .bind(external_id)
        .fetch_optional(&mut conn)
        .await
    })?
    .map(|row| row.get::<String, _>(0));

    let uuid = existing_uuid.unwrap_or_else(crate::gen_nanoid);
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO agent_artifact_registry
                (uuid, kind, external_id, name, source_path, content_hash, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(kind, external_id) DO UPDATE SET
               name = excluded.name,
               source_path = excluded.source_path,
               content_hash = excluded.content_hash,
               updated_at = excluded.updated_at",
        )
        .bind(&uuid)
        .bind(kind)
        .bind(external_id)
        .bind(name)
        .bind(source_path)
        .bind(content_hash)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;

    Ok(uuid)
}

pub fn get_agent_artifact_registry_by_uuid_db(
    ship_dir: &Path,
    kind: &str,
    uuid: &str,
) -> Result<Option<AgentArtifactRegistryDb>> {
    let mut conn = open_project_db(ship_dir)?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT uuid, kind, external_id, name, source_path, content_hash
             FROM agent_artifact_registry
             WHERE kind = ? AND uuid = ?",
        )
        .bind(kind)
        .bind(uuid)
        .fetch_optional(&mut conn)
        .await
    })?;

    let Some(row) = row_opt else {
        return Ok(None);
    };

    Ok(Some(AgentArtifactRegistryDb {
        uuid: row.get(0),
        kind: row.get(1),
        external_id: row.get(2),
        name: row.get(3),
        source_path: row.get(4),
        content_hash: row.get(5),
    }))
}

pub fn get_agent_artifact_registry_by_external_id_db(
    ship_dir: &Path,
    kind: &str,
    external_id: &str,
) -> Result<Option<AgentArtifactRegistryDb>> {
    let mut conn = open_project_db(ship_dir)?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT uuid, kind, external_id, name, source_path, content_hash
             FROM agent_artifact_registry
             WHERE kind = ? AND external_id = ?",
        )
        .bind(kind)
        .bind(external_id)
        .fetch_optional(&mut conn)
        .await
    })?;

    let Some(row) = row_opt else {
        return Ok(None);
    };

    Ok(Some(AgentArtifactRegistryDb {
        uuid: row.get(0),
        kind: row.get(1),
        external_id: row.get(2),
        name: row.get(3),
        source_path: row.get(4),
        content_hash: row.get(5),
    }))
}

pub fn list_agent_modes_db(ship_dir: &Path) -> Result<Vec<AgentModeDb>> {
    let mut conn = open_project_db(ship_dir)?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT id, name, description, active_tools_json, mcp_refs_json, skill_refs_json, rule_refs_json, prompt_id, hooks_json, permissions_json, target_agents_json
             FROM agent_mode
             ORDER BY id ASC",
        )
        .fetch_all(&mut conn)
        .await
    })?;

    let mut modes = Vec::with_capacity(rows.len());
    for row in rows {
        modes.push(AgentModeDb {
            id: row.get(0),
            name: row.get(1),
            description: row.get(2),
            active_tools_json: row.get(3),
            mcp_refs_json: row.get(4),
            skill_refs_json: row.get(5),
            rule_refs_json: row.get(6),
            prompt_id: row.get(7),
            hooks_json: row.get(8),
            permissions_json: row.get(9),
            target_agents_json: row.get(10),
        });
    }
    Ok(modes)
}

pub fn upsert_agent_mode_db(ship_dir: &Path, mode: &AgentModeDb) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO agent_mode
                (id, name, description, active_tools_json, mcp_refs_json, skill_refs_json, rule_refs_json, prompt_id, hooks_json, permissions_json, target_agents_json, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name,
               description = excluded.description,
               active_tools_json = excluded.active_tools_json,
               mcp_refs_json = excluded.mcp_refs_json,
               skill_refs_json = excluded.skill_refs_json,
               rule_refs_json = excluded.rule_refs_json,
               prompt_id = excluded.prompt_id,
               hooks_json = excluded.hooks_json,
               permissions_json = excluded.permissions_json,
               target_agents_json = excluded.target_agents_json,
               updated_at = excluded.updated_at",
        )
        .bind(&mode.id)
        .bind(&mode.name)
        .bind(&mode.description)
        .bind(&mode.active_tools_json)
        .bind(&mode.mcp_refs_json)
        .bind(&mode.skill_refs_json)
        .bind(&mode.rule_refs_json)
        .bind(&mode.prompt_id)
        .bind(&mode.hooks_json)
        .bind(&mode.permissions_json)
        .bind(&mode.target_agents_json)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn delete_agent_mode_db(ship_dir: &Path, id: &str) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    block_on(async {
        sqlx::query("DELETE FROM agent_mode WHERE id = ?")
            .bind(id)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

/// Look up which linked entity is associated with `branch`.
/// Returns `(link_type, link_id)` or `None`.
pub fn get_branch_link(ship_dir: &Path, branch: &str) -> Result<Option<(String, String)>> {
    let mut conn = open_project_db(ship_dir)?;
    let has_legacy_doc_columns = column_exists(&mut conn, "branch_context", "doc_type")?
        && column_exists(&mut conn, "branch_context", "doc_id")?;
    let sql = if has_legacy_doc_columns {
        "SELECT
           COALESCE(NULLIF(link_type, ''), doc_type),
           COALESCE(NULLIF(link_id, ''), doc_id)
         FROM branch_context
         WHERE branch = ?"
    } else {
        "SELECT link_type, link_id FROM branch_context WHERE branch = ?"
    };
    let row_opt = block_on(async {
        sqlx::query(sql)
            .bind(branch)
            .fetch_optional(&mut conn)
            .await
    })?;
    if let Some(row) = row_opt {
        use sqlx::Row;
        Ok(Some((row.get(0), row.get(1))))
    } else {
        Ok(None)
    }
}

/// Record that `branch` is associated with `link_type` and entity id.
pub fn set_branch_link(
    ship_dir: &Path,
    branch: &str,
    link_type: &str,
    link_id: &str,
) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    let has_legacy_doc_columns = column_exists(&mut conn, "branch_context", "doc_type")?
        && column_exists(&mut conn, "branch_context", "doc_id")?;
    let now = Utc::now().to_rfc3339();
    if has_legacy_doc_columns {
        block_on(async {
            sqlx::query(
                "INSERT INTO branch_context
                   (branch, link_type, link_id, doc_type, doc_id, last_synced)
                 VALUES (?, ?, ?, ?, ?, ?)
                 ON CONFLICT(branch) DO UPDATE SET
                   link_type = excluded.link_type,
                   link_id = excluded.link_id,
                   doc_type = excluded.doc_type,
                   doc_id = excluded.doc_id,
                   last_synced = excluded.last_synced",
            )
            .bind(branch)
            .bind(link_type)
            .bind(link_id)
            .bind(link_type)
            .bind(link_id)
            .bind(&now)
            .execute(&mut conn)
            .await
        })?;
    } else {
        block_on(async {
            sqlx::query(
                "INSERT INTO branch_context (branch, link_type, link_id, last_synced)
                 VALUES (?, ?, ?, ?)
                 ON CONFLICT(branch) DO UPDATE SET
                   link_type = excluded.link_type,
                   link_id = excluded.link_id,
                   last_synced = excluded.last_synced",
            )
            .bind(branch)
            .bind(link_type)
            .bind(link_id)
            .bind(&now)
            .execute(&mut conn)
            .await
        })?;
    }
    Ok(())
}

/// Remove branch link mapping for `branch` when no entity is associated anymore.
pub fn clear_branch_link(ship_dir: &Path, branch: &str) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    block_on(async {
        sqlx::query("DELETE FROM branch_context WHERE branch = ?")
            .bind(branch)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

/// Legacy alias kept for compatibility with older call sites.
pub fn get_branch_doc(ship_dir: &Path, branch: &str) -> Result<Option<(String, String)>> {
    get_branch_link(ship_dir, branch)
}

/// Legacy alias kept for compatibility with older call sites.
pub fn set_branch_doc(ship_dir: &Path, branch: &str, doc_type: &str, doc_uuid: &str) -> Result<()> {
    set_branch_link(ship_dir, branch, doc_type, doc_uuid)
}

/// Legacy alias kept for compatibility with older call sites.
pub fn clear_branch_doc(ship_dir: &Path, branch: &str) -> Result<()> {
    clear_branch_link(ship_dir, branch)
}

/// Look up feature-linked ids used by workspace hydration.
/// Returns `(spec_id, release_id)` when the feature exists.
pub fn get_feature_links(
    ship_dir: &Path,
    feature_id: &str,
) -> Result<Option<(Option<String>, Option<String>)>> {
    let mut conn = open_project_db(ship_dir)?;
    let row_opt = block_on(async {
        sqlx::query("SELECT spec_id, release_id FROM feature WHERE id = ?")
            .bind(feature_id)
            .fetch_optional(&mut conn)
            .await
    })?;
    if let Some(row) = row_opt {
        use sqlx::Row;
        let spec_id: Option<String> = row.get(0);
        let release_id: Option<String> = row.get(1);
        Ok(Some((spec_id, release_id)))
    } else {
        Ok(None)
    }
}

/// Resolve a feature by git branch and return `(feature_id, spec_id, release_id)`.
/// Uses most recently updated row when multiple features share the same branch.
pub fn get_feature_by_branch_links(
    ship_dir: &Path,
    branch: &str,
) -> Result<Option<FeatureBranchLinks>> {
    let mut conn = open_project_db(ship_dir)?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT id, spec_id, release_id
             FROM feature
             WHERE branch = ?
             ORDER BY updated_at DESC
             LIMIT 1",
        )
        .bind(branch)
        .fetch_optional(&mut conn)
        .await
    })?;
    if let Some(row) = row_opt {
        let feature_id: String = row.get(0);
        let spec_id: Option<String> = row.get(1);
        let release_id: Option<String> = row.get(2);
        Ok(Some((feature_id, spec_id, release_id)))
    } else {
        Ok(None)
    }
}

// ─── Path helpers ─────────────────────────────────────────────────────────────

fn ship_global_dir() -> Result<PathBuf> {
    crate::project::get_global_dir()
}

fn ensure_global_dir_outside_project(ship_dir: &Path, global_dir: &Path) -> Result<()> {
    let project_root = ship_dir.parent().unwrap_or(ship_dir);
    let normalize =
        |path: &Path| std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    let ship_dir_norm = normalize(ship_dir);
    let project_root_norm = normalize(project_root);
    let global_dir_norm = normalize(global_dir);

    if global_dir_norm.starts_with(&ship_dir_norm)
        || global_dir_norm.starts_with(&project_root_norm)
    {
        return Err(anyhow!(
            "Resolved global state directory {} is inside project {}. Refusing to write project state locally; expected ~/.ship (or another external absolute path).",
            global_dir_norm.display(),
            project_root_norm.display()
        ));
    }
    Ok(())
}

/// Derives a filesystem-safe slug from the project root path.
/// `/home/alice/dev/my-app` → `home-alice-dev-my-app`
fn project_slug(ship_dir: &Path) -> Result<String> {
    let slug = crate::project::project_slug_from_ship_dir(ship_dir);

    if slug.is_empty() {
        return Err(anyhow!(
            "Could not derive a project slug from {}",
            ship_dir.display()
        ));
    }
    Ok(slug)
}

// ─── Workspace ────────────────────────────────────────────────────────────────

/// Retrieve the workspace record for the given branch, or None if none exists.
pub fn get_workspace_db(ship_dir: &Path, branch: &str) -> Result<Option<WorkspaceDbRow>> {
    let mut conn = open_project_db(ship_dir)?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT COALESCE(id, branch), workspace_type, status, feature_id, spec_id, release_id, active_mode, providers_json, resolved_at, is_worktree, worktree_path, last_activated_at, context_hash, COALESCE(config_generation, 0), compiled_at, compile_error
             FROM workspace WHERE branch = ?",
        )
        .bind(branch)
        .fetch_optional(&mut conn)
        .await
    })?;
    if let Some(row) = row_opt {
        use sqlx::Row;
        let id: String = row.get(0);
        let workspace_type: String = row.get(1);
        let status: String = row.get(2);
        let feature_id: Option<String> = row.get(3);
        let spec_id: Option<String> = row.get(4);
        let release_id: Option<String> = row.get(5);
        let active_mode: Option<String> = row.get(6);
        let providers_json: String = row.get(7);
        let resolved_at: String = row.get(8);
        let is_worktree: i64 = row.get(9);
        let worktree_path: Option<String> = row.get(10);
        let last_activated_at: Option<String> = row.get(11);
        let context_hash: Option<String> = row.get(12);
        let config_generation: i64 = row.get(13);
        let compiled_at: Option<String> = row.get(14);
        let compile_error: Option<String> = row.get(15);
        let providers: Vec<String> = serde_json::from_str(&providers_json).unwrap_or_default();
        Ok(Some((
            id,
            workspace_type,
            status,
            feature_id,
            spec_id,
            release_id,
            active_mode,
            providers,
            resolved_at,
            is_worktree != 0,
            worktree_path,
            last_activated_at,
            context_hash,
            config_generation,
            compiled_at,
            compile_error,
        )))
    } else {
        Ok(None)
    }
}

pub fn list_workspaces_db(ship_dir: &Path) -> Result<Vec<WorkspaceDbListRow>> {
    let mut conn = open_project_db(ship_dir)?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT branch, COALESCE(id, branch), workspace_type, status, feature_id, spec_id, release_id, active_mode, providers_json, resolved_at, is_worktree, worktree_path, last_activated_at, context_hash, COALESCE(config_generation, 0), compiled_at, compile_error
             FROM workspace
             ORDER BY
               CASE status
                 WHEN 'active' THEN 0
                 WHEN 'idle' THEN 1
                 WHEN 'planned' THEN 2
                 WHEN 'review' THEN 3
                 WHEN 'merged' THEN 4
                 WHEN 'archived' THEN 5
                 ELSE 6
               END,
               COALESCE(last_activated_at, resolved_at) DESC",
        )
        .fetch_all(&mut conn)
        .await
    })?;

    use sqlx::Row;
    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let branch: String = row.get(0);
        let id: String = row.get(1);
        let workspace_type: String = row.get(2);
        let status: String = row.get(3);
        let feature_id: Option<String> = row.get(4);
        let spec_id: Option<String> = row.get(5);
        let release_id: Option<String> = row.get(6);
        let active_mode: Option<String> = row.get(7);
        let providers_json: String = row.get(8);
        let resolved_at: String = row.get(9);
        let is_worktree: i64 = row.get(10);
        let worktree_path: Option<String> = row.get(11);
        let last_activated_at: Option<String> = row.get(12);
        let context_hash: Option<String> = row.get(13);
        let config_generation: i64 = row.get(14);
        let compiled_at: Option<String> = row.get(15);
        let compile_error: Option<String> = row.get(16);
        let providers: Vec<String> = serde_json::from_str(&providers_json).unwrap_or_default();

        result.push((
            branch,
            id,
            workspace_type,
            status,
            feature_id,
            spec_id,
            release_id,
            active_mode,
            providers,
            resolved_at,
            is_worktree != 0,
            worktree_path,
            last_activated_at,
            context_hash,
            config_generation,
            compiled_at,
            compile_error,
        ));
    }
    Ok(result)
}

/// Upsert the workspace record for the given branch.
pub fn upsert_workspace_db(ship_dir: &Path, record: WorkspaceUpsert<'_>) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    let providers_json = serde_json::to_string(record.providers)
        .with_context(|| "Failed to serialize workspace providers")?;
    block_on(async {
        sqlx::query(
            "INSERT INTO workspace (branch, id, workspace_type, status, feature_id, spec_id, release_id, active_mode, providers_json, resolved_at, is_worktree, worktree_path, last_activated_at, context_hash, config_generation, compiled_at, compile_error)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(branch) DO UPDATE SET
               id            = excluded.id,
               workspace_type = excluded.workspace_type,
               status        = excluded.status,
               feature_id    = excluded.feature_id,
               spec_id       = excluded.spec_id,
               release_id    = excluded.release_id,
               active_mode   = excluded.active_mode,
               providers_json = excluded.providers_json,
               resolved_at   = excluded.resolved_at,
               is_worktree   = excluded.is_worktree,
               worktree_path = excluded.worktree_path,
               last_activated_at = excluded.last_activated_at,
               context_hash = excluded.context_hash,
               config_generation = excluded.config_generation,
               compiled_at = excluded.compiled_at,
               compile_error = excluded.compile_error",
        )
        .bind(record.branch)
        .bind(record.workspace_id)
        .bind(record.workspace_type)
        .bind(record.status)
        .bind(record.feature_id)
        .bind(record.spec_id)
        .bind(record.release_id)
        .bind(record.active_mode)
        .bind(&providers_json)
        .bind(record.resolved_at)
        .bind(if record.is_worktree { 1i64 } else { 0i64 })
        .bind(record.worktree_path)
        .bind(record.last_activated_at)
        .bind(record.context_hash)
        .bind(record.config_generation)
        .bind(record.compiled_at)
        .bind(record.compile_error)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

/// Delete workspace state for a branch, including any session history.
pub fn delete_workspace_db(ship_dir: &Path, branch: &str) -> Result<bool> {
    let mut conn = open_project_db(ship_dir)?;
    let workspace_id = block_on(async {
        sqlx::query_scalar::<_, String>(
            "SELECT COALESCE(id, branch) FROM workspace WHERE branch = ?",
        )
        .bind(branch)
        .fetch_optional(&mut conn)
        .await
    })?;

    let Some(workspace_id) = workspace_id else {
        return Ok(false);
    };

    let deleted = block_on(async {
        sqlx::query("DELETE FROM workspace_session WHERE workspace_id = ? OR workspace_branch = ?")
            .bind(&workspace_id)
            .bind(branch)
            .execute(&mut conn)
            .await?;

        sqlx::query("UPDATE spec SET workspace_id = NULL WHERE workspace_id = ?")
            .bind(&workspace_id)
            .execute(&mut conn)
            .await?;

        let result = sqlx::query("DELETE FROM workspace WHERE branch = ?")
            .bind(branch)
            .execute(&mut conn)
            .await?;

        Ok::<bool, sqlx::Error>(result.rows_affected() > 0)
    })?;

    Ok(deleted)
}

/// Mark any currently active workspace as idle except `active_branch`.
pub fn demote_other_active_workspaces_db(
    ship_dir: &Path,
    active_branch: &str,
    resolved_at: &str,
) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    block_on(async {
        sqlx::query(
            "UPDATE workspace
             SET status = 'idle', resolved_at = ?
             WHERE status = 'active' AND branch != ?",
        )
        .bind(resolved_at)
        .bind(active_branch)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

fn parse_workspace_session_row(row: &sqlx::sqlite::SqliteRow) -> WorkspaceSessionDb {
    let updated_feature_ids_json: String = row.get(10);
    let updated_spec_ids_json: String = row.get(11);
    let updated_feature_ids = serde_json::from_str(&updated_feature_ids_json).unwrap_or_default();
    let updated_spec_ids = serde_json::from_str(&updated_spec_ids_json).unwrap_or_default();
    WorkspaceSessionDb {
        id: row.get(0),
        workspace_id: row.get(1),
        workspace_branch: row.get(2),
        status: row.get(3),
        started_at: row.get(4),
        ended_at: row.get(5),
        mode_id: row.get(6),
        primary_provider: row.get(7),
        goal: row.get(8),
        summary: row.get(9),
        updated_feature_ids,
        updated_spec_ids,
        compiled_at: row.get(12),
        compile_error: row.get(13),
        config_generation_at_start: row.get(14),
        created_at: row.get(15),
        updated_at: row.get(16),
    }
}

pub fn get_workspace_session_db(
    ship_dir: &Path,
    session_id: &str,
) -> Result<Option<WorkspaceSessionDb>> {
    let mut conn = open_project_db(ship_dir)?;
    let row = block_on(async {
        sqlx::query(
            "SELECT id, workspace_id, workspace_branch, status, started_at, ended_at, mode_id, primary_provider, goal, summary, updated_feature_ids_json, updated_spec_ids_json, compiled_at, compile_error, config_generation_at_start, created_at, updated_at
             FROM workspace_session
             WHERE id = ?",
        )
        .bind(session_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(parse_workspace_session_row))
}

pub fn get_active_workspace_session_db(
    ship_dir: &Path,
    workspace_id: &str,
) -> Result<Option<WorkspaceSessionDb>> {
    let mut conn = open_project_db(ship_dir)?;
    let row = block_on(async {
        sqlx::query(
            "SELECT id, workspace_id, workspace_branch, status, started_at, ended_at, mode_id, primary_provider, goal, summary, updated_feature_ids_json, updated_spec_ids_json, compiled_at, compile_error, config_generation_at_start, created_at, updated_at
             FROM workspace_session
             WHERE workspace_id = ? AND status = 'active'
             ORDER BY started_at DESC
             LIMIT 1",
        )
        .bind(workspace_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(parse_workspace_session_row))
}

pub fn list_workspace_sessions_db(
    ship_dir: &Path,
    workspace_id: Option<&str>,
    limit: usize,
) -> Result<Vec<WorkspaceSessionDb>> {
    let mut conn = open_project_db(ship_dir)?;
    let clamped_limit = limit.clamp(1, 500) as i64;
    let rows = if let Some(workspace_id) = workspace_id {
        block_on(async {
            sqlx::query(
                "SELECT id, workspace_id, workspace_branch, status, started_at, ended_at, mode_id, primary_provider, goal, summary, updated_feature_ids_json, updated_spec_ids_json, compiled_at, compile_error, config_generation_at_start, created_at, updated_at
                 FROM workspace_session
                 WHERE workspace_id = ?
                 ORDER BY started_at DESC
                 LIMIT ?",
            )
            .bind(workspace_id)
            .bind(clamped_limit)
            .fetch_all(&mut conn)
            .await
        })?
    } else {
        block_on(async {
            sqlx::query(
                "SELECT id, workspace_id, workspace_branch, status, started_at, ended_at, mode_id, primary_provider, goal, summary, updated_feature_ids_json, updated_spec_ids_json, compiled_at, compile_error, config_generation_at_start, created_at, updated_at
                 FROM workspace_session
                 ORDER BY started_at DESC
                 LIMIT ?",
            )
            .bind(clamped_limit)
            .fetch_all(&mut conn)
            .await
        })?
    };

    Ok(rows.iter().map(parse_workspace_session_row).collect())
}

pub fn insert_workspace_session_db(ship_dir: &Path, session: &WorkspaceSessionDb) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    let updated_feature_ids_json = serde_json::to_string(&session.updated_feature_ids)
        .with_context(|| "Failed to serialize workspace session updated_feature_ids")?;
    let updated_spec_ids_json = serde_json::to_string(&session.updated_spec_ids)
        .with_context(|| "Failed to serialize workspace session updated_spec_ids")?;
    block_on(async {
        sqlx::query(
            "INSERT INTO workspace_session
             (id, workspace_id, workspace_branch, status, started_at, ended_at, mode_id, primary_provider, goal, summary, updated_feature_ids_json, updated_spec_ids_json, compiled_at, compile_error, config_generation_at_start, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&session.id)
        .bind(&session.workspace_id)
        .bind(&session.workspace_branch)
        .bind(&session.status)
        .bind(&session.started_at)
        .bind(&session.ended_at)
        .bind(&session.mode_id)
        .bind(&session.primary_provider)
        .bind(&session.goal)
        .bind(&session.summary)
        .bind(&updated_feature_ids_json)
        .bind(&updated_spec_ids_json)
        .bind(&session.compiled_at)
        .bind(&session.compile_error)
        .bind(session.config_generation_at_start)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn update_workspace_session_db(ship_dir: &Path, session: &WorkspaceSessionDb) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    let updated_feature_ids_json = serde_json::to_string(&session.updated_feature_ids)
        .with_context(|| "Failed to serialize workspace session updated_feature_ids")?;
    let updated_spec_ids_json = serde_json::to_string(&session.updated_spec_ids)
        .with_context(|| "Failed to serialize workspace session updated_spec_ids")?;
    block_on(async {
        sqlx::query(
            "UPDATE workspace_session
             SET workspace_id = ?,
                 workspace_branch = ?,
                 status = ?,
                 started_at = ?,
                 ended_at = ?,
                 mode_id = ?,
                 primary_provider = ?,
                 goal = ?,
                 summary = ?,
                 updated_feature_ids_json = ?,
                 updated_spec_ids_json = ?,
                 compiled_at = ?,
                 compile_error = ?,
                 config_generation_at_start = ?,
                 created_at = ?,
                 updated_at = ?
             WHERE id = ?",
        )
        .bind(&session.workspace_id)
        .bind(&session.workspace_branch)
        .bind(&session.status)
        .bind(&session.started_at)
        .bind(&session.ended_at)
        .bind(&session.mode_id)
        .bind(&session.primary_provider)
        .bind(&session.goal)
        .bind(&session.summary)
        .bind(&updated_feature_ids_json)
        .bind(&updated_spec_ids_json)
        .bind(&session.compiled_at)
        .bind(&session.compile_error)
        .bind(session.config_generation_at_start)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .bind(&session.id)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

// ─── Core ─────────────────────────────────────────────────────────────────────

fn ensure_database(db_path: &Path, migrations: &[(&str, &str)]) -> Result<DatabaseMigrationReport> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let created = !db_path.exists();

    let db_url = sqlite_url(db_path);
    let options = SqliteConnectOptions::from_str(&db_url)
        .with_context(|| format!("Invalid sqlite url {}", db_url))?
        .create_if_missing(true);
    let mut connection = block_on(async { SqliteConnection::connect_with(&options).await })?;

    // Keep write behavior stable across CLI/UI/MCP call paths.
    block_on(async {
        sqlx::query("PRAGMA journal_mode = WAL;")
            .execute(&mut connection)
            .await
    })?;
    block_on(async {
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&mut connection)
            .await
    })?;

    let mut applied = 0usize;
    for (version, ddl) in migrations {
        let has_row = block_on(async {
            sqlx::query("SELECT version FROM schema_migrations WHERE version = ?")
                .bind(*version)
                .fetch_optional(&mut connection)
                .await
        })
        .ok()
        .flatten()
        .is_some();
        if has_row {
            continue;
        }

        block_on(async { sqlx::query(ddl).execute(&mut connection).await }).with_context(|| {
            format!(
                "Failed applying schema migration {} for {}",
                version,
                db_path.display()
            )
        })?;
        block_on(async {
            sqlx::query("INSERT INTO schema_migrations (version, applied_at) VALUES (?, ?)")
                .bind(*version)
                .bind(Utc::now().to_rfc3339())
                .execute(&mut connection)
                .await
        })
        .with_context(|| {
            format!(
                "Failed recording schema migration {} for {}",
                version,
                db_path.display()
            )
        })?;
        applied += 1;
    }

    Ok(DatabaseMigrationReport {
        db_path: db_path.to_path_buf(),
        created,
        applied_migrations: applied,
    })
}

fn ensure_project_schema_compat(connection: &mut SqliteConnection) -> Result<()> {
    ensure_column(
        connection,
        "feature",
        "branch",
        "ALTER TABLE feature ADD COLUMN branch TEXT",
    )?;
    ensure_column(
        connection,
        "feature",
        "agent_json",
        "ALTER TABLE feature ADD COLUMN agent_json TEXT",
    )?;
    ensure_column(
        connection,
        "feature",
        "tags_json",
        "ALTER TABLE feature ADD COLUMN tags_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    let added_feature_active_target = ensure_column(
        connection,
        "feature",
        "active_target_id",
        "ALTER TABLE feature ADD COLUMN active_target_id TEXT",
    )?;
    if table_exists(connection, "feature")? && column_exists(connection, "feature", "active_target_id")? {
        block_on(async {
            sqlx::query(
                "CREATE INDEX IF NOT EXISTS feature_active_target_idx ON feature(active_target_id)",
            )
            .execute(&mut *connection)
            .await
        })?;
    }
    if added_feature_active_target
        && table_exists(connection, "feature")?
        && column_exists(connection, "feature", "release_id")?
    {
        block_on(async {
            sqlx::query(
                "UPDATE feature
                 SET active_target_id = release_id
                 WHERE (active_target_id IS NULL OR active_target_id = '')
                   AND release_id IS NOT NULL
                   AND release_id != ''",
            )
            .execute(&mut *connection)
            .await
        })?;
    }
    ensure_column(
        connection,
        "release",
        "target_date",
        "ALTER TABLE release ADD COLUMN target_date TEXT",
    )?;
    ensure_column(
        connection,
        "release",
        "supported",
        "ALTER TABLE release ADD COLUMN supported INTEGER",
    )?;
    ensure_column(
        connection,
        "issue",
        "description",
        "ALTER TABLE issue ADD COLUMN description TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        connection,
        "spec",
        "body",
        "ALTER TABLE spec ADD COLUMN body TEXT NOT NULL DEFAULT ''",
    )?;
    let added_spec_workspace_id = ensure_column(
        connection,
        "spec",
        "workspace_id",
        "ALTER TABLE spec ADD COLUMN workspace_id TEXT",
    )?;
    if table_exists(connection, "spec")? && column_exists(connection, "spec", "workspace_id")? {
        block_on(async {
            sqlx::query("CREATE INDEX IF NOT EXISTS spec_workspace_idx ON spec(workspace_id)")
                .execute(&mut *connection)
                .await
        })?;
    }

    let added_branch_link_type = ensure_column(
        connection,
        "branch_context",
        "link_type",
        "ALTER TABLE branch_context ADD COLUMN link_type TEXT",
    )?;
    let added_branch_link_id = ensure_column(
        connection,
        "branch_context",
        "link_id",
        "ALTER TABLE branch_context ADD COLUMN link_id TEXT",
    )?;
    if table_exists(connection, "branch_context")?
        && (added_branch_link_type || added_branch_link_id)
        && column_exists(connection, "branch_context", "doc_type")?
        && column_exists(connection, "branch_context", "doc_id")?
    {
        block_on(async {
            sqlx::query(
                "UPDATE branch_context
                 SET link_type = COALESCE(NULLIF(link_type, ''), doc_type),
                     link_id = COALESCE(NULLIF(link_id, ''), doc_id)",
            )
            .execute(&mut *connection)
            .await
        })?;
    }

    let added_workspace_id = ensure_column(
        connection,
        "workspace",
        "id",
        "ALTER TABLE workspace ADD COLUMN id TEXT",
    )?;
    ensure_column(
        connection,
        "workspace",
        "workspace_type",
        "ALTER TABLE workspace ADD COLUMN workspace_type TEXT NOT NULL DEFAULT 'feature'",
    )?;
    let added_workspace_status = ensure_column(
        connection,
        "workspace",
        "status",
        "ALTER TABLE workspace ADD COLUMN status TEXT NOT NULL DEFAULT 'idle'",
    )?;
    ensure_column(
        connection,
        "workspace",
        "release_id",
        "ALTER TABLE workspace ADD COLUMN release_id TEXT",
    )?;
    ensure_column(
        connection,
        "workspace",
        "last_activated_at",
        "ALTER TABLE workspace ADD COLUMN last_activated_at TEXT",
    )?;
    ensure_column(
        connection,
        "workspace",
        "context_hash",
        "ALTER TABLE workspace ADD COLUMN context_hash TEXT",
    )?;
    ensure_column(
        connection,
        "workspace",
        "config_generation",
        "ALTER TABLE workspace ADD COLUMN config_generation INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        connection,
        "workspace",
        "compiled_at",
        "ALTER TABLE workspace ADD COLUMN compiled_at TEXT",
    )?;
    ensure_column(
        connection,
        "workspace",
        "compile_error",
        "ALTER TABLE workspace ADD COLUMN compile_error TEXT",
    )?;
    ensure_column(
        connection,
        "workspace_session",
        "primary_provider",
        "ALTER TABLE workspace_session ADD COLUMN primary_provider TEXT",
    )?;
    ensure_column(
        connection,
        "workspace_session",
        "compiled_at",
        "ALTER TABLE workspace_session ADD COLUMN compiled_at TEXT",
    )?;
    ensure_column(
        connection,
        "workspace_session",
        "compile_error",
        "ALTER TABLE workspace_session ADD COLUMN compile_error TEXT",
    )?;
    ensure_column(
        connection,
        "workspace_session",
        "config_generation_at_start",
        "ALTER TABLE workspace_session ADD COLUMN config_generation_at_start INTEGER",
    )?;

    if table_exists(connection, "workspace")? {
        if added_workspace_id {
            block_on(async {
                sqlx::query(
                    "UPDATE workspace
                     SET id = branch
                     WHERE id IS NULL OR id = ''",
                )
                .execute(&mut *connection)
                .await
            })?;
        }
        if added_workspace_status {
            // Existing pre-lifecycle rows represented currently checked-out work.
            // Preserve that behavior once when the status column is introduced.
            block_on(async {
                sqlx::query(
                    "UPDATE workspace
                     SET status = 'active'
                     WHERE status IS NULL OR status = ''",
                )
                .execute(&mut *connection)
                .await
            })?;
        }
    }

    if table_exists(connection, "spec")?
        && table_exists(connection, "workspace")?
        && (added_spec_workspace_id || column_exists(connection, "spec", "workspace_id")?)
    {
        block_on(async {
            sqlx::query(
                "UPDATE spec
                 SET workspace_id = (
                   SELECT w.id
                   FROM workspace w
                   WHERE (spec.branch IS NOT NULL AND spec.branch != '' AND w.branch = spec.branch)
                      OR (spec.feature_id IS NOT NULL AND spec.feature_id != '' AND w.feature_id = spec.feature_id)
                   ORDER BY
                     CASE WHEN w.status = 'active' THEN 0 ELSE 1 END,
                     COALESCE(w.last_activated_at, w.resolved_at) DESC
                   LIMIT 1
                 )
                 WHERE (workspace_id IS NULL OR workspace_id = '')",
            )
            .execute(&mut *connection)
            .await
        })?;
    }

    Ok(())
}

fn ensure_column(
    connection: &mut SqliteConnection,
    table: &str,
    column: &str,
    alter_sql: &str,
) -> Result<bool> {
    if !table_exists(connection, table)? {
        return Ok(false);
    }

    if column_exists(connection, table, column)? {
        return Ok(false);
    }

    block_on(async { sqlx::query(alter_sql).execute(&mut *connection).await })
        .with_context(|| format!("Failed applying compatibility column {}.{}", table, column))?;
    Ok(true)
}

fn table_exists(connection: &mut SqliteConnection, table: &str) -> Result<bool> {
    let row = block_on(async {
        sqlx::query("SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?")
            .bind(table)
            .fetch_optional(&mut *connection)
            .await
    })?;
    Ok(row.is_some())
}

fn column_exists(connection: &mut SqliteConnection, table: &str, column: &str) -> Result<bool> {
    let pragma = format!("PRAGMA table_info({})", table);
    let rows = block_on(async { sqlx::query(&pragma).fetch_all(&mut *connection).await })?;
    Ok(rows
        .iter()
        .any(|row| row.get::<String, _>(1).eq_ignore_ascii_case(column)))
}

fn sqlite_url(path: &Path) -> String {
    let mut raw = path.to_string_lossy().replace('\\', "/");
    if !raw.starts_with('/') {
        raw = format!("/{}", raw);
    }
    format!("sqlite://{}", raw)
}

pub fn block_on<F, T>(future: F) -> Result<T>
where
    F: std::future::Future<Output = std::result::Result<T, sqlx::Error>>,
{
    // If we are inside a Tokio runtime (e.g. spawn_blocking thread or MCP
    // async handler), use block_in_place to run the future without blocking
    // the scheduler thread. block_in_place is safe to call from any thread
    // that is within the Tokio threadpool, including blocking threads.
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return tokio::task::block_in_place(|| {
            handle
                .block_on(future)
                .map_err(|err| anyhow!("SQLite operation failed: {}", err))
        });
    }
    // No runtime active — create a lightweight single-threaded one.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|err| anyhow!("Failed to create SQLite runtime: {}", err))?;
    runtime
        .block_on(future)
        .map_err(|err| anyhow!("SQLite operation failed: {}", err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn ensure_project_database_is_idempotent() -> Result<()> {
        let tmp = tempdir()?;
        // ship_dir must have a parent (project root) to derive the slug
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        // Resolve once to avoid environment-race induced global-dir drift between calls.
        let db_path = project_db_path(&ship_dir)?;
        let report_a = ensure_database(&db_path, PROJECT_MIGRATIONS)?;
        let report_b = ensure_database(&db_path, PROJECT_MIGRATIONS)?;

        assert!(report_a.created);
        assert!(report_a.applied_migrations >= 1);
        assert_eq!(report_a.db_path, report_b.db_path);
        assert!(!report_b.created);
        assert_eq!(report_b.applied_migrations, 0);
        // DB lives outside the project dir
        assert!(!report_a.db_path.starts_with(tmp.path()));
        assert!(report_a.db_path.to_string_lossy().contains("ship.db"));

        // Clean up the DB we just created in ~/.ship/state/
        std::fs::remove_file(&report_a.db_path).ok();
        Ok(())
    }

    #[test]
    fn project_slug_strips_leading_slash_and_collapses_separators() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        let slug = project_slug(&ship_dir)?;
        assert!(!slug.starts_with('-'), "slug should not start with a dash");
        assert!(
            !slug.contains("--"),
            "slug should not contain consecutive dashes"
        );
        assert!(!slug.is_empty());
        Ok(())
    }

    #[test]
    fn compat_workspace_backfills_only_when_columns_are_added() -> Result<()> {
        let tmp = tempdir()?;
        let db_path = tmp.path().join("compat.db");
        let db_url = sqlite_url(&db_path);
        let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
        let mut conn = block_on(async { SqliteConnection::connect_with(&options).await })?;

        block_on(async {
            sqlx::query(
                "CREATE TABLE workspace (
                   branch TEXT PRIMARY KEY,
                   feature_id TEXT,
                   spec_id TEXT,
                   active_mode TEXT,
                   providers_json TEXT NOT NULL DEFAULT '[]',
                   resolved_at TEXT NOT NULL,
                   is_worktree INTEGER NOT NULL DEFAULT 0,
                   worktree_path TEXT
                 )",
            )
            .execute(&mut conn)
            .await
        })?;
        block_on(async {
            sqlx::query(
                "INSERT INTO workspace (branch, resolved_at) VALUES ('feature/demo', '2026-01-01T00:00:00Z')",
            )
            .execute(&mut conn)
            .await
        })?;

        ensure_project_schema_compat(&mut conn)?;

        let row = block_on(async {
            sqlx::query("SELECT id, status FROM workspace WHERE branch = 'feature/demo'")
                .fetch_one(&mut conn)
                .await
        })?;
        let id: Option<String> = row.get(0);
        let status: Option<String> = row.get(1);
        assert_eq!(id.as_deref(), Some("feature/demo"));
        assert_eq!(status.as_deref(), Some("idle"));
        Ok(())
    }

    #[test]
    fn compat_branch_context_backfills_link_columns_from_legacy_doc_columns() -> Result<()> {
        let tmp = tempdir()?;
        let db_path = tmp.path().join("branch-context-compat.db");
        let db_url = sqlite_url(&db_path);
        let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
        let mut conn = block_on(async { SqliteConnection::connect_with(&options).await })?;

        block_on(async {
            sqlx::query(
                "CREATE TABLE branch_context (
                   branch TEXT PRIMARY KEY,
                   doc_type TEXT NOT NULL,
                   doc_id TEXT NOT NULL,
                   last_synced TEXT NOT NULL
                 )",
            )
            .execute(&mut conn)
            .await
        })?;
        block_on(async {
            sqlx::query(
                "INSERT INTO branch_context (branch, doc_type, doc_id, last_synced)
                 VALUES ('feature/auth', 'feature', 'ABC123', '2026-01-01T00:00:00Z')",
            )
            .execute(&mut conn)
            .await
        })?;

        ensure_project_schema_compat(&mut conn)?;

        let row = block_on(async {
            sqlx::query(
                "SELECT link_type, link_id
                 FROM branch_context
                 WHERE branch = 'feature/auth'",
            )
            .fetch_one(&mut conn)
            .await
        })?;
        let link_type: Option<String> = row.get(0);
        let link_id: Option<String> = row.get(1);
        assert_eq!(link_type.as_deref(), Some("feature"));
        assert_eq!(link_id.as_deref(), Some("ABC123"));
        Ok(())
    }

    #[test]
    fn compat_adds_issue_description_and_spec_workspace_columns() -> Result<()> {
        let tmp = tempdir()?;
        let db_path = tmp.path().join("issue-spec-compat.db");
        let db_url = sqlite_url(&db_path);
        let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
        let mut conn = block_on(async { SqliteConnection::connect_with(&options).await })?;

        block_on(async {
            sqlx::query(
                "CREATE TABLE issue (
                   id TEXT PRIMARY KEY,
                   title TEXT NOT NULL,
                   status TEXT NOT NULL DEFAULT 'backlog',
                   assignee TEXT,
                   priority TEXT,
                   release_id TEXT,
                   feature_id TEXT,
                   spec_id TEXT,
                   tags_json TEXT NOT NULL DEFAULT '[]',
                   links_json TEXT NOT NULL DEFAULT '[]',
                   created_at TEXT NOT NULL,
                   updated_at TEXT NOT NULL
                 )",
            )
            .execute(&mut conn)
            .await
        })?;
        block_on(async {
            sqlx::query(
                "CREATE TABLE spec (
                   id TEXT PRIMARY KEY,
                   title TEXT NOT NULL,
                   status TEXT NOT NULL DEFAULT 'draft',
                   author TEXT,
                   branch TEXT,
                   feature_id TEXT,
                   release_id TEXT,
                   tags_json TEXT NOT NULL DEFAULT '[]',
                   created_at TEXT NOT NULL,
                   updated_at TEXT NOT NULL
                 )",
            )
            .execute(&mut conn)
            .await
        })?;

        ensure_project_schema_compat(&mut conn)?;

        assert!(column_exists(&mut conn, "issue", "description")?);
        assert!(column_exists(&mut conn, "spec", "body")?);
        assert!(column_exists(&mut conn, "spec", "workspace_id")?);
        Ok(())
    }

    #[test]
    fn compat_backfills_spec_workspace_id_from_workspace_branch_or_feature() -> Result<()> {
        let tmp = tempdir()?;
        let db_path = tmp.path().join("spec-workspace-backfill.db");
        let db_url = sqlite_url(&db_path);
        let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
        let mut conn = block_on(async { SqliteConnection::connect_with(&options).await })?;

        block_on(async {
            sqlx::query(
                "CREATE TABLE workspace (
                   branch TEXT PRIMARY KEY,
                   id TEXT,
                   workspace_type TEXT NOT NULL DEFAULT 'feature',
                   status TEXT NOT NULL DEFAULT 'planned',
                   feature_id TEXT,
                   spec_id TEXT,
                   release_id TEXT,
                   active_mode TEXT,
                   providers_json TEXT NOT NULL DEFAULT '[]',
                   resolved_at TEXT NOT NULL DEFAULT '',
                   is_worktree INTEGER NOT NULL DEFAULT 0,
                   worktree_path TEXT,
                   last_activated_at TEXT,
                   context_hash TEXT
                 )",
            )
            .execute(&mut conn)
            .await?;
            sqlx::query(
                "INSERT INTO workspace (branch, id, status, feature_id, resolved_at)
                 VALUES ('feature/auth', 'feature-auth', 'active', 'feat-auth', '2026-01-01T00:00:00Z')",
            )
            .execute(&mut conn)
            .await?;
            Ok::<_, sqlx::Error>(())
        })?;

        block_on(async {
            sqlx::query(
                "CREATE TABLE spec (
                   id TEXT PRIMARY KEY,
                   title TEXT NOT NULL,
                   status TEXT NOT NULL DEFAULT 'draft',
                   author TEXT,
                   branch TEXT,
                   feature_id TEXT,
                   release_id TEXT,
                   tags_json TEXT NOT NULL DEFAULT '[]',
                   created_at TEXT NOT NULL,
                   updated_at TEXT NOT NULL
                 )",
            )
            .execute(&mut conn)
            .await?;
            sqlx::query(
                "INSERT INTO spec (id, title, branch, feature_id, tags_json, created_at, updated_at)
                 VALUES ('spec-auth', 'Auth Spec', 'feature/auth', 'feat-auth', '[]', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            )
            .execute(&mut conn)
            .await?;
            Ok::<_, sqlx::Error>(())
        })?;

        ensure_project_schema_compat(&mut conn)?;

        let workspace_id: Option<String> = block_on(async {
            sqlx::query("SELECT workspace_id FROM spec WHERE id = 'spec-auth'")
                .fetch_one(&mut conn)
                .await
                .map(|row| row.get(0))
        })?;
        assert_eq!(workspace_id.as_deref(), Some("feature-auth"));

        Ok(())
    }

    #[test]
    fn migration_meta_project_roundtrip() -> Result<()> {
        let tmp = tempdir()?;
        let db_path = tmp.path().join("migration-meta.db");
        ensure_database(&db_path, PROJECT_MIGRATIONS)?;

        let db_url = sqlite_url(&db_path);
        let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
        let mut conn = block_on(async { SqliteConnection::connect_with(&options).await })?;

        assert!(!migration_meta_complete(&mut conn, "feature")?);
        assert!(!migration_meta_complete(&mut conn, "release")?);

        mark_migration_meta_complete(&mut conn, "feature", 3)?;
        mark_migration_meta_complete(&mut conn, "release", 1)?;

        assert!(migration_meta_complete(&mut conn, "feature")?);
        assert!(migration_meta_complete(&mut conn, "release")?);

        let cleared = clear_migration_meta(&mut conn)?;
        assert_eq!(cleared, 2);
        assert!(!migration_meta_complete(&mut conn, "feature")?);
        assert!(!migration_meta_complete(&mut conn, "release")?);
        Ok(())
    }

    #[test]
    fn rejects_global_state_dir_inside_project_tree() -> Result<()> {
        let tmp = tempdir()?;
        let project_root = tmp.path().join("repo");
        let ship_dir = project_root.join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        let local_global = ship_dir.join("state");
        std::fs::create_dir_all(&local_global)?;

        let err = ensure_global_dir_outside_project(&ship_dir, &local_global).unwrap_err();
        assert!(err.to_string().contains("inside project"));
        Ok(())
    }

    #[test]
    fn allows_global_state_dir_outside_project_tree() -> Result<()> {
        let tmp = tempdir()?;
        let project_root = tmp.path().join("repo");
        let ship_dir = project_root.join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        let external_global = tmp.path().join("global-ship-dir");
        std::fs::create_dir_all(&external_global)?;

        ensure_global_dir_outside_project(&ship_dir, &external_global)?;
        Ok(())
    }
}
