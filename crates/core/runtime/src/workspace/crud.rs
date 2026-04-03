use crate::db::branch_context::clear_branch_link;
use crate::db::workspace_state::{
    delete_workspace_db, get_workspace_by_id_db, get_workspace_db, list_workspaces_db,
};
use super::event_upserts::upsert_workspace_on_deleted;
use anyhow::{Result, anyhow};
use std::path::Path;

use super::helpers::*;
use super::types::*;
use super::types_session::*;

// ---- Get / list / delete ---------------------------------------------------

/// Look up a workspace by its id (or branch as fallback).
pub fn get_workspace_by_id(_ship_dir: &Path, id: &str) -> Result<Option<Workspace>> {
    let row = get_workspace_by_id_db(id)?;
    let Some((branch, (ws_id, status, is_worktree, worktree_path, active_agent, last_activated_at))) = row else {
        return Ok(None);
    };

    let status = parse_workspace_status_required(&status)
        .map_err(|err| anyhow!("Workspace '{}' has invalid status value: {}", id, err))?;

    Ok(Some(Workspace {
        id: ws_id,
        branch,
        status,
        is_worktree,
        worktree_path,
        active_agent,
        last_activated_at: parse_datetime_opt(last_activated_at),
    }))
}

pub fn get_workspace(_ship_dir: &Path, branch: &str) -> Result<Option<Workspace>> {
    let row = get_workspace_db(branch)?;
    let Some((id, status, is_worktree, worktree_path, active_agent, last_activated_at)) = row else {
        return Ok(None);
    };

    let status = parse_workspace_status_required(&status)
        .map_err(|err| anyhow!("Workspace '{}' has invalid status value: {}", branch, err))?;

    Ok(Some(Workspace {
        id,
        branch: branch.to_string(),
        status,
        is_worktree,
        worktree_path,
        active_agent,
        last_activated_at: parse_datetime_opt(last_activated_at),
    }))
}

pub fn list_workspaces(_ship_dir: &Path) -> Result<Vec<Workspace>> {
    let rows = list_workspaces_db()?;
    let mut workspaces = Vec::with_capacity(rows.len());
    for (branch, id, status, is_worktree, worktree_path, active_agent, last_activated_at) in rows {
        let parsed_status = parse_workspace_status_required(&status)
            .map_err(|err| anyhow!("Workspace '{}' has invalid status value: {}", branch, err))?;

        workspaces.push(Workspace {
            id,
            branch,
            status: parsed_status,
            is_worktree,
            worktree_path,
            active_agent,
            last_activated_at: parse_datetime_opt(last_activated_at),
        });
    }
    Ok(workspaces)
}

pub fn delete_workspace(ship_dir: &Path, branch: &str) -> Result<()> {
    let branch = ensure_branch_key(branch)?;
    let _ = delete_workspace_db(branch)?;
    clear_branch_link(branch)?;
    upsert_workspace_on_deleted(ship_dir, branch)?;
    Ok(())
}

// ---- Provider matrix / repair (public) -------------------------------------

pub fn get_workspace_provider_matrix(
    ship_dir: &Path,
    branch: &str,
    agent_id: Option<&str>,
) -> Result<WorkspaceProviderMatrix> {
    let branch = ensure_branch_key(branch)?;
    let workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow!("Workspace not found for branch '{}'", branch))?;
    super::compile::build_workspace_provider_matrix(ship_dir, &workspace, agent_id)
}

pub fn repair_workspace(
    ship_dir: &Path,
    branch: &str,
    dry_run: bool,
) -> Result<WorkspaceRepairReport> {
    let branch = ensure_branch_key(branch)?;
    let workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow!("Workspace not found for branch '{}'", branch))?;
    let mut matrix =
        get_workspace_provider_matrix(ship_dir, branch, workspace.active_agent.as_deref())?;

    let mut actions = Vec::new();
    let mut missing_provider_configs =
        super::compile::missing_provider_configs_for_workspace(
            &super::compile::resolve_workspace_context_root(ship_dir, &workspace),
            &matrix.allowed_providers,
        );
    if !missing_provider_configs.is_empty() {
        actions.push(format!(
            "missing provider configs: {}",
            missing_provider_configs.join(",")
        ));
    }

    let mut reapplied_compile = false;
    let needs_recompile = !missing_provider_configs.is_empty();

    if !dry_run && needs_recompile && matrix.resolution_error.is_none() {
        if workspace.status == WorkspaceStatus::Active {
            let active = workspace.active_agent.clone();
            let _workspace =
                super::lifecycle::set_workspace_active_agent(ship_dir, branch, active.as_deref())?;
            matrix =
                get_workspace_provider_matrix(ship_dir, branch, _workspace.active_agent.as_deref())?;
            let context_root = super::compile::resolve_workspace_context_root(ship_dir, &_workspace);
            missing_provider_configs =
                super::compile::missing_provider_configs_for_workspace(&context_root, &matrix.allowed_providers);
            reapplied_compile = true;
            actions.push("recompiled active workspace context".to_string());
        } else {
            actions.push(
                "workspace is not active; activate workspace to apply compile repair".to_string(),
            );
        }
    }

    Ok(WorkspaceRepairReport {
        workspace_branch: workspace.branch.clone(),
        dry_run,
        agent_id: workspace.active_agent.clone(),
        status: workspace.status,
        providers_expected: matrix.allowed_providers,
        needs_recompile: !missing_provider_configs.is_empty(),
        missing_provider_configs,
        had_compile_error: false,
        reapplied_compile,
        resolution_error: matrix.resolution_error,
        actions,
    })
}
