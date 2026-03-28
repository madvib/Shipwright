//! Lifecycle-event-aware workspace upsert functions.
//!
//! Each function writes the workspace row AND inserts a typed event in one
//! atomic SQLite transaction. Callers in `lifecycle.rs` and `compile.rs`
//! use these instead of plain `upsert_workspace` for the four transitions
//! specified in ADR GHihs2tn.

use crate::db::types::WorkspaceUpsert;
use crate::db::workspace_events::{
    insert_workspace_agent_changed_event, insert_workspace_archived_event,
    insert_workspace_deleted_event, upsert_workspace_activated, upsert_workspace_agent_changed,
    upsert_workspace_archived, upsert_workspace_compile_failed, upsert_workspace_compiled,
    upsert_workspace_created, upsert_workspace_status_changed,
};
use crate::events::types::{
    WorkspaceActivated, WorkspaceAgentChanged, WorkspaceArchived, WorkspaceCompileFailed,
    WorkspaceCompiled, WorkspaceCreated, WorkspaceStatusChanged,
};
use anyhow::Result;
use std::path::Path;

use super::helpers::workspace_id_from_branch;
use super::types::Workspace;

// ── private helpers ───────────────────────────────────────────────────────────

fn workspace_string_fields(
    workspace: &Workspace,
) -> (String, String, String, Option<String>, Option<String>) {
    let workspace_id = if workspace.id.trim().is_empty() {
        workspace_id_from_branch(&workspace.branch)
    } else {
        workspace.id.clone()
    };
    let workspace_type = workspace.workspace_type.to_string();
    let status = workspace.status.to_string();
    let last_activated_at = workspace.last_activated_at.as_ref().map(|ts| ts.to_rfc3339());
    let compiled_at = workspace.compiled_at.as_ref().map(|ts| ts.to_rfc3339());
    (workspace_id, workspace_type, status, last_activated_at, compiled_at)
}

fn build_upsert_record<'a>(
    workspace: &'a Workspace,
    workspace_id: &'a str,
    workspace_type: &'a str,
    status: &'a str,
    last_activated_at: Option<&'a str>,
    compiled_at: Option<&'a str>,
) -> WorkspaceUpsert<'a> {
    WorkspaceUpsert {
        branch: &workspace.branch,
        workspace_id,
        workspace_type,
        status,
        active_agent: workspace.active_agent.as_deref(),
        providers: &workspace.providers,
        mcp_servers: &workspace.mcp_servers,
        skills: &workspace.skills,
        is_worktree: workspace.is_worktree,
        worktree_path: workspace.worktree_path.as_deref(),
        last_activated_at,
        context_hash: workspace.context_hash.as_deref(),
        config_generation: workspace.config_generation,
        compiled_at,
        compile_error: workspace.compile_error.as_deref(),
    }
}

// ── public event-emitting upserts ─────────────────────────────────────────────

/// Upsert workspace state and emit `workspace.activated` in one transaction.
pub fn upsert_workspace_on_activate(_ship_dir: &Path, workspace: &Workspace) -> Result<()> {
    let (workspace_id, workspace_type, status, last_activated_at, compiled_at) =
        workspace_string_fields(workspace);
    let payload = WorkspaceActivated {
        agent_id: workspace.active_agent.clone(),
        providers: workspace.providers.clone(),
    };
    upsert_workspace_activated(
        build_upsert_record(
            workspace,
            &workspace_id,
            &workspace_type,
            &status,
            last_activated_at.as_deref(),
            compiled_at.as_deref(),
        ),
        &payload,
    )
}

/// Upsert workspace state and emit `workspace.compiled` in one transaction.
pub fn upsert_workspace_on_compiled(
    _ship_dir: &Path,
    workspace: &Workspace,
    duration_ms: u64,
) -> Result<()> {
    let (workspace_id, workspace_type, status, last_activated_at, compiled_at) =
        workspace_string_fields(workspace);
    let payload = WorkspaceCompiled {
        config_generation: workspace.config_generation as u32,
        duration_ms,
    };
    upsert_workspace_compiled(
        build_upsert_record(
            workspace,
            &workspace_id,
            &workspace_type,
            &status,
            last_activated_at.as_deref(),
            compiled_at.as_deref(),
        ),
        &payload,
    )
}

/// Upsert workspace state and emit `workspace.compile_failed` in one transaction.
pub fn upsert_workspace_on_compile_failed(
    _ship_dir: &Path,
    workspace: &Workspace,
    error: &str,
) -> Result<()> {
    let (workspace_id, workspace_type, status, last_activated_at, compiled_at) =
        workspace_string_fields(workspace);
    let payload = WorkspaceCompileFailed {
        error: error.to_string(),
    };
    upsert_workspace_compile_failed(
        build_upsert_record(
            workspace,
            &workspace_id,
            &workspace_type,
            &status,
            last_activated_at.as_deref(),
            compiled_at.as_deref(),
        ),
        &payload,
    )
}

/// Upsert workspace state and emit `workspace.archived` in one transaction.
pub fn upsert_workspace_on_archived(_ship_dir: &Path, workspace: &Workspace) -> Result<()> {
    let (workspace_id, workspace_type, status, last_activated_at, compiled_at) =
        workspace_string_fields(workspace);
    let payload = WorkspaceArchived {};
    upsert_workspace_archived(
        build_upsert_record(
            workspace,
            &workspace_id,
            &workspace_type,
            &status,
            last_activated_at.as_deref(),
            compiled_at.as_deref(),
        ),
        &payload,
    )
}

/// Upsert workspace state and emit `workspace.created` in one transaction.
pub fn upsert_workspace_on_created(_ship_dir: &Path, workspace: &Workspace) -> Result<()> {
    let (workspace_id, workspace_type, status, last_activated_at, compiled_at) =
        workspace_string_fields(workspace);
    let payload = WorkspaceCreated {
        workspace_type: workspace_type.clone(),
        status: status.clone(),
    };
    upsert_workspace_created(
        build_upsert_record(
            workspace,
            &workspace_id,
            &workspace_type,
            &status,
            last_activated_at.as_deref(),
            compiled_at.as_deref(),
        ),
        &payload,
    )
}

/// Emit `workspace.deleted` event and clear the workspace row.
///
/// Emits the event first, then the caller is responsible for deleting the row.
/// The event uses the branch as `entity_id` so it is queryable after deletion.
pub fn upsert_workspace_on_deleted(_ship_dir: &Path, branch: &str) -> Result<()> {
    insert_workspace_deleted_event(branch)
}

/// Upsert workspace state and emit `workspace.status_changed` in one transaction.
pub fn upsert_workspace_on_status_changed(
    _ship_dir: &Path,
    workspace: &Workspace,
    old_status: &str,
    new_status: &str,
) -> Result<()> {
    let (workspace_id, workspace_type, status, last_activated_at, compiled_at) =
        workspace_string_fields(workspace);
    let payload = WorkspaceStatusChanged {
        old_status: old_status.to_string(),
        new_status: new_status.to_string(),
    };
    upsert_workspace_status_changed(
        build_upsert_record(
            workspace,
            &workspace_id,
            &workspace_type,
            &status,
            last_activated_at.as_deref(),
            compiled_at.as_deref(),
        ),
        &payload,
    )
}

/// Emit `workspace.archived` event for a bulk-demoted workspace (no upsert).
pub fn emit_workspace_archived_event(_ship_dir: &Path, branch: &str) -> Result<()> {
    insert_workspace_archived_event(branch)
}

/// Emit `workspace.agent_changed` without upserting the workspace row.
///
/// Used on the active-workspace path where compile handles its own upsert.
pub fn emit_workspace_agent_changed_event(
    _ship_dir: &Path,
    branch: &str,
    agent_id: Option<&str>,
) -> Result<()> {
    insert_workspace_agent_changed_event(branch, agent_id)
}

/// Upsert workspace state and emit `workspace.agent_changed` in one transaction.
pub fn upsert_workspace_on_agent_changed(_ship_dir: &Path, workspace: &Workspace) -> Result<()> {
    let (workspace_id, workspace_type, status, last_activated_at, compiled_at) =
        workspace_string_fields(workspace);
    let payload = WorkspaceAgentChanged {
        agent_id: workspace.active_agent.clone(),
    };
    upsert_workspace_agent_changed(
        build_upsert_record(
            workspace,
            &workspace_id,
            &workspace_type,
            &status,
            last_activated_at.as_deref(),
            compiled_at.as_deref(),
        ),
        &payload,
    )
}
