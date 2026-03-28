use crate::db::branch_context::clear_branch_link;
use crate::db::types::WorkspaceUpsert;
use crate::db::workspace_state::{
    delete_workspace_db, get_workspace_db, list_workspaces_db, upsert_workspace_db,
};
use super::event_upserts::upsert_workspace_on_deleted;
use anyhow::{Result, anyhow};
use std::path::Path;

use super::compile::{
    build_workspace_provider_matrix, missing_provider_configs_for_workspace,
    resolve_workspace_context_root,
};
use super::helpers::*;
use super::lifecycle::set_workspace_active_agent;
use super::types::*;
use super::types_session::*;

// ---- Get / list / delete ---------------------------------------------------

pub fn get_workspace(_ship_dir: &Path, branch: &str) -> Result<Option<Workspace>> {
    let row = get_workspace_db(branch)?;
    let Some((
        id,
        workspace_type,
        status,
        active_agent,
        providers,
        mcp_servers,
        skills,
        is_worktree,
        worktree_path,
        last_activated_at,
        context_hash,
        config_generation,
        compiled_at,
        compile_error,
    )) = row
    else {
        return Ok(None);
    };

    let workspace_type = parse_workspace_type_required(&workspace_type)
        .map_err(|err| anyhow!("Workspace '{}' has invalid type value: {}", branch, err))?;
    let status = parse_workspace_status_required(&status)
        .map_err(|err| anyhow!("Workspace '{}' has invalid status value: {}", branch, err))?;

    Ok(Some(Workspace {
        id,
        branch: branch.to_string(),
        workspace_type,
        status,
        active_agent,
        providers,
        mcp_servers,
        skills,
        last_activated_at: parse_datetime_opt(last_activated_at),
        is_worktree,
        worktree_path,
        context_hash,
        config_generation,
        compiled_at: parse_datetime_opt(compiled_at),
        compile_error,
    }))
}

pub fn list_workspaces(_ship_dir: &Path) -> Result<Vec<Workspace>> {
    let rows = list_workspaces_db()?;
    let mut workspaces = Vec::with_capacity(rows.len());
    for (
        branch,
        id,
        workspace_type,
        status,
        active_agent,
        providers,
        mcp_servers,
        skills,
        is_worktree,
        worktree_path,
        last_activated_at,
        context_hash,
        config_generation,
        compiled_at,
        compile_error,
    ) in rows
    {
        let parsed_type = parse_workspace_type_required(&workspace_type)
            .map_err(|err| anyhow!("Workspace '{}' has invalid type value: {}", branch, err))?;
        let parsed_status = parse_workspace_status_required(&status)
            .map_err(|err| anyhow!("Workspace '{}' has invalid status value: {}", branch, err))?;

        workspaces.push(Workspace {
            id,
            branch,
            workspace_type: parsed_type,
            status: parsed_status,
            active_agent,
            providers,
            mcp_servers,
            skills,
            last_activated_at: parse_datetime_opt(last_activated_at),
            is_worktree,
            worktree_path,
            context_hash,
            config_generation,
            compiled_at: parse_datetime_opt(compiled_at),
            compile_error,
        });
    }
    Ok(workspaces)
}

pub fn delete_workspace(ship_dir: &Path, branch: &str) -> Result<()> {
    let branch = ensure_branch_key(branch)?;
    upsert_workspace_on_deleted(ship_dir, branch)?;
    clear_branch_link(branch)?;
    let _ = delete_workspace_db(branch)?;
    Ok(())
}

pub fn upsert_workspace(_ship_dir: &Path, workspace: &Workspace) -> Result<()> {
    let workspace_id = if workspace.id.trim().is_empty() {
        workspace_id_from_branch(&workspace.branch)
    } else {
        workspace.id.clone()
    };

    let workspace_type = workspace.workspace_type.to_string();
    let status = workspace.status.to_string();
    let last_activated_at = workspace
        .last_activated_at
        .as_ref()
        .map(|ts| ts.to_rfc3339());
    let compiled_at = workspace.compiled_at.as_ref().map(|ts| ts.to_rfc3339());

    upsert_workspace_db(WorkspaceUpsert {
        branch: &workspace.branch,
        workspace_id: &workspace_id,
        workspace_type: &workspace_type,
        status: &status,
        active_agent: workspace.active_agent.as_deref(),
        providers: &workspace.providers,
        mcp_servers: &workspace.mcp_servers,
        skills: &workspace.skills,
        is_worktree: workspace.is_worktree,
        worktree_path: workspace.worktree_path.as_deref(),
        last_activated_at: last_activated_at.as_deref(),
        context_hash: workspace.context_hash.as_deref(),
        config_generation: workspace.config_generation,
        compiled_at: compiled_at.as_deref(),
        compile_error: workspace.compile_error.as_deref(),
    })
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
    build_workspace_provider_matrix(ship_dir, &workspace, agent_id)
}

pub fn repair_workspace(
    ship_dir: &Path,
    branch: &str,
    dry_run: bool,
) -> Result<WorkspaceRepairReport> {
    let branch = ensure_branch_key(branch)?;
    let mut workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow!("Workspace not found for branch '{}'", branch))?;
    let mut matrix =
        get_workspace_provider_matrix(ship_dir, branch, workspace.active_agent.as_deref())?;
    let context_root = resolve_workspace_context_root(ship_dir, &workspace);

    let mut actions = Vec::new();
    let had_compile_error = workspace.compile_error.is_some();
    if had_compile_error {
        actions.push("workspace has compile_error set".to_string());
    }
    if workspace.compiled_at.is_none() {
        actions.push("workspace compiled_at is missing".to_string());
    }

    let mut missing_provider_configs =
        missing_provider_configs_for_workspace(&context_root, &matrix.allowed_providers);
    if !missing_provider_configs.is_empty() {
        actions.push(format!(
            "missing provider configs: {}",
            missing_provider_configs.join(",")
        ));
    }

    let mut reapplied_compile = false;
    let mut needs_recompile = had_compile_error
        || workspace.compiled_at.is_none()
        || !missing_provider_configs.is_empty();

    if !dry_run && needs_recompile && matrix.resolution_error.is_none() {
        if workspace.status == WorkspaceStatus::Active {
            let active = workspace.active_agent.clone();
            workspace = set_workspace_active_agent(ship_dir, branch, active.as_deref())?;
            matrix =
                get_workspace_provider_matrix(ship_dir, branch, workspace.active_agent.as_deref())?;
            missing_provider_configs =
                missing_provider_configs_for_workspace(&context_root, &matrix.allowed_providers);
            reapplied_compile = true;
            needs_recompile = workspace.compile_error.is_some()
                || workspace.compiled_at.is_none()
                || !missing_provider_configs.is_empty();
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
        missing_provider_configs,
        had_compile_error,
        needs_recompile,
        reapplied_compile,
        resolution_error: matrix.resolution_error,
        actions,
    })
}
