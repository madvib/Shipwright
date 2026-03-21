//! Schema DDL for agent runtime tables.

/// Agent runtime settings: singleton row (id=1) holding global agent config.
/// Stores active agent identity, provider list, hooks, AI/git config, and
/// namespace scoping. Written by `ship use` and read on every compilation.
pub const AGENT_RUNTIME_SETTINGS: &str = r#"
-- agent_runtime_settings: singleton (id=1) global agent configuration.
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
"#;

/// Agent artifact registry: content-addressed registry of compiled artifacts
/// (skills, rules, MCP server configs). Keyed by (kind, external_id).
/// UUID provides a stable reference across re-compilations.
pub const AGENT_ARTIFACT_REGISTRY: &str = r#"
-- agent_artifact_registry: content-addressed compiled artifact store.
-- UNIQUE(kind, external_id) for dedup; uuid is the stable reference.
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

/// Agent config: named agent configuration profiles.
/// Each profile specifies which tools, MCP servers, skills, rules, and
/// permissions are active. Referenced by workspace.active_agent.
///
/// Historical note: this table was renamed from `agent_mode` to `agent_config`.
/// The migration in `db::mod.rs::ensure_db` handles the rename idempotently.
pub const AGENT_CONFIG: &str = r#"
-- agent_config: named agent profiles (tools, MCP, skills, rules, permissions).
-- Renamed from agent_mode; migration in ensure_db handles the rename.
CREATE TABLE IF NOT EXISTS agent_config (
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
"#;

/// Managed MCP state: tracks which MCP server processes Ship manages
/// per provider, and the last agent config that was applied.
/// Used to detect drift and perform incremental server management.
pub const MANAGED_MCP_STATE: &str = r#"
-- managed_mcp_state: Ship-managed MCP server tracking per provider.
CREATE TABLE IF NOT EXISTS managed_mcp_state (
  provider         TEXT PRIMARY KEY,
  server_ids_json  TEXT NOT NULL DEFAULT '[]',
  last_mode        TEXT,
  updated_at       TEXT NOT NULL
);
"#;
