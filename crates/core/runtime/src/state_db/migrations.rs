use super::schema::*;
use super::schema_ext::*;

pub(super) const PROJECT_MIGRATIONS: &[(&str, &str)] = &[
    ("0001_project_schema", PROJECT_SCHEMA_V1),
    ("0002_operational_state", PROJECT_SCHEMA_OPERATIONAL),
    ("0003_workspace", PROJECT_SCHEMA_WORKSPACE),
    ("0004_adrs", PROJECT_SCHEMA_ADRS),
    ("0005_notes", PROJECT_SCHEMA_NOTES),
    ("0006_features_releases", PROJECT_SCHEMA_FEATURES_RELEASES),
    ("0007_workspace_lifecycle", PROJECT_SCHEMA_WORKSPACE_V2),
    ("0008_specs", PROJECT_SCHEMA_SPECS),
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
    (
        "0016_feature_body_release_status",
        "ALTER TABLE feature ADD COLUMN body TEXT NOT NULL DEFAULT '';
         UPDATE release SET status = 'upcoming' WHERE status = 'planned';
         UPDATE release SET status = 'deprecated' WHERE status IN ('shipped', 'archived');",
    ),
    (
        "0017_workspace_runtime_contract",
        "UPDATE workspace
         SET workspace_type = lower(trim(workspace_type))
         WHERE workspace_type IS NOT NULL
           AND trim(workspace_type) != '';
         UPDATE workspace
         SET workspace_type = 'feature'
         WHERE workspace_type IS NULL
            OR trim(workspace_type) = '';
         UPDATE workspace
         SET status = 'active'
         WHERE lower(trim(status)) = 'active';
         UPDATE workspace
         SET status = 'archived'
         WHERE lower(trim(status)) = 'archived';
         UPDATE workspace
         SET status = 'archived'
         WHERE status IS NOT NULL
           AND trim(status) != ''
           AND lower(trim(status)) NOT IN ('active', 'archived');
         UPDATE workspace
         SET status = 'active'
         WHERE status IS NULL OR trim(status) = '';",
    ),
    (
        "0018_runtime_primitives_v3",
        PROJECT_SCHEMA_RUNTIME_PRIMITIVES_V3,
    ),
    (
        "0019_workspace_target_and_session_records",
        "CREATE TABLE IF NOT EXISTS workspace_session_record (
           id                 TEXT PRIMARY KEY,
           session_id         TEXT NOT NULL UNIQUE REFERENCES workspace_session(id) ON DELETE CASCADE,
           workspace_id       TEXT NOT NULL,
           workspace_branch   TEXT NOT NULL,
           summary            TEXT,
           updated_feature_ids_json TEXT NOT NULL DEFAULT '[]',
           created_at         TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS workspace_session_record_workspace_idx
           ON workspace_session_record(workspace_id, created_at DESC);",
    ),
    (
        "0020_capability_and_target_links",
        "CREATE TABLE IF NOT EXISTS capability_map (
           id            TEXT PRIMARY KEY,
           vision_ref    TEXT,
           created_at    TEXT NOT NULL,
           updated_at    TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS capability (
           id                    TEXT PRIMARY KEY,
           map_id                TEXT NOT NULL REFERENCES capability_map(id) ON DELETE CASCADE,
           title                 TEXT NOT NULL,
           description           TEXT NOT NULL DEFAULT '',
           parent_capability_id  TEXT REFERENCES capability(id) ON DELETE SET NULL,
           status                TEXT NOT NULL DEFAULT 'active',
           ord                   INTEGER NOT NULL DEFAULT 0,
           created_at            TEXT NOT NULL,
           updated_at            TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS capability_map_idx
           ON capability(map_id, ord ASC, updated_at DESC);
         CREATE TABLE IF NOT EXISTS feature_capability (
           feature_id      TEXT NOT NULL REFERENCES feature(id) ON DELETE CASCADE,
           capability_id   TEXT NOT NULL REFERENCES capability(id) ON DELETE CASCADE,
           is_primary      INTEGER NOT NULL DEFAULT 1,
           created_at      TEXT NOT NULL,
           PRIMARY KEY(feature_id, capability_id)
         );
         CREATE UNIQUE INDEX IF NOT EXISTS feature_capability_primary_idx
           ON feature_capability(feature_id)
           WHERE is_primary = 1;
         CREATE TABLE IF NOT EXISTS target_feature (
           target_id       TEXT NOT NULL REFERENCES release(id) ON DELETE CASCADE,
           feature_id      TEXT NOT NULL REFERENCES feature(id) ON DELETE CASCADE,
           ord             INTEGER NOT NULL DEFAULT 0,
           created_at      TEXT NOT NULL,
           PRIMARY KEY(target_id, feature_id)
         );
         CREATE INDEX IF NOT EXISTS target_feature_feature_idx
           ON target_feature(feature_id, target_id);",
    ),
    (
        "0021_workspace_agent_overrides",
        "ALTER TABLE workspace ADD COLUMN mcp_servers_json TEXT NOT NULL DEFAULT '[]';
         ALTER TABLE workspace ADD COLUMN skills_json TEXT NOT NULL DEFAULT '[]';",
    ),
];

pub(super) const GLOBAL_MIGRATIONS: &[(&str, &str)] = &[
    ("0001_global_schema", GLOBAL_SCHEMA_V1),
    ("0002_notes", PROJECT_SCHEMA_NOTES),
    ("0003_migration_meta", SCHEMA_MIGRATION_META),
];
