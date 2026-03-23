//! Unified schema — single DDL, no migration versioning. Schema = code.
//!
//! Two layers, one database:
//!
//! ## Platform (`platform.rs`) — portable runtime
//! - `kv_state` -- generic namespaced key-value store
//! - `workspace` -- branch-keyed unit of work
//! - `workspace_session` -- time-bounded work interval
//! - `workspace_session_record` -- immutable end-of-session snapshot
//! - `branch_config` -- compiled preset state per branch
//! - `branch_context` -- branch-to-entity links
//! - `event_log` -- append-only audit trail
//! - `agent_runtime_settings` -- singleton global agent config
//! - `agent_artifact_registry` -- content-addressed artifact store
//! - `agent_config` -- named agent profiles
//! - `managed_mcp_state` -- Ship-managed MCP server tracking
//!
//! ## Workflow (`workflow.rs`) — opinionated planning layer
//! - `target` -- named goals (milestones and surfaces)
//! - `capability` -- concrete requirements under targets
//! - `job` -- queued unit of work
//! - `job_file` -- exclusive file claims per job
//! - `file_claim` -- batch-atomic file claims with workspace tracking
//! - `note` -- human-facing scratchpad
//! - `adr` -- architecture decision records

mod platform;
mod workflow;

/// All DDL fragments in execution order. `ensure_db` iterates these,
/// splitting each on `;` and running every statement.
pub const SCHEMA_PARTS: &[&str] = &[
    // Pragmas
    "PRAGMA journal_mode = WAL;\nPRAGMA foreign_keys = ON;",
    // Platform
    platform::KV_STATE,
    platform::WORKSPACE,
    platform::WORKSPACE_SESSION,
    platform::WORKSPACE_SESSION_RECORD,
    platform::BRANCH_CONFIG,
    platform::BRANCH_CONTEXT,
    platform::EVENT_LOG,
    platform::AGENT_RUNTIME_SETTINGS,
    platform::AGENT_ARTIFACT_REGISTRY,
    platform::AGENT_CONFIG,
    platform::MANAGED_MCP_STATE,
    // Workflow
    workflow::TARGET,
    workflow::CAPABILITY,
    workflow::JOB,
    workflow::JOB_FILE,
    workflow::FILE_CLAIM,
    workflow::NOTE,
    workflow::ADR,
];
