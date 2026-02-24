use logic::config::{
    generate_gitignore, get_config, save_config, AiConfig, McpServerConfig, ModeConfig,
    ProjectConfig, ProjectDiscovery,
    add_mode, remove_mode, set_active_mode, get_active_mode,
    add_mcp_server, remove_mcp_server, list_mcp_servers,
};
use logic::{
    create_adr, create_issue, create_spec, delete_issue, get_issue, get_project_dir,
    get_project_name, get_spec_raw as get_spec_content, init_project, list_adrs, list_issues_full,
    list_specs, log_action, move_issue, read_log_entries, register_project, update_issue,
    update_spec, ADR, AdrEntry, Issue, IssueEntry, LogEntry, SHIP_DIR_NAME,
    list_registered_projects,
};
use logic::project::{get_active_project_global, set_active_project_global};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, State};
use tauri_plugin_dialog::DialogExt;

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

fn get_active_dir(state: &State<AppState>) -> Result<PathBuf, String> {
    let guard = state.active_project.lock().unwrap();
    guard
        .as_ref()
        .cloned()
        .ok_or_else(|| "No active project".to_string())
}

fn ensure_ship_path(path: &Path) -> PathBuf {
    if path.file_name().map(|name| name == SHIP_DIR_NAME).unwrap_or(false) {
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
        .find_map(|line| line.strip_prefix("# ").map(|value| value.trim().to_string()))
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

// ─── AI helper ────────────────────────────────────────────────────────────────

fn invoke_ai_cli(ai: &AiConfig, prompt: &str) -> Result<String, String> {
    let cli = ai.effective_cli().to_string();
    let mut cmd = std::process::Command::new(&cli);
    match ai.effective_provider() {
        "codex" => { cmd.arg("exec").arg(prompt); }
        _ => { cmd.arg("-p").arg(prompt); }
    }
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to launch '{}': {}", cli, e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("AI CLI error: {}", stderr.trim()))
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

/// Auto-detect current project from the working directory (for dogfooding).
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

    let mut final_config = config.unwrap_or_else(|| {
        get_config(Some(ship_path.clone())).unwrap_or_default()
    });

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
        let issues_dir = ship_root.join("issues");
        let specs_dir = ship_root.join("specs");
        let log_file = ship_root.join("log.md");
        let config_file = ship_root.join("config.toml");

        let mut last_issues = issues_signature(&issues_dir);
        let mut last_specs = issues_signature(&specs_dir);
        let mut last_log = file_signature(&log_file);
        let mut last_config = file_signature(&config_file);

        loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }
            thread::sleep(Duration::from_millis(250));

            let next_issues = issues_signature(&issues_dir);
            if next_issues != last_issues {
                let _ = app_handle.emit("ship://issues-changed", ());
                last_issues = next_issues;
            }

            let next_specs = issues_signature(&specs_dir);
            if next_specs != last_specs {
                let _ = app_handle.emit("ship://issues-changed", ());
                last_specs = next_specs;
            }

            let next_log = file_signature(&log_file);
            if next_log != last_log {
                let _ = app_handle.emit("ship://log-changed", ());
                last_log = next_log;
            }

            let next_config = file_signature(&config_file);
            if next_config != last_config {
                let _ = app_handle.emit("ship://config-changed", ());
                last_config = next_config;
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

    let issue_path = project_dir
        .join("issues")
        .join(&from_status)
        .join(&file_name);
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

    let adr_data = logic::get_adr(path.clone()).map_err(|e| e.to_string())?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    Ok(AdrEntry {
        file_name,
        path: path.to_string_lossy().to_string(),
        adr: adr_data,
    })
}

#[tauri::command]
#[specta::specta]
fn get_adr_cmd(file_name: String, state: State<AppState>) -> Result<AdrEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let path = project_dir.join("adrs").join(&file_name);
    if !path.exists() {
        return Err(format!("ADR not found: {}", file_name));
    }
    let adr = logic::get_adr(path.clone()).map_err(|e| e.to_string())?;
    Ok(AdrEntry {
        file_name,
        path: path.to_string_lossy().to_string(),
        adr,
    })
}

#[tauri::command]
#[specta::specta]
fn update_adr_cmd(file_name: String, adr: ADR, state: State<AppState>) -> Result<AdrEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let path = project_dir.join("adrs").join(&file_name);
    if !path.exists() {
        return Err(format!("ADR not found: {}", file_name));
    }
    let content = adr.to_markdown().map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    log_action(
        project_dir,
        "adr update",
        &format!("Updated ADR: {}", file_name),
    )
    .ok();
    let refreshed = logic::get_adr(path.clone()).map_err(|e| e.to_string())?;
    Ok(AdrEntry {
        file_name,
        path: path.to_string_lossy().to_string(),
        adr: refreshed,
    })
}

#[tauri::command]
#[specta::specta]
fn delete_adr_cmd(file_name: String, state: State<AppState>) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    let path = project_dir.join("adrs").join(&file_name);
    if !path.exists() {
        return Err(format!("ADR not found: {}", file_name));
    }
    fs::remove_file(&path).map_err(|e| e.to_string())?;
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
    let path = project_dir.join("specs").join(&file_name);
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
    let path = create_spec(project_dir.clone(), &title, &content).map_err(|e| e.to_string())?;
    log_action(project_dir, "spec create", &format!("Created Spec: {}", title)).ok();
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
    let path = project_dir.join("specs").join(&file_name);
    if !path.exists() {
        return Err(format!("Spec not found: {}", file_name));
    }
    update_spec(path.clone(), &content).map_err(|e| e.to_string())?;
    log_action(project_dir, "spec update", &format!("Updated Spec: {}", file_name)).ok();
    spec_document_from_path(path)
}

#[tauri::command]
#[specta::specta]
fn delete_spec_cmd(file_name: String, state: State<AppState>) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    let path = project_dir.join("specs").join(&file_name);
    if !path.exists() {
        return Err(format!("Spec not found: {}", file_name));
    }
    fs::remove_file(&path).map_err(|e| e.to_string())?;
    log_action(project_dir, "spec delete", &format!("Deleted Spec: {}", file_name)).ok();
    Ok(())
}

// ─── Commands: Log ────────────────────────────────────────────────────────────

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

// ─── Commands: AI ─────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
async fn generate_issue_description_cmd(
    title: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let dir = get_active_dir(&state)?;
    let config = get_config(Some(dir)).map_err(|e| e.to_string())?;
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
    let config = get_config(Some(dir)).map_err(|e| e.to_string())?;
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
    let config = get_config(Some(dir)).map_err(|e| e.to_string())?;
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
    let builder = tauri_specta::Builder::<tauri::Wry>::new().commands(
        tauri_specta::collect_commands![
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
            // Log
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
            // AI
            generate_issue_description_cmd,
            generate_adr_cmd,
            brainstorm_issues_cmd,
        ],
    );

    // In debug builds, regenerate src/bindings.ts automatically.
    #[cfg(debug_assertions)]
    let bindings_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts");

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default()
                .bigint(specta_typescript::BigIntExportBehavior::Number)
                .header("// This file is auto-generated by tauri-specta. Do not edit manually."),
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
