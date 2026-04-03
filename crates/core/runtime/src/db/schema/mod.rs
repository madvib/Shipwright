//! Schema reference — platform and workflow table definitions.
//!
//! Two layers, one database:
//!
//! ## Platform (`platform.rs`) — portable runtime
//! - `kv_state` -- generic namespaced key-value store
//! - `workspace` -- branch-keyed unit of work
//! - `workspace_session` -- time-bounded work interval
//! - `workspace_session_record` -- immutable end-of-session snapshot
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
//! - `jobs` -- agent work queue
//! - `file_claim` -- batch-atomic file claims with workspace tracking
//! - `note` -- human-facing scratchpad
//! - `adr` -- architecture decision records
//!
//! The canonical DDL lives in `migrations/0001_initial.sql`.
//! These modules retain the constants as code-level documentation.

#[allow(dead_code)]
pub mod platform;
#[allow(dead_code)]
pub mod workflow;
