//! Unified platform schema -- single DDL, no migration versioning.
//!
//! Every table uses `CREATE TABLE IF NOT EXISTS` so the schema is
//! idempotent: run it on every connection and new tables appear
//! automatically.  Schema = code.
//!
//! # Table inventory (19 tables)
//!
//! ## State & workspace (`state.rs`)
//! - `kv_state` -- generic namespaced key-value store
//! - `workspace` -- branch-keyed unit of work
//! - `workspace_session` -- time-bounded work interval
//! - `workspace_session_record` -- immutable end-of-session snapshot
//! - `branch_config` -- compiled preset state per branch
//! - `branch_context` -- branch-to-entity links
//!
//! ## Work items (`work.rs`)
//! - `job` -- queued unit of work
//! - `job_log` -- **DEPRECATED** in favor of `event_log`
//! - `job_file` -- exclusive file claims per job
//! - `file_claim` -- batch-atomic file claims with workspace tracking
//! - `note` -- human-facing scratchpad
//! - `adr` -- architecture decision records
//! - `target` -- named goals (milestones and surfaces)
//! - `capability` -- concrete requirements under targets
//!
//! ## Events (`events.rs`)
//! - `event_log` -- append-only audit trail
//!
//! ## Agent runtime (`agents.rs`)
//! - `agent_runtime_settings` -- singleton global agent config
//! - `agent_artifact_registry` -- content-addressed artifact store
//! - `agent_config` -- named agent profiles
//! - `managed_mcp_state` -- Ship-managed MCP server tracking

mod agents;
mod events;
mod state;
mod work;

/// All DDL fragments in execution order. `ensure_db` iterates these,
/// splitting each on `;` and running every statement.
pub const SCHEMA_PARTS: &[&str] = &[
    "PRAGMA journal_mode = WAL;\nPRAGMA foreign_keys = ON;",
    state::KV_STATE,
    state::WORKSPACE,
    state::WORKSPACE_SESSION,
    state::WORKSPACE_SESSION_RECORD,
    state::BRANCH_CONFIG,
    state::BRANCH_CONTEXT,
    work::JOB,
    work::JOB_LOG_DEPRECATED,
    work::JOB_FILE,
    work::FILE_CLAIM,
    work::NOTE,
    work::ADR,
    work::TARGET,
    work::CAPABILITY,
    events::EVENT_LOG,
    agents::AGENT_RUNTIME_SETTINGS,
    agents::AGENT_ARTIFACT_REGISTRY,
    agents::AGENT_CONFIG,
    agents::MANAGED_MCP_STATE,
];
