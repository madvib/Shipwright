use crate::db::branch_context::{clear_branch_link, get_branch_link, set_branch_link};
use crate::project::{get_global_dir, project_slug_from_ship_dir, sanitize_file_name};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::types::*;

// ---- Parsing helpers -------------------------------------------------------

pub(crate) fn parse_datetime(value: &str) -> DateTime<Utc> {
    DateTime::from_str(value).unwrap_or_else(|_| Utc::now())
}

pub(crate) fn parse_datetime_opt(value: Option<String>) -> Option<DateTime<Utc>> {
    value.and_then(|entry| DateTime::from_str(&entry).ok())
}

pub(crate) fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|entry| {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(crate) fn normalize_agent_ref(agent: &str) -> Option<String> {
    let trimmed = agent.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn normalize_provider_ref(provider: &str) -> Option<String> {
    let trimmed = provider.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

pub(crate) fn normalize_nonempty_id_list(ids: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for raw in ids {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value = trimmed.to_string();
        if !normalized.iter().any(|existing| existing == &value) {
            normalized.push(value);
        }
    }
    normalized
}

// ---- Branch key ------------------------------------------------------------

pub(crate) fn ensure_branch_key(branch: &str) -> Result<&str> {
    let trimmed = branch.trim();
    if trimmed.is_empty() {
        return Err(anyhow::anyhow!("Workspace branch/key cannot be empty"));
    }
    Ok(trimmed)
}

// ---- Workspace construction ------------------------------------------------

pub(crate) fn workspace_id_from_branch(branch: &str) -> String {
    sanitize_file_name(branch)
}

pub(crate) fn infer_workspace_type(branch: &str, feature_id: Option<&str>) -> ShipWorkspaceKind {
    if feature_id.is_some() {
        return ShipWorkspaceKind::Feature;
    }
    if branch.starts_with("patch/") {
        return ShipWorkspaceKind::Patch;
    }
    ShipWorkspaceKind::Feature
}

pub(crate) fn new_workspace(branch: &str, now: DateTime<Utc>) -> Workspace {
    Workspace {
        id: workspace_id_from_branch(branch),
        branch: branch.to_string(),
        workspace_type: ShipWorkspaceKind::Feature,
        status: WorkspaceStatus::Active,
        environment_id: None,
        feature_id: None,
        target_id: None,
        active_agent: None,
        providers: Vec::new(),
        mcp_servers: Vec::new(),
        skills: Vec::new(),
        resolved_at: now,
        last_activated_at: None,
        is_worktree: false,
        worktree_path: None,
        context_hash: None,
        config_generation: 0,
        compiled_at: None,
        compile_error: None,
    }
}

// ---- Branch links ----------------------------------------------------------

pub(crate) fn hydrate_from_branch_links(
    ship_dir: &Path,
    branch: &str,
    workspace: &mut Workspace,
) -> Result<()> {
    if let Some((link_type, link_id)) = get_branch_link(ship_dir, branch)? {
        match link_type.as_str() {
            "feature" => {
                workspace.feature_id = Some(link_id);
            }
            "target" | "release" => {
                workspace.target_id = Some(link_id);
            }
            _ => {}
        }
    }
    Ok(())
}

pub(crate) fn hydrate_from_feature_links(
    _ship_dir: &Path,
    _workspace: &mut Workspace,
) -> Result<()> {
    // Feature links have been migrated to branch_context; this function is
    // retained as a no-op to preserve existing call-sites without changing
    // public surface area.
    Ok(())
}

pub(crate) fn persist_branch_link_from_workspace(
    ship_dir: &Path,
    workspace: &Workspace,
) -> Result<()> {
    if let Some(feature_id) = workspace.feature_id.as_deref() {
        return set_branch_link(ship_dir, &workspace.branch, "feature", feature_id);
    }
    clear_branch_link(ship_dir, &workspace.branch)
}

// ---- Lifecycle validation --------------------------------------------------

fn lifecycle_allows_transition(from: WorkspaceStatus, to: WorkspaceStatus) -> bool {
    from == to
        || matches!(
            (from, to),
            (WorkspaceStatus::Active, WorkspaceStatus::Archived)
                | (WorkspaceStatus::Archived, WorkspaceStatus::Active)
        )
}

fn type_allows_status(workspace_type: ShipWorkspaceKind, status: WorkspaceStatus) -> bool {
    let _ = workspace_type;
    let _ = status;
    true
}

pub fn validate_workspace_transition(
    workspace_type: ShipWorkspaceKind,
    from: WorkspaceStatus,
    to: WorkspaceStatus,
) -> Result<()> {
    if !type_allows_status(workspace_type, to) {
        return Err(anyhow::anyhow!(
            "Workspace type '{}' cannot enter status '{}'",
            workspace_type,
            to
        ));
    }
    if !lifecycle_allows_transition(from, to) {
        return Err(anyhow::anyhow!(
            "Invalid workspace transition: {} -> {}",
            from,
            to
        ));
    }
    Ok(())
}

// ---- Agent validation ------------------------------------------------------

pub(crate) fn validate_agent_exists(ship_dir: &Path, agent_id: &str) -> Result<String> {
    let normalized = normalize_agent_ref(agent_id)
        .ok_or_else(|| anyhow::anyhow!("Workspace agent cannot be empty"))?;
    let effective = crate::config::get_effective_config(Some(ship_dir.to_path_buf()))?;
    if effective.modes.iter().any(|mode| mode.id == normalized) {
        Ok(normalized)
    } else {
        Err(anyhow::anyhow!("Agent '{}' not found", normalized))
    }
}

// ---- Worktree path defaults ------------------------------------------------

pub(crate) fn default_project_worktree_root(ship_dir: &Path) -> PathBuf {
    ship_dir.join("worktrees")
}

pub(crate) fn default_global_worktree_root(ship_dir: &Path) -> Option<PathBuf> {
    let slug = project_slug_from_ship_dir(ship_dir);
    let global_dir = get_global_dir().ok()?;
    Some(global_dir.join("projects").join(slug).join("worktrees"))
}

pub(crate) fn default_worktree_path(ship_dir: &Path, branch: &str) -> Option<String> {
    let branch_token = sanitize_file_name(branch);
    let project_root = default_project_worktree_root(ship_dir);
    if std::fs::create_dir_all(&project_root).is_ok() {
        return Some(
            project_root
                .join(&branch_token)
                .to_string_lossy()
                .to_string(),
        );
    }
    if let Some(global_root) = default_global_worktree_root(ship_dir)
        && std::fs::create_dir_all(&global_root).is_ok()
    {
        return Some(global_root.join(branch_token).to_string_lossy().to_string());
    }
    None
}
