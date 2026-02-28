use runtime::config::{
    add_mcp_server, add_mode, generate_gitignore, get_active_mode, get_config,
    get_effective_config, list_mcp_servers, remove_mcp_server, remove_mode, save_config,
    set_active_mode, AiConfig, McpServerConfig, ModeConfig, ProjectConfig, ProjectDiscovery,
};
use runtime::project::{
    adrs_dir, features_dir, get_active_project_global, issues_dir, releases_dir,
    set_active_project_global, specs_dir,
};
use runtime::{
    create_adr, create_feature, create_issue, create_prompt, create_release, create_skill,
    create_spec, create_user_skill, delete_adr, delete_issue, delete_prompt, delete_skill,
    delete_spec, delete_user_skill, get_effective_skill, get_feature,
    get_feature_raw as get_feature_content, get_issue, get_project_dir, get_project_name,
    get_prompt, get_release, get_release_raw as get_release_content, get_skill,
    get_spec_raw as get_spec_content, get_user_skill, get_workspace, ingest_external_events,
    init_project, list_adrs, list_catalog, list_catalog_by_kind, list_effective_skills,
    list_events_since, list_features, list_issues_full, list_models, list_prompts, list_providers,
    list_registered_projects, list_releases, list_skills, list_specs, list_user_skills, log_action,
    move_issue, read_log_entries, read_template, register_project, resolve_agent_config,
    search_catalog, update_adr, update_feature, update_issue, update_prompt, update_release,
    update_skill, update_spec, update_user_skill, AdrEntry, AgentConfig, CatalogEntry, CatalogKind,
    EventRecord, FeatureStatus, Issue, IssueEntry, LogEntry, ModelInfo, Prompt, ProviderInfo,
    ReleaseStatus, Skill, Workspace, ADR, SHIP_DIR_NAME,
};
use serde::{Deserialize, Serialize};
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
pub struct ReleaseInfo {
    pub file_name: String,
    pub version: String,
    pub status: ReleaseStatus,
    pub path: String,
    pub updated: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ReleaseDocument {
    pub file_name: String,
    pub version: String,
    pub status: ReleaseStatus,
    pub path: String,
    pub updated: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureInfo {
    pub file_name: String,
    pub title: String,
    pub status: FeatureStatus,
    pub release_id: Option<String>,
    pub spec_id: Option<String>,
    pub branch: Option<String>,
    pub description: Option<String>,
    pub path: String,
    pub updated: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureDocument {
    pub file_name: String,
    pub title: String,
    pub status: FeatureStatus,
    pub release_id: Option<String>,
    pub spec_id: Option<String>,
    pub branch: Option<String>,
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
    let ship_path = ensure_ship_path(registered_path);
    let root = ship_path.parent().unwrap_or(&ship_path);
    cwd.starts_with(root)
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

fn derive_spec_title(file_name: &str, content: &str) -> String {
    content
        .lines()
        .find_map(|line| {
            line.strip_prefix("# ")
                .map(|value| value.trim().to_string())
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            Path::new(file_name)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or(file_name)
                .replace('-', " ")
        })
}

fn spec_document_from_path(path: PathBuf) -> Result<SpecDocument, String> {
    let content = get_spec_content(path.clone()).map_err(|e| e.to_string())?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    Ok(SpecDocument {
        title: derive_spec_title(&file_name, &content),
        file_name,
        path: path.to_string_lossy().to_string(),
        content,
    })
}

fn release_document_from_path(path: PathBuf) -> Result<ReleaseDocument, String> {
    let release = get_release(path.clone()).map_err(|e| e.to_string())?;
    let content = get_release_content(path.clone()).map_err(|e| e.to_string())?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    Ok(ReleaseDocument {
        file_name,
        version: release.metadata.version,
        status: release.metadata.status.clone(),
        path: path.to_string_lossy().to_string(),
        updated: release.metadata.updated,
        content,
    })
}

fn feature_document_from_path(path: PathBuf) -> Result<FeatureDocument, String> {
    let feature = get_feature(path.clone()).map_err(|e| e.to_string())?;
    let content = get_feature_content(path.clone()).map_err(|e| e.to_string())?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    Ok(FeatureDocument {
        file_name,
        title: feature.metadata.title,
        status: FeatureStatus::from_path(&path),
        release_id: feature.metadata.release_id,
        spec_id: feature.metadata.spec_id,
        branch: feature.metadata.branch,
        description: feature.metadata.description,
        path: path.to_string_lossy().to_string(),
        updated: feature.metadata.updated,
        content,
    })
}

// ─── AI helper ────────────────────────────────────────────────────────────────

fn invoke_ai_cli(ai: &AiConfig, prompt: &str) -> Result<String, String> {
    let cli = ai.effective_cli().to_string();
    let provider = ai.effective_provider().to_ascii_lowercase();
    let attempts: Vec<Vec<String>> = match provider.as_str() {
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
    };

    let mut last_error = String::new();
    for args in attempts {
        let output = std::process::Command::new(&cli)
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to launch '{}': {}", cli, e))?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }
        last_error = String::from_utf8_lossy(&output.stderr).trim().to_string();
    }

    if last_error.is_empty() {
        Err("AI CLI failed with no error output".to_string())
    } else {
        Err(format!("AI CLI error: {}", last_error))
    }
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
            let issue_count = list_issues_full(ship_path.clone())
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
                    let issues = list_issues_full(path.clone()).unwrap_or_default();
                    return Ok(Some(ProjectInfo {
                        name: get_project_name(&path),
                        path: path.to_string_lossy().to_string(),
                        issue_count: issues.len(),
                    }));
                }
            }
            Ok(None)
        }
        Some(path) => {
            let issues = list_issues_full(path.clone()).unwrap_or_default();
            Ok(Some(ProjectInfo {
                name: get_project_name(path),
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
    let issues = list_issues_full(ship_path.clone()).unwrap_or_default();
    let info = ProjectInfo {
        name: get_project_name(&ship_path),
        path: ship_path.to_string_lossy().to_string(),
        issue_count: issues.len(),
    };
    *state.active_project.lock().unwrap() = Some(ship_path.clone());
    register_project(get_project_name(&ship_path), ship_path.clone()).map_err(|e| e.to_string())?;
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

    let issues = list_issues_full(final_ship_path.clone()).unwrap_or_default();
    let info = ProjectInfo {
        name: get_project_name(&final_ship_path),
        path: final_ship_path.to_string_lossy().to_string(),
        issue_count: issues.len(),
    };
    *state.active_project.lock().unwrap() = Some(final_ship_path.clone());
    register_project(get_project_name(&final_ship_path), final_ship_path.clone())
        .map_err(|e| e.to_string())?;
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
            let issues = list_issues_full(ship_path.clone()).unwrap_or_default();
            let info = ProjectInfo {
                name: get_project_name(&ship_path),
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
            let issues = list_issues_full(ship_path.clone()).unwrap_or_default();
            let info = ProjectInfo {
                name: get_project_name(&ship_path),
                path: ship_path.to_string_lossy().to_string(),
                issue_count: issues.len(),
            };
            // Also set as active
            *state.active_project.lock().unwrap() = Some(ship_path.clone());
            register_project(get_project_name(&ship_path), ship_path.clone())
                .map_err(|e| e.to_string())?;
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

    let issues = list_issues_full(ship_path.clone()).unwrap_or_default();
    let info = ProjectInfo {
        name: get_project_name(&ship_path),
        path: ship_path.to_string_lossy().to_string(),
        issue_count: issues.len(),
    };
    *state.active_project.lock().unwrap() = Some(ship_path.clone());
    register_project(get_project_name(&ship_path), ship_path.clone()).map_err(|e| e.to_string())?;
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

    let issues = list_issues_full(ship_path.clone()).unwrap_or_default();
    let info = ProjectInfo {
        name: display_name.clone(),
        path: ship_path.to_string_lossy().to_string(),
        issue_count: issues.len(),
    };

    *state.active_project.lock().unwrap() = Some(ship_path.clone());
    register_project(display_name, ship_path.clone()).map_err(|e| e.to_string())?;

    if let Err(err) = start_project_watcher(&app, &state, &ship_path) {
        eprintln!("Failed to start project watcher: {}", err);
    }

    set_active_project_global(ship_path).map_err(|e| e.to_string())?;
    Ok(info)
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
        let events_file = ship_root.join("events.ndjson");
        let config_file = ship_root.join(runtime::config::PRIMARY_CONFIG_FILE);

        let mut last_issues = issues_signature(&issues_dir);
        let mut last_specs = issues_signature(&specs_dir);
        let mut last_adrs = issues_signature(&adrs_dir);
        let mut last_features = issues_signature(&features_dir);
        let mut last_releases = issues_signature(&releases_dir);
        let mut last_events = file_signature(&events_file);
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

            let next_events = file_signature(&events_file);
            if next_events != last_events {
                let _ = ShipEvent::EventsChanged.emit(&app_handle);
                let _ = ShipEvent::LogChanged.emit(&app_handle);
                last_events = next_events;
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
    list_issues_full(project_dir).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn get_issue_by_path(path: String) -> Result<Issue, String> {
    get_issue(PathBuf::from(path)).map_err(|e| e.to_string())
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

    let path = create_issue(project_dir.clone(), &title, &description, &status)
        .map_err(|e| e.to_string())?;
    if assignee.is_some() || tags.is_some() {
        let mut issue = get_issue(path.clone()).map_err(|e| e.to_string())?;
        issue.metadata.assignee = assignee;
        issue.metadata.tags = tags.unwrap_or_default();
        update_issue(path.clone(), issue).map_err(|e| e.to_string())?;
    }
    log_action(project_dir, "issue create", &format!("Created: {}", title)).ok();

    let issue = get_issue(path.clone()).map_err(|e| e.to_string())?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    Ok(IssueEntry {
        file_name,
        status,
        path: path.to_string_lossy().to_string(),
        issue,
    })
}

#[tauri::command]
#[specta::specta]
fn update_issue_by_path(path: String, issue: Issue) -> Result<(), String> {
    update_issue(PathBuf::from(&path), issue.clone()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn move_issue_status(
    file_name: String,
    from_status: String,
    to_status: String,
    state: State<AppState>,
) -> Result<IssueEntry, String> {
    let project_dir = get_active_dir(&state)?;

    let issue_path = issues_dir(&project_dir).join(&from_status).join(&file_name);
    let new_path = move_issue(project_dir.clone(), issue_path, &from_status, &to_status)
        .map_err(|e| e.to_string())?;
    log_action(
        project_dir,
        "issue move",
        &format!("Moved {} → {}", file_name, to_status),
    )
    .ok();

    let issue = get_issue(new_path.clone()).map_err(|e| e.to_string())?;
    Ok(IssueEntry {
        file_name,
        status: to_status,
        path: new_path.to_string_lossy().to_string(),
        issue,
    })
}

#[tauri::command]
#[specta::specta]
fn delete_issue_by_path(path: String, state: State<AppState>) -> Result<(), String> {
    let guard = state.active_project.lock().unwrap();
    let project_dir = guard.as_ref().cloned();
    drop(guard);

    let p = PathBuf::from(&path);
    let name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    delete_issue(p).map_err(|e| e.to_string())?;
    if let Some(dir) = project_dir {
        log_action(dir, "issue delete", &format!("Deleted: {}", name)).ok();
    }
    Ok(())
}

// ─── Commands: ADRs ───────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_adrs_cmd(state: State<AppState>) -> Result<Vec<AdrEntry>, String> {
    let project_dir = get_active_dir(&state)?;
    list_adrs(project_dir).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn create_new_adr(
    title: String,
    decision: String,
    state: State<AppState>,
) -> Result<AdrEntry, String> {
    let project_dir = get_active_dir(&state)?;

    let path = create_adr(project_dir.clone(), &title, &decision, "accepted")
        .map_err(|e| e.to_string())?;
    log_action(
        project_dir,
        "adr create",
        &format!("Created ADR: {}", title),
    )
    .ok();

    let adr_data = runtime::get_adr(path.clone()).map_err(|e| e.to_string())?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let status_str = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("proposed");
    let status = status_str.parse::<runtime::AdrStatus>().unwrap_or_default();
    Ok(AdrEntry {
        file_name,
        path: path.to_string_lossy().to_string(),
        status,
        adr: adr_data,
    })
}

#[tauri::command]
#[specta::specta]
fn get_adr_cmd(file_name: String, state: State<AppState>) -> Result<AdrEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let path = runtime::find_adr_path(&project_dir, &file_name).map_err(|e| e.to_string())?;
    let adr = runtime::get_adr(path.clone()).map_err(|e| e.to_string())?;
    let status_str = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("proposed");
    let status = status_str.parse::<runtime::AdrStatus>().unwrap_or_default();
    Ok(AdrEntry {
        file_name,
        path: path.to_string_lossy().to_string(),
        status,
        adr,
    })
}

#[tauri::command]
#[specta::specta]
fn update_adr_cmd(file_name: String, adr: ADR, state: State<AppState>) -> Result<AdrEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let path = runtime::find_adr_path(&project_dir, &file_name).map_err(|e| e.to_string())?;
    update_adr(path.clone(), adr).map_err(|e| e.to_string())?;
    log_action(
        project_dir,
        "adr update",
        &format!("Updated ADR: {}", file_name),
    )
    .ok();
    let refreshed = runtime::get_adr(path.clone()).map_err(|e| e.to_string())?;
    let status_str = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("proposed");
    let status = status_str.parse::<runtime::AdrStatus>().unwrap_or_default();
    Ok(AdrEntry {
        file_name,
        path: path.to_string_lossy().to_string(),
        status,
        adr: refreshed,
    })
}

#[tauri::command]
#[specta::specta]
fn delete_adr_cmd(file_name: String, state: State<AppState>) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    let path = adrs_dir(&project_dir).join(&file_name);
    if !path.exists() {
        return Err(format!("ADR not found: {}", file_name));
    }
    delete_adr(path).map_err(|e| e.to_string())?;
    log_action(
        project_dir,
        "adr delete",
        &format!("Deleted ADR: {}", file_name),
    )
    .ok();
    Ok(())
}

// ─── Commands: Specs ─────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_specs_cmd(state: State<AppState>) -> Result<Vec<SpecInfo>, String> {
    let project_dir = get_active_dir(&state)?;
    let entries = list_specs(project_dir).map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .map(|entry| SpecInfo {
            file_name: entry.file_name,
            title: entry.title,
            path: entry.path,
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
fn get_spec_cmd(file_name: String, state: State<AppState>) -> Result<SpecDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let path = specs_dir(&project_dir).join(&file_name);
    if !path.exists() {
        return Err(format!("Spec not found: {}", file_name));
    }
    spec_document_from_path(path)
}

#[tauri::command]
#[specta::specta]
fn create_spec_cmd(
    title: String,
    content: String,
    state: State<AppState>,
) -> Result<SpecDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let path =
        create_spec(project_dir.clone(), &title, &content, "draft").map_err(|e| e.to_string())?;
    log_action(
        project_dir,
        "spec create",
        &format!("Created Spec: {}", title),
    )
    .ok();
    spec_document_from_path(path)
}

#[tauri::command]
#[specta::specta]
fn update_spec_cmd(
    file_name: String,
    content: String,
    state: State<AppState>,
) -> Result<SpecDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let path = specs_dir(&project_dir).join(&file_name);
    if !path.exists() {
        return Err(format!("Spec not found: {}", file_name));
    }
    update_spec(path.clone(), &content).map_err(|e| e.to_string())?;
    log_action(
        project_dir,
        "spec update",
        &format!("Updated Spec: {}", file_name),
    )
    .ok();
    spec_document_from_path(path)
}

#[tauri::command]
#[specta::specta]
fn delete_spec_cmd(file_name: String, state: State<AppState>) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    let path = specs_dir(&project_dir).join(&file_name);
    if !path.exists() {
        return Err(format!("Spec not found: {}", file_name));
    }
    delete_spec(path).map_err(|e| e.to_string())?;
    log_action(
        project_dir,
        "spec delete",
        &format!("Deleted Spec: {}", file_name),
    )
    .ok();
    Ok(())
}

// ─── Commands: Releases ──────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_releases_cmd(state: State<AppState>) -> Result<Vec<ReleaseInfo>, String> {
    let project_dir = get_active_dir(&state)?;
    let entries = list_releases(project_dir).map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .map(|entry| ReleaseInfo {
            file_name: entry.file_name,
            version: entry.version,
            status: entry.status,
            path: entry.path,
            updated: entry.updated,
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
fn get_release_cmd(file_name: String, state: State<AppState>) -> Result<ReleaseDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let path = releases_dir(&project_dir).join(&file_name);
    if !path.exists() {
        return Err(format!("Release not found: {}", file_name));
    }
    release_document_from_path(path)
}

#[tauri::command]
#[specta::specta]
fn create_release_cmd(
    version: String,
    content: String,
    state: State<AppState>,
) -> Result<ReleaseDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let path =
        create_release(project_dir.clone(), &version, &content).map_err(|e| e.to_string())?;
    log_action(
        project_dir,
        "release create",
        &format!("Created Release: {}", version),
    )
    .ok();
    release_document_from_path(path)
}

#[tauri::command]
#[specta::specta]
fn update_release_cmd(
    file_name: String,
    content: String,
    state: State<AppState>,
) -> Result<ReleaseDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let path = releases_dir(&project_dir).join(&file_name);
    if !path.exists() {
        return Err(format!("Release not found: {}", file_name));
    }
    update_release(path.clone(), &content).map_err(|e| e.to_string())?;
    log_action(
        project_dir,
        "release update",
        &format!("Updated Release: {}", file_name),
    )
    .ok();
    release_document_from_path(path)
}

// ─── Commands: Features ──────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_features_cmd(state: State<AppState>) -> Result<Vec<FeatureInfo>, String> {
    let project_dir = get_active_dir(&state)?;
    let entries = list_features(project_dir, None).map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .map(|entry| FeatureInfo {
            file_name: entry.file_name,
            title: entry.title,
            status: entry.status,
            release_id: entry.release_id,
            spec_id: entry.spec_id,
            branch: entry.branch,
            description: entry.description,
            path: entry.path,
            updated: entry.updated,
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
fn get_feature_cmd(file_name: String, state: State<AppState>) -> Result<FeatureDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let path = features_dir(&project_dir).join(&file_name);
    if !path.exists() {
        return Err(format!("Feature not found: {}", file_name));
    }
    feature_document_from_path(path)
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
    let path = create_feature(
        project_dir.clone(),
        &title,
        &content,
        release.as_deref(),
        spec.as_deref(),
        None,
    )
    .map_err(|e| e.to_string())?;
    log_action(
        project_dir,
        "feature create",
        &format!("Created Feature: {}", title),
    )
    .ok();
    feature_document_from_path(path)
}

#[tauri::command]
#[specta::specta]
fn update_feature_cmd(
    file_name: String,
    content: String,
    state: State<AppState>,
) -> Result<FeatureDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let path = features_dir(&project_dir).join(&file_name);
    if !path.exists() {
        return Err(format!("Feature not found: {}", file_name));
    }
    update_feature(path.clone(), &content).map_err(|e| e.to_string())?;
    log_action(
        project_dir,
        "feature update",
        &format!("Updated Feature: {}", file_name),
    )
    .ok();
    feature_document_from_path(path)
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
fn get_vision_cmd(state: State<AppState>) -> Result<runtime::vision::Vision, String> {
    let project_dir = get_active_dir(&state)?;
    runtime::get_vision(project_dir).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn update_vision_cmd(content: String, state: State<AppState>) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    let vision = runtime::vision::Vision::from_markdown(&content).map_err(|e| e.to_string())?;
    runtime::update_vision(project_dir, &vision).map_err(|e| e.to_string())
}

// ─── Commands: Notes ──────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct NoteInfo {
    pub file_name: String,
    pub title: String,
    pub path: String,
    pub updated: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct NoteDocument {
    pub file_name: String,
    pub title: String,
    pub path: String,
    pub updated: String,
    pub content: String,
}

#[tauri::command]
#[specta::specta]
fn list_notes_cmd(state: State<AppState>) -> Result<Vec<NoteInfo>, String> {
    let project_dir = get_active_dir(&state)?;
    let entries = runtime::list_notes(runtime::NoteScope::Project, Some(project_dir))
        .map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .map(|e| NoteInfo {
            file_name: e.file_name,
            title: e.title,
            path: e.path,
            updated: e.updated,
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
fn get_note_cmd(file_name: String, state: State<AppState>) -> Result<NoteDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let path = runtime::project::notes_dir(&project_dir).join(&file_name);
    let note = runtime::get_note(path.clone()).map_err(|e| e.to_string())?;
    Ok(NoteDocument {
        file_name,
        title: note.metadata.title,
        path: path.to_string_lossy().to_string(),
        updated: note.metadata.updated,
        content: note.body,
    })
}

#[tauri::command]
#[specta::specta]
fn create_note_cmd(
    title: String,
    content: String,
    state: State<AppState>,
) -> Result<NoteDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let path = runtime::create_note(
        runtime::NoteScope::Project,
        Some(project_dir.clone()),
        &title,
        &content,
    )
    .map_err(|e| e.to_string())?;
    let note = runtime::get_note(path.clone()).map_err(|e| e.to_string())?;
    Ok(NoteDocument {
        file_name: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
        title: note.metadata.title,
        path: path.to_string_lossy().to_string(),
        updated: note.metadata.updated,
        content: note.body,
    })
}

#[tauri::command]
#[specta::specta]
fn update_note_cmd(
    file_name: String,
    content: String,
    state: State<AppState>,
) -> Result<NoteDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let path = runtime::project::notes_dir(&project_dir).join(&file_name);
    runtime::update_note(path.clone(), &content).map_err(|e| e.to_string())?;
    let note = runtime::get_note(path.clone()).map_err(|e| e.to_string())?;
    Ok(NoteDocument {
        file_name,
        title: note.metadata.title,
        path: path.to_string_lossy().to_string(),
        updated: note.metadata.updated,
        content: note.body,
    })
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
fn get_workspace_cmd(branch: String, state: State<AppState>) -> Result<Option<Workspace>, String> {
    let project_dir = get_active_dir(&state)?;
    get_workspace(&project_dir, &branch).map_err(|e| e.to_string())
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
    read_log_entries(project_dir).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn get_project_config(state: State<AppState>) -> Result<ProjectConfig, String> {
    let project_dir = get_active_dir(&state)?;
    get_config(Some(project_dir)).map_err(|e| e.to_string())
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
        Some("project") => get_skill(&dir, &id).map_err(|e| e.to_string()),
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

// ─── Commands: Prompts ────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn list_prompts_cmd(state: State<AppState>) -> Result<Vec<Prompt>, String> {
    let dir = get_active_dir(&state)?;
    list_prompts(&dir).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn get_prompt_cmd(id: String, state: State<AppState>) -> Result<Prompt, String> {
    let dir = get_active_dir(&state)?;
    get_prompt(&dir, &id).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn create_prompt_cmd(
    id: String,
    name: String,
    content: String,
    state: State<AppState>,
) -> Result<Prompt, String> {
    let dir = get_active_dir(&state)?;
    create_prompt(&dir, &id, &name, &content).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn update_prompt_cmd(
    id: String,
    name: Option<String>,
    content: Option<String>,
    state: State<AppState>,
) -> Result<Prompt, String> {
    let dir = get_active_dir(&state)?;
    update_prompt(&dir, &id, name.as_deref(), content.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn delete_prompt_cmd(id: String, state: State<AppState>) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    delete_prompt(&dir, &id).map_err(|e| e.to_string())
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

// ─── App Entry ────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<tauri::Wry>::new()
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
            detect_current_project,
            // Issues
            list_items,
            get_issue_by_path,
            create_new_issue,
            update_issue_by_path,
            move_issue_status,
            delete_issue_by_path,
            // ADRs
            list_adrs_cmd,
            create_new_adr,
            get_adr_cmd,
            update_adr_cmd,
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
            // Prompts
            list_prompts_cmd,
            get_prompt_cmd,
            create_prompt_cmd,
            update_prompt_cmd,
            delete_prompt_cmd,
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
        ]);

    // In debug builds, regenerate src/bindings.ts automatically.
    #[cfg(debug_assertions)]
    let bindings_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts");

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default()
                .bigint(specta_typescript::BigIntExportBehavior::Number)
                .header(
                    "// @ts-nocheck\n// This file is auto-generated by tauri-specta. Do not edit manually."
                ),
            &bindings_path,
        )
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
