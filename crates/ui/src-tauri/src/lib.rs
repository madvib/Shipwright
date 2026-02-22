use logic::config::{discover_projects, get_config, save_config, Config, ProjectDiscovery};
use logic::{
    create_adr, create_issue, delete_issue, get_issue, get_project_dir, get_project_name,
    init_project, list_adrs, list_issues_full, log_action, move_issue, read_log_entries,
    update_issue, AdrEntry, Issue, IssueEntry, LogEntry,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

// ─── App State ────────────────────────────────────────────────────────────────

/// Holds the currently active project directory (the `.ship` dir path).
#[derive(Default)]
pub struct AppState {
    pub active_project: Mutex<Option<PathBuf>>,
}

// ─── Project Info ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub issue_count: usize,
}

// ─── Commands: Project ────────────────────────────────────────────────────────

#[tauri::command]
fn list_projects() -> Result<Vec<ProjectDiscovery>, String> {
    let home = dirs::home_dir().unwrap_or_default();
    let current = std::env::current_dir().unwrap_or(home.clone());

    let mut projects = Vec::new();

    // Discover in current dir
    if let Ok(mut found) = discover_projects(current) {
        projects.append(&mut found);
    }

    // Also discover in home dir if different
    if let Ok(found) = discover_projects(home) {
        for p in found {
            if !projects.iter().any(|x: &ProjectDiscovery| x.path == p.path) {
                projects.push(p);
            }
        }
    }

    Ok(projects)
}

#[tauri::command]
fn get_active_project(state: State<AppState>) -> Result<Option<ProjectInfo>, String> {
    let guard = state.active_project.lock().unwrap();
    match &*guard {
        None => Ok(None),
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
fn set_active_project(path: String, state: State<AppState>) -> Result<ProjectInfo, String> {
    let ship_path = PathBuf::from(&path);
    if !ship_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    let issues = list_issues_full(ship_path.clone()).unwrap_or_default();
    let info = ProjectInfo {
        name: get_project_name(&ship_path),
        path: path.clone(),
        issue_count: issues.len(),
    };
    *state.active_project.lock().unwrap() = Some(ship_path);
    Ok(info)
}

/// Opens a folder picker. If the chosen directory has no .ship, initialises one.
/// Sets the result as the active project.
#[tauri::command]
async fn pick_and_open_project(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<ProjectInfo, String> {
    let picked = app.dialog().file().blocking_pick_folder();
    let base_dir = match picked {
        Some(p) => p
            .as_path()
            .ok_or_else(|| "Invalid path".to_string())?
            .to_path_buf(),
        None => return Err("No directory selected".to_string()),
    };

    let ship_path = base_dir.join(".ship");
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
    *state.active_project.lock().unwrap() = Some(final_ship_path);
    Ok(info)
}

/// Auto-detect current project from the working directory (for dogfooding).
#[tauri::command]
fn detect_current_project(state: State<AppState>) -> Result<Option<ProjectInfo>, String> {
    match get_project_dir(None) {
        Ok(ship_path) => {
            let issues = list_issues_full(ship_path.clone()).unwrap_or_default();
            let info = ProjectInfo {
                name: get_project_name(&ship_path),
                path: ship_path.to_string_lossy().to_string(),
                issue_count: issues.len(),
            };
            // Also set as active
            *state.active_project.lock().unwrap() = Some(ship_path);
            Ok(Some(info))
        }
        Err(_) => Ok(None),
    }
}

// ─── Commands: Issues ─────────────────────────────────────────────────────────

#[tauri::command]
fn list_items(state: State<AppState>) -> Result<Vec<IssueEntry>, String> {
    let guard = state.active_project.lock().unwrap();
    let project_dir = guard
        .as_ref()
        .ok_or_else(|| "No active project".to_string())?
        .clone();
    drop(guard);
    list_issues_full(project_dir).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_issue_by_path(path: String) -> Result<Issue, String> {
    get_issue(PathBuf::from(path)).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_new_issue(
    title: String,
    description: String,
    status: String,
    state: State<AppState>,
) -> Result<IssueEntry, String> {
    let guard = state.active_project.lock().unwrap();
    let project_dir = guard
        .as_ref()
        .ok_or_else(|| "No active project".to_string())?
        .clone();
    drop(guard);

    let path = create_issue(project_dir.clone(), &title, &description, &status)
        .map_err(|e| e.to_string())?;
    log_action(project_dir, "issue create", &format!("Created: {}", title)).ok();

    let issue = get_issue(path.clone()).map_err(|e| e.to_string())?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    Ok(IssueEntry {
        file_name,
        status: issue.metadata.status.clone(),
        path: path.to_string_lossy().to_string(),
        issue,
    })
}

#[tauri::command]
fn update_issue_by_path(path: String, issue: Issue) -> Result<(), String> {
    update_issue(PathBuf::from(&path), issue.clone()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn move_issue_status(
    file_name: String,
    from_status: String,
    to_status: String,
    state: State<AppState>,
) -> Result<IssueEntry, String> {
    let guard = state.active_project.lock().unwrap();
    let project_dir = guard
        .as_ref()
        .ok_or_else(|| "No active project".to_string())?
        .clone();
    drop(guard);

    let issue_path = project_dir
        .join("Issues")
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
fn list_adrs_cmd(state: State<AppState>) -> Result<Vec<AdrEntry>, String> {
    let guard = state.active_project.lock().unwrap();
    let project_dir = guard
        .as_ref()
        .ok_or_else(|| "No active project".to_string())?
        .clone();
    drop(guard);
    list_adrs(project_dir).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_new_adr(
    title: String,
    decision: String,
    state: State<AppState>,
) -> Result<AdrEntry, String> {
    let guard = state.active_project.lock().unwrap();
    let project_dir = guard
        .as_ref()
        .ok_or_else(|| "No active project".to_string())?
        .clone();
    drop(guard);

    let path = create_adr(project_dir.clone(), &title, &decision, "accepted")
        .map_err(|e| e.to_string())?;
    log_action(
        project_dir,
        "adr create",
        &format!("Created ADR: {}", title),
    )
    .ok();

    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let adr_data: logic::ADR = serde_json::from_str(&content).map_err(|e| e.to_string())?;
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

// ─── Commands: Log ────────────────────────────────────────────────────────────

#[tauri::command]
fn get_log(state: State<AppState>) -> Result<Vec<LogEntry>, String> {
    let guard = state.active_project.lock().unwrap();
    let project_dir = guard
        .as_ref()
        .ok_or_else(|| "No active project".to_string())?
        .clone();
    drop(guard);
    read_log_entries(project_dir).map_err(|e| e.to_string())
}

// ─── Commands: Settings ───────────────────────────────────────────────────────

#[tauri::command]
fn get_app_settings() -> Result<Config, String> {
    get_config(None).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_app_settings(config: Config) -> Result<(), String> {
    save_config(&config, None).map_err(|e| e.to_string())
}

// ─── App Entry ────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Project
            list_projects,
            get_active_project,
            set_active_project,
            pick_and_open_project,
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
            // Log
            get_log,
            // Settings
            get_app_settings,
            save_app_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
