use anyhow::Result;
use chrono::Utc;
use std::path::Path;

use super::compile::compile_workspace_context;
use super::crud::{get_workspace, list_workspaces};
use super::event_upserts::{
    emit_workspace_agent_changed_event, upsert_workspace_on_activate,
    upsert_workspace_on_agent_changed, upsert_workspace_on_archived, upsert_workspace_on_created,
    upsert_workspace_on_status_changed,
};
use super::helpers::*;
use super::types::*;
use super::types_session::*;

use crate::db::actor_events::{emit_actor_created, emit_actor_stopped};
use crate::events::types::ActorCreated;

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
    upsert_workspace_on_created(ship_dir, &workspace)?;
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

    let old_status = workspace.status.to_string();
    let now = Utc::now();
    if next_status == WorkspaceStatus::Active {
        workspace.last_activated_at = Some(now);
    }

    workspace.status = next_status;
    let new_status = workspace.status.to_string();
    if next_status == WorkspaceStatus::Archived {
        upsert_workspace_on_archived(ship_dir, &workspace)?;
    } else {
        upsert_workspace_on_status_changed(ship_dir, &workspace, &old_status, &new_status)?;
    }
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
    // Emit workspace.activated (overwrites the compile upsert with the same
    // data; the transaction guarantees the event is never orphaned).
    upsert_workspace_on_activate(ship_dir, &workspace)?;

    // Auto-create actor for this workspace/agent pair.
    ensure_actor_for_workspace(&workspace)?;

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
        emit_workspace_agent_changed_event(ship_dir, &workspace.branch, active_agent.as_deref())?;
        compile_workspace_context(ship_dir, &mut workspace, active_agent.as_deref())?;
    } else {
        upsert_workspace_on_agent_changed(ship_dir, &workspace)?;
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

// ── Actor auto-creation ──────────────────────────────────────────────────────

/// Derive the actor ID for a workspace: `{workspace_id}/{agent_id}`.
fn actor_id_for_workspace(workspace: &Workspace) -> String {
    let agent = workspace
        .active_agent
        .as_deref()
        .unwrap_or("default");
    format!("{}/{}", workspace.id, agent)
}

/// Query the current actor for this workspace from the workspace DB.
/// Returns `(actor_id, kind)` if an active (non-stopped) actor exists.
fn current_actor_in_workspace(workspace: &Workspace) -> Result<Option<String>> {
    let mut conn = crate::db::workspace_db::open_workspace_db_for_id(&workspace.id)?;
    let rows: Vec<(String,)> = crate::db::block_on(async {
        sqlx::query_as(
            "SELECT id FROM actors WHERE workspace_id = ? AND status != 'stopped' \
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(&workspace.id)
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows.first().map(|r| r.0.clone()))
}

/// Ensure an actor exists for the workspace's current agent.
/// If the agent changed, stop the old actor and create a new one.
fn ensure_actor_for_workspace(workspace: &Workspace) -> Result<()> {
    let desired_id = actor_id_for_workspace(workspace);
    let ws_id = Some(workspace.id.as_str());

    match current_actor_in_workspace(workspace)? {
        Some(existing_id) if existing_id == desired_id => {
            // Actor already exists for this agent — nothing to do.
        }
        Some(existing_id) => {
            // Agent changed — stop old actor, create new one.
            emit_actor_stopped(&existing_id, "agent changed", ws_id, None)?;
            emit_actor_created(
                &desired_id,
                &ActorCreated {
                    kind: "workspace".to_string(),
                    environment_type: "local".to_string(),
                    workspace_id: ws_id.map(str::to_string),
                    parent_actor_id: None,
                    restart_count: 0,
                },
                ws_id,
                None,
            )?;
        }
        None => {
            // No actor yet — create one.
            emit_actor_created(
                &desired_id,
                &ActorCreated {
                    kind: "workspace".to_string(),
                    environment_type: "local".to_string(),
                    workspace_id: ws_id.map(str::to_string),
                    parent_actor_id: None,
                    restart_count: 0,
                },
                ws_id,
                None,
            )?;
        }
    }

    Ok(())
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

    upsert_workspace_on_created(ship_dir, &workspace)?;

    Ok(())
}
