use crate::project::sanitize_file_name;
use crate::state_db::{
    WorkspaceSessionDb, WorkspaceUpsert, clear_branch_link, delete_workspace_db,
    demote_other_active_workspaces_db, get_active_workspace_session_db, get_workspace_db,
    get_workspace_session_db, insert_workspace_session_db, list_workspace_sessions_db,
    list_workspaces_db, set_branch_link, update_workspace_session_db, upsert_workspace_db,
};
use crate::state_db::{get_branch_link, get_feature_by_branch_links, get_feature_links};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    /// Project-manager workspace: planning, triage, releases, specs. Not tied
    /// to a single feature branch. Automatically unlocks the full PM tool surface.
    Project,
}

impl std::fmt::Display for WorkspaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceType::Feature => write!(f, "feature"),
            WorkspaceType::Refactor => write!(f, "refactor"),
            WorkspaceType::Experiment => write!(f, "experiment"),
            WorkspaceType::Hotfix => write!(f, "hotfix"),
            WorkspaceType::Project => write!(f, "project"),
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
            "project" => Ok(WorkspaceType::Project),
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
    #[serde(default)]
    pub config_generation: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiled_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compile_error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceSessionStatus {
    #[default]
    Active,
    Ended,
}

impl std::fmt::Display for WorkspaceSessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceSessionStatus::Active => write!(f, "active"),
            WorkspaceSessionStatus::Ended => write!(f, "ended"),
        }
    }
}

impl std::str::FromStr for WorkspaceSessionStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "active" => Ok(WorkspaceSessionStatus::Active),
            "ended" => Ok(WorkspaceSessionStatus::Ended),
            _ => Err(anyhow::anyhow!(
                "Invalid workspace session status: {}",
                value
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WorkspaceSession {
    pub id: String,
    pub workspace_id: String,
    pub workspace_branch: String,
    pub status: WorkspaceSessionStatus,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default)]
    pub updated_feature_ids: Vec<String>,
    #[serde(default)]
    pub updated_spec_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiled_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compile_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_generation_at_start: Option<i64>,
    #[serde(default)]
    pub stale_context: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct EndWorkspaceSessionRequest {
    pub summary: Option<String>,
    pub updated_feature_ids: Vec<String>,
    pub updated_spec_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WorkspaceProviderMatrix {
    pub workspace_branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_id: Option<String>,
    pub source: String,
    pub allowed_providers: Vec<String>,
    pub supported_providers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution_error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WorkspaceRepairReport {
    pub workspace_branch: String,
    pub dry_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_id: Option<String>,
    pub status: WorkspaceStatus,
    pub providers_expected: Vec<String>,
    pub missing_provider_configs: Vec<String>,
    pub had_compile_error: bool,
    pub needs_recompile: bool,
    pub reapplied_compile: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution_error: Option<String>,
    pub actions: Vec<String>,
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

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|entry| {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn hydrate_workspace_session(row: WorkspaceSessionDb) -> WorkspaceSession {
    WorkspaceSession {
        id: row.id,
        workspace_id: row.workspace_id,
        workspace_branch: row.workspace_branch,
        status: row.status.parse().unwrap_or(WorkspaceSessionStatus::Active),
        started_at: parse_datetime(&row.started_at),
        ended_at: parse_datetime_opt(row.ended_at),
        mode_id: row.mode_id,
        primary_provider: row.primary_provider,
        goal: row.goal,
        summary: row.summary,
        updated_feature_ids: row.updated_feature_ids,
        updated_spec_ids: row.updated_spec_ids,
        compiled_at: parse_datetime_opt(row.compiled_at),
        compile_error: row.compile_error,
        config_generation_at_start: row.config_generation_at_start,
        stale_context: false,
        created_at: parse_datetime(&row.created_at),
        updated_at: parse_datetime(&row.updated_at),
    }
}

fn annotate_session_stale_state(
    session: &mut WorkspaceSession,
    workspace_generation_by_branch: &HashMap<String, i64>,
) {
    session.stale_context = session
        .config_generation_at_start
        .and_then(|session_generation| {
            workspace_generation_by_branch
                .get(&session.workspace_branch)
                .map(|workspace_generation| *workspace_generation > session_generation)
        })
        .unwrap_or(false);
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

fn normalize_mode_ref(mode: &str) -> Option<String> {
    let trimmed = mode.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_provider_ref(provider: &str) -> Option<String> {
    let trimmed = provider.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn validate_mode_exists(ship_dir: &Path, mode_id: &str) -> Result<String> {
    let normalized = normalize_mode_ref(mode_id)
        .ok_or_else(|| anyhow::anyhow!("Workspace mode cannot be empty"))?;
    let effective = crate::config::get_effective_config(Some(ship_dir.to_path_buf()))?;
    if effective.modes.iter().any(|mode| mode.id == normalized) {
        Ok(normalized)
    } else {
        Err(anyhow::anyhow!("Mode '{}' not found", normalized))
    }
}

fn resolve_session_providers(
    ship_dir: &Path,
    workspace: &Workspace,
    mode_id: Option<&str>,
) -> Result<Vec<String>> {
    let matrix = build_workspace_provider_matrix(ship_dir, workspace, mode_id)?;
    if let Some(error) = matrix.resolution_error {
        return Err(anyhow!(error));
    }
    Ok(matrix.allowed_providers)
}

fn resolve_provider_candidates(
    config: &crate::config::ProjectConfig,
    workspace: &Workspace,
    mode_id: Option<&str>,
) -> (Vec<String>, &'static str, Option<String>) {
    let resolved_mode_id = mode_id.and_then(normalize_mode_ref).or_else(|| {
        workspace
            .active_mode
            .as_deref()
            .and_then(normalize_mode_ref)
    });

    let mode_targets = resolved_mode_id
        .as_deref()
        .and_then(|id| config.modes.iter().find(|mode| mode.id == id))
        .map(|mode| mode.target_agents.clone())
        .unwrap_or_default();

    let (candidates, source) = if !workspace.providers.is_empty() {
        (workspace.providers.clone(), "workspace")
    } else if !mode_targets.is_empty() {
        (mode_targets, "mode")
    } else if !config.providers.is_empty() {
        (config.providers.clone(), "config")
    } else {
        (vec!["claude".to_string()], "default")
    };
    (candidates, source, resolved_mode_id)
}

fn build_workspace_provider_matrix(
    ship_dir: &Path,
    workspace: &Workspace,
    mode_id: Option<&str>,
) -> Result<WorkspaceProviderMatrix> {
    let config = crate::config::get_effective_config(Some(ship_dir.to_path_buf()))?;
    let (candidates, source, resolved_mode_id) =
        resolve_provider_candidates(&config, workspace, mode_id);
    let mut providers = Vec::new();
    for candidate in candidates {
        let Some(normalized) = normalize_provider_ref(&candidate) else {
            continue;
        };
        if crate::agents::export::get_provider(&normalized).is_some()
            && !providers.iter().any(|p| p == &normalized)
        {
            providers.push(normalized);
        }
    }

    let supported_providers = crate::agents::export::list_providers(ship_dir)?
        .into_iter()
        .map(|provider| provider.id)
        .collect::<Vec<_>>();

    let resolution_error = if providers.is_empty() {
        Some(format!(
            "No valid providers resolved for workspace '{}' (source: {})",
            workspace.branch, source
        ))
    } else {
        None
    };

    Ok(WorkspaceProviderMatrix {
        workspace_branch: workspace.branch.clone(),
        mode_id: resolved_mode_id,
        source: source.to_string(),
        allowed_providers: providers,
        supported_providers,
        resolution_error,
    })
}

fn resolve_workspace_context_root(ship_dir: &Path, workspace: &Workspace) -> PathBuf {
    if workspace.is_worktree
        && let Some(path) = workspace.worktree_path.as_deref()
    {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    ship_dir.parent().unwrap_or(ship_dir).to_path_buf()
}

fn missing_provider_configs_for_workspace(
    context_root: &Path,
    providers: &[String],
) -> Vec<String> {
    providers
        .iter()
        .filter_map(|provider| {
            let desc = crate::agents::export::get_provider(provider)?;
            let target = context_root.join(desc.project_config);
            if target.exists() {
                None
            } else {
                Some(provider.clone())
            }
        })
        .collect()
}

pub fn get_workspace_provider_matrix(
    ship_dir: &Path,
    branch: &str,
    mode_id: Option<&str>,
) -> Result<WorkspaceProviderMatrix> {
    let branch = ensure_branch_key(branch)?;
    let workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow!("Workspace not found for branch '{}'", branch))?;
    build_workspace_provider_matrix(ship_dir, &workspace, mode_id)
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
        get_workspace_provider_matrix(ship_dir, branch, workspace.active_mode.as_deref())?;
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
            let mode = workspace.active_mode.clone();
            workspace = set_workspace_active_mode(ship_dir, branch, mode.as_deref())?;
            matrix =
                get_workspace_provider_matrix(ship_dir, branch, workspace.active_mode.as_deref())?;
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
        mode_id: workspace.active_mode.clone(),
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

fn compile_workspace_context(
    ship_dir: &Path,
    workspace: &mut Workspace,
    mode_id_override: Option<&str>,
) -> Result<()> {
    let mode_id = mode_id_override
        .map(|mode| mode.to_string())
        .or_else(|| workspace.active_mode.clone());
    let mode_id = mode_id.and_then(|value| normalize_optional_text(Some(value)));
    let providers = match resolve_session_providers(ship_dir, workspace, mode_id.as_deref()) {
        Ok(providers) => providers,
        Err(error) => {
            let now = Utc::now();
            workspace.compiled_at = Some(now);
            workspace.compile_error = Some(error.to_string());
            workspace.resolved_at = now;
            upsert_workspace(ship_dir, workspace)?;
            return Err(error);
        }
    };

    let now = Utc::now();
    for provider in &providers {
        if let Err(error) = crate::agents::export::export_to_with_mode_override(
            ship_dir.to_path_buf(),
            provider,
            mode_id.as_deref(),
        ) {
            let contextual = error.context(format!(
                "Failed to compile provider '{}' for workspace '{}'",
                provider, workspace.branch
            ));
            workspace.compiled_at = Some(now);
            workspace.compile_error = Some(contextual.to_string());
            workspace.resolved_at = now;
            upsert_workspace(ship_dir, workspace)?;
            return Err(contextual);
        }
    }

    workspace.config_generation = workspace.config_generation.saturating_add(1);
    workspace.compiled_at = Some(now);
    workspace.compile_error = None;
    workspace.resolved_at = now;
    upsert_workspace(ship_dir, workspace)?;
    Ok(())
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
        config_generation: 0,
        compiled_at: None,
        compile_error: None,
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
    if workspace_type == WorkspaceType::Experiment && status == WorkspaceStatus::Merged {
        return false;
    }
    // Project workspace has no lifecycle end — it doesn't get merged or reviewed.
    if workspace_type == WorkspaceType::Project
        && matches!(status, WorkspaceStatus::Merged | WorkspaceStatus::Review)
    {
        return false;
    }
    true
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
            config_generation,
            compiled_at,
            compile_error,
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
                config_generation,
                compiled_at: parse_datetime_opt(compiled_at),
                compile_error,
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
        config_generation,
        compiled_at,
        compile_error,
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
            config_generation,
            compiled_at: parse_datetime_opt(compiled_at),
            compile_error,
        });
    }
    Ok(workspaces)
}

pub fn delete_workspace(ship_dir: &Path, branch: &str) -> Result<()> {
    let branch = ensure_branch_key(branch)?;
    clear_branch_link(ship_dir, branch)?;
    let _ = delete_workspace_db(ship_dir, branch)?;
    Ok(())
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
    let compiled_at = workspace.compiled_at.as_ref().map(|ts| ts.to_rfc3339());

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
            config_generation: workspace.config_generation,
            compiled_at: compiled_at.as_deref(),
            compile_error: workspace.compile_error.as_deref(),
        },
    )
}

pub fn get_active_workspace_session(
    ship_dir: &Path,
    branch: &str,
) -> Result<Option<WorkspaceSession>> {
    let branch = ensure_branch_key(branch)?;
    let workspace = match get_workspace(ship_dir, branch)? {
        Some(workspace) => workspace,
        None => return Ok(None),
    };
    let mut generation_by_branch = HashMap::new();
    generation_by_branch.insert(workspace.branch.clone(), workspace.config_generation);
    Ok(
        get_active_workspace_session_db(ship_dir, &workspace.id)?.map(|row| {
            let mut session = hydrate_workspace_session(row);
            annotate_session_stale_state(&mut session, &generation_by_branch);
            session
        }),
    )
}

pub fn list_workspace_sessions(
    ship_dir: &Path,
    branch: Option<&str>,
    limit: usize,
) -> Result<Vec<WorkspaceSession>> {
    let mut workspace_generation_by_branch = HashMap::new();
    let workspace_id = if let Some(branch) = branch {
        let branch = ensure_branch_key(branch)?;
        match get_workspace(ship_dir, branch)? {
            Some(workspace) => {
                workspace_generation_by_branch
                    .insert(workspace.branch.clone(), workspace.config_generation);
                Some(workspace.id)
            }
            None => return Ok(Vec::new()),
        }
    } else {
        for workspace in list_workspaces(ship_dir)? {
            workspace_generation_by_branch.insert(workspace.branch, workspace.config_generation);
        }
        None
    };

    let rows = list_workspace_sessions_db(ship_dir, workspace_id.as_deref(), limit)?;
    let mut sessions: Vec<WorkspaceSession> =
        rows.into_iter().map(hydrate_workspace_session).collect();
    for session in &mut sessions {
        annotate_session_stale_state(session, &workspace_generation_by_branch);
    }
    Ok(sessions)
}

pub fn start_workspace_session(
    ship_dir: &Path,
    branch: &str,
    goal: Option<String>,
    mode_id: Option<String>,
    primary_provider: Option<String>,
) -> Result<WorkspaceSession> {
    let branch = ensure_branch_key(branch)?;
    let mut workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow::anyhow!("Workspace not found for branch '{}'", branch))?;

    if workspace.status != WorkspaceStatus::Active {
        workspace = activate_workspace(ship_dir, branch)?;
    }

    if let Some(mode_id) = mode_id.as_deref() {
        workspace = set_workspace_active_mode(ship_dir, branch, Some(mode_id))?;
    }

    if let Some(active) = get_active_workspace_session_db(ship_dir, &workspace.id)? {
        return Err(anyhow::anyhow!(
            "Workspace session '{}' is already active for '{}'",
            active.id,
            workspace.branch
        ));
    }

    let mode_id = mode_id
        .or(workspace.active_mode.clone())
        .and_then(|value| normalize_optional_text(Some(value)));
    let providers = resolve_session_providers(ship_dir, &workspace, mode_id.as_deref())?;
    let primary_provider = if let Some(requested_provider) = primary_provider {
        let normalized = normalize_provider_ref(&requested_provider)
            .ok_or_else(|| anyhow!("Session provider cannot be empty"))?;
        if !providers.iter().any(|provider| provider == &normalized) {
            return Err(anyhow!(
                "Provider '{}' is not allowed for workspace '{}' (allowed: {})",
                normalized,
                workspace.branch,
                providers.join(", ")
            ));
        }
        normalized
    } else {
        providers
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("No providers resolved for workspace '{}'", workspace.branch))?
    };

    if workspace.compiled_at.is_none() || workspace.compile_error.is_some() {
        compile_workspace_context(ship_dir, &mut workspace, mode_id.as_deref())?;
    }

    let now = Utc::now();
    let session = WorkspaceSessionDb {
        id: crate::gen_nanoid(),
        workspace_id: workspace.id.clone(),
        workspace_branch: workspace.branch.clone(),
        status: WorkspaceSessionStatus::Active.to_string(),
        started_at: now.to_rfc3339(),
        ended_at: None,
        mode_id,
        primary_provider: Some(primary_provider),
        goal: normalize_optional_text(goal),
        summary: None,
        updated_feature_ids: Vec::new(),
        updated_spec_ids: Vec::new(),
        compiled_at: workspace.compiled_at.as_ref().map(|ts| ts.to_rfc3339()),
        compile_error: workspace.compile_error.clone(),
        config_generation_at_start: Some(workspace.config_generation),
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
    };
    insert_workspace_session_db(ship_dir, &session)?;
    let created = get_workspace_session_db(ship_dir, &session.id)?
        .ok_or_else(|| anyhow::anyhow!("Failed to load created workspace session"))?;
    Ok(hydrate_workspace_session(created))
}

pub fn end_workspace_session(
    ship_dir: &Path,
    branch: &str,
    request: EndWorkspaceSessionRequest,
) -> Result<WorkspaceSession> {
    let branch = ensure_branch_key(branch)?;
    let workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow::anyhow!("Workspace not found for branch '{}'", branch))?;

    let mut active = get_active_workspace_session_db(ship_dir, &workspace.id)?
        .ok_or_else(|| anyhow::anyhow!("No active workspace session for '{}'", workspace.branch))?;

    let now = Utc::now().to_rfc3339();
    active.status = WorkspaceSessionStatus::Ended.to_string();
    active.ended_at = Some(now.clone());
    active.summary = normalize_optional_text(request.summary);
    active.updated_feature_ids = request.updated_feature_ids;
    active.updated_spec_ids = request.updated_spec_ids;
    active.updated_at = now;

    update_workspace_session_db(ship_dir, &active)?;

    let ended = get_workspace_session_db(ship_dir, &active.id)?
        .ok_or_else(|| anyhow::anyhow!("Failed to load ended workspace session"))?;
    Ok(hydrate_workspace_session(ended))
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
        workspace.active_mode = Some(validate_mode_exists(ship_dir, &active_mode)?);
    }
    if let Some(providers) = request.providers {
        workspace.providers = providers;
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
    let active_mode = workspace.active_mode.clone();
    compile_workspace_context(ship_dir, &mut workspace, active_mode.as_deref())?;
    Ok(workspace)
}

/// Set or clear workspace-level mode override for a branch workspace.
pub fn set_workspace_active_mode(
    ship_dir: &Path,
    branch: &str,
    mode_id: Option<&str>,
) -> Result<Workspace> {
    let branch = ensure_branch_key(branch)?;
    let mut workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow::anyhow!("Workspace not found for branch '{}'", branch))?;

    workspace.active_mode = match mode_id {
        Some(mode) => Some(validate_mode_exists(ship_dir, mode)?),
        None => None,
    };
    workspace.resolved_at = Utc::now();
    if workspace.status == WorkspaceStatus::Active {
        let active_mode = workspace.active_mode.clone();
        compile_workspace_context(ship_dir, &mut workspace, active_mode.as_deref())?;
    } else {
        upsert_workspace(ship_dir, &workspace)?;
    }
    Ok(workspace)
}

/// Reconcile the current branch into an active workspace record.
pub fn sync_workspace(ship_dir: &Path, branch: &str) -> Result<Workspace> {
    activate_workspace(ship_dir, branch)
}

/// Returns the type of the currently active workspace, or `None` if no workspace is active.
pub fn get_active_workspace_type(ship_dir: &Path) -> Result<Option<WorkspaceType>> {
    let workspaces = list_workspaces(ship_dir)?;
    Ok(workspaces
        .iter()
        .find(|w| w.status == WorkspaceStatus::Active)
        .map(|w| w.workspace_type))
}

/// Create the project-manager workspace ("ship") if it doesn't already exist.
/// Called from `init_project`. The workspace starts Active so it's immediately
/// usable, and uses the branch name "ship".
pub fn seed_project_workspace(ship_dir: &Path) -> Result<()> {
    const PROJECT_BRANCH: &str = "ship";

    // Don't re-seed if any project workspace already exists.
    let existing = list_workspaces(ship_dir)?;
    if existing
        .iter()
        .any(|w| w.workspace_type == WorkspaceType::Project)
    {
        return Ok(());
    }

    let now = Utc::now();
    let mut workspace = new_workspace(PROJECT_BRANCH, now);
    workspace.workspace_type = WorkspaceType::Project;
    workspace.status = WorkspaceStatus::Active;
    workspace.last_activated_at = Some(now);

    // Demote any currently active workspace before seeding.
    demote_other_active_workspaces_db(ship_dir, PROJECT_BRANCH, &now.to_rfc3339())?;
    upsert_workspace(ship_dir, &workspace)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Connection;
    use tempfile::tempdir;

    fn insert_feature_for_branch(
        ship_dir: &Path,
        feature_id: &str,
        branch: &str,
        spec_id: Option<&str>,
        release_id: Option<&str>,
    ) -> Result<()> {
        crate::state_db::ensure_project_database(ship_dir)?;
        let mut conn = crate::state_db::open_project_connection(ship_dir)?;
        let now = Utc::now().to_rfc3339();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(async {
            sqlx::query(
                "INSERT INTO feature (id, title, description, status, release_id, spec_id, branch, agent_json, tags_json, created_at, updated_at)
                 VALUES (?, ?, '', 'planned', ?, ?, ?, '{}', '[]', ?, ?)",
            )
            .bind(feature_id)
            .bind(format!("Feature {}", feature_id))
            .bind(release_id)
            .bind(spec_id)
            .bind(branch)
            .bind(&now)
            .bind(&now)
            .execute(&mut conn)
            .await
        })?;
        rt.block_on(async { conn.close().await })?;
        Ok(())
    }

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

    #[test]
    fn create_workspace_mixed_branch_links_preserve_spec_context_and_hydrate_feature_release()
    -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        crate::state_db::ensure_project_database(&ship_dir)?;

        insert_feature_for_branch(
            &ship_dir,
            "feat-mixed",
            "feature/mixed",
            Some("spec-from-feature"),
            Some("release-from-feature"),
        )?;
        crate::state_db::set_branch_link(&ship_dir, "feature/mixed", "spec", "spec-direct")?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/mixed".to_string(),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        assert_eq!(workspace.feature_id.as_deref(), Some("feat-mixed"));
        assert_eq!(workspace.spec_id.as_deref(), Some("spec-direct"));
        assert_eq!(
            workspace.release_id.as_deref(),
            Some("release-from-feature")
        );
        let stored_link = get_branch_link(&ship_dir, "feature/mixed")?;
        assert_eq!(
            stored_link,
            Some(("feature".to_string(), "feat-mixed".to_string()))
        );
        Ok(())
    }

    #[test]
    fn workspace_never_persists_spec_as_branch_owner() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        crate::state_db::ensure_project_database(&ship_dir)?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "experiment/spec-context".to_string(),
                workspace_type: Some(WorkspaceType::Experiment),
                spec_id: Some("spec-only".to_string()),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        assert_eq!(workspace.spec_id.as_deref(), Some("spec-only"));
        assert!(get_branch_link(&ship_dir, "experiment/spec-context")?.is_none());
        Ok(())
    }

    #[test]
    fn activating_workspace_demotes_other_active_workspace() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        crate::state_db::ensure_project_database(&ship_dir)?;

        let first = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/alpha".to_string(),
                status: Some(WorkspaceStatus::Active),
                feature_id: Some("feat-alpha".to_string()),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        assert_eq!(first.status, WorkspaceStatus::Active);

        let second = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/beta".to_string(),
                status: Some(WorkspaceStatus::Active),
                feature_id: Some("feat-beta".to_string()),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        assert_eq!(second.status, WorkspaceStatus::Active);

        let first_after = get_workspace(&ship_dir, "feature/alpha")?
            .ok_or_else(|| anyhow::anyhow!("feature/alpha workspace missing"))?;
        let second_after = get_workspace(&ship_dir, "feature/beta")?
            .ok_or_else(|| anyhow::anyhow!("feature/beta workspace missing"))?;
        assert_eq!(first_after.status, WorkspaceStatus::Idle);
        assert_eq!(second_after.status, WorkspaceStatus::Active);
        assert!(second_after.last_activated_at.is_some());
        Ok(())
    }

    #[test]
    fn activate_workspace_main_branch_stays_unlinked() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        crate::state_db::ensure_project_database(&ship_dir)?;

        let workspace = activate_workspace(&ship_dir, "main")?;
        assert_eq!(workspace.status, WorkspaceStatus::Active);
        assert!(workspace.feature_id.is_none());
        assert!(workspace.spec_id.is_none());
        assert!(get_branch_link(&ship_dir, "main")?.is_none());
        Ok(())
    }

    #[test]
    fn delete_workspace_removes_workspace_links_and_sessions() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        crate::state_db::ensure_project_database(&ship_dir)?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/delete-me".to_string(),
                status: Some(WorkspaceStatus::Active),
                feature_id: Some("feat-delete".to_string()),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        let now = Utc::now().to_rfc3339();
        insert_workspace_session_db(
            &ship_dir,
            &WorkspaceSessionDb {
                id: "session-delete-me".to_string(),
                workspace_id: workspace.id.clone(),
                workspace_branch: workspace.branch.clone(),
                status: WorkspaceSessionStatus::Ended.to_string(),
                started_at: now.clone(),
                ended_at: Some(now.clone()),
                mode_id: None,
                primary_provider: None,
                goal: None,
                summary: Some("done".to_string()),
                updated_feature_ids: Vec::new(),
                updated_spec_ids: Vec::new(),
                compiled_at: None,
                compile_error: None,
                config_generation_at_start: None,
                created_at: now.clone(),
                updated_at: now,
            },
        )?;
        assert_eq!(list_workspace_sessions(&ship_dir, None, 10)?.len(), 1);

        delete_workspace(&ship_dir, "feature/delete-me")?;

        assert!(get_workspace(&ship_dir, "feature/delete-me")?.is_none());
        assert!(get_branch_link(&ship_dir, "feature/delete-me")?.is_none());
        assert!(list_workspace_sessions(&ship_dir, None, 10)?.is_empty());
        Ok(())
    }

    #[test]
    fn create_workspace_clears_worktree_metadata_when_switched_to_non_worktree() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        crate::state_db::ensure_project_database(&ship_dir)?;

        let branch = "feature/worktree-cleanup";
        let initial = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                is_worktree: Some(true),
                worktree_path: Some("../worktrees/worktree-cleanup".to_string()),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        assert!(initial.is_worktree);
        assert_eq!(
            initial.worktree_path.as_deref(),
            Some("../worktrees/worktree-cleanup")
        );

        let updated = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                is_worktree: Some(false),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        assert!(!updated.is_worktree);
        assert!(updated.worktree_path.is_none());

        let stored = get_workspace(&ship_dir, branch)?
            .ok_or_else(|| anyhow::anyhow!("workspace missing after update"))?;
        assert!(!stored.is_worktree);
        assert!(stored.worktree_path.is_none());
        Ok(())
    }

    #[test]
    fn create_workspace_requires_path_for_worktree_records() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir)?;
        crate::state_db::ensure_project_database(&ship_dir)?;

        let err = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/missing-path".to_string(),
                is_worktree: Some(true),
                ..CreateWorkspaceRequest::default()
            },
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("Worktree workspace requires a worktree path")
        );
        Ok(())
    }

    #[test]
    fn workspace_session_start_and_end_happy_path() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/session-flow".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let started = start_workspace_session(
            &ship_dir,
            "feature/session-flow",
            Some("Implement parser".to_string()),
            None,
            None,
        )?;
        assert_eq!(started.status, WorkspaceSessionStatus::Active);
        assert_eq!(started.goal.as_deref(), Some("Implement parser"));
        assert_eq!(started.primary_provider.as_deref(), Some("claude"));
        assert!(started.compiled_at.is_some());
        assert!(started.compile_error.is_none());
        assert!(started.config_generation_at_start.is_some());
        assert!(!started.stale_context);
        assert!(started.ended_at.is_none());

        let active = get_active_workspace_session(&ship_dir, "feature/session-flow")?
            .ok_or_else(|| anyhow::anyhow!("active session not found"))?;
        assert_eq!(active.id, started.id);
        assert!(!active.stale_context);

        let ended = end_workspace_session(
            &ship_dir,
            "feature/session-flow",
            EndWorkspaceSessionRequest {
                summary: Some("Implemented parser + tests".to_string()),
                updated_feature_ids: vec!["feat-parser".to_string()],
                updated_spec_ids: vec!["spec-parser".to_string()],
            },
        )?;
        assert_eq!(ended.status, WorkspaceSessionStatus::Ended);
        assert!(ended.ended_at.is_some());
        assert_eq!(ended.summary.as_deref(), Some("Implemented parser + tests"));
        assert_eq!(ended.updated_feature_ids, vec!["feat-parser".to_string()]);
        assert_eq!(ended.updated_spec_ids, vec!["spec-parser".to_string()]);
        assert!(get_active_workspace_session(&ship_dir, "feature/session-flow")?.is_none());
        Ok(())
    }

    #[test]
    fn workspace_session_start_rejects_duplicate_active_sessions() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/session-dupe".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let first = start_workspace_session(
            &ship_dir,
            "feature/session-dupe",
            Some("one".into()),
            None,
            None,
        )?;
        let err = start_workspace_session(
            &ship_dir,
            "feature/session-dupe",
            Some("two".into()),
            None,
            None,
        )
        .expect_err("second active session should be rejected");

        assert!(err.to_string().contains(&first.id));
        Ok(())
    }

    #[test]
    fn workspace_session_list_filters_by_branch_workspace() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/a".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/b".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let a = start_workspace_session(&ship_dir, "feature/a", None, None, None)?;
        end_workspace_session(
            &ship_dir,
            "feature/a",
            EndWorkspaceSessionRequest::default(),
        )?;
        let b = start_workspace_session(&ship_dir, "feature/b", None, None, None)?;

        let all = list_workspace_sessions(&ship_dir, None, 10)?;
        assert!(all.iter().any(|session| session.id == a.id));
        assert!(all.iter().any(|session| session.id == b.id));

        let only_a = list_workspace_sessions(&ship_dir, Some("feature/a"), 10)?;
        assert!(
            only_a
                .iter()
                .all(|session| session.workspace_branch == "feature/a")
        );
        assert_eq!(only_a.len(), 1);
        assert_eq!(only_a[0].id, a.id);
        Ok(())
    }

    #[test]
    fn workspace_session_start_allows_explicit_primary_provider() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-ok".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/provider-ok",
            Some("Pin provider".to_string()),
            None,
            Some("claude".to_string()),
        )?;
        assert_eq!(session.primary_provider.as_deref(), Some("claude"));
        assert!(session.compiled_at.is_some());
        Ok(())
    }

    #[test]
    fn workspace_session_start_rejects_provider_outside_allowed_targets() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-deny".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let err = start_workspace_session(
            &ship_dir,
            "feature/provider-deny",
            None,
            None,
            Some("gemini".to_string()),
        )
        .expect_err("provider outside allowed targets should be rejected");

        assert!(
            err.to_string()
                .contains("Provider 'gemini' is not allowed for workspace")
        );
        Ok(())
    }

    #[test]
    fn create_workspace_rejects_unknown_active_mode() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let err = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/no-mode".to_string(),
                active_mode: Some("ghost".to_string()),
                ..Default::default()
            },
        )
        .expect_err("expected invalid mode to be rejected");

        assert!(err.to_string().contains("Mode 'ghost' not found"));
        Ok(())
    }

    #[test]
    fn set_workspace_active_mode_updates_and_clears_override() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::ProjectConfig::default();
        config.modes = vec![crate::config::ModeConfig {
            id: "planning".to_string(),
            name: "Planning".to_string(),
            target_agents: vec!["codex".to_string()],
            ..Default::default()
        }];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/mode-override".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let updated =
            set_workspace_active_mode(&ship_dir, "feature/mode-override", Some("planning"))?;
        assert_eq!(updated.active_mode.as_deref(), Some("planning"));
        assert!(updated.config_generation >= 1);
        assert!(updated.compiled_at.is_some());
        assert!(updated.compile_error.is_none());
        assert!(tmp.path().join(".codex").join("config.toml").exists());

        let cleared = set_workspace_active_mode(&ship_dir, "feature/mode-override", None)?;
        assert!(cleared.active_mode.is_none());
        assert!(cleared.config_generation > updated.config_generation);
        Ok(())
    }

    #[test]
    fn provider_matrix_prefers_workspace_provider_overrides() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::ProjectConfig::default();
        config.providers = vec!["claude".to_string()];
        config.modes = vec![crate::config::ModeConfig {
            id: "planning".to_string(),
            name: "Planning".to_string(),
            target_agents: vec!["gemini".to_string()],
            ..Default::default()
        }];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-matrix".to_string(),
                providers: Some(vec!["codex".to_string()]),
                active_mode: Some("planning".to_string()),
                ..Default::default()
            },
        )?;

        let matrix = get_workspace_provider_matrix(&ship_dir, "feature/provider-matrix", None)?;
        assert_eq!(matrix.source, "workspace");
        assert_eq!(matrix.allowed_providers, vec!["codex".to_string()]);
        assert!(matrix.resolution_error.is_none());
        Ok(())
    }

    #[test]
    fn provider_matrix_reports_resolution_error_for_invalid_candidates() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-invalid".to_string(),
                providers: Some(vec!["ghost-provider".to_string()]),
                ..Default::default()
            },
        )?;

        let matrix = get_workspace_provider_matrix(&ship_dir, "feature/provider-invalid", None)?;
        assert!(matrix.allowed_providers.is_empty());
        assert!(matrix.resolution_error.is_some());
        Ok(())
    }

    #[test]
    fn repair_workspace_dry_run_reports_missing_provider_config() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/repair-dry-run".to_string(),
                providers: Some(vec!["codex".to_string()]),
                ..Default::default()
            },
        )?;

        let report = repair_workspace(&ship_dir, "feature/repair-dry-run", true)?;
        assert_eq!(report.workspace_branch, "feature/repair-dry-run");
        assert!(report.dry_run);
        assert!(report.needs_recompile);
        assert!(report.missing_provider_configs.iter().any(|p| p == "codex"));
        assert!(!report.reapplied_compile);
        Ok(())
    }

    #[test]
    fn repair_workspace_apply_recompiles_active_workspace() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let created = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/repair-apply".to_string(),
                providers: Some(vec!["codex".to_string()]),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;
        assert_eq!(created.status, WorkspaceStatus::Active);

        let codex_config = tmp.path().join(".codex").join("config.toml");
        if codex_config.exists() {
            std::fs::remove_file(&codex_config)?;
        }

        let report = repair_workspace(&ship_dir, "feature/repair-apply", false)?;
        assert!(report.reapplied_compile);
        assert!(!report.needs_recompile);
        assert!(report.missing_provider_configs.is_empty());
        assert!(codex_config.exists());
        Ok(())
    }

    #[test]
    fn repair_workspace_apply_on_idle_workspace_reports_activation_action() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/repair-idle".to_string(),
                providers: Some(vec!["codex".to_string()]),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;
        transition_workspace_status(&ship_dir, "feature/repair-idle", WorkspaceStatus::Idle)?;

        let report = repair_workspace(&ship_dir, "feature/repair-idle", false)?;
        assert!(report.needs_recompile);
        assert!(!report.reapplied_compile);
        assert!(
            report
                .actions
                .iter()
                .any(|action| action.contains("activate workspace"))
        );
        Ok(())
    }

    #[test]
    fn start_workspace_session_errors_when_no_valid_providers_resolve() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/no-provider".to_string(),
                providers: Some(vec!["ghost-provider".to_string()]),
                ..Default::default()
            },
        )?;

        let err = start_workspace_session(
            &ship_dir,
            "feature/no-provider",
            Some("should fail".to_string()),
            None,
            None,
        )
        .expect_err("session start should fail when no providers resolve");
        assert!(
            err.to_string()
                .contains("No valid providers resolved for workspace"),
            "unexpected error: {}",
            err
        );
        Ok(())
    }

    #[test]
    fn activate_workspace_compiles_and_bumps_generation() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let created = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/activation-compile".to_string(),
                ..Default::default()
            },
        )?;
        assert_eq!(created.config_generation, 0);

        let activated = activate_workspace(&ship_dir, "feature/activation-compile")?;
        assert_eq!(activated.status, WorkspaceStatus::Active);
        assert!(activated.config_generation >= 1);
        assert!(activated.compiled_at.is_some());
        assert!(activated.compile_error.is_none());
        Ok(())
    }

    #[test]
    fn session_stale_context_turns_true_after_recompile() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::ProjectConfig::default();
        config.providers = vec!["claude".to_string()];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/stale-session".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let started = start_workspace_session(
            &ship_dir,
            "feature/stale-session",
            Some("test stale".to_string()),
            None,
            None,
        )?;
        let start_generation = started
            .config_generation_at_start
            .ok_or_else(|| anyhow::anyhow!("missing config generation on start"))?;

        let workspace_after_start = get_workspace(&ship_dir, "feature/stale-session")?
            .ok_or_else(|| anyhow::anyhow!("workspace missing"))?;
        assert_eq!(workspace_after_start.config_generation, start_generation);

        let _ = set_workspace_active_mode(&ship_dir, "feature/stale-session", None)?;

        let active = get_active_workspace_session(&ship_dir, "feature/stale-session")?
            .ok_or_else(|| anyhow::anyhow!("active session missing"))?;
        assert!(active.stale_context);

        let sessions = list_workspace_sessions(&ship_dir, Some("feature/stale-session"), 10)?;
        assert!(!sessions.is_empty());
        assert!(sessions[0].stale_context);
        Ok(())
    }
}
