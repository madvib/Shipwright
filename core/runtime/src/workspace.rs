use crate::agents::config::{
    AgentConfig, FeatureAgentConfig, resolve_agent_config_with_mode_override,
};
use crate::events::{EventAction, EventEntity, append_event};
use crate::project::{get_global_dir, project_slug_from_ship_dir, sanitize_file_name};
use crate::state_db::{
    WorkspaceSessionDb, WorkspaceUpsert, clear_branch_link, delete_workspace_db,
    get_active_workspace_session_db, get_workspace_db, get_workspace_session_db,
    get_workspace_session_record_db, insert_workspace_session_db,
    insert_workspace_session_record_db, list_workspace_sessions_db, list_workspaces_db,
    set_branch_link, update_workspace_session_db, upsert_workspace_db,
};
use crate::state_db::{
    get_branch_link, get_feature_agent_config, get_feature_by_branch_links, get_feature_links,
};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

// ─── Data types ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ShipWorkspaceKind {
    #[default]
    Feature,
    Patch,
    Service,
}

impl std::fmt::Display for ShipWorkspaceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShipWorkspaceKind::Feature => write!(f, "feature"),
            ShipWorkspaceKind::Patch => write!(f, "patch"),
            ShipWorkspaceKind::Service => write!(f, "service"),
        }
    }
}

impl std::str::FromStr for ShipWorkspaceKind {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        parse_workspace_kind(value)
            .ok_or_else(|| anyhow::anyhow!("Invalid workspace type: {}", value))
    }
}

fn parse_workspace_kind(value: &str) -> Option<ShipWorkspaceKind> {
    match value.trim().to_lowercase().as_str() {
        "feature" => Some(ShipWorkspaceKind::Feature),
        "patch" => Some(ShipWorkspaceKind::Patch),
        "service" => Some(ShipWorkspaceKind::Service),
        _ => None,
    }
}

fn parse_workspace_type_required(value: &str) -> Result<ShipWorkspaceKind> {
    parse_workspace_kind(value).ok_or_else(|| {
        anyhow!(
            "Invalid workspace type '{}'; expected one of: feature, patch, service",
            value
        )
    })
}

fn parse_workspace_status(value: &str) -> Option<WorkspaceStatus> {
    match value.trim().to_lowercase().as_str() {
        "active" => Some(WorkspaceStatus::Active),
        "archived" => Some(WorkspaceStatus::Archived),
        _ => None,
    }
}

fn parse_workspace_status_required(value: &str) -> Result<WorkspaceStatus> {
    parse_workspace_status(value).ok_or_else(|| {
        anyhow!(
            "Invalid workspace status '{}'; expected one of: active, archived",
            value
        )
    })
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceStatus {
    #[default]
    Active,
    Archived,
}

impl std::fmt::Display for WorkspaceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceStatus::Active => write!(f, "active"),
            WorkspaceStatus::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for WorkspaceStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "active" => Ok(WorkspaceStatus::Active),
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
    pub workspace_type: ShipWorkspaceKind,
    #[serde(default)]
    pub status: WorkspaceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_mode: Option<String>,
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
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

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Environment {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    pub permissions_json: String,
    #[serde(default)]
    pub providers: Vec<String>,
    pub hooks_json: String,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessStatus {
    #[default]
    Running,
    Paused,
    Completed,
    Error,
    Interrupted,
}

impl std::fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessStatus::Running => write!(f, "running"),
            ProcessStatus::Paused => write!(f, "paused"),
            ProcessStatus::Completed => write!(f, "completed"),
            ProcessStatus::Error => write!(f, "error"),
            ProcessStatus::Interrupted => write!(f, "interrupted"),
        }
    }
}

impl std::str::FromStr for ProcessStatus {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "running" => Ok(ProcessStatus::Running),
            "paused" => Ok(ProcessStatus::Paused),
            "completed" => Ok(ProcessStatus::Completed),
            "error" => Ok(ProcessStatus::Error),
            "interrupted" => Ok(ProcessStatus::Interrupted),
            _ => Err(anyhow::anyhow!("Invalid process status: {}", value)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Process {
    pub id: String,
    pub workspace_id: String,
    pub status: ProcessStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_record_id: Option<String>,
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

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WorkspaceSessionRecord {
    pub id: String,
    pub session_id: String,
    pub workspace_id: String,
    pub workspace_branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default)]
    pub updated_feature_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct EndWorkspaceSessionRequest {
    pub summary: Option<String>,
    pub updated_feature_ids: Vec<String>,
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
    pub workspace_type: Option<ShipWorkspaceKind>,
    pub status: Option<WorkspaceStatus>,
    pub environment_id: Option<String>,
    pub feature_id: Option<String>,
    pub target_id: Option<String>,
    pub active_mode: Option<String>,
    pub providers: Option<Vec<String>>,
    pub mcp_servers: Option<Vec<String>>,
    pub skills: Option<Vec<String>>,
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
        session_record_id: None,
        compiled_at: parse_datetime_opt(row.compiled_at),
        compile_error: row.compile_error,
        config_generation_at_start: row.config_generation_at_start,
        stale_context: false,
        created_at: parse_datetime(&row.created_at),
        updated_at: parse_datetime(&row.updated_at),
    }
}

fn hydrate_workspace_session_record(
    row: crate::state_db::WorkspaceSessionRecordDb,
) -> WorkspaceSessionRecord {
    WorkspaceSessionRecord {
        id: row.id,
        session_id: row.session_id,
        workspace_id: row.workspace_id,
        workspace_branch: row.workspace_branch,
        summary: row.summary,
        updated_feature_ids: row.updated_feature_ids,
        created_at: parse_datetime(&row.created_at),
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

fn annotate_session_record(ship_dir: &Path, session: &mut WorkspaceSession) -> Result<()> {
    session.session_record_id =
        get_workspace_session_record_db(ship_dir, &session.id)?.map(|record| record.id);
    Ok(())
}

fn infer_workspace_type(branch: &str, feature_id: Option<&str>) -> ShipWorkspaceKind {
    if feature_id.is_some() {
        return ShipWorkspaceKind::Feature;
    }
    if branch.starts_with("patch/") {
        return ShipWorkspaceKind::Patch;
    }
    ShipWorkspaceKind::Feature
}

fn workspace_id_from_branch(branch: &str) -> String {
    sanitize_file_name(branch)
}

fn default_project_worktree_root(ship_dir: &Path) -> PathBuf {
    ship_dir.join("worktrees")
}

fn default_global_worktree_root(ship_dir: &Path) -> Option<PathBuf> {
    let slug = project_slug_from_ship_dir(ship_dir);
    let global_dir = get_global_dir().ok()?;
    Some(global_dir.join("projects").join(slug).join("worktrees"))
}

fn default_worktree_path(ship_dir: &Path, branch: &str) -> Option<String> {
    let branch_token = sanitize_file_name(branch);

    let project_root = default_project_worktree_root(ship_dir);
    if fs::create_dir_all(&project_root).is_ok() {
        return Some(
            project_root
                .join(&branch_token)
                .to_string_lossy()
                .to_string(),
        );
    }

    if let Some(global_root) = default_global_worktree_root(ship_dir)
        && fs::create_dir_all(&global_root).is_ok()
    {
        return Some(global_root.join(branch_token).to_string_lossy().to_string());
    }

    None
}

fn session_artifacts_root(ship_dir: &Path) -> Option<PathBuf> {
    let slug = project_slug_from_ship_dir(ship_dir);
    let global_dir = get_global_dir().ok()?;
    Some(global_dir.join("projects").join(slug).join("sessions"))
}

fn session_artifacts_dir(ship_dir: &Path, session_id: &str) -> Option<PathBuf> {
    let root = session_artifacts_root(ship_dir)?;
    Some(root.join(sanitize_file_name(session_id)))
}

fn persist_session_artifact(
    ship_dir: &Path,
    session: &WorkspaceSession,
    phase: &str,
) -> Result<()> {
    let Some(session_dir) = session_artifacts_dir(ship_dir, &session.id) else {
        return Ok(());
    };
    fs::create_dir_all(&session_dir)?;

    let snapshot_path = session_dir.join("session.json");
    let snapshot = serde_json::to_string_pretty(session)?;
    fs::write(snapshot_path, snapshot)?;

    let timeline_path = session_dir.join("timeline.ndjson");
    let entry = serde_json::json!({
        "phase": phase,
        "timestamp": Utc::now().to_rfc3339(),
        "session_id": session.id,
        "branch": session.workspace_branch,
        "status": session.status.to_string(),
        "provider": session.primary_provider,
        "goal": session.goal,
        "summary": session.summary,
        "updated_feature_ids": session.updated_feature_ids,
        "session_record_id": session.session_record_id,
    });
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(timeline_path)?;
    writeln!(file, "{}", entry)?;

    Ok(())
}

fn append_session_note_artifact(ship_dir: &Path, session_id: &str, note: &str) -> Result<()> {
    let Some(session_dir) = session_artifacts_dir(ship_dir, session_id) else {
        return Ok(());
    };
    fs::create_dir_all(&session_dir)?;

    let notes_path = session_dir.join("notes.ndjson");
    let entry = serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "note": note,
    });
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(notes_path)?;
    writeln!(file, "{}", entry)?;

    Ok(())
}

fn run_post_session_hooks(ship_dir: &Path, session: &WorkspaceSession) -> Result<()> {
    let effective = crate::config::get_effective_config(Some(ship_dir.to_path_buf()))?;
    let hooks: Vec<_> = effective
        .hooks
        .into_iter()
        .filter(|hook| {
            hook.trigger == crate::config::HookTrigger::SessionEnd
                || hook.trigger == crate::config::HookTrigger::Stop
        })
        .collect();

    for hook in hooks {
        let command = hook.command.trim();
        if command.is_empty() {
            continue;
        }

        let mut process = if cfg!(windows) {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", command]);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.args(["-lc", command]);
            cmd
        };

        let output = process
            .current_dir(ship_dir)
            .env("SHIP_SESSION_ID", &session.id)
            .env("SHIP_SESSION_BRANCH", &session.workspace_branch)
            .output();

        match output {
            Ok(out) => {
                if !out.status.success() {
                    eprintln!(
                        "Post-session hook '{}' failed with status {:?}",
                        hook.id,
                        out.status.code()
                    );
                }
            }
            Err(error) => {
                eprintln!(
                    "Failed to execute post-session hook '{}': {}",
                    hook.id, error
                );
            }
        }
    }

    Ok(())
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

fn normalize_nonempty_id_list(ids: &[String]) -> Vec<String> {
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

fn merge_feature_agent_with_workspace(
    feature_agent: Option<FeatureAgentConfig>,
    workspace: &Workspace,
) -> Option<FeatureAgentConfig> {
    let mut has_override = feature_agent.is_some();
    let mut merged = feature_agent.unwrap_or_default();

    let workspace_providers = normalize_nonempty_id_list(&workspace.providers);
    if !workspace_providers.is_empty() {
        merged.providers = workspace_providers;
        has_override = true;
    }

    let workspace_mcp_servers = normalize_nonempty_id_list(&workspace.mcp_servers);
    if !workspace_mcp_servers.is_empty() {
        merged.mcp_servers = workspace_mcp_servers;
        has_override = true;
    }

    let workspace_skills = normalize_nonempty_id_list(&workspace.skills);
    if !workspace_skills.is_empty() {
        merged.skills = workspace_skills;
        has_override = true;
    }

    if has_override { Some(merged) } else { None }
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
    let resolved = resolve_workspace_agent_config(ship_dir, workspace, mode_id)?;
    if resolved.providers.is_empty() {
        return Err(anyhow!(
            "No valid providers resolved for workspace '{}'",
            workspace.branch
        ));
    }
    Ok(resolved.providers)
}

fn resolve_workspace_agent_config(
    ship_dir: &Path,
    workspace: &Workspace,
    mode_id: Option<&str>,
) -> Result<AgentConfig> {
    let mode_id = mode_id
        .and_then(normalize_mode_ref)
        .or_else(|| {
            workspace
                .active_mode
                .as_deref()
                .and_then(normalize_mode_ref)
        })
        .map(|mode| mode.to_string());

    let feature_agent = workspace
        .feature_id
        .as_deref()
        .map(|feature_id| get_feature_agent_config(ship_dir, feature_id))
        .transpose()?
        .flatten();
    let workspace_agent = merge_feature_agent_with_workspace(feature_agent, workspace);

    resolve_agent_config_with_mode_override(ship_dir, workspace_agent.as_ref(), mode_id.as_deref())
}

fn build_workspace_provider_matrix(
    ship_dir: &Path,
    workspace: &Workspace,
    mode_id: Option<&str>,
) -> Result<WorkspaceProviderMatrix> {
    let resolved_mode_id = mode_id.and_then(normalize_mode_ref).or_else(|| {
        workspace
            .active_mode
            .as_deref()
            .and_then(normalize_mode_ref)
    });
    let resolved = resolve_workspace_agent_config(ship_dir, workspace, mode_id)?;
    let providers = resolved.providers;

    let supported_providers = crate::agents::export::list_providers(ship_dir)?
        .into_iter()
        .map(|provider| provider.id)
        .collect::<Vec<_>>();

    let resolution_error = if providers.is_empty() {
        Some(format!(
            "No valid providers resolved for workspace '{}'",
            workspace.branch
        ))
    } else {
        None
    };

    Ok(WorkspaceProviderMatrix {
        workspace_branch: workspace.branch.clone(),
        mode_id: resolved_mode_id,
        source: if !workspace.providers.is_empty() {
            "workspace".to_string()
        } else if workspace.feature_id.is_some() {
            "feature".to_string()
        } else if resolved.active_mode.is_some() {
            "mode/config".to_string()
        } else {
            "config/default".to_string()
        },
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

fn stable_hash(value: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn compute_workspace_context_hash(
    ship_dir: &Path,
    workspace: &Workspace,
    resolved_agent: &AgentConfig,
) -> Result<String> {
    let config = crate::config::get_effective_config(Some(ship_dir.to_path_buf()))?;
    let config_hash = stable_hash(&toml::to_string(&config)?);

    let permissions_hash = stable_hash(&toml::to_string(&resolved_agent.permissions)?);

    let mut skill_hashes = resolved_agent
        .skills
        .iter()
        .map(|skill| (skill.id.clone(), stable_hash(&skill.content)))
        .collect::<Vec<_>>();
    skill_hashes.sort_by(|left, right| left.0.cmp(&right.0));

    let mut rule_hashes = resolved_agent
        .rules
        .iter()
        .map(|rule| (rule.file_name.clone(), stable_hash(&rule.content)))
        .collect::<Vec<_>>();
    rule_hashes.sort_by(|left, right| left.0.cmp(&right.0));

    let mut mcp_hashes = resolved_agent
        .mcp_servers
        .iter()
        .map(|server| -> Result<(String, String)> {
            let digest = stable_hash(&toml::to_string(&server)?);
            Ok((server.id.clone(), digest))
        })
        .collect::<Result<Vec<_>>>()?;
    mcp_hashes.sort_by(|left, right| left.0.cmp(&right.0));

    let mut normalized_providers = resolved_agent
        .providers
        .iter()
        .map(|provider| provider.trim().to_ascii_lowercase())
        .filter(|provider| !provider.is_empty())
        .collect::<Vec<_>>();
    normalized_providers.sort();
    normalized_providers.dedup();

    let fingerprint = serde_json::json!({
        "workspace": {
            "branch": workspace.branch,
            "workspace_type": workspace.workspace_type.to_string(),
            "environment_id": workspace.environment_id,
            "feature_id": workspace.feature_id,
            "target_id": workspace.target_id,
            "mode_id": resolved_agent.active_mode,
            "provider_overrides": normalize_nonempty_id_list(&workspace.providers),
            "mcp_server_overrides": normalize_nonempty_id_list(&workspace.mcp_servers),
            "skill_overrides": normalize_nonempty_id_list(&workspace.skills),
        },
        "providers": normalized_providers,
        "model": resolved_agent.model,
        "config_hash": config_hash,
        "permissions_hash": permissions_hash,
        "skill_hashes": skill_hashes,
        "rule_hashes": rule_hashes,
        "mcp_hashes": mcp_hashes,
    });
    Ok(stable_hash(&serde_json::to_string(&fingerprint)?))
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
    let resolved_agent =
        match resolve_workspace_agent_config(ship_dir, workspace, mode_id.as_deref()) {
            Ok(agent) => agent,
            Err(error) => {
                let now = Utc::now();
                workspace.compiled_at = Some(now);
                workspace.compile_error = Some(error.to_string());
                workspace.resolved_at = now;
                upsert_workspace(ship_dir, workspace)?;
                return Err(error);
            }
        };
    let providers = resolved_agent.providers.clone();
    if providers.is_empty() {
        let error = anyhow!(
            "No valid providers resolved for workspace '{}'",
            workspace.branch
        );
        let now = Utc::now();
        workspace.compiled_at = Some(now);
        workspace.compile_error = Some(error.to_string());
        workspace.resolved_at = now;
        upsert_workspace(ship_dir, workspace)?;
        return Err(error);
    }

    let mcp_server_filter = resolved_agent
        .mcp_servers
        .iter()
        .map(|server| server.id.clone())
        .collect::<Vec<_>>();
    let skill_filter = resolved_agent
        .skills
        .iter()
        .map(|skill| skill.id.clone())
        .collect::<Vec<_>>();

    let now = Utc::now();
    let next_context_hash =
        match compute_workspace_context_hash(ship_dir, workspace, &resolved_agent) {
            Ok(hash) => hash,
            Err(error) => {
                workspace.compiled_at = Some(now);
                workspace.compile_error = Some(error.to_string());
                workspace.resolved_at = now;
                upsert_workspace(ship_dir, workspace)?;
                return Err(error);
            }
        };

    let context_root = resolve_workspace_context_root(ship_dir, workspace);
    for provider in &providers {
        if let Err(error) =
            crate::agents::export::export_to_filtered_with_mode_override_and_skills_at_root(
                ship_dir.to_path_buf(),
                provider,
                Some(mcp_server_filter.as_slice()),
                Some(skill_filter.as_slice()),
                mode_id.as_deref(),
                &context_root,
            )
        {
            let contextual = error.context(format!(
                "Failed to compile provider '{}' for workspace '{}'",
                provider, workspace.branch
            ));
            workspace.compiled_at = Some(now);
            workspace.compile_error = Some(contextual.to_string());
            workspace.context_hash = Some(next_context_hash.clone());
            workspace.resolved_at = now;
            upsert_workspace(ship_dir, workspace)?;
            return Err(contextual);
        }
    }

    workspace.config_generation = workspace.config_generation.saturating_add(1);
    workspace.compiled_at = Some(now);
    workspace.compile_error = None;
    workspace.context_hash = Some(next_context_hash);
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
        workspace_type: ShipWorkspaceKind::Feature,
        status: WorkspaceStatus::Active,
        environment_id: None,
        feature_id: None,
        target_id: None,
        active_mode: None,
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

fn hydrate_from_branch_links(
    ship_dir: &Path,
    branch: &str,
    workspace: &mut Workspace,
) -> Result<()> {
    if let Some((link_type, link_id)) = get_branch_link(ship_dir, branch)? {
        match link_type.as_str() {
            "feature" => {
                workspace.feature_id = Some(link_id.clone());
                if let Some(target_id) = get_feature_links(ship_dir, &link_id)? {
                    workspace.target_id = target_id;
                }
            }
            "target" | "release" => {
                workspace.target_id = Some(link_id);
            }
            _ => {}
        }
    }

    // Git branch linkage also lives on feature rows; hydrate from there when
    // no explicit branch_context mapping is present.
    if workspace.feature_id.is_none()
        && let Some((feature_id, target_id)) = get_feature_by_branch_links(ship_dir, branch)?
    {
        workspace.feature_id = Some(feature_id);
        if workspace.target_id.is_none() {
            workspace.target_id = target_id;
        }
    }

    Ok(())
}

fn hydrate_from_feature_links(ship_dir: &Path, workspace: &mut Workspace) -> Result<()> {
    if let Some(feature_id) = workspace.feature_id.clone()
        && let Some(target_id) = get_feature_links(ship_dir, &feature_id)?
    {
        if workspace.target_id.is_none() {
            workspace.target_id = target_id;
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

pub fn get_workspace(ship_dir: &Path, branch: &str) -> Result<Option<Workspace>> {
    let row = get_workspace_db(ship_dir, branch)?;
    let Some((
        id,
        workspace_type,
        status,
        environment_id,
        feature_id,
        target_id,
        active_mode,
        providers,
        mcp_servers,
        skills,
        resolved_at,
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
        environment_id,
        feature_id,
        target_id,
        active_mode,
        providers,
        mcp_servers,
        skills,
        resolved_at: parse_datetime(&resolved_at),
        last_activated_at: parse_datetime_opt(last_activated_at),
        is_worktree,
        worktree_path,
        context_hash,
        config_generation,
        compiled_at: parse_datetime_opt(compiled_at),
        compile_error,
    }))
}

pub fn list_workspaces(ship_dir: &Path) -> Result<Vec<Workspace>> {
    let rows = list_workspaces_db(ship_dir)?;
    let mut workspaces = Vec::with_capacity(rows.len());
    for (
        branch,
        id,
        workspace_type,
        status,
        environment_id,
        feature_id,
        target_id,
        active_mode,
        providers,
        mcp_servers,
        skills,
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
        let parsed_workspace_type = parse_workspace_type_required(&workspace_type)
            .map_err(|err| anyhow!("Workspace '{}' has invalid type value: {}", branch, err))?;
        let parsed_status = parse_workspace_status_required(&status)
            .map_err(|err| anyhow!("Workspace '{}' has invalid status value: {}", branch, err))?;

        workspaces.push(Workspace {
            id,
            branch,
            workspace_type: parsed_workspace_type,
            status: parsed_status,
            environment_id,
            feature_id,
            target_id,
            active_mode,
            providers,
            mcp_servers,
            skills,
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
            environment_id: workspace.environment_id.as_deref(),
            feature_id: workspace.feature_id.as_deref(),
            target_id: workspace.target_id.as_deref(),
            active_mode: workspace.active_mode.as_deref(),
            providers: &workspace.providers,
            mcp_servers: &workspace.mcp_servers,
            skills: &workspace.skills,
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
            let _ = annotate_session_record(ship_dir, &mut session);
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
        annotate_session_record(ship_dir, session)?;
    }
    Ok(sessions)
}

pub fn get_workspace_session_record(
    ship_dir: &Path,
    session_id: &str,
) -> Result<Option<WorkspaceSessionRecord>> {
    let session_id = session_id.trim();
    if session_id.is_empty() {
        return Err(anyhow!("Session ID cannot be empty"));
    }
    Ok(
        get_workspace_session_record_db(ship_dir, session_id)?
            .map(hydrate_workspace_session_record),
    )
}

pub fn record_workspace_session_progress(ship_dir: &Path, branch: &str, note: &str) -> Result<()> {
    let branch = ensure_branch_key(branch)?;
    let workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow::anyhow!("Workspace not found for branch '{}'", branch))?;
    let active = get_active_workspace_session_db(ship_dir, &workspace.id)?
        .ok_or_else(|| anyhow::anyhow!("No active workspace session for '{}'", workspace.branch))?;

    let normalized_note = note.trim();
    if normalized_note.is_empty() {
        return Err(anyhow!("Session note cannot be empty"));
    }

    append_event(
        ship_dir,
        "agent",
        EventEntity::Session,
        EventAction::Note,
        active.id.clone(),
        Some(format!("branch={} {}", workspace.branch, normalized_note)),
    )?;
    if let Err(error) = append_session_note_artifact(ship_dir, &active.id, normalized_note) {
        eprintln!("Failed to persist session note artifact: {}", error);
    }
    Ok(())
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
        let mut existing = hydrate_workspace_session(active);
        let _ = annotate_session_record(ship_dir, &mut existing);
        if let Err(error) = persist_session_artifact(ship_dir, &existing, "attach") {
            eprintln!("Failed to persist attached session artifact: {}", error);
        }
        return Ok(existing);
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

    compile_workspace_context(ship_dir, &mut workspace, mode_id.as_deref())?;

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
        compiled_at: workspace.compiled_at.as_ref().map(|ts| ts.to_rfc3339()),
        compile_error: workspace.compile_error.clone(),
        config_generation_at_start: Some(workspace.config_generation),
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
    };
    insert_workspace_session_db(ship_dir, &session)?;
    let created = get_workspace_session_db(ship_dir, &session.id)?
        .ok_or_else(|| anyhow::anyhow!("Failed to load created workspace session"))?;
    let started = hydrate_workspace_session(created);

    let mut details = vec![format!("branch={}", started.workspace_branch)];
    if let Some(provider) = started.primary_provider.as_deref() {
        details.push(format!("provider={provider}"));
    }
    if let Some(mode_id) = started.mode_id.as_deref() {
        details.push(format!("mode={mode_id}"));
    }
    if let Some(goal) = started.goal.as_deref() {
        details.push(format!("goal={goal}"));
    }
    append_event(
        ship_dir,
        "ship",
        EventEntity::Session,
        EventAction::Start,
        started.id.clone(),
        Some(details.join(" ")),
    )?;

    if let Err(error) = persist_session_artifact(ship_dir, &started, "start") {
        eprintln!("Failed to persist session artifact on start: {}", error);
    }

    Ok(started)
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
    active.updated_at = now;

    update_workspace_session_db(ship_dir, &active)?;

    let ended = get_workspace_session_db(ship_dir, &active.id)?
        .ok_or_else(|| anyhow::anyhow!("Failed to load ended workspace session"))?;
    let ended = hydrate_workspace_session(ended);

    let mut details = vec![format!("branch={}", ended.workspace_branch)];
    if let Some(summary) = ended.summary.as_deref() {
        details.push(format!("summary={summary}"));
    }
    if !ended.updated_feature_ids.is_empty() {
        details.push(format!(
            "updated_features={}",
            ended.updated_feature_ids.join(",")
        ));
    }
    append_event(
        ship_dir,
        "ship",
        EventEntity::Session,
        EventAction::Stop,
        ended.id.clone(),
        Some(details.join(" ")),
    )?;

    let record = crate::state_db::WorkspaceSessionRecordDb {
        id: crate::gen_nanoid(),
        session_id: ended.id.clone(),
        workspace_id: ended.workspace_id.clone(),
        workspace_branch: ended.workspace_branch.clone(),
        summary: ended.summary.clone(),
        updated_feature_ids: ended.updated_feature_ids.clone(),
        created_at: Utc::now().to_rfc3339(),
    };
    insert_workspace_session_record_db(ship_dir, &record)?;

    let mut ended = ended;
    ended.session_record_id = Some(record.id);

    if let Err(error) = persist_session_artifact(ship_dir, &ended, "end") {
        eprintln!("Failed to persist session artifact on end: {}", error);
    }
    if let Err(error) = run_post_session_hooks(ship_dir, &ended) {
        eprintln!("Failed to run post-session hooks: {}", error);
    }

    Ok(ended)
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
    if let Some(environment_id) = request.environment_id {
        workspace.environment_id = normalize_optional_text(Some(environment_id));
    }
    if let Some(target_id) = request.target_id {
        workspace.target_id = Some(target_id);
    }
    if let Some(active_mode) = request.active_mode {
        workspace.active_mode = Some(validate_mode_exists(ship_dir, &active_mode)?);
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
            .unwrap_or_else(|| infer_workspace_type(&branch, workspace.feature_id.as_deref()))
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
    workspace.resolved_at = now;
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
    if let Some(feature_id) = workspace.feature_id.as_deref() {
        details.push(format!("feature={feature_id}"));
    }
    if let Some(target_id) = workspace.target_id.as_deref() {
        details.push(format!("target={target_id}"));
    }
    if let Some(mode_id) = workspace.active_mode.as_deref() {
        details.push(format!("mode={mode_id}"));
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
    workspace.resolved_at = now;
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
        workspace.workspace_type = infer_workspace_type(branch, workspace.feature_id.as_deref());
    }
    validate_workspace_transition(
        workspace.workspace_type,
        workspace.status,
        WorkspaceStatus::Active,
    )?;

    workspace.status = WorkspaceStatus::Active;
    workspace.resolved_at = now;
    workspace.last_activated_at = Some(now);

    persist_branch_link_from_workspace(ship_dir, &workspace)?;
    let active_mode = workspace.active_mode.clone();
    compile_workspace_context(ship_dir, &mut workspace, active_mode.as_deref())?;
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

    // Don't re-seed if any service workspace already exists.
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Connection;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn insert_feature_for_branch(
        ship_dir: &Path,
        feature_id: &str,
        branch: &str,
        target_id: Option<&str>,
    ) -> Result<()> {
        insert_feature_for_branch_with_agent(ship_dir, feature_id, branch, target_id, "{}")
    }

    fn insert_feature_for_branch_with_agent(
        ship_dir: &Path,
        feature_id: &str,
        branch: &str,
        target_id: Option<&str>,
        agent_json: &str,
    ) -> Result<()> {
        crate::state_db::ensure_project_database(ship_dir)?;
        let mut conn = crate::state_db::open_project_connection(ship_dir)?;
        let now = Utc::now().to_rfc3339();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(async {
            sqlx::query(
                "INSERT INTO feature (id, title, description, status, active_target_id, branch, agent_json, tags_json, created_at, updated_at)
                 VALUES (?, ?, '', 'planned', ?, ?, ?, '[]', ?, ?)",
            )
            .bind(feature_id)
            .bind(format!("Feature {}", feature_id))
            .bind(target_id)
            .bind(branch)
            .bind(agent_json)
            .bind(&now)
            .bind(&now)
            .execute(&mut conn)
            .await
        })?;
        rt.block_on(async { conn.close().await })?;
        Ok(())
    }

    fn stdio_server(id: &str, command: &str) -> crate::config::McpServerConfig {
        crate::config::McpServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            command: command.to_string(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: crate::config::McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        }
    }

    #[test]
    fn lifecycle_transition_matrix_covers_expected_paths() {
        assert!(
            validate_workspace_transition(
                ShipWorkspaceKind::Feature,
                WorkspaceStatus::Archived,
                WorkspaceStatus::Active
            )
            .is_ok()
        );
        assert!(
            validate_workspace_transition(
                ShipWorkspaceKind::Feature,
                WorkspaceStatus::Active,
                WorkspaceStatus::Archived
            )
            .is_ok()
        );
        assert!(
            validate_workspace_transition(
                ShipWorkspaceKind::Feature,
                WorkspaceStatus::Archived,
                WorkspaceStatus::Archived
            )
            .is_ok()
        );
        assert!(
            validate_workspace_transition(
                ShipWorkspaceKind::Feature,
                WorkspaceStatus::Archived,
                WorkspaceStatus::Archived
            )
            .is_ok()
        );
    }

    #[test]
    fn runtime_status_transitions_cover_active_archived() {
        assert!(
            validate_workspace_transition(
                ShipWorkspaceKind::Feature,
                WorkspaceStatus::Archived,
                WorkspaceStatus::Archived,
            )
            .is_ok()
        );
    }

    #[test]
    fn workspace_kind_does_not_restrict_runtime_status() {
        assert!(
            validate_workspace_transition(
                ShipWorkspaceKind::Service,
                WorkspaceStatus::Active,
                WorkspaceStatus::Archived,
            )
            .is_ok()
        );
    }

    #[test]
    fn workspace_kind_from_str_accepts_only_canonical_values() {
        assert_eq!(
            ShipWorkspaceKind::from_str("feature").unwrap(),
            ShipWorkspaceKind::Feature
        );
        assert_eq!(
            ShipWorkspaceKind::from_str("patch").unwrap(),
            ShipWorkspaceKind::Patch
        );
        assert_eq!(
            ShipWorkspaceKind::from_str("service").unwrap(),
            ShipWorkspaceKind::Service
        );
        assert_eq!(
            ShipWorkspaceKind::from_str("hotfix")
                .expect_err("hotfix should not parse")
                .to_string(),
            "Invalid workspace type: hotfix"
        );
        assert_eq!(
            ShipWorkspaceKind::from_str("refactor")
                .expect_err("refactor should not parse")
                .to_string(),
            "Invalid workspace type: refactor"
        );
        assert_eq!(
            ShipWorkspaceKind::from_str("experiment")
                .expect_err("experiment should not parse")
                .to_string(),
            "Invalid workspace type: experiment"
        );
    }

    #[test]
    fn workspace_read_model_parsers_reject_unknown_values() {
        assert_eq!(
            parse_workspace_type_required("weird-type")
                .expect_err("invalid workspace type should fail")
                .to_string(),
            "Invalid workspace type 'weird-type'; expected one of: feature, patch, service"
        );
        assert_eq!(
            parse_workspace_status_required("in-progress")
                .expect_err("invalid status should fail")
                .to_string(),
            "Invalid workspace status 'in-progress'; expected one of: active, archived"
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
            ShipWorkspaceKind::Feature
        );
        assert_eq!(
            infer_workspace_type("service/agent-lab", None),
            ShipWorkspaceKind::Feature
        );
        assert_eq!(
            infer_workspace_type("patch/token", None),
            ShipWorkspaceKind::Patch
        );
    }

    #[test]
    fn create_workspace_hydrates_feature_link_from_branch_context() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;
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
    fn create_workspace_mixed_branch_links_preserve_target_context() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        insert_feature_for_branch(&ship_dir, "feat-mixed", "feature/mixed", Some("target-a"))?;
        crate::state_db::set_branch_link(&ship_dir, "feature/mixed", "target", "target-direct")?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/mixed".to_string(),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        assert_eq!(workspace.feature_id.as_deref(), Some("feat-mixed"));
        assert_eq!(workspace.target_id.as_deref(), Some("target-direct"));
        let stored_link = get_branch_link(&ship_dir, "feature/mixed")?;
        assert_eq!(
            stored_link,
            Some(("feature".to_string(), "feat-mixed".to_string()))
        );
        Ok(())
    }

    #[test]
    fn workspace_never_persists_target_as_branch_owner() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "service/target-context".to_string(),
                workspace_type: Some(ShipWorkspaceKind::Feature),
                target_id: Some("target-only".to_string()),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        assert_eq!(workspace.target_id.as_deref(), Some("target-only"));
        assert!(get_branch_link(&ship_dir, "service/target-context")?.is_none());
        Ok(())
    }

    #[test]
    fn activating_workspace_keeps_other_workspace_status_when_both_are_feature_workspaces()
    -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

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
        let _ = activate_workspace(&ship_dir, "feature/beta")?;

        let first_after = get_workspace(&ship_dir, "feature/alpha")?
            .ok_or_else(|| anyhow::anyhow!("feature/alpha workspace missing"))?;
        let second_after = get_workspace(&ship_dir, "feature/beta")?
            .ok_or_else(|| anyhow::anyhow!("feature/beta workspace missing"))?;
        assert_eq!(first_after.status, WorkspaceStatus::Active);
        assert_eq!(second_after.status, WorkspaceStatus::Active);
        assert!(second_after.last_activated_at.is_some());
        Ok(())
    }

    #[test]
    fn activate_workspace_main_branch_stays_unlinked() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let workspace = activate_workspace(&ship_dir, "main")?;
        assert_eq!(workspace.status, WorkspaceStatus::Active);
        assert!(workspace.feature_id.is_none());
        assert!(workspace.target_id.is_none());
        assert!(get_branch_link(&ship_dir, "main")?.is_none());
        Ok(())
    }

    #[test]
    fn delete_workspace_removes_workspace_links_and_sessions() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

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
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

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
    fn create_workspace_auto_populates_worktree_path_when_missing() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/missing-path".to_string(),
                is_worktree: Some(true),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        assert!(workspace.is_worktree);
        let worktree_path = workspace
            .worktree_path
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("expected auto-generated worktree path"))?;
        assert!(worktree_path.contains("feature-missing-path"));
        Ok(())
    }

    #[test]
    fn activate_worktree_workspace_compiles_agent_config_into_worktree_root() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
        config.providers = vec!["claude".to_string()];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        let worktree_root = tmp
            .path()
            .join(".worktrees")
            .join("feature-worktree-export");
        let worktree_path = worktree_root.to_string_lossy().to_string();
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/worktree-export".to_string(),
                status: Some(WorkspaceStatus::Active),
                is_worktree: Some(true),
                worktree_path: Some(worktree_path),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        let workspace = activate_workspace(&ship_dir, "feature/worktree-export")?;
        assert!(workspace.compiled_at.is_some());
        assert!(workspace.compile_error.is_none());
        assert!(
            worktree_root.join(".mcp.json").exists(),
            "expected provider config to be written to worktree root"
        );
        assert!(
            worktree_root
                .join(".ship")
                .join("generated")
                .join("runtime")
                .join("envelope.json")
                .exists(),
            "expected runtime envelope artifacts in worktree root"
        );
        assert!(
            !tmp.path().join(".mcp.json").exists(),
            "main checkout root should not receive worktree provider config"
        );
        Ok(())
    }

    #[test]
    fn workspace_feature_agent_invalid_provider_override_falls_back_to_project_providers()
    -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
        config.providers = vec!["codex".to_string()];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        insert_feature_for_branch_with_agent(
            &ship_dir,
            "feat-provider-fallback",
            "feature/provider-fallback",
            None,
            r#"{"providers":["totally-unknown-provider"]}"#,
        )?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-fallback".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        let workspace = activate_workspace(&ship_dir, "feature/provider-fallback")?;
        assert!(workspace.compile_error.is_none());

        let matrix = get_workspace_provider_matrix(&ship_dir, "feature/provider-fallback", None)?;
        assert_eq!(matrix.allowed_providers, vec!["codex".to_string()]);
        assert!(matrix.resolution_error.is_none());
        Ok(())
    }

    #[test]
    fn workspace_compile_exports_only_feature_filtered_skills() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
        config.providers = vec!["codex".to_string()];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        crate::skill::create_skill(&ship_dir, "selected-skill", "Selected", "selected content")?;
        crate::skill::create_skill(&ship_dir, "other-skill", "Other", "other content")?;

        insert_feature_for_branch_with_agent(
            &ship_dir,
            "feat-skill-filter",
            "feature/skill-filter",
            None,
            r#"{"providers":["codex"],"skills":["selected-skill"]}"#,
        )?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/skill-filter".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        let workspace = activate_workspace(&ship_dir, "feature/skill-filter")?;
        assert!(workspace.compile_error.is_none());

        let project_root = ship_dir.parent().unwrap_or(&ship_dir).to_path_buf();
        assert!(
            project_root
                .join(".agents")
                .join("skills")
                .join("selected-skill")
                .join("SKILL.md")
                .exists(),
            "selected feature skill should be exported"
        );
        assert!(
            !project_root
                .join(".agents")
                .join("skills")
                .join("other-skill")
                .join("SKILL.md")
                .exists(),
            "non-selected feature skill should not be exported"
        );
        Ok(())
    }

    #[test]
    fn workspace_agent_overrides_persist_and_round_trip() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/agent-overrides".to_string(),
                providers: Some(vec!["codex".to_string()]),
                mcp_servers: Some(vec!["github".to_string()]),
                skills: Some(vec!["task-policy".to_string()]),
                ..Default::default()
            },
        )?;

        let workspace = get_workspace(&ship_dir, "feature/agent-overrides")?
            .ok_or_else(|| anyhow::anyhow!("workspace missing"))?;
        assert_eq!(workspace.providers, vec!["codex".to_string()]);
        assert_eq!(workspace.mcp_servers, vec!["github".to_string()]);
        assert_eq!(workspace.skills, vec!["task-policy".to_string()]);
        Ok(())
    }

    #[test]
    fn workspace_agent_overrides_take_precedence_over_feature_agent_filters() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::ProjectConfig::default();
        config.providers = vec!["claude".to_string()];
        config.mcp_servers = vec![
            stdio_server("github", "gh"),
            stdio_server("linear", "linear"),
        ];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        crate::skill::create_skill(&ship_dir, "selected-skill", "Selected", "selected content")?;
        crate::skill::create_skill(
            &ship_dir,
            "workspace-skill",
            "Workspace",
            "workspace content",
        )?;

        insert_feature_for_branch_with_agent(
            &ship_dir,
            "feat-workspace-agent-override",
            "feature/workspace-agent-override",
            None,
            r#"{"providers":["gemini"],"mcp_servers":["github"],"skills":["selected-skill"]}"#,
        )?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/workspace-agent-override".to_string(),
                feature_id: Some("feat-workspace-agent-override".to_string()),
                providers: Some(vec!["codex".to_string()]),
                mcp_servers: Some(vec!["linear".to_string()]),
                skills: Some(vec!["workspace-skill".to_string()]),
                ..Default::default()
            },
        )?;

        let workspace = get_workspace(&ship_dir, "feature/workspace-agent-override")?
            .ok_or_else(|| anyhow::anyhow!("workspace missing"))?;
        let resolved = resolve_workspace_agent_config(&ship_dir, &workspace, None)?;

        assert_eq!(resolved.providers, vec!["codex".to_string()]);
        assert_eq!(
            resolved
                .mcp_servers
                .iter()
                .map(|server| server.id.as_str())
                .collect::<Vec<_>>(),
            vec!["linear"]
        );
        assert_eq!(
            resolved
                .skills
                .iter()
                .map(|skill| skill.id.as_str())
                .collect::<Vec<_>>(),
            vec!["workspace-skill"]
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
            },
        )?;
        assert_eq!(ended.status, WorkspaceSessionStatus::Ended);
        assert!(ended.ended_at.is_some());
        assert_eq!(ended.summary.as_deref(), Some("Implemented parser + tests"));
        assert_eq!(ended.updated_feature_ids, vec!["feat-parser".to_string()]);
        assert!(ended.session_record_id.is_some());
        assert!(get_active_workspace_session(&ship_dir, "feature/session-flow")?.is_none());

        let events = crate::events::read_events(&ship_dir)?;
        assert!(events.iter().any(|event| {
            event.entity == crate::events::EventEntity::Session
                && event.action == crate::events::EventAction::Start
                && event.subject == started.id
        }));
        assert!(events.iter().any(|event| {
            event.entity == crate::events::EventEntity::Session
                && event.action == crate::events::EventAction::Stop
                && event.subject == ended.id
        }));
        Ok(())
    }

    #[test]
    fn workspace_session_start_attaches_existing_active_session() -> Result<()> {
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
        let attached = start_workspace_session(
            &ship_dir,
            "feature/session-dupe",
            Some("two".into()),
            None,
            None,
        )?;

        assert_eq!(attached.id, first.id);
        let sessions = list_workspace_sessions(&ship_dir, Some("feature/session-dupe"), 10)?;
        assert_eq!(sessions.len(), 1);
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
    fn provider_matrix_prefers_feature_agent_providers_over_mode_and_config() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::ProjectConfig::default();
        config.providers = vec!["claude".to_string()];
        config.modes = vec![crate::config::ModeConfig {
            id: "planning".to_string(),
            name: "Planning".to_string(),
            target_agents: vec!["codex".to_string()],
            ..Default::default()
        }];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        insert_feature_for_branch_with_agent(
            &ship_dir,
            "feat-provider-feature",
            "feature/provider-feature",
            None,
            r#"{"providers":["gemini"]}"#,
        )?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-feature".to_string(),
                feature_id: Some("feat-provider-feature".to_string()),
                active_mode: Some("planning".to_string()),
                ..Default::default()
            },
        )?;

        let matrix = get_workspace_provider_matrix(&ship_dir, "feature/provider-feature", None)?;
        assert_eq!(matrix.source, "feature");
        assert_eq!(matrix.allowed_providers, vec!["gemini".to_string()]);
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
        transition_workspace_status(&ship_dir, "feature/repair-idle", WorkspaceStatus::Archived)?;

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
    fn activate_workspace_always_recompiles_and_bumps_generation() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/hash-short-circuit".to_string(),
                ..Default::default()
            },
        )?;

        let first = activate_workspace(&ship_dir, "feature/hash-short-circuit")?;
        let second = activate_workspace(&ship_dir, "feature/hash-short-circuit")?;

        assert!(second.config_generation > first.config_generation);
        assert!(second.compile_error.is_none());
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

        let mut updated_config = crate::config::get_config(Some(ship_dir.clone()))?;
        updated_config.providers = vec!["codex".to_string()];
        crate::config::save_config(&updated_config, Some(ship_dir.clone()))?;

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
