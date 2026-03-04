use crate::project::sanitize_file_name;
use crate::state_db::{
    WorkspaceUpsert, clear_branch_link, demote_other_active_workspaces_db, get_workspace_db,
    list_workspaces_db, set_branch_link, upsert_workspace_db,
};
use crate::state_db::{get_branch_link, get_feature_by_branch_links, get_feature_links};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::Path;
use std::str::FromStr;

// ─── Data types ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceType {
    #[default]
    Feature,
    Refactor,
    Experiment,
    Hotfix,
}

impl std::fmt::Display for WorkspaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceType::Feature => write!(f, "feature"),
            WorkspaceType::Refactor => write!(f, "refactor"),
            WorkspaceType::Experiment => write!(f, "experiment"),
            WorkspaceType::Hotfix => write!(f, "hotfix"),
        }
    }
}

impl std::str::FromStr for WorkspaceType {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "feature" => Ok(WorkspaceType::Feature),
            "refactor" => Ok(WorkspaceType::Refactor),
            "experiment" => Ok(WorkspaceType::Experiment),
            "hotfix" => Ok(WorkspaceType::Hotfix),
            _ => Err(anyhow::anyhow!("Invalid workspace type: {}", value)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceStatus {
    #[default]
    Planned,
    Active,
    Idle,
    Review,
    Merged,
    Archived,
}

impl std::fmt::Display for WorkspaceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceStatus::Planned => write!(f, "planned"),
            WorkspaceStatus::Active => write!(f, "active"),
            WorkspaceStatus::Idle => write!(f, "idle"),
            WorkspaceStatus::Review => write!(f, "review"),
            WorkspaceStatus::Merged => write!(f, "merged"),
            WorkspaceStatus::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for WorkspaceStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "planned" => Ok(WorkspaceStatus::Planned),
            "active" => Ok(WorkspaceStatus::Active),
            "idle" => Ok(WorkspaceStatus::Idle),
            "review" => Ok(WorkspaceStatus::Review),
            "merged" => Ok(WorkspaceStatus::Merged),
            "archived" => Ok(WorkspaceStatus::Archived),
            _ => Err(anyhow::anyhow!("Invalid workspace status: {}", value)),
        }
    }
}

/// Workspace runtime state — SQLite only, no frontmatter file.
/// `branch` is the workspace key and can represent either a git branch or a
/// non-git runtime workspace identifier.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Workspace {
    pub id: String,
    pub branch: String,
    #[serde(default)]
    pub workspace_type: WorkspaceType,
    #[serde(default)]
    pub status: WorkspaceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_mode: Option<String>,
    pub providers: Vec<String>,
    pub resolved_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activated_at: Option<DateTime<Utc>>,
    pub is_worktree: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_hash: Option<String>,
}

/// Input for creating or updating a workspace runtime record.
#[derive(Debug, Clone, Default)]
pub struct CreateWorkspaceRequest {
    pub branch: String,
    pub workspace_type: Option<WorkspaceType>,
    pub status: Option<WorkspaceStatus>,
    pub feature_id: Option<String>,
    pub spec_id: Option<String>,
    pub release_id: Option<String>,
    pub active_mode: Option<String>,
    pub providers: Option<Vec<String>>,
    pub is_worktree: Option<bool>,
    pub worktree_path: Option<String>,
    pub context_hash: Option<String>,
}

// ─── CRUD ─────────────────────────────────────────────────────────────────────

fn parse_datetime(value: &str) -> DateTime<Utc> {
    DateTime::from_str(value).unwrap_or_else(|_| Utc::now())
}

fn parse_datetime_opt(value: Option<String>) -> Option<DateTime<Utc>> {
    value.and_then(|entry| DateTime::from_str(&entry).ok())
}

fn infer_workspace_type(branch: &str, feature_id: Option<&str>) -> WorkspaceType {
    if feature_id.is_some() {
        return WorkspaceType::Feature;
    }
    if branch.starts_with("refactor/") {
        return WorkspaceType::Refactor;
    }
    if branch.starts_with("experiment/") {
        return WorkspaceType::Experiment;
    }
    if branch.starts_with("hotfix/") {
        return WorkspaceType::Hotfix;
    }
    WorkspaceType::Feature
}

fn workspace_id_from_branch(branch: &str) -> String {
    sanitize_file_name(branch)
}

fn ensure_branch_key(branch: &str) -> Result<&str> {
    let trimmed = branch.trim();
    if trimmed.is_empty() {
        return Err(anyhow::anyhow!("Workspace branch/key cannot be empty"));
    }
    Ok(trimmed)
}

fn new_workspace(branch: &str, now: DateTime<Utc>) -> Workspace {
    Workspace {
        id: workspace_id_from_branch(branch),
        branch: branch.to_string(),
        workspace_type: WorkspaceType::Feature,
        status: WorkspaceStatus::Planned,
        feature_id: None,
        spec_id: None,
        release_id: None,
        active_mode: None,
        providers: Vec::new(),
        resolved_at: now,
        last_activated_at: None,
        is_worktree: false,
        worktree_path: None,
        context_hash: None,
    }
}

fn hydrate_from_branch_links(
    ship_dir: &Path,
    branch: &str,
    workspace: &mut Workspace,
) -> Result<()> {
    if let Some((link_type, link_id)) = get_branch_link(ship_dir, branch)? {
        match link_type.as_str() {
            "feature" => {
                workspace.feature_id = Some(link_id.clone());
                if let Some((spec_id, release_id)) = get_feature_links(ship_dir, &link_id)? {
                    workspace.spec_id = spec_id;
                    workspace.release_id = release_id;
                }
            }
            "spec" => {
                workspace.spec_id = Some(link_id);
            }
            _ => {}
        }
    }

    // Git branch linkage also lives on feature rows; hydrate from there when
    // no explicit branch_context mapping is present.
    if workspace.feature_id.is_none()
        && let Some((feature_id, spec_id, release_id)) =
            get_feature_by_branch_links(ship_dir, branch)?
    {
        workspace.feature_id = Some(feature_id);
        if workspace.spec_id.is_none() {
            workspace.spec_id = spec_id;
        }
        workspace.release_id = release_id;
    }

    Ok(())
}

fn hydrate_from_feature_links(ship_dir: &Path, workspace: &mut Workspace) -> Result<()> {
    if let Some(feature_id) = workspace.feature_id.clone()
        && let Some((spec_id, release_id)) = get_feature_links(ship_dir, &feature_id)?
    {
        if workspace.spec_id.is_none() {
            workspace.spec_id = spec_id;
        }
        if workspace.release_id.is_none() {
            workspace.release_id = release_id;
        }
    }
    Ok(())
}

fn persist_branch_link_from_workspace(ship_dir: &Path, workspace: &Workspace) -> Result<()> {
    if let Some(feature_id) = workspace.feature_id.as_deref() {
        return set_branch_link(ship_dir, &workspace.branch, "feature", feature_id);
    }
    if let Some(spec_id) = workspace.spec_id.as_deref() {
        return set_branch_link(ship_dir, &workspace.branch, "spec", spec_id);
    }
    clear_branch_link(ship_dir, &workspace.branch)
}

fn lifecycle_allows_transition(from: WorkspaceStatus, to: WorkspaceStatus) -> bool {
    if from == to {
        return true;
    }

    match from {
        WorkspaceStatus::Planned => {
            matches!(to, WorkspaceStatus::Active | WorkspaceStatus::Archived)
        }
        WorkspaceStatus::Active => matches!(
            to,
            WorkspaceStatus::Idle | WorkspaceStatus::Review | WorkspaceStatus::Merged
        ),
        WorkspaceStatus::Idle => matches!(
            to,
            WorkspaceStatus::Active | WorkspaceStatus::Review | WorkspaceStatus::Archived
        ),
        WorkspaceStatus::Review => matches!(
            to,
            WorkspaceStatus::Active | WorkspaceStatus::Merged | WorkspaceStatus::Archived
        ),
        WorkspaceStatus::Merged => {
            matches!(to, WorkspaceStatus::Archived | WorkspaceStatus::Active)
        }
        WorkspaceStatus::Archived => matches!(to, WorkspaceStatus::Active),
    }
}

fn type_allows_status(workspace_type: WorkspaceType, status: WorkspaceStatus) -> bool {
    !(workspace_type == WorkspaceType::Experiment && status == WorkspaceStatus::Merged)
}

pub fn validate_workspace_transition(
    workspace_type: WorkspaceType,
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

pub fn get_workspace(ship_dir: &Path, branch: &str) -> Result<Option<Workspace>> {
    let row = get_workspace_db(ship_dir, branch)?;
    Ok(row.map(
        |(
            id,
            workspace_type,
            status,
            feature_id,
            spec_id,
            release_id,
            active_mode,
            providers,
            resolved_at,
            is_worktree,
            worktree_path,
            last_activated_at,
            context_hash,
        )| {
            let resolved_at = parse_datetime(&resolved_at);
            Workspace {
                id,
                branch: branch.to_string(),
                workspace_type: workspace_type.parse().unwrap_or_default(),
                status: status.parse().unwrap_or_default(),
                feature_id,
                spec_id,
                release_id,
                active_mode,
                providers,
                resolved_at,
                last_activated_at: parse_datetime_opt(last_activated_at),
                is_worktree,
                worktree_path,
                context_hash,
            }
        },
    ))
}

pub fn list_workspaces(ship_dir: &Path) -> Result<Vec<Workspace>> {
    let rows = list_workspaces_db(ship_dir)?;
    let mut workspaces = Vec::with_capacity(rows.len());
    for (
        branch,
        id,
        workspace_type,
        status,
        feature_id,
        spec_id,
        release_id,
        active_mode,
        providers,
        resolved_at,
        is_worktree,
        worktree_path,
        last_activated_at,
        context_hash,
    ) in rows
    {
        workspaces.push(Workspace {
            id,
            branch,
            workspace_type: workspace_type.parse().unwrap_or_default(),
            status: status.parse().unwrap_or_default(),
            feature_id,
            spec_id,
            release_id,
            active_mode,
            providers,
            resolved_at: parse_datetime(&resolved_at),
            last_activated_at: parse_datetime_opt(last_activated_at),
            is_worktree,
            worktree_path,
            context_hash,
        });
    }
    Ok(workspaces)
}

pub fn upsert_workspace(ship_dir: &Path, workspace: &Workspace) -> Result<()> {
    let workspace_id = if workspace.id.trim().is_empty() {
        workspace_id_from_branch(&workspace.branch)
    } else {
        workspace.id.clone()
    };

    let resolved_at = workspace.resolved_at.to_rfc3339();
    let workspace_type = workspace.workspace_type.to_string();
    let status = workspace.status.to_string();
    let last_activated_at = workspace
        .last_activated_at
        .as_ref()
        .map(|ts| ts.to_rfc3339());

    upsert_workspace_db(
        ship_dir,
        WorkspaceUpsert {
            branch: &workspace.branch,
            workspace_id: &workspace_id,
            workspace_type: &workspace_type,
            status: &status,
            feature_id: workspace.feature_id.as_deref(),
            spec_id: workspace.spec_id.as_deref(),
            release_id: workspace.release_id.as_deref(),
            active_mode: workspace.active_mode.as_deref(),
            providers: &workspace.providers,
            resolved_at: &resolved_at,
            is_worktree: workspace.is_worktree,
            worktree_path: workspace.worktree_path.as_deref(),
            last_activated_at: last_activated_at.as_deref(),
            context_hash: workspace.context_hash.as_deref(),
        },
    )
}

/// Create or update a workspace record without requiring a git checkout.
/// This is the runtime-native entrypoint for workspace lifecycle management.
pub fn create_workspace(ship_dir: &Path, request: CreateWorkspaceRequest) -> Result<Workspace> {
    let branch = ensure_branch_key(&request.branch)?.to_string();
    let now = Utc::now();

    let existing = get_workspace(ship_dir, &branch)?;
    let mut workspace = existing
        .clone()
        .unwrap_or_else(|| new_workspace(&branch, now));

    if let Some(feature_id) = request.feature_id {
        workspace.feature_id = Some(feature_id);
    }
    if let Some(spec_id) = request.spec_id {
        workspace.spec_id = Some(spec_id);
    }
    if let Some(release_id) = request.release_id {
        workspace.release_id = Some(release_id);
    }
    if let Some(active_mode) = request.active_mode {
        workspace.active_mode = Some(active_mode);
    }
    if let Some(providers) = request.providers {
        workspace.providers = providers;
    }
    if let Some(is_worktree) = request.is_worktree {
        workspace.is_worktree = is_worktree;
    }
    if let Some(worktree_path) = request.worktree_path {
        workspace.worktree_path = Some(worktree_path);
    }
    if let Some(context_hash) = request.context_hash {
        workspace.context_hash = Some(context_hash);
    }

    hydrate_from_branch_links(ship_dir, &branch, &mut workspace)?;
    workspace.workspace_type = request.workspace_type.unwrap_or_else(|| {
        existing
            .as_ref()
            .map(|entry| entry.workspace_type)
            .unwrap_or_else(|| infer_workspace_type(&branch, workspace.feature_id.as_deref()))
    });

    hydrate_from_feature_links(ship_dir, &mut workspace)?;

    let base_status = existing
        .as_ref()
        .map(|entry| entry.status)
        .unwrap_or(WorkspaceStatus::Planned);
    let next_status = request.status.unwrap_or(base_status);

    validate_workspace_transition(workspace.workspace_type, base_status, next_status)?;

    workspace.id = workspace_id_from_branch(&branch);
    workspace.branch = branch;
    workspace.status = next_status;
    workspace.resolved_at = now;
    if next_status == WorkspaceStatus::Active {
        demote_other_active_workspaces_db(ship_dir, &workspace.branch, &now.to_rfc3339())?;
        workspace.last_activated_at = Some(now);
    }

    persist_branch_link_from_workspace(ship_dir, &workspace)?;
    upsert_workspace(ship_dir, &workspace)?;
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
        demote_other_active_workspaces_db(ship_dir, &workspace.branch, &now.to_rfc3339())?;
        workspace.last_activated_at = Some(now);
    }

    workspace.status = next_status;
    workspace.resolved_at = now;
    upsert_workspace(ship_dir, &workspace)?;
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
    if workspace.workspace_type == WorkspaceType::Feature {
        workspace.workspace_type = infer_workspace_type(branch, workspace.feature_id.as_deref());
    }

    validate_workspace_transition(
        workspace.workspace_type,
        workspace.status,
        WorkspaceStatus::Active,
    )?;

    demote_other_active_workspaces_db(ship_dir, branch, &now.to_rfc3339())?;
    workspace.status = WorkspaceStatus::Active;
    workspace.resolved_at = now;
    workspace.last_activated_at = Some(now);

    persist_branch_link_from_workspace(ship_dir, &workspace)?;
    upsert_workspace(ship_dir, &workspace)?;
    Ok(workspace)
}

/// Reconcile the current branch into an active workspace record.
pub fn sync_workspace(ship_dir: &Path, branch: &str) -> Result<Workspace> {
    activate_workspace(ship_dir, branch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn lifecycle_transition_matrix_covers_expected_paths() {
        assert!(
            validate_workspace_transition(
                WorkspaceType::Feature,
                WorkspaceStatus::Planned,
                WorkspaceStatus::Active
            )
            .is_ok()
        );
        assert!(
            validate_workspace_transition(
                WorkspaceType::Feature,
                WorkspaceStatus::Active,
                WorkspaceStatus::Review
            )
            .is_ok()
        );
        assert!(
            validate_workspace_transition(
                WorkspaceType::Feature,
                WorkspaceStatus::Review,
                WorkspaceStatus::Merged
            )
            .is_ok()
        );
        assert!(
            validate_workspace_transition(
                WorkspaceType::Feature,
                WorkspaceStatus::Merged,
                WorkspaceStatus::Archived
            )
            .is_ok()
        );
    }

    #[test]
    fn invalid_transitions_are_rejected() {
        let err = validate_workspace_transition(
            WorkspaceType::Feature,
            WorkspaceStatus::Planned,
            WorkspaceStatus::Review,
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("Invalid workspace transition: planned -> review")
        );
    }

    #[test]
    fn experiment_workspace_can_never_merge() {
        let err = validate_workspace_transition(
            WorkspaceType::Experiment,
            WorkspaceStatus::Review,
            WorkspaceStatus::Merged,
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("Workspace type 'experiment' cannot enter status 'merged'")
        );
    }

    #[test]
    fn workspace_branch_key_validation_rejects_empty_values() {
        let err = ensure_branch_key("   ").unwrap_err();
        assert!(
            err.to_string()
                .contains("Workspace branch/key cannot be empty")
        );
    }

    #[test]
    fn inferred_workspace_type_prefers_feature_links_then_prefixes() {
        assert_eq!(
            infer_workspace_type("sandbox/personal", Some("auth-redesign")),
            WorkspaceType::Feature
        );
        assert_eq!(
            infer_workspace_type("experiment/agent-lab", None),
            WorkspaceType::Experiment
        );
        assert_eq!(
            infer_workspace_type("hotfix/token", None),
            WorkspaceType::Hotfix
        );
    }

    #[test]
    fn create_workspace_hydrates_feature_link_from_branch_context() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        crate::state_db::ensure_project_database(&ship_dir)?;
        crate::state_db::set_branch_link(
            &ship_dir,
            "feature/auth-redesign",
            "feature",
            "feat-auth",
        )?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/auth-redesign".to_string(),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        assert_eq!(workspace.feature_id.as_deref(), Some("feat-auth"));
        Ok(())
    }
}
