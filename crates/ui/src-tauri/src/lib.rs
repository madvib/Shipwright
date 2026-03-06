use runtime::config::{
    add_mcp_server, add_mode, generate_gitignore, get_active_mode, get_config,
    get_effective_config, list_mcp_servers, remove_mcp_server, remove_mode, save_config,
    set_active_mode, AiConfig, McpServerConfig, ModeConfig, ProjectConfig, ProjectDiscovery,
};
use runtime::project::{
    adrs_dir, features_dir, get_active_project_global, get_project_dir, issues_dir, releases_dir,
    resolve_project_ship_dir, set_active_project_global, specs_dir, SHIP_DIR_NAME,
};
use runtime::{
    activate_workspace, create_skill, create_user_skill, create_workspace, delete_skill,
    delete_user_skill, get_effective_skill, get_skill, get_user_skill, get_workspace,
    ingest_external_events, list_catalog, list_catalog_by_kind, list_effective_skills,
    list_events_since, list_models, list_providers, list_skills, list_user_skills, list_workspaces,
    log_action, read_log_entries, resolve_agent_config, search_catalog, sync_workspace,
    transition_workspace_status, update_skill, update_user_skill, AgentConfig, CatalogEntry,
    CatalogKind, CreateWorkspaceRequest, EventRecord, LogEntry, ModelInfo, ProviderInfo, Skill,
    Workspace, WorkspaceStatus, WorkspaceType,
};
use serde::{Deserialize, Serialize};
use ship_module_project::{
    create_adr, create_feature, create_issue, create_note, create_release, create_spec, delete_adr,
    delete_issue, delete_spec, get_adr_by_id, get_feature_by_id, get_issue_by_id, get_note_by_id,
    get_project_name, get_release_by_id, get_spec_by_id, init_project, list_adrs, list_features,
    list_issues, list_notes, list_registered_projects, list_releases, list_specs, move_adr,
    move_issue, read_template, register_project, rename_project, update_adr, update_feature,
    update_issue, update_note_content, update_release, update_spec, AdrEntry, AdrStatus,
    FeatureEntry as ProjectFeatureEntry, Issue, IssueEntry, IssueStatus, NoteScope,
    ReleaseEntry as ProjectReleaseEntry, Spec, SpecEntry, ADR,
};
use specta::Type;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use tauri_specta::Event;

// ─── Typed Events ─────────────────────────────────────────────────────────────

/// Typed push events from the backend to the UI.
/// Each variant maps to a `{ type: "..." }` payload on the TypeScript side.
#[derive(Clone, Serialize, Type, tauri_specta::Event)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ShipEvent {
    /// One or more issue files changed (created, moved, deleted).
    IssuesChanged,
    /// Spec files changed.
    SpecsChanged,
    /// ADR files changed.
    AdrsChanged,
    /// Feature files changed.
    FeaturesChanged,
    /// Release files changed.
    ReleasesChanged,
    /// Project config (ship.toml) changed.
    ConfigChanged,
    /// Event log changed (new events ingested).
    EventsChanged,
    /// Human-readable log changed.
    LogChanged,
    /// Note files or DB entries changed.
    NotesChanged,
}

// ─── App State ────────────────────────────────────────────────────────────────

struct ProjectPoller {
    stop_tx: mpsc::Sender<()>,
    handle: thread::JoinHandle<()>,
}

/// Holds the currently active project directory (the `.ship` dir path).
#[derive(Default)]
pub struct AppState {
    active_project: Mutex<Option<PathBuf>>,
    project_watcher: Mutex<Option<ProjectPoller>>,
}

// ─── Project Info ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub issue_count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct SpecInfo {
    pub file_name: String,
    pub title: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct SpecDocument {
    pub file_name: String,
    pub title: String,
    pub path: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct VisionDocument {
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ReleaseInfo {
    pub id: String,
    pub file_name: String,
    pub version: String,
    pub status: String,
    pub path: String,
    pub updated: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ReleaseDocument {
    pub id: String,
    pub file_name: String,
    pub version: String,
    pub status: String,
    pub path: String,
    pub updated: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureInfo {
    pub id: String,
    pub file_name: String,
    pub title: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub path: String,
    pub updated: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureDocument {
    pub id: String,
    pub file_name: String,
    pub title: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub path: String,
    pub updated: String,
    pub content: String,
}

fn get_active_dir(state: &State<AppState>) -> Result<PathBuf, String> {
    let guard = state.active_project.lock().unwrap();
    guard
        .as_ref()
        .cloned()
        .ok_or_else(|| "No active project".to_string())
}

fn resolve_note_scope_and_dir(
    state: &State<AppState>,
    scope: Option<String>,
) -> Result<(NoteScope, Option<PathBuf>), String> {
    let resolved_scope = scope
        .as_deref()
        .map(|value| value.parse::<NoteScope>())
        .transpose()
        .map_err(|e| e.to_string())?
        .unwrap_or(NoteScope::Project);

    match resolved_scope {
        NoteScope::Project => Ok((resolved_scope, Some(get_active_dir(state)?))),
        NoteScope::User => Ok((resolved_scope, None)),
    }
}

fn ensure_ship_path(path: &Path) -> PathBuf {
    if path
        .file_name()
        .map(|name| name == SHIP_DIR_NAME)
        .unwrap_or(false)
    {
        path.to_path_buf()
    } else {
        path.join(SHIP_DIR_NAME)
    }
}

fn selected_base_dir(path: &Path) -> PathBuf {
    if path
        .file_name()
        .map(|name| name == SHIP_DIR_NAME)
        .unwrap_or(false)
    {
        path.parent().unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

fn current_inside_project(cwd: &Path, registered_path: &Path) -> bool {
    if let (Some(cwd_ship), Some(registered_ship)) = (
        resolve_project_ship_dir(cwd),
        resolve_project_ship_dir(registered_path),
    ) {
        return cwd_ship == registered_ship;
    }

    let ship_path = ensure_ship_path(registered_path);
    let root = ship_path.parent().unwrap_or(&ship_path);
    cwd.starts_with(root)
}

fn project_display_name(ship_path: &Path) -> String {
    if let Ok(config) = get_config(Some(ship_path.to_path_buf())) {
        if let Some(name) = config.name {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    let canonical_ship = fs::canonicalize(ship_path).unwrap_or_else(|_| ship_path.to_path_buf());
    if let Ok(registry) = list_registered_projects() {
        for entry in registry {
            let entry_ship = ensure_ship_path(&entry.path);
            let entry_ship = fs::canonicalize(&entry_ship).unwrap_or(entry_ship);
            if entry_ship == canonical_ship {
                let trimmed = entry.name.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }

    get_project_name(ship_path)
}

fn timestamp_nanos(time: SystemTime) -> u128 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn file_signature(path: &Path) -> Option<u128> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .map(timestamp_nanos)
}

fn issues_signature(dir: &Path) -> (u64, u128) {
    let mut count = 0_u64;
    let mut latest = 0_u128;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let (nested_count, nested_latest) = issues_signature(&path);
                count += nested_count;
                latest = latest.max(nested_latest);
                continue;
            }
            if path.extension().map(|ext| ext == "md").unwrap_or(false) {
                count += 1;
                if let Some(sig) = file_signature(&path) {
                    latest = latest.max(sig);
                }
            }
        }
    }

    (count, latest)
}

fn find_markdown_file_by_id(root: &Path, id: &str) -> Option<PathBuf> {
    if !root.exists() {
        return None;
    }
    let needle = format!("id = \"{}\"", id);
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if !path.extension().map(|ext| ext == "md").unwrap_or(false) {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            if content.contains(&needle) {
                return Some(path);
            }
        }
    }
    None
}

fn resolve_feature_markdown_path(
    project_dir: &Path,
    entry: &ProjectFeatureEntry,
) -> Option<PathBuf> {
    if !entry.path.is_empty() {
        let direct = PathBuf::from(&entry.path);
        if direct.exists() {
            return Some(direct);
        }
    }

    let features_root = features_dir(project_dir);
    let status = entry.status.to_string();
    let primary = features_root.join(&status).join(&entry.file_name);
    if primary.exists() {
        return Some(primary);
    }

    for candidate_dir in [
        features_root.join("planned"),
        features_root.join("in-progress"),
        features_root.join("implemented"),
        features_root.join("deprecated"),
        features_root.clone(),
    ] {
        let candidate = candidate_dir.join(&entry.file_name);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    find_markdown_file_by_id(&features_root, &entry.id)
}

fn resolve_release_markdown_path(
    project_dir: &Path,
    entry: &ProjectReleaseEntry,
) -> Option<PathBuf> {
    if !entry.path.is_empty() {
        let direct = PathBuf::from(&entry.path);
        if direct.exists() {
            return Some(direct);
        }
    }

    let releases_root = releases_dir(project_dir);
    let dashed_file = format!("{}.md", entry.version.replace('.', "-"));
    let candidates = [
        entry.file_name.clone(),
        format!("{}.md", entry.version),
        dashed_file,
    ];

    for base_dir in [releases_root.clone(), releases_root.join("upcoming")] {
        for file_name in &candidates {
            let candidate = base_dir.join(file_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    find_markdown_file_by_id(&releases_root, &entry.id)
}

fn map_feature_info(project_dir: &Path, entry: &ProjectFeatureEntry) -> FeatureInfo {
    let path = resolve_feature_markdown_path(project_dir, entry)
        .unwrap_or_else(|| features_dir(project_dir).join(&entry.file_name));
    FeatureInfo {
        id: entry.id.clone(),
        file_name: entry.file_name.clone(),
        title: entry.feature.metadata.title.clone(),
        status: entry.status.to_string(),
        release_id: entry.feature.metadata.release_id.clone(),
        spec_id: entry.feature.metadata.spec_id.clone(),
        branch: entry.feature.metadata.branch.clone(),
        description: entry.feature.metadata.description.clone(),
        path: path.to_string_lossy().to_string(),
        updated: entry.feature.metadata.updated.clone(),
    }
}

fn map_feature_document(project_dir: &Path, entry: &ProjectFeatureEntry) -> FeatureDocument {
    let info = map_feature_info(project_dir, entry);
    let content = resolve_feature_markdown_path(project_dir, entry)
        .and_then(|path| fs::read_to_string(path).ok())
        .unwrap_or_else(|| entry.feature.body.clone());
    FeatureDocument {
        id: info.id,
        file_name: info.file_name,
        title: info.title,
        status: info.status,
        release_id: info.release_id,
        spec_id: info.spec_id,
        branch: info.branch,
        description: info.description,
        path: info.path,
        updated: info.updated,
        content,
    }
}

fn map_release_info(project_dir: &Path, entry: &ProjectReleaseEntry) -> ReleaseInfo {
    let path = resolve_release_markdown_path(project_dir, entry)
        .unwrap_or_else(|| releases_dir(project_dir).join(&entry.file_name));
    ReleaseInfo {
        id: entry.id.clone(),
        file_name: entry.file_name.clone(),
        version: entry.version.clone(),
        status: entry.status.to_string(),
        path: path.to_string_lossy().to_string(),
        updated: entry.release.metadata.updated.clone(),
    }
}

fn map_release_document(project_dir: &Path, entry: &ProjectReleaseEntry) -> ReleaseDocument {
    let info = map_release_info(project_dir, entry);
    let content = resolve_release_markdown_path(project_dir, entry)
        .and_then(|path| fs::read_to_string(path).ok())
        .unwrap_or_else(|| entry.release.body.clone());
    ReleaseDocument {
        id: info.id,
        file_name: info.file_name,
        version: info.version,
        status: info.status,
        path: info.path,
        updated: info.updated,
        content,
    }
}

// ─── AI helper ────────────────────────────────────────────────────────────────

fn ai_cli_attempts(provider: &str, prompt: &str) -> Vec<Vec<String>> {
    match provider.to_ascii_lowercase().as_str() {
        "claude" | "gemini" => {
            vec![
                vec!["-p".to_string(), prompt.to_string()],
                vec![prompt.to_string()],
            ]
        }
        "codex" | "chatgpt" => vec![
            vec!["exec".to_string(), prompt.to_string()],
            vec!["-p".to_string(), prompt.to_string()],
            vec![prompt.to_string()],
        ],
        _ => vec![
            vec!["-p".to_string(), prompt.to_string()],
            vec![prompt.to_string()],
        ],
    }
}

fn ai_cli_success_text(stdout: &[u8], stderr: &[u8]) -> Option<String> {
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();
    if !stdout.is_empty() {
        return Some(stdout);
    }
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    if !stderr.is_empty() {
        return Some(stderr);
    }
    None
}

fn invoke_ai_cli(ai: &AiConfig, prompt: &str) -> Result<String, String> {
    let cli = ai.effective_cli().to_string();
    let provider = ai.effective_provider().to_ascii_lowercase();
    let attempts = ai_cli_attempts(&provider, prompt);

    let mut attempt_errors: Vec<String> = Vec::new();
    for args in attempts {
        let output = std::process::Command::new(&cli)
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to launch '{}': {}", cli, e))?;
        if output.status.success() {
            if let Some(text) = ai_cli_success_text(&output.stdout, &output.stderr) {
                return Ok(text);
            }
            return Err(format!("AI CLI '{}' succeeded but returned no output", cli));
        }

        let status = output
            .status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "signal".to_string());
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let body = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            "no output".to_string()
        };
        attempt_errors.push(format!(
            "{} {} -> exit {}: {}",
            cli,
            args.join(" "),
            status,
            body
        ));
    }

    Err(format!(
        "AI CLI failed after {} attempt(s): {}",
        attempt_errors.len(),
        attempt_errors.join(" | ")
    ))
}

// ─── Commands: Project ────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_projects() -> Result<Vec<ProjectDiscovery>, String> {
    let registry = list_registered_projects().map_err(|e| e.to_string())?;
    let mut projects = Vec::new();
    let mut seen_paths = HashSet::new();
    for entry in registry {
        let ship_path = ensure_ship_path(&entry.path);
        let key = ship_path.to_string_lossy().to_string();
        if ship_path.exists() && seen_paths.insert(key) {
            let issue_count = list_issues(&ship_path)
                .map(|issues| issues.len())
                .unwrap_or(0);
            projects.push(ProjectDiscovery {
                name: entry.name,
                path: ship_path,
                issue_count,
            });
        }
    }
    Ok(projects)
}

#[tauri::command]
#[specta::specta]
fn get_active_project(state: State<AppState>) -> Result<Option<ProjectInfo>, String> {
    let guard = state.active_project.lock().unwrap();
    match &*guard {
        None => {
            // Try to load from global state
            drop(guard);
            if let Ok(Some(path)) = get_active_project_global() {
                if path.exists() {
                    let issues = list_issues(&path).unwrap_or_default();
                    return Ok(Some(ProjectInfo {
                        name: project_display_name(&path),
                        path: path.to_string_lossy().to_string(),
                        issue_count: issues.len(),
                    }));
                }
            }
            Ok(None)
        }
        Some(path) => {
            let issues = list_issues(path).unwrap_or_default();
            Ok(Some(ProjectInfo {
                name: project_display_name(path),
                path: path.to_string_lossy().to_string(),
                issue_count: issues.len(),
            }))
        }
    }
}

#[tauri::command]
#[specta::specta]
fn set_active_project(
    path: String,
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<ProjectInfo, String> {
    let ship_path = ensure_ship_path(Path::new(&path));
    if !ship_path.exists() {
        return Err(format!("Path does not exist: {}", ship_path.display()));
    }
    let issues = list_issues(&ship_path).unwrap_or_default();
    let display_name = project_display_name(&ship_path);
    let info = ProjectInfo {
        name: display_name.clone(),
        path: ship_path.to_string_lossy().to_string(),
        issue_count: issues.len(),
    };
    *state.active_project.lock().unwrap() = Some(ship_path.clone());
    register_project(display_name, ship_path.clone()).map_err(|e: anyhow::Error| e.to_string())?;
    if let Err(err) = start_project_watcher(&app, &state, &ship_path) {
        eprintln!("Failed to start project watcher: {}", err);
    }
    // Persist to global state
    set_active_project_global(ship_path).map_err(|e| e.to_string())?;
    Ok(info)
}

/// Opens a folder picker. If the chosen directory has no .ship, initialises one.
/// Sets the result as the active project.
#[tauri::command]
#[specta::specta]
async fn pick_and_open_project(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<ProjectInfo, String> {
    let picked = app.dialog().file().blocking_pick_folder();
    let selected_dir = match picked {
        Some(p) => p
            .as_path()
            .ok_or_else(|| "Invalid path".to_string())?
            .to_path_buf(),
        None => return Err("No directory selected".to_string()),
    };
    let base_dir = selected_base_dir(&selected_dir);
    let ship_path = ensure_ship_path(&selected_dir);
    let final_ship_path = if ship_path.exists() {
        ship_path
    } else {
        init_project(base_dir).map_err(|e| e.to_string())?
    };

    let issues = list_issues(&final_ship_path).unwrap_or_default();
    let display_name = project_display_name(&final_ship_path);
    let info = ProjectInfo {
        name: display_name.clone(),
        path: final_ship_path.to_string_lossy().to_string(),
        issue_count: issues.len(),
    };
    *state.active_project.lock().unwrap() = Some(final_ship_path.clone());
    register_project(display_name, final_ship_path.clone())
        .map_err(|e: anyhow::Error| e.to_string())?;
    if let Err(err) = start_project_watcher(&app, &state, &final_ship_path) {
        eprintln!("Failed to start project watcher: {}", err);
    }
    // Persist to global state
    set_active_project_global(final_ship_path).map_err(|e| e.to_string())?;
    Ok(info)
}

/// Auto-detect current project from the working directory (for local e2e).
#[tauri::command]
#[specta::specta]
fn detect_current_project(
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<Option<ProjectInfo>, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let cwd = fs::canonicalize(cwd).map_err(|e| e.to_string())?;
    let registry = list_registered_projects().map_err(|e| e.to_string())?;
    for entry in registry {
        if current_inside_project(&cwd, &entry.path) {
            let ship_path = ensure_ship_path(&entry.path);
            if !ship_path.exists() {
                continue;
            }
            let issues = list_issues(&ship_path).unwrap_or_default();
            let display_name = project_display_name(&ship_path);
            let info = ProjectInfo {
                name: display_name,
                path: ship_path.to_string_lossy().to_string(),
                issue_count: issues.len(),
            };
            *state.active_project.lock().unwrap() = Some(ship_path.clone());
            if let Err(err) = start_project_watcher(&app, &state, &ship_path) {
                eprintln!("Failed to start project watcher: {}", err);
            }
            set_active_project_global(ship_path).map_err(|e| e.to_string())?;
            return Ok(Some(info));
        }
    }

    // Fallback: detect local .ship via cwd traversal and register it.
    match get_project_dir(None) {
        Ok(ship_path) => {
            let issues = list_issues(&ship_path).unwrap_or_default();
            let display_name = project_display_name(&ship_path);
            let info = ProjectInfo {
                name: display_name.clone(),
                path: ship_path.to_string_lossy().to_string(),
                issue_count: issues.len(),
            };
            // Also set as active
            *state.active_project.lock().unwrap() = Some(ship_path.clone());
            register_project(display_name, ship_path.clone())
                .map_err(|e: anyhow::Error| e.to_string())?;
            if let Err(err) = start_project_watcher(&app, &state, &ship_path) {
                eprintln!("Failed to start project watcher: {}", err);
            }
            // Persist to global state
            set_active_project_global(ship_path).map_err(|e| e.to_string())?;
            Ok(Some(info))
        }
        Err(_) => Ok(None),
    }
}

/// Creates a new project by picking a folder and initializing .ship
#[tauri::command]
#[specta::specta]
async fn create_new_project(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<ProjectInfo, String> {
    let picked = app.dialog().file().blocking_pick_folder();
    let selected_dir = match picked {
        Some(p) => p
            .as_path()
            .ok_or_else(|| "Invalid path".to_string())?
            .to_path_buf(),
        None => return Err("No directory selected".to_string()),
    };
    let base_dir = selected_base_dir(&selected_dir);
    let existing_ship = ensure_ship_path(&selected_dir);

    // Initialize the project unless it already points to a .ship directory.
    let ship_path = if existing_ship.exists() {
        existing_ship
    } else {
        init_project(base_dir.clone()).map_err(|e| e.to_string())?
    };

    let issues = list_issues(&ship_path).unwrap_or_default();
    let display_name = project_display_name(&ship_path);
    let info = ProjectInfo {
        name: display_name.clone(),
        path: ship_path.to_string_lossy().to_string(),
        issue_count: issues.len(),
    };
    *state.active_project.lock().unwrap() = Some(ship_path.clone());
    register_project(display_name, ship_path.clone()).map_err(|e: anyhow::Error| e.to_string())?;
    if let Err(err) = start_project_watcher(&app, &state, &ship_path) {
        eprintln!("Failed to start project watcher: {}", err);
    }
    // Persist to global state
    set_active_project_global(ship_path).map_err(|e| e.to_string())?;
    Ok(info)
}

/// Opens a folder picker and returns the selected directory path.
#[tauri::command]
#[specta::specta]
async fn pick_project_directory(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let picked = app.dialog().file().blocking_pick_folder();
    let selected = match picked {
        Some(p) => p
            .as_path()
            .ok_or_else(|| "Invalid path".to_string())?
            .to_path_buf(),
        None => return Ok(None),
    };
    Ok(Some(selected.to_string_lossy().to_string()))
}

/// Creates (or initializes) a project from explicit onboarding options.
#[tauri::command]
#[specta::specta]
fn create_project_with_options(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    directory: String,
    name: Option<String>,
    description: Option<String>,
    config: Option<ProjectConfig>,
) -> Result<ProjectInfo, String> {
    let selected_dir = PathBuf::from(directory);
    let base_dir = selected_base_dir(&selected_dir);
    let existing_ship = ensure_ship_path(&selected_dir);

    let ship_path = if existing_ship.exists() {
        existing_ship
    } else {
        init_project(base_dir.clone()).map_err(|e| e.to_string())?
    };

    let mut final_config =
        config.unwrap_or_else(|| get_config(Some(ship_path.clone())).unwrap_or_default());

    if let Some(raw_name) = name {
        let trimmed = raw_name.trim();
        if !trimmed.is_empty() {
            final_config.name = Some(trimmed.to_string());
        }
    }

    if let Some(raw_desc) = description {
        let trimmed = raw_desc.trim();
        final_config.description = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }

    save_config(&final_config, Some(ship_path.clone())).map_err(|e| e.to_string())?;
    generate_gitignore(&ship_path, &final_config.git).map_err(|e| e.to_string())?;

    let display_name = final_config
        .name
        .clone()
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| get_project_name(&ship_path));

    let issues = list_issues(&ship_path).unwrap_or_default();
    let info = ProjectInfo {
        name: display_name.clone(),
        path: ship_path.to_string_lossy().to_string(),
        issue_count: issues.len(),
    };

    *state.active_project.lock().unwrap() = Some(ship_path.clone());
    register_project(display_name, ship_path.clone()).map_err(|e: anyhow::Error| e.to_string())?;

    if let Err(err) = start_project_watcher(&app, &state, &ship_path) {
        eprintln!("Failed to start project watcher: {}", err);
    }

    set_active_project_global(ship_path).map_err(|e| e.to_string())?;
    Ok(info)
}

#[tauri::command]
#[specta::specta]
fn rename_project_cmd(
    path: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<ProjectInfo, String> {
    let ship_path = ensure_ship_path(Path::new(&path));
    rename_project(ship_path.clone(), name).map_err(|e| e.to_string())?;

    if ship_path.exists() {
        *state.active_project.lock().unwrap() = Some(ship_path.clone());
    }

    let issues = list_issues(&ship_path).unwrap_or_default();
    Ok(ProjectInfo {
        name: project_display_name(&ship_path),
        path: ship_path.to_string_lossy().to_string(),
        issue_count: issues.len(),
    })
}

fn start_project_watcher(
    app: &tauri::AppHandle,
    state: &State<AppState>,
    ship_dir: &PathBuf,
) -> Result<(), String> {
    let app_handle = app.clone();
    let ship_root = ship_dir.clone();
    let (stop_tx, stop_rx) = mpsc::channel::<()>();

    let poller = thread::spawn(move || {
        let issues_dir = issues_dir(&ship_root);
        let specs_dir = specs_dir(&ship_root);
        let adrs_dir = adrs_dir(&ship_root);
        let features_dir = features_dir(&ship_root);
        let releases_dir = releases_dir(&ship_root);
        let events_db = runtime::state_db::project_db_path(&ship_root).ok();
        let config_file = ship_root.join(runtime::config::PRIMARY_CONFIG_FILE);

        let mut last_issues = issues_signature(&issues_dir);
        let mut last_specs = issues_signature(&specs_dir);
        let mut last_adrs = issues_signature(&adrs_dir);
        let mut last_features = issues_signature(&features_dir);
        let mut last_releases = issues_signature(&releases_dir);
        let mut last_events = events_db.as_ref().and_then(|path| file_signature(path));
        let mut last_config = file_signature(&config_file);

        loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }
            thread::sleep(Duration::from_millis(250));

            let mut tracked_files_changed = false;

            let next_issues = issues_signature(&issues_dir);
            if next_issues != last_issues {
                let _ = ShipEvent::IssuesChanged.emit(&app_handle);
                last_issues = next_issues;
                tracked_files_changed = true;
            }

            let next_specs = issues_signature(&specs_dir);
            if next_specs != last_specs {
                let _ = ShipEvent::SpecsChanged.emit(&app_handle);
                last_specs = next_specs;
                tracked_files_changed = true;
            }

            let next_adrs = issues_signature(&adrs_dir);
            if next_adrs != last_adrs {
                let _ = ShipEvent::AdrsChanged.emit(&app_handle);
                last_adrs = next_adrs;
                tracked_files_changed = true;
            }

            let next_features = issues_signature(&features_dir);
            if next_features != last_features {
                last_features = next_features;
                let _ = ShipEvent::FeaturesChanged.emit(&app_handle);
                tracked_files_changed = true;
            }

            let next_releases = issues_signature(&releases_dir);
            if next_releases != last_releases {
                last_releases = next_releases;
                let _ = ShipEvent::ReleasesChanged.emit(&app_handle);
                tracked_files_changed = true;
            }

            let next_config = file_signature(&config_file);
            if next_config != last_config {
                let _ = ShipEvent::ConfigChanged.emit(&app_handle);
                last_config = next_config;
                tracked_files_changed = true;
            }

            if tracked_files_changed {
                if let Ok(emitted) = ingest_external_events(&ship_root) {
                    if !emitted.is_empty() {
                        let _ = ShipEvent::EventsChanged.emit(&app_handle);
                    }
                }
            }

            if let Some(events_db) = events_db.as_ref() {
                let next_events = file_signature(events_db);
                if next_events != last_events {
                    let _ = ShipEvent::IssuesChanged.emit(&app_handle);
                    let _ = ShipEvent::SpecsChanged.emit(&app_handle);
                    let _ = ShipEvent::EventsChanged.emit(&app_handle);
                    let _ = ShipEvent::LogChanged.emit(&app_handle);
                    last_events = next_events;
                }
            }
        }
    });

    let mut guard = state.project_watcher.lock().unwrap();
    if let Some(old) = guard.take() {
        let _ = old.stop_tx.send(());
        let _ = old.handle.join();
    }
    *guard = Some(ProjectPoller {
        stop_tx,
        handle: poller,
    });

    Ok(())
}

// ─── Commands: Issues ─────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_items(state: State<AppState>) -> Result<Vec<IssueEntry>, String> {
    let project_dir = get_active_dir(&state)?;
    list_issues(&project_dir).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn get_issue_by_path(path: String) -> Result<Issue, String> {
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    Issue::from_markdown(&content).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn create_new_issue(
    title: String,
    description: String,
    status: String,
    assignee: Option<String>,
    tags: Option<Vec<String>>,
    state: State<AppState>,
) -> Result<IssueEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let issue_status = status
        .parse::<IssueStatus>()
        .map_err(|_| format!("Invalid issue status: {}", status))?;

    let entry = create_issue(
        &project_dir,
        &title,
        &description,
        issue_status,
        assignee,
        None, // priority
        None, // release_id
        None, // feature_id
    )
    .map_err(|e| e.to_string())?;

    if let Some(t) = tags {
        let mut issue = entry.issue.clone();
        issue.metadata.tags = t;
        update_issue(&project_dir, &entry.id, issue).map_err(|e| e.to_string())?;
    }

    log_action(&project_dir, "issue create", &format!("Created: {}", title)).ok();

    get_issue_by_id(&project_dir, &entry.id).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn update_issue_command(id: String, issue: Issue, state: State<AppState>) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    update_issue(&project_dir, &id, issue).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn move_issue_status(
    id: String,
    new_status: String,
    state: State<AppState>,
) -> Result<IssueEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let status = new_status
        .parse::<IssueStatus>()
        .map_err(|_| format!("Invalid issue status: {}", new_status))?;

    let entry = move_issue(&project_dir, &id, status).map_err(|e| e.to_string())?;
    log_action(
        &project_dir,
        "issue move",
        &format!("Moved {} → {}", entry.file_name, new_status),
    )
    .ok();

    Ok(entry)
}

#[tauri::command]
#[specta::specta]
fn delete_issue_cmd(id: String, state: State<AppState>) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    delete_issue(&project_dir, &id).map_err(|e| e.to_string())?;
    log_action(&project_dir, "issue delete", &format!("Deleted: {}", id)).ok();
    Ok(())
}

// ─── Commands: ADRs ───────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_adrs_cmd(state: State<AppState>) -> Result<Vec<AdrEntry>, String> {
    let project_dir = get_active_dir(&state)?;
    list_adrs(&project_dir).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn create_new_adr(
    title: String,
    context: String,
    decision: String,
    state: State<AppState>,
) -> Result<AdrEntry, String> {
    let project_dir = get_active_dir(&state)?;
    create_adr(&project_dir, &title, &context, &decision, "proposed").map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn get_adr_cmd(id: String, state: State<AppState>) -> Result<AdrEntry, String> {
    let project_dir = get_active_dir(&state)?;
    get_adr_by_id(&project_dir, &id).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn update_adr_cmd(id: String, adr: ADR, state: State<AppState>) -> Result<AdrEntry, String> {
    let project_dir = get_active_dir(&state)?;
    update_adr(&project_dir, &id, adr).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn move_adr_cmd(
    id: String,
    new_status: String,
    state: State<AppState>,
) -> Result<AdrEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let status = new_status
        .parse::<AdrStatus>()
        .map_err(|_| format!("Invalid ADR status: {}", new_status))?;
    move_adr(&project_dir, &id, status).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn delete_adr_cmd(id: String, state: State<AppState>) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    delete_adr(&project_dir, &id).map_err(|e| e.to_string())
}

// ─── Commands: Specs ─────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_specs_cmd(state: State<AppState>) -> Result<Vec<SpecEntry>, String> {
    let project_dir = get_active_dir(&state)?;
    list_specs(&project_dir).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn get_spec_cmd(id: String, state: State<AppState>) -> Result<SpecEntry, String> {
    let project_dir = get_active_dir(&state)?;
    get_spec_by_id(&project_dir, &id).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn create_spec_cmd(
    title: String,
    content: String,
    state: State<AppState>,
) -> Result<SpecEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let entry = create_spec(&project_dir, &title, &content, None).map_err(|e| e.to_string())?;
    log_action(
        &project_dir,
        "spec create",
        &format!("Created Spec: {}", title),
    )
    .ok();
    Ok(entry)
}

#[tauri::command]
#[specta::specta]
fn update_spec_cmd(id: String, spec: Spec, state: State<AppState>) -> Result<SpecEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let entry = update_spec(&project_dir, &id, spec).map_err(|e| e.to_string())?;
    log_action(
        &project_dir,
        "spec update",
        &format!("Updated Spec: {}", entry.file_name),
    )
    .ok();
    Ok(entry)
}

#[tauri::command]
#[specta::specta]
fn delete_spec_cmd(id: String, state: State<AppState>) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    delete_spec(&project_dir, &id).map_err(|e| e.to_string())?;
    log_action(
        &project_dir,
        "spec delete",
        &format!("Deleted Spec: {}", id),
    )
    .ok();
    Ok(())
}

// ─── Commands: Releases ──────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_releases_cmd(state: State<AppState>) -> Result<Vec<ReleaseInfo>, String> {
    let project_dir = get_active_dir(&state)?;
    let entries = list_releases(&project_dir).map_err(|e| e.to_string())?;
    Ok(entries
        .iter()
        .map(|entry| map_release_info(&project_dir, entry))
        .collect())
}

#[tauri::command]
#[specta::specta]
fn get_release_cmd(file_name: String, state: State<AppState>) -> Result<ReleaseDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let entry = get_release_by_id(&project_dir, file_name.trim_end_matches(".md"))
        .map_err(|e| e.to_string())?;
    Ok(map_release_document(&project_dir, &entry))
}

#[tauri::command]
#[specta::specta]
fn create_release_cmd(
    version: String,
    content: String,
    state: State<AppState>,
) -> Result<ReleaseDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let entry = create_release(&project_dir, &version, &content).map_err(|e| e.to_string())?;
    log_action(
        &project_dir,
        "release create",
        &format!("Created Release: {}", version),
    )
    .ok();
    Ok(map_release_document(&project_dir, &entry))
}

#[tauri::command]
#[specta::specta]
fn update_release_cmd(
    file_name: String,
    content: String,
    state: State<AppState>,
) -> Result<ReleaseDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let id = file_name.trim_end_matches(".md");
    let release_entry = get_release_by_id(&project_dir, id).map_err(|e| e.to_string())?;
    let entry =
        update_release(&project_dir, id, release_entry.release).map_err(|e| e.to_string())?;
    let path = resolve_release_markdown_path(&project_dir, &entry)
        .unwrap_or_else(|| releases_dir(&project_dir).join(&file_name));
    fs::write(&path, &content).map_err(|e| e.to_string())?;
    log_action(
        &project_dir,
        "release update",
        &format!("Updated Release: {}", file_name),
    )
    .ok();
    let updated = get_release_by_id(&project_dir, id).map_err(|e| e.to_string())?;
    Ok(map_release_document(&project_dir, &updated))
}

// ─── Commands: Features ──────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_features_cmd(state: State<AppState>) -> Result<Vec<FeatureInfo>, String> {
    let project_dir = get_active_dir(&state)?;
    let entries = list_features(&project_dir).map_err(|e| e.to_string())?;
    Ok(entries
        .iter()
        .map(|entry| map_feature_info(&project_dir, entry))
        .collect())
}

#[tauri::command]
#[specta::specta]
fn get_feature_cmd(
    file_name: String,
    state: State<'_, AppState>,
) -> Result<FeatureDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let entry = get_feature_by_id(&project_dir, file_name.trim_end_matches(".md"))
        .map_err(|e| e.to_string())?;
    Ok(map_feature_document(&project_dir, &entry))
}

#[tauri::command]
#[specta::specta]
fn create_feature_cmd(
    title: String,
    content: String,
    release: Option<String>,
    spec: Option<String>,
    state: State<AppState>,
) -> Result<FeatureDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let entry = create_feature(
        &project_dir,
        &title,
        &content,
        release.as_deref(),
        spec.as_deref(),
        None,
    )
    .map_err(|e| e.to_string())?;
    log_action(
        &project_dir,
        "feature create",
        &format!("Created Feature: {}", title),
    )
    .ok();
    Ok(map_feature_document(&project_dir, &entry))
}

#[tauri::command]
#[specta::specta]
fn update_feature_cmd(
    file_name: String,
    content: String,
    state: State<AppState>,
) -> Result<FeatureDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let id = file_name.trim_end_matches(".md");
    // Get existing feature to update its content
    let feature_entry = get_feature_by_id(&project_dir, id).map_err(|e| e.to_string())?;
    let _entry =
        update_feature(&project_dir, id, feature_entry.feature).map_err(|e| e.to_string())?;
    let refreshed = get_feature_by_id(&project_dir, id).map_err(|e| e.to_string())?;
    let path = resolve_feature_markdown_path(&project_dir, &refreshed)
        .unwrap_or_else(|| features_dir(&project_dir).join(&file_name));
    fs::write(&path, &content).map_err(|e| e.to_string())?;
    log_action(
        &project_dir,
        "feature update",
        &format!("Updated Feature: {}", file_name),
    )
    .ok();
    let updated = get_feature_by_id(&project_dir, id).map_err(|e| e.to_string())?;
    Ok(map_feature_document(&project_dir, &updated))
}

#[tauri::command]
#[specta::specta]
fn get_template_cmd(kind: String, state: State<AppState>) -> Result<String, String> {
    let project_dir = get_active_dir(&state)?;
    read_template(&project_dir, &kind).map_err(|e| e.to_string())
}

// ─── Commands: Vision ─────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn get_vision_cmd(state: State<AppState>) -> Result<VisionDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let vision_path = runtime::project::project_ns(&project_dir).join("vision.md");
    let content = std::fs::read_to_string(&vision_path).unwrap_or_default();
    Ok(VisionDocument { content })
}

#[tauri::command]
#[specta::specta]
fn update_vision_cmd(content: String, state: State<AppState>) -> Result<VisionDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let vision_path = runtime::project::project_ns(&project_dir).join("vision.md");
    if let Some(parent) = vision_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&vision_path, &content).map_err(|e| e.to_string())?;
    Ok(VisionDocument { content })
}

// ─── Commands: Notes ──────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct NoteInfo {
    pub id: String,
    pub title: String,
    pub updated: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct NoteDocument {
    pub id: String,
    pub title: String,
    pub updated: String,
    pub content: String,
}

#[tauri::command]
#[specta::specta]
fn list_notes_cmd(scope: Option<String>, state: State<AppState>) -> Result<Vec<NoteInfo>, String> {
    let (note_scope, project_dir) = resolve_note_scope_and_dir(&state, scope)?;
    let entries = list_notes(note_scope, project_dir.as_deref()).map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .map(|e| NoteInfo {
            id: e.id,
            title: e.title,
            updated: e.updated,
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
fn get_note_cmd(
    id: String,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<NoteDocument, String> {
    let (note_scope, project_dir) = resolve_note_scope_and_dir(&state, scope)?;
    let note =
        get_note_by_id(note_scope, project_dir.as_deref(), &id).map_err(|e| e.to_string())?;
    Ok(NoteDocument {
        id: note.id,
        title: note.title,
        updated: note.updated_at,
        content: note.content,
    })
}

#[tauri::command]
#[specta::specta]
fn create_note_cmd(
    title: String,
    content: String,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<NoteDocument, String> {
    let (note_scope, project_dir) = resolve_note_scope_and_dir(&state, scope)?;
    let note = create_note(note_scope, project_dir.as_deref(), &title, &content)
        .map_err(|e| e.to_string())?;
    Ok(NoteDocument {
        id: note.id,
        title: note.title,
        updated: note.updated_at,
        content: note.content,
    })
}

#[tauri::command]
#[specta::specta]
fn update_note_cmd(
    id: String,
    content: String,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<NoteDocument, String> {
    let (note_scope, project_dir) = resolve_note_scope_and_dir(&state, scope)?;
    let note = update_note_content(note_scope, project_dir.as_deref(), &id, &content)
        .map_err(|e| e.to_string())?;
    Ok(NoteDocument {
        id: note.id,
        title: note.title,
        updated: note.updated_at,
        content: note.content,
    })
}

#[tauri::command]
#[specta::specta]
fn delete_note_cmd(
    id: String,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let (note_scope, project_dir) = resolve_note_scope_and_dir(&state, scope)?;
    ship_module_project::delete_note(note_scope, project_dir.as_deref(), &id)
        .map_err(|e| e.to_string())
}

// ─── Commands: Rules ──────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_rules_cmd(state: State<AppState>) -> Result<Vec<runtime::rule::Rule>, String> {
    let project_dir = get_active_dir(&state)?;
    runtime::list_rules(project_dir).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn get_rule_cmd(file_name: String, state: State<AppState>) -> Result<runtime::rule::Rule, String> {
    let project_dir = get_active_dir(&state)?;
    runtime::get_rule(project_dir, &file_name).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn create_rule_cmd(
    file_name: String,
    content: String,
    state: State<AppState>,
) -> Result<runtime::rule::Rule, String> {
    let project_dir = get_active_dir(&state)?;
    runtime::create_rule(project_dir, &file_name, &content).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn update_rule_cmd(
    file_name: String,
    content: String,
    state: State<AppState>,
) -> Result<runtime::rule::Rule, String> {
    let project_dir = get_active_dir(&state)?;
    runtime::update_rule(project_dir, &file_name, &content).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn delete_rule_cmd(file_name: String, state: State<AppState>) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    runtime::delete_rule(project_dir, &file_name).map_err(|e| e.to_string())
}

// ─── Commands: Permissions ────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn get_permissions_cmd(
    state: State<AppState>,
) -> Result<runtime::permissions::Permissions, String> {
    let project_dir = get_active_dir(&state)?;
    runtime::get_permissions(project_dir).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn save_permissions_cmd(
    permissions: runtime::permissions::Permissions,
    state: State<AppState>,
) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    runtime::save_permissions(project_dir, &permissions).map_err(|e| e.to_string())
}

// ─── Commands: Workspace ──────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
async fn get_workspace_cmd(
    branch: String,
    state: State<'_, AppState>,
) -> Result<Option<Workspace>, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        get_workspace(&project_dir, &branch).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn list_workspaces_cmd(state: State<'_, AppState>) -> Result<Vec<Workspace>, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        list_workspaces(&project_dir).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn sync_workspace_cmd(
    branch: Option<String>,
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let resolved_branch = if let Some(value) = branch {
            value
        } else {
            let git_root = project_dir.parent().unwrap_or(&project_dir).to_path_buf();
            let output = std::process::Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(&git_root)
                .output()
                .map_err(|e| e.to_string())?;
            if !output.status.success() {
                return Err("Failed to resolve active git branch".to_string());
            }
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        };

        if resolved_branch.is_empty() || resolved_branch == "HEAD" {
            return Err("No active branch to sync".to_string());
        }

        sync_workspace(&project_dir, &resolved_branch).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn create_workspace_cmd(
    branch: String,
    workspace_type: Option<String>,
    feature_id: Option<String>,
    spec_id: Option<String>,
    release_id: Option<String>,
    activate: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let parsed_workspace_type = workspace_type
            .as_deref()
            .map(|value| value.parse::<WorkspaceType>())
            .transpose()
            .map_err(|e| e.to_string())?;

        let status = if activate.unwrap_or(false) {
            Some(WorkspaceStatus::Active)
        } else {
            None
        };

        create_workspace(
            &project_dir,
            CreateWorkspaceRequest {
                branch,
                workspace_type: parsed_workspace_type,
                status,
                feature_id,
                spec_id,
                release_id,
                ..CreateWorkspaceRequest::default()
            },
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn activate_workspace_cmd(
    branch: String,
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        activate_workspace(&project_dir, &branch).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn transition_workspace_cmd(
    branch: String,
    status: String,
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    let project_dir = get_active_dir(&state)?;
    let next_status: WorkspaceStatus = status.parse().map_err(|e: anyhow::Error| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        transition_workspace_status(&project_dir, &branch, next_status).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn get_current_branch_cmd(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        // Walk up from the .ship dir to find the git repo root
        let git_root = project_dir.parent().unwrap_or(&project_dir).to_path_buf();
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&git_root)
            .output();
        match output {
            Ok(out) if out.status.success() => {
                let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if branch.is_empty() || branch == "HEAD" {
                    Ok(None)
                } else {
                    Ok(Some(branch))
                }
            }
            _ => Ok(None),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

// ─── Commands: Log ────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_events_cmd(
    since: Option<u64>,
    limit: Option<usize>,
    state: State<AppState>,
) -> Result<Vec<EventRecord>, String> {
    let project_dir = get_active_dir(&state)?;
    list_events_since(&project_dir, since.unwrap_or(0), limit).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn ingest_events_cmd(state: State<AppState>) -> Result<usize, String> {
    let project_dir = get_active_dir(&state)?;
    let events = ingest_external_events(&project_dir).map_err(|e| e.to_string())?;
    Ok(events.len())
}

#[tauri::command]
#[specta::specta]
fn get_log(state: State<AppState>) -> Result<Vec<LogEntry>, String> {
    let project_dir = get_active_dir(&state)?;
    read_log_entries(&project_dir).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn get_project_config(state: State<AppState>) -> Result<ProjectConfig, String> {
    let project_dir = get_active_dir(&state)?;
    get_config(Some(project_dir.clone())).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn save_project_config(config: ProjectConfig, state: State<AppState>) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    save_config(&config, Some(project_dir)).map_err(|e| e.to_string())
}

// ─── Commands: Settings ───────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn get_app_settings() -> Result<ProjectConfig, String> {
    get_config(None).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn save_app_settings(config: ProjectConfig) -> Result<(), String> {
    save_config(&config, None).map_err(|e| e.to_string())
}

// ─── Commands: Modes ──────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_modes_cmd(state: State<AppState>) -> Result<Vec<ModeConfig>, String> {
    let dir = get_active_dir(&state)?;
    let config = get_config(Some(dir)).map_err(|e| e.to_string())?;
    Ok(config.modes)
}

#[tauri::command]
#[specta::specta]
fn add_mode_cmd(mode: ModeConfig, state: State<AppState>) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    add_mode(Some(dir), mode).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn remove_mode_cmd(id: String, state: State<AppState>) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    remove_mode(Some(dir), &id).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn set_active_mode_cmd(id: Option<String>, state: State<AppState>) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    set_active_mode(Some(dir), id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn get_active_mode_cmd(state: State<AppState>) -> Result<Option<ModeConfig>, String> {
    let dir = get_active_dir(&state)?;
    get_active_mode(Some(dir)).map_err(|e| e.to_string())
}

// ─── Commands: MCP Servers ────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_mcp_servers_cmd(state: State<AppState>) -> Result<Vec<McpServerConfig>, String> {
    let dir = get_active_dir(&state)?;
    list_mcp_servers(Some(dir)).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn add_mcp_server_cmd(server: McpServerConfig, state: State<AppState>) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    add_mcp_server(Some(dir), server).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn remove_mcp_server_cmd(id: String, state: State<AppState>) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    remove_mcp_server(Some(dir), &id).map_err(|e| e.to_string())
}

// ─── Commands: Skills ─────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_skills_cmd(scope: Option<String>, state: State<AppState>) -> Result<Vec<Skill>, String> {
    let dir = get_active_dir(&state)?;
    match scope.as_deref() {
        Some("user") => list_user_skills().map_err(|e| e.to_string()),
        Some("project") => list_skills(&dir).map_err(|e| e.to_string()),
        _ => list_effective_skills(&dir).map_err(|e| e.to_string()),
    }
}

#[tauri::command]
#[specta::specta]
fn get_skill_cmd(
    id: String,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<Skill, String> {
    let dir = get_active_dir(&state)?;
    match scope.as_deref() {
        Some("user") => get_user_skill(&id).map_err(|e| e.to_string()),
        Some("project") => get_skill(&dir, &id).map_err(|e: anyhow::Error| e.to_string()),
        _ => get_effective_skill(&dir, &id).map_err(|e| e.to_string()),
    }
}

#[tauri::command]
#[specta::specta]
fn create_skill_cmd(
    id: String,
    name: String,
    content: String,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<Skill, String> {
    let dir = get_active_dir(&state)?;
    match scope.as_deref() {
        Some("user") => create_user_skill(&id, &name, &content).map_err(|e| e.to_string()),
        _ => create_skill(&dir, &id, &name, &content).map_err(|e| e.to_string()),
    }
}

#[tauri::command]
#[specta::specta]
fn update_skill_cmd(
    id: String,
    name: Option<String>,
    content: Option<String>,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<Skill, String> {
    let dir = get_active_dir(&state)?;
    match scope.as_deref() {
        Some("user") => {
            update_user_skill(&id, name.as_deref(), content.as_deref()).map_err(|e| e.to_string())
        }
        _ => {
            update_skill(&dir, &id, name.as_deref(), content.as_deref()).map_err(|e| e.to_string())
        }
    }
}

#[tauri::command]
#[specta::specta]
fn delete_skill_cmd(
    id: String,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    match scope.as_deref() {
        Some("user") => delete_user_skill(&id).map_err(|e| e.to_string()),
        _ => delete_skill(&dir, &id).map_err(|e| e.to_string()),
    }
}

// ─── Commands: Agents / Providers ─────────────────────────────────────────────

/// List all supported agent providers with enabled + installed status and known models.
#[tauri::command]
#[specta::specta]
fn list_providers_cmd(state: State<AppState>) -> Result<Vec<ProviderInfo>, String> {
    let dir = get_active_dir(&state)?;
    list_providers(&dir).map_err(|e| e.to_string())
}

/// Return the known models for a specific provider (static list).
#[tauri::command]
#[specta::specta]
fn list_models_cmd(provider_id: String) -> Result<Vec<ModelInfo>, String> {
    list_models(&provider_id).map_err(|e| e.to_string())
}

/// Return the resolved AgentConfig for the current branch/project state.
/// Pass the feature's agent config JSON string if on a feature branch; `null` otherwise.
#[tauri::command]
#[specta::specta]
fn get_agent_config_cmd(state: State<AppState>) -> Result<AgentConfig, String> {
    let ship_dir = get_active_dir(&state)?;
    // Resolve without a feature override — the UI can supply one separately if needed.
    resolve_agent_config(&ship_dir, None).map_err(|e| e.to_string())
}

// ─── Commands: Catalog ────────────────────────────────────────────────────────

/// Return all embedded catalog entries (skills + MCP servers).
#[tauri::command]
#[specta::specta]
fn list_catalog_cmd() -> Vec<CatalogEntry> {
    list_catalog()
}

/// Return catalog entries filtered by kind ("skill" or "mcp-server").
#[tauri::command]
#[specta::specta]
fn list_catalog_by_kind_cmd(kind: CatalogKind) -> Vec<CatalogEntry> {
    list_catalog_by_kind(kind)
}

/// Search the catalog by name, description, or tag.
#[tauri::command]
#[specta::specta]
fn search_catalog_cmd(query: String) -> Vec<CatalogEntry> {
    search_catalog(&query)
}

// ─── Commands: Agent Export ───────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
async fn export_agent_config_cmd(target: String, state: State<'_, AppState>) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    tokio::task::spawn_blocking(move || {
        runtime::agent_export::export_to(dir, &target).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

// ─── Commands: AI ─────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
async fn generate_issue_description_cmd(
    title: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let dir = get_active_dir(&state)?;
    let config = get_effective_config(Some(dir)).map_err(|e| e.to_string())?;
    let ai = config.ai.unwrap_or_default();
    let prompt = format!(
        "Write a concise issue description for a software task titled: \"{}\". \
         Return only the description body in markdown, no title or preamble.",
        title
    );
    tokio::task::spawn_blocking(move || invoke_ai_cli(&ai, &prompt))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn generate_adr_cmd(
    title: String,
    context: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let dir = get_active_dir(&state)?;
    let config = get_effective_config(Some(dir)).map_err(|e| e.to_string())?;
    let ai = config.ai.unwrap_or_default();
    let ctx = if context.trim().is_empty() {
        String::new()
    } else {
        format!(" Context: {}", context.trim())
    };
    let prompt = format!(
        "Write an Architecture Decision Record body for the decision: \"{}\".{} \
         Include sections: ## Status, ## Context, ## Decision, ## Consequences. \
         Return only the markdown body, no title or preamble.",
        title, ctx
    );
    tokio::task::spawn_blocking(move || invoke_ai_cli(&ai, &prompt))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn brainstorm_issues_cmd(
    topic: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let dir = get_active_dir(&state)?;
    let config = get_effective_config(Some(dir)).map_err(|e| e.to_string())?;
    let ai = config.ai.unwrap_or_default();
    let prompt = format!(
        "List 5 actionable software task titles for: \"{}\". \
         Return one task title per line, no numbering, bullets, or extra text.",
        topic
    );
    let output = tokio::task::spawn_blocking(move || invoke_ai_cli(&ai, &prompt))
        .await
        .map_err(|e| e.to_string())??;
    Ok(output
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}
#[tauri::command]
#[specta::specta]
async fn transform_text_cmd(
    instruction: String,
    text: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let dir = get_active_dir(&state)?;
    let config = get_effective_config(Some(dir)).map_err(|e| e.to_string())?;
    let ai = config.ai.unwrap_or_default();
    let prompt = format!(
        "Role: Senior Software Engineer & Technical Writer.\n\
         Task: {}.\n\n\
         Target Text:\n\
         ```\n\
         {}\n\
         ```\n\n\
         Constraint: Return ONLY the processed text with preserved or improved markdown structure. No preamble, conversational filler, or triple backticks unless required by the content.",
        instruction, text
    );
    tokio::task::spawn_blocking(move || invoke_ai_cli(&ai, &prompt))
        .await
        .map_err(|e| e.to_string())?
}

// ─── App Entry ────────────────────────────────────────────────────────────────

fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
        .events(tauri_specta::collect_events![ShipEvent])
        .commands(tauri_specta::collect_commands![
            // Project
            list_projects,
            get_active_project,
            set_active_project,
            pick_and_open_project,
            create_new_project,
            pick_project_directory,
            create_project_with_options,
            rename_project_cmd,
            detect_current_project,
            // Issues
            list_items,
            get_issue_by_path,
            create_new_issue,
            update_issue_command,
            move_issue_status,
            delete_issue_cmd,
            // ADRs
            list_adrs_cmd,
            create_new_adr,
            get_adr_cmd,
            update_adr_cmd,
            move_adr_cmd,
            delete_adr_cmd,
            // Specs
            list_specs_cmd,
            get_spec_cmd,
            create_spec_cmd,
            update_spec_cmd,
            delete_spec_cmd,
            // Releases
            list_releases_cmd,
            get_release_cmd,
            create_release_cmd,
            update_release_cmd,
            // Features
            list_features_cmd,
            get_feature_cmd,
            create_feature_cmd,
            update_feature_cmd,
            get_template_cmd,
            // Vision
            get_vision_cmd,
            update_vision_cmd,
            // Notes
            list_notes_cmd,
            get_note_cmd,
            create_note_cmd,
            update_note_cmd,
            delete_note_cmd,
            // Rules
            list_rules_cmd,
            get_rule_cmd,
            create_rule_cmd,
            update_rule_cmd,
            delete_rule_cmd,
            // Permissions
            get_permissions_cmd,
            save_permissions_cmd,
            // Workspace
            get_workspace_cmd,
            list_workspaces_cmd,
            sync_workspace_cmd,
            create_workspace_cmd,
            activate_workspace_cmd,
            transition_workspace_cmd,
            get_current_branch_cmd,
            // Log
            list_events_cmd,
            ingest_events_cmd,
            get_log,
            // Settings
            get_app_settings,
            get_project_config,
            save_project_config,
            save_app_settings,
            // Modes
            list_modes_cmd,
            add_mode_cmd,
            remove_mode_cmd,
            set_active_mode_cmd,
            get_active_mode_cmd,
            // MCP servers
            list_mcp_servers_cmd,
            add_mcp_server_cmd,
            remove_mcp_server_cmd,
            // Skills
            list_skills_cmd,
            get_skill_cmd,
            create_skill_cmd,
            update_skill_cmd,
            delete_skill_cmd,
            // Agents / Providers
            list_providers_cmd,
            list_models_cmd,
            get_agent_config_cmd,
            // Catalog
            list_catalog_cmd,
            list_catalog_by_kind_cmd,
            search_catalog_cmd,
            // Agent export
            export_agent_config_cmd,
            // AI
            generate_issue_description_cmd,
            generate_adr_cmd,
            brainstorm_issues_cmd,
            transform_text_cmd,
        ])
}

fn default_bindings_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts")
}

fn export_bindings_to(path: &Path) -> Result<(), String> {
    specta_builder()
        .export(
            specta_typescript::Typescript::default()
                .bigint(specta_typescript::BigIntExportBehavior::Number)
                .header(
                    "// @ts-nocheck\n// This file is auto-generated by tauri-specta. Do not edit manually.",
                ),
            path,
        )
        .map_err(|err| format!("Failed to export TypeScript bindings: {}", err))
}

pub fn export_bindings() -> Result<PathBuf, String> {
    let path = default_bindings_path();
    export_bindings_to(&path)?;
    Ok(path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();

    // In debug builds, regenerate src/bindings.ts automatically.
    #[cfg(debug_assertions)]
    if let Err(err) = export_bindings_to(&default_bindings_path()) {
        panic!("{}", err);
    }

    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_cli_attempts_codex_prefers_exec_then_prompt() {
        let attempts = ai_cli_attempts("codex", "hello");
        assert_eq!(
            attempts,
            vec![
                vec!["exec".to_string(), "hello".to_string()],
                vec!["-p".to_string(), "hello".to_string()],
                vec!["hello".to_string()],
            ]
        );
    }

    #[test]
    fn ai_cli_attempts_claude_uses_prompt_fallback() {
        let attempts = ai_cli_attempts("claude", "hello");
        assert_eq!(
            attempts,
            vec![
                vec!["-p".to_string(), "hello".to_string()],
                vec!["hello".to_string()],
            ]
        );
    }

    #[test]
    fn ai_cli_success_text_prefers_stdout_then_stderr() {
        let stdout = ai_cli_success_text(b"  primary output  ", b"secondary");
        assert_eq!(stdout.as_deref(), Some("primary output"));

        let stderr = ai_cli_success_text(b"   ", b" fallback stderr ");
        assert_eq!(stderr.as_deref(), Some("fallback stderr"));

        let none = ai_cli_success_text(b"", b"");
        assert!(none.is_none());
    }
}
