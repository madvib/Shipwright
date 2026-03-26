use crate::events::{EventAction, EventEntity, append_event};
use anyhow::Result;
use chrono::Utc;
use std::path::Path;

use super::compile::compile_workspace_context;
use super::crud::{get_workspace, list_workspaces, upsert_workspace};
use super::helpers::*;
use super::types::*;
use super::types_session::*;

/// Create or update a workspace record without requiring a git checkout.
/// This is the runtime-native entrypoint for workspace lifecycle management.
pub fn create_workspace(ship_dir: &Path, request: CreateWorkspaceRequest) -> Result<Workspace> {
    let branch = ensure_branch_key(&request.branch)?.to_string();
    let now = Utc::now();

    let existing = get_workspace(ship_dir, &branch)?;
    let mut workspace = existing
        .clone()
        .unwrap_or_else(|| new_workspace(&branch, now));

    if let Some(active_agent) = request.active_agent {
        workspace.active_agent = Some(validate_agent_exists(ship_dir, &active_agent)?);
    }
    if let Some(providers) = request.providers {
        workspace.providers = providers;
    }
    if let Some(mcp_servers) = request.mcp_servers {
        workspace.mcp_servers = mcp_servers;
    }
    if let Some(skills) = request.skills {
        workspace.skills = skills;
    }
    if let Some(is_worktree) = request.is_worktree {
        workspace.is_worktree = is_worktree;
    }
    if let Some(worktree_path) = request.worktree_path {
        let path = worktree_path.trim();
        if path.is_empty() {
            workspace.worktree_path = None;
        } else if workspace.is_worktree {
            workspace.worktree_path = Some(path.to_string());
        } else {
            return Err(anyhow::anyhow!(
                "Worktree path can only be set when is_worktree=true"
            ));
        }
    }
    if !workspace.is_worktree {
        workspace.worktree_path = None;
    } else if workspace.worktree_path.is_none() {
        workspace.worktree_path = default_worktree_path(ship_dir, &branch);
    }
    if workspace.is_worktree && workspace.worktree_path.is_none() {
        return Err(anyhow::anyhow!(
            "Worktree workspace requires a worktree path"
        ));
    }
    if let Some(context_hash) = request.context_hash {
        workspace.context_hash = Some(context_hash);
    }

    hydrate_from_branch_links(ship_dir, &branch, &mut workspace)?;
    workspace.workspace_type = request.workspace_type.unwrap_or_else(|| {
        existing
            .as_ref()
            .map(|entry| entry.workspace_type)
            .unwrap_or_else(|| infer_workspace_type(&branch))
    });

    hydrate_from_feature_links(ship_dir, &mut workspace)?;
    let base_status = existing
        .as_ref()
        .map(|entry| entry.status)
        .unwrap_or(WorkspaceStatus::Active);
    let next_status = request.status.unwrap_or(base_status);

    validate_workspace_transition(workspace.workspace_type, base_status, next_status)?;

    workspace.id = workspace_id_from_branch(&branch);
    workspace.branch = branch;
    workspace.status = next_status;
    if next_status == WorkspaceStatus::Active {
        workspace.last_activated_at = Some(now);
    }

    persist_branch_link_from_workspace(ship_dir, &workspace)?;
    upsert_workspace(ship_dir, &workspace)?;
    let action = if existing.is_some() {
        EventAction::Update
    } else {
        EventAction::Create
    };
    let mut details = vec![
        format!("type={}", workspace.workspace_type),
        format!("status={}", workspace.status),
    ];
    if let Some(agent_id) = workspace.active_agent.as_deref() {
        details.push(format!("agent={agent_id}"));
    }
    if !workspace.mcp_servers.is_empty() {
        details.push(format!("mcp={}", workspace.mcp_servers.len()));
    }
    if !workspace.skills.is_empty() {
        details.push(format!("skills={}", workspace.skills.len()));
    }
    if workspace.is_worktree {
        details.push("worktree=true".to_string());
        if let Some(path) = workspace.worktree_path.as_deref() {
            details.push(format!("worktree_path={path}"));
        }
    }
    append_event(
        ship_dir,
        "ship",
        EventEntity::Workspace,
        action,
        workspace.branch.clone(),
        Some(details.join(" ")),
    )?;
    Ok(workspace)
}

pub fn transition_workspace_status(
    ship_dir: &Path,
    branch: &str,
    next_status: WorkspaceStatus,
) -> Result<Workspace> {
    let mut workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow::anyhow!("Workspace not found for branch '{}'", branch))?;

    validate_workspace_transition(workspace.workspace_type, workspace.status, next_status)?;

    let now = Utc::now();
    if next_status == WorkspaceStatus::Active {
        workspace.last_activated_at = Some(now);
    }

    workspace.status = next_status;
    upsert_workspace(ship_dir, &workspace)?;
    append_event(
        ship_dir,
        "ship",
        EventEntity::Workspace,
        EventAction::Set,
        workspace.branch.clone(),
        Some(format!(
            "status={} type={}",
            workspace.status, workspace.workspace_type
        )),
    )?;
    Ok(workspace)
}

/// Activate a workspace by key (branch/id) as a runtime operation.
/// Git hooks may call this after branch checkout, but it can be used standalone.
pub fn activate_workspace(ship_dir: &Path, branch: &str) -> Result<Workspace> {
    let branch = ensure_branch_key(branch)?;
    let now = Utc::now();

    let mut workspace =
        get_workspace(ship_dir, branch)?.unwrap_or_else(|| new_workspace(branch, now));

    hydrate_from_branch_links(ship_dir, branch, &mut workspace)?;

    workspace.id = workspace_id_from_branch(branch);
    workspace.branch = branch.to_string();
    if workspace.workspace_type == ShipWorkspaceKind::Feature {
        workspace.workspace_type = infer_workspace_type(branch);
    }
    validate_workspace_transition(
        workspace.workspace_type,
        workspace.status,
        WorkspaceStatus::Active,
    )?;

    workspace.status = WorkspaceStatus::Active;
    workspace.last_activated_at = Some(now);

    persist_branch_link_from_workspace(ship_dir, &workspace)?;
    let active_agent = workspace.active_agent.clone();
    compile_workspace_context(ship_dir, &mut workspace, active_agent.as_deref())?;
    append_event(
        ship_dir,
        "ship",
        EventEntity::Workspace,
        EventAction::Start,
        workspace.branch.clone(),
        Some(format!(
            "status={} type={} generation={}",
            workspace.status, workspace.workspace_type, workspace.config_generation
        )),
    )?;
    Ok(workspace)
}

/// Set or clear workspace-level agent override for a branch workspace.
pub fn set_workspace_active_agent(
    ship_dir: &Path,
    branch: &str,
    agent_id: Option<&str>,
) -> Result<Workspace> {
    let branch = ensure_branch_key(branch)?;
    let mut workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow::anyhow!("Workspace not found for branch '{}'", branch))?;

    workspace.active_agent = match agent_id {
        Some(a) => Some(validate_agent_exists(ship_dir, a)?),
        None => None,
    };
    if workspace.status == WorkspaceStatus::Active {
        let active_agent = workspace.active_agent.clone();
        compile_workspace_context(ship_dir, &mut workspace, active_agent.as_deref())?;
    } else {
        upsert_workspace(ship_dir, &workspace)?;
    }
    Ok(workspace)
}

/// Reconcile the current branch into an active workspace record.
pub fn sync_workspace(ship_dir: &Path, branch: &str) -> Result<Workspace> {
    activate_workspace(ship_dir, branch)
}

/// Returns the type of the currently active workspace, or `None` if none is active.
pub fn get_active_workspace_type(ship_dir: &Path) -> Result<Option<ShipWorkspaceKind>> {
    let workspaces = list_workspaces(ship_dir)?;
    Ok(workspaces
        .iter()
        .find(|w| w.status == WorkspaceStatus::Active)
        .map(|w| w.workspace_type))
}

/// Create the default service workspace ("ship") if it doesn't already exist.
/// Called from `init_project`. The workspace starts Active so it's immediately
/// usable, and uses the branch name "ship".
pub fn seed_service_workspace(ship_dir: &Path) -> Result<()> {
    const PROJECT_BRANCH: &str = "ship";

    let existing = list_workspaces(ship_dir)?;
    if existing
        .iter()
        .any(|w| w.workspace_type == ShipWorkspaceKind::Service)
    {
        return Ok(());
    }

    let now = Utc::now();
    let mut workspace = new_workspace(PROJECT_BRANCH, now);
    workspace.workspace_type = ShipWorkspaceKind::Service;
    workspace.status = WorkspaceStatus::Active;
    workspace.last_activated_at = Some(now);

    upsert_workspace(ship_dir, &workspace)?;

    Ok(())
}
