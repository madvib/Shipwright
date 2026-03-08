use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use runtime::config::{
    add_mcp_server, add_mode, generate_gitignore, get_active_mode, get_config,
    get_effective_config, list_mcp_servers, remove_mcp_server, remove_mode, save_config,
    set_active_mode, AiConfig, McpServerConfig, ModeConfig, ProjectConfig, ProjectDiscovery,
};
use runtime::project::{
    adrs_dir, features_dir, get_active_project_global, get_project_dir, issues_dir, notes_dir,
    releases_dir, resolve_project_ship_dir, set_active_project_global, specs_dir, SHIP_DIR_NAME,
};
use runtime::{
    activate_workspace, autodetect_providers, create_skill, create_user_skill, create_workspace,
    delete_skill, delete_user_skill, delete_workspace, end_workspace_session,
    get_active_workspace_session, get_effective_skill, get_skill, get_user_skill, get_workspace,
    get_workspace_provider_matrix, ingest_external_events, list_catalog, list_catalog_by_kind,
    list_effective_skills, list_events_since, list_models, list_providers, list_skills,
    list_user_skills, list_workspace_sessions, list_workspaces, log_action, read_log_entries,
    repair_workspace, resolve_agent_config, search_catalog, set_workspace_active_mode,
    start_workspace_session, sync_workspace, transition_workspace_status, update_skill,
    update_user_skill, AgentConfig, CatalogEntry, CatalogKind, CreateWorkspaceRequest,
    EndWorkspaceSessionRequest, EventRecord, LogEntry, ModelInfo, ProviderInfo, Skill, Workspace,
    WorkspaceProviderMatrix, WorkspaceRepairReport, WorkspaceSession, WorkspaceStatus,
    WorkspaceType,
};
use serde::{Deserialize, Serialize};
use ship_module_project::{
    create_adr, create_feature, create_issue, create_note, create_release, create_spec, delete_adr,
    delete_issue, delete_spec, get_adr_by_id, get_feature_by_id, get_feature_documentation,
    get_issue_by_id, get_note_by_id, get_project_name, get_release_by_id, get_spec_by_id,
    init_project, list_adrs, list_features, list_issues, list_notes, list_registered_projects,
    list_releases, list_specs, move_adr, move_issue, read_template, register_project,
    rename_project, update_adr, update_issue, update_note_content, update_release, update_spec,
    update_feature_content, AdrEntry, AdrStatus, FeatureEntry as ProjectFeatureEntry, Issue,
    IssueEntry, IssueStatus, NoteScope, ReleaseEntry as ProjectReleaseEntry, Spec, SpecEntry, ADR,
};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use notify::{
    Config as NotifyConfig, Event as NotifyEvent, EventKind as NotifyEventKind,
    RecommendedWatcher, RecursiveMode, Watcher,
};
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
    handle: thread::JoinHandle<Result<(), String>>,
}

#[derive(Default)]
struct RuntimePerfCounters {
    terminal_start_calls: AtomicU64,
    terminal_start_errors: AtomicU64,
    terminal_start_last_micros: AtomicU64,
    terminal_read_calls: AtomicU64,
    terminal_read_bytes: AtomicU64,
    terminal_read_errors: AtomicU64,
    terminal_last_read_micros: AtomicU64,
    terminal_write_calls: AtomicU64,
    terminal_write_errors: AtomicU64,
    terminal_write_last_micros: AtomicU64,
    terminal_resize_calls: AtomicU64,
    terminal_resize_errors: AtomicU64,
    terminal_resize_last_micros: AtomicU64,
    terminal_stop_calls: AtomicU64,
    terminal_stop_errors: AtomicU64,
    terminal_stop_last_micros: AtomicU64,
    watcher_fs_events: AtomicU64,
    watcher_flushes: AtomicU64,
    watcher_ingest_runs: AtomicU64,
    watcher_last_ingest_micros: AtomicU64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RuntimePerfSnapshot {
    pub terminal_start_calls: u64,
    pub terminal_start_errors: u64,
    pub terminal_start_last_micros: u64,
    pub terminal_read_calls: u64,
    pub terminal_read_bytes: u64,
    pub terminal_read_errors: u64,
    pub terminal_last_read_micros: u64,
    pub terminal_write_calls: u64,
    pub terminal_write_errors: u64,
    pub terminal_write_last_micros: u64,
    pub terminal_resize_calls: u64,
    pub terminal_resize_errors: u64,
    pub terminal_resize_last_micros: u64,
    pub terminal_stop_calls: u64,
    pub terminal_stop_errors: u64,
    pub terminal_stop_last_micros: u64,
    pub watcher_fs_events: u64,
    pub watcher_flushes: u64,
    pub watcher_ingest_runs: u64,
    pub watcher_last_ingest_micros: u64,
}

struct PtySession {
    id: String,
    branch: String,
    provider: String,
    cwd: String,
    cols: Mutex<u16>,
    rows: Mutex<u16>,
    master: Mutex<Box<dyn MasterPty + Send>>,
    writer: Mutex<Box<dyn Write + Send>>,
    child: Mutex<Box<dyn Child + Send>>,
    output_rx: Mutex<mpsc::Receiver<Vec<u8>>>,
    reader_handle: Mutex<Option<thread::JoinHandle<()>>>,
    closed: AtomicBool,
    exit_code: Mutex<Option<u32>>,
}

impl PtySession {
    fn mark_closed(&self, exit_code: Option<u32>) {
        self.closed.store(true, Ordering::SeqCst);
        if let Some(code_value) = exit_code {
            if let Ok(mut code) = self.exit_code.lock() {
                *code = Some(code_value);
            }
        }
    }

    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    fn exit_code(&self) -> Option<u32> {
        self.exit_code.lock().ok().and_then(|code| *code)
    }

    fn refresh_exit_state(&self) -> Result<bool, String> {
        if self.is_closed() {
            return Ok(true);
        }
        let mut child = self
            .child
            .lock()
            .map_err(|_| "PTY child lock poisoned".to_string())?;
        match child.try_wait() {
            Ok(Some(status)) => {
                self.mark_closed(Some(status.exit_code()));
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(error) => {
                self.mark_closed(None);
                Err(error.to_string())
            }
        }
    }

    fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        let master = self
            .master
            .lock()
            .map_err(|_| "PTY session lock poisoned".to_string())?;
        master.resize(size).map_err(|e| e.to_string())?;
        drop(master);
        if let Ok(mut current_cols) = self.cols.lock() {
            *current_cols = cols;
        }
        if let Ok(mut current_rows) = self.rows.lock() {
            *current_rows = rows;
        }
        Ok(())
    }

    fn write_input(&self, input: &str) -> Result<(), String> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| "PTY writer lock poisoned".to_string())?;
        writer
            .write_all(input.as_bytes())
            .map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())
    }

    fn drain_output(&self, max_bytes: usize) -> Result<String, String> {
        let rx = self
            .output_rx
            .lock()
            .map_err(|_| "PTY output lock poisoned".to_string())?;
        let mut output = Vec::new();
        while output.len() < max_bytes {
            match rx.try_recv() {
                Ok(chunk) => {
                    let remaining = max_bytes.saturating_sub(output.len());
                    if chunk.len() <= remaining {
                        output.extend_from_slice(&chunk);
                    } else {
                        output.extend_from_slice(&chunk[..remaining]);
                        break;
                    }
                }
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
        Ok(String::from_utf8_lossy(&output).to_string())
    }

    fn stop(&self) {
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
            if let Ok(status) = child.wait() {
                self.mark_closed(Some(status.exit_code()));
            } else {
                self.mark_closed(None);
            }
        }
        if let Ok(mut handle) = self.reader_handle.lock() {
            if let Some(join_handle) = handle.take() {
                let _ = join_handle.join();
            }
        }
        self.mark_closed(self.exit_code());
    }
}

/// Holds the currently active project directory (the `.ship` dir path).
#[derive(Default)]
pub struct AppState {
    active_project: Mutex<Option<PathBuf>>,
    project_watcher: Mutex<Option<ProjectPoller>>,
    terminal_sessions: Mutex<HashMap<String, Arc<PtySession>>>,
    perf: Arc<RuntimePerfCounters>,
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
    pub active_target_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_status: Option<String>,
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
    pub active_target_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub path: String,
    pub updated: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_revision: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_content: Option<String>,
    #[serde(default)]
    pub todos: Vec<FeatureTodoItem>,
    #[serde(default)]
    pub acceptance_criteria: Vec<FeatureCriterionItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureTodoItem {
    pub id: String,
    pub text: String,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureCriterionItem {
    pub id: String,
    pub text: String,
    pub met: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WorkspaceEditorInfo {
    pub id: String,
    pub name: String,
    pub binary: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WorkspaceFileChange {
    pub status: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WorkspaceTerminalSessionInfo {
    pub session_id: String,
    pub branch: String,
    pub provider: String,
    pub cwd: String,
    pub cols: u16,
    pub rows: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_error: Option<String>,
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

fn detect_project_providers_non_blocking(ship_path: &Path) {
    let project_root = ship_path.parent().unwrap_or(ship_path);
    if let Err(err) = autodetect_providers(project_root) {
        eprintln!(
            "[ship-ui] warning: provider autodetect failed for {}: {}",
            project_root.display(),
            err
        );
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

fn u128_to_u64_saturating(value: u128) -> u64 {
    if value > u64::MAX as u128 {
        u64::MAX
    } else {
        value as u64
    }
}

fn find_markdown_file_by_id(root: &Path, id: &str) -> Option<PathBuf> {
    if !root.exists() {
        return None;
    }
    let legacy_needle = format!("id = \"{}\"", id);
    let feature_needle = format!("ship:feature id={}", id);
    let release_needle = format!("ship:release id={}", id);
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
            if content.contains(&legacy_needle)
                || content.contains(&feature_needle)
                || content.contains(&release_needle)
            {
                return Some(path);
            }
        }
    }
    None
}

fn strip_generated_export_header(content: String) -> String {
    let mut lines = content.lines();
    if let Some(first) = lines.next() {
        let trimmed = first.trim();
        if trimmed.starts_with("<!-- ship:feature ") || trimmed.starts_with("<!-- ship:release ") {
            return lines.collect::<Vec<_>>().join("\n").trim_start_matches('\n').to_string();
        }
    }
    if let Some(stripped) = strip_legacy_toml_frontmatter(&content) {
        return stripped;
    }
    content
}

fn strip_legacy_toml_frontmatter(content: &str) -> Option<String> {
    if !content.starts_with("+++\n") {
        return None;
    }
    let rest = &content[4..];
    let end = rest.find("\n+++")?;
    let body = rest[end + 4..].trim_start_matches('\n').to_string();
    Some(body)
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
    let docs_status = get_feature_documentation(project_dir, &entry.id)
        .ok()
        .map(|doc| doc.status.to_string());
    FeatureInfo {
        id: entry.id.clone(),
        file_name: entry.file_name.clone(),
        title: entry.feature.metadata.title.clone(),
        status: entry.status.to_string(),
        release_id: entry.feature.metadata.release_id.clone(),
        active_target_id: entry.feature.metadata.active_target_id.clone(),
        spec_id: entry.feature.metadata.spec_id.clone(),
        branch: entry.feature.metadata.branch.clone(),
        description: entry.feature.metadata.description.clone(),
        docs_status,
        path: path.to_string_lossy().to_string(),
        updated: entry.feature.metadata.updated.clone(),
    }
}

fn map_feature_document(project_dir: &Path, entry: &ProjectFeatureEntry) -> FeatureDocument {
    let info = map_feature_info(project_dir, entry);
    let content = resolve_feature_markdown_path(project_dir, entry)
        .and_then(|path| fs::read_to_string(path).ok())
        .map(strip_generated_export_header)
        .unwrap_or_else(|| entry.feature.body.clone());
    let docs = get_feature_documentation(project_dir, &entry.id).ok();
    FeatureDocument {
        id: info.id,
        file_name: info.file_name,
        title: info.title,
        status: info.status,
        release_id: info.release_id,
        active_target_id: info.active_target_id,
        spec_id: info.spec_id,
        branch: info.branch,
        description: info.description,
        path: info.path,
        updated: info.updated,
        content,
        docs_status: info.docs_status,
        docs_revision: docs.as_ref().map(|doc| doc.revision),
        docs_updated_at: docs.as_ref().map(|doc| doc.updated_at.clone()),
        docs_content: docs.as_ref().map(|doc| doc.content.clone()),
        todos: entry
            .feature
            .todos
            .iter()
            .map(|todo| FeatureTodoItem {
                id: todo.id.clone(),
                text: todo.text.clone(),
                completed: todo.completed,
            })
            .collect(),
        acceptance_criteria: entry
            .feature
            .criteria
            .iter()
            .map(|criterion| FeatureCriterionItem {
                id: criterion.id.clone(),
                text: criterion.text.clone(),
                met: criterion.met,
            })
            .collect(),
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
        .map(strip_generated_export_header)
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
    let initialized = !ship_path.exists();
    let final_ship_path = if initialized {
        init_project(base_dir).map_err(|e| e.to_string())?
    } else {
        ship_path
    };
    if initialized {
        detect_project_providers_non_blocking(&final_ship_path);
    }

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
    let initialized = !existing_ship.exists();
    let ship_path = if initialized {
        init_project(base_dir.clone()).map_err(|e| e.to_string())?
    } else {
        existing_ship
    };
    if initialized {
        detect_project_providers_non_blocking(&ship_path);
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

    let initialized = !existing_ship.exists();
    let ship_path = if initialized {
        init_project(base_dir.clone()).map_err(|e| e.to_string())?
    } else {
        existing_ship
    };
    if initialized {
        detect_project_providers_non_blocking(&ship_path);
    }

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
    clear_terminal_sessions(state)?;

    let app_handle = app.clone();
    let ship_root = ship_dir.clone();
    let perf = Arc::clone(&state.perf);
    let (stop_tx, stop_rx) = mpsc::channel::<()>();

    let poller = thread::spawn(move || -> Result<(), String> {
        #[derive(Default)]
        struct PendingWatchChanges {
            issues: bool,
            specs: bool,
            adrs: bool,
            features: bool,
            releases: bool,
            notes: bool,
            config: bool,
            tracked_content: bool,
            events_db: bool,
        }

        impl PendingWatchChanges {
            fn any(&self) -> bool {
                self.issues
                    || self.specs
                    || self.adrs
                    || self.features
                    || self.releases
                    || self.notes
                    || self.config
                    || self.events_db
            }

            fn clear(&mut self) {
                *self = Self::default();
            }
        }

        let issues_dir = issues_dir(&ship_root);
        let specs_dir = specs_dir(&ship_root);
        let adrs_dir = adrs_dir(&ship_root);
        let features_dir = features_dir(&ship_root);
        let releases_dir = releases_dir(&ship_root);
        let notes_dir = notes_dir(&ship_root);
        let events_db = runtime::state_db::project_db_path(&ship_root).ok();
        let config_file = ship_root.join(runtime::config::PRIMARY_CONFIG_FILE);
        let (event_tx, event_rx) = mpsc::channel::<notify::Result<NotifyEvent>>();

        let mut watcher = RecommendedWatcher::new(
            move |result| {
                let _ = event_tx.send(result);
            },
            NotifyConfig::default(),
        )
        .map_err(|e| e.to_string())?;

        watcher
            .watch(&ship_root, RecursiveMode::Recursive)
            .map_err(|e| e.to_string())?;

        if let Some(events_db) = events_db.as_ref() {
            if let Some(parent) = events_db.parent() {
                let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
            }
        }

        let mut pending = PendingWatchChanges::default();
        let mut last_flush = Instant::now();

        let flush = |pending: &mut PendingWatchChanges| {
            if pending.issues {
                let _ = ShipEvent::IssuesChanged.emit(&app_handle);
            }
            if pending.specs {
                let _ = ShipEvent::SpecsChanged.emit(&app_handle);
            }
            if pending.adrs {
                let _ = ShipEvent::AdrsChanged.emit(&app_handle);
            }
            if pending.features {
                let _ = ShipEvent::FeaturesChanged.emit(&app_handle);
            }
            if pending.releases {
                let _ = ShipEvent::ReleasesChanged.emit(&app_handle);
            }
            if pending.notes {
                let _ = ShipEvent::NotesChanged.emit(&app_handle);
            }
            if pending.config {
                let _ = ShipEvent::ConfigChanged.emit(&app_handle);
            }

            if pending.tracked_content {
                perf.watcher_ingest_runs.fetch_add(1, Ordering::Relaxed);
                let ingest_started = Instant::now();
                if let Ok(emitted) = ingest_external_events(&ship_root) {
                    if !emitted.is_empty() {
                        let _ = ShipEvent::EventsChanged.emit(&app_handle);
                    }
                }
                perf.watcher_last_ingest_micros.store(
                    u128_to_u64_saturating(ingest_started.elapsed().as_micros()),
                    Ordering::Relaxed,
                );
            }

            if pending.events_db {
                let _ = ShipEvent::EventsChanged.emit(&app_handle);
                let _ = ShipEvent::LogChanged.emit(&app_handle);
            }

            perf.watcher_flushes.fetch_add(1, Ordering::Relaxed);
            pending.clear();
        };

        loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }

            match event_rx.recv_timeout(Duration::from_millis(200)) {
                Ok(Ok(event)) => {
                    perf.watcher_fs_events.fetch_add(1, Ordering::Relaxed);
                    if matches!(event.kind, NotifyEventKind::Access(_)) {
                        continue;
                    }
                    for path in event.paths {
                        if path.starts_with(&issues_dir) {
                            pending.issues = true;
                            pending.tracked_content = true;
                            continue;
                        }
                        if path.starts_with(&specs_dir) {
                            pending.specs = true;
                            pending.tracked_content = true;
                            continue;
                        }
                        if path.starts_with(&adrs_dir) {
                            pending.adrs = true;
                            pending.tracked_content = true;
                            continue;
                        }
                        if path.starts_with(&features_dir) {
                            pending.features = true;
                            pending.tracked_content = true;
                            continue;
                        }
                        if path.starts_with(&releases_dir) {
                            pending.releases = true;
                            pending.tracked_content = true;
                            continue;
                        }
                        if path.starts_with(&notes_dir) {
                            pending.notes = true;
                            pending.tracked_content = true;
                            continue;
                        }
                        if path == config_file {
                            pending.config = true;
                            pending.tracked_content = true;
                            continue;
                        }
                        if let Some(events_db) = events_db.as_ref() {
                            if path == *events_db {
                                pending.events_db = true;
                            }
                        }
                    }
                }
                Ok(Err(error)) => {
                    eprintln!("Project watcher error: {}", error);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }

            if pending.any() && last_flush.elapsed() >= Duration::from_millis(180) {
                flush(&mut pending);
                last_flush = Instant::now();
            }
        }

        if pending.any() {
            flush(&mut pending);
        }

        Ok(())
    });

    let mut guard = state.project_watcher.lock().unwrap();
    if let Some(old) = guard.take() {
        let _ = old.stop_tx.send(());
        match old.handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => eprintln!("Project watcher exited with error: {}", error),
            Err(_) => eprintln!("Project watcher thread panicked"),
        }
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
    let _ = update_feature_content(&project_dir, id, &content).map_err(|e| e.to_string())?;
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

const WORKSPACE_EDITOR_ALIASES_CURSOR: &[&str] = &["cursor"];
const WORKSPACE_EDITOR_ALIASES_VSCODE: &[&str] = &["vscode", "code", "vs-code"];
const WORKSPACE_EDITOR_ALIASES_WINDSURF: &[&str] = &["windsurf", "codeium-windsurf", "codeium"];
const WORKSPACE_EDITOR_ALIASES_ANTIGRAVITY: &[&str] = &["antigravity", "ag"];
const WORKSPACE_EDITOR_ALIASES_ZED: &[&str] = &["zed"];
const WORKSPACE_EDITOR_ALIASES_INTELLIJ: &[&str] = &["intellij", "idea", "idea-ultimate"];
const WORKSPACE_EDITOR_ALIASES_WEBSTORM: &[&str] = &["webstorm", "web"];
const WORKSPACE_EDITOR_ALIASES_PYCHARM: &[&str] = &["pycharm", "charm"];
const WORKSPACE_EDITOR_ALIASES_CLION: &[&str] = &["clion"];
const WORKSPACE_EDITOR_ALIASES_GOLAND: &[&str] = &["goland", "gol"];
const WORKSPACE_EDITOR_ALIASES_RUSTROVER: &[&str] = &["rustrover", "rust"];

const SUPPORTED_WORKSPACE_EDITORS: [(&str, &str, &str, &[&str]); 11] = [
    (
        "cursor",
        "Cursor",
        "cursor",
        WORKSPACE_EDITOR_ALIASES_CURSOR,
    ),
    ("vscode", "VS Code", "code", WORKSPACE_EDITOR_ALIASES_VSCODE),
    (
        "windsurf",
        "Windsurf",
        "windsurf",
        WORKSPACE_EDITOR_ALIASES_WINDSURF,
    ),
    (
        "antigravity",
        "Antigravity",
        "antigravity",
        WORKSPACE_EDITOR_ALIASES_ANTIGRAVITY,
    ),
    ("zed", "Zed", "zed", WORKSPACE_EDITOR_ALIASES_ZED),
    (
        "intellij",
        "IntelliJ IDEA",
        "idea",
        WORKSPACE_EDITOR_ALIASES_INTELLIJ,
    ),
    (
        "webstorm",
        "WebStorm",
        "webstorm",
        WORKSPACE_EDITOR_ALIASES_WEBSTORM,
    ),
    (
        "pycharm",
        "PyCharm",
        "pycharm",
        WORKSPACE_EDITOR_ALIASES_PYCHARM,
    ),
    ("clion", "CLion", "clion", WORKSPACE_EDITOR_ALIASES_CLION),
    (
        "goland",
        "GoLand",
        "goland",
        WORKSPACE_EDITOR_ALIASES_GOLAND,
    ),
    (
        "rustrover",
        "RustRover",
        "rustrover",
        WORKSPACE_EDITOR_ALIASES_RUSTROVER,
    ),
];

fn resolve_command_in_path(binary: &str) -> Option<PathBuf> {
    let path_env = match std::env::var_os("PATH") {
        Some(value) => value,
        None => return None,
    };

    for dir in std::env::split_paths(&path_env) {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            return Some(candidate);
        }

        #[cfg(windows)]
        {
            let exts = ["exe", "cmd", "bat", "com"];
            if exts
                .iter()
                .map(|ext| dir.join(format!("{}.{}", binary, ext)))
                .find(|path| path.is_file())
                .is_some()
            {
                return exts
                    .iter()
                    .map(|ext| dir.join(format!("{}.{}", binary, ext)))
                    .find(|path| path.is_file());
            }
        }
    }

    None
}

fn normalize_workspace_editor_id(raw: &str) -> Option<&'static str> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }

    SUPPORTED_WORKSPACE_EDITORS
        .iter()
        .find(|(id, _, _, aliases)| {
            id.eq_ignore_ascii_case(&normalized)
                || aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(&normalized))
        })
        .map(|(id, _, _, _)| *id)
}

fn supported_workspace_editor_ids() -> String {
    SUPPORTED_WORKSPACE_EDITORS
        .iter()
        .map(|(id, _, _, _)| *id)
        .collect::<Vec<_>>()
        .join(", ")
}

fn editor_command_candidates(editor_id: &str, binary: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    candidates.push(binary.to_string());
    match editor_id {
        "vscode" => {
            candidates.extend(["code-insiders"].iter().map(|value| value.to_string()));
        }
        "windsurf" => {
            candidates.extend(
                ["windsurf-cli", "codeium"]
                    .iter()
                    .map(|value| value.to_string()),
            );
        }
        "antigravity" => {
            candidates.extend(["ag"].iter().map(|value| value.to_string()));
        }
        "intellij" => {
            candidates.extend(
                ["idea-ultimate", "idea-community", "idea64"]
                    .iter()
                    .map(|value| value.to_string()),
            );
        }
        "webstorm" => {
            candidates.extend(["webstorm64"].iter().map(|value| value.to_string()));
        }
        "pycharm" => {
            candidates.extend(
                ["pycharm-professional", "pycharm-community", "pycharm64"]
                    .iter()
                    .map(|value| value.to_string()),
            );
        }
        "goland" => {
            candidates.extend(["goland64"].iter().map(|value| value.to_string()));
        }
        "rustrover" => {
            candidates.extend(["rustrover64"].iter().map(|value| value.to_string()));
        }
        _ => {}
    }
    candidates
}

#[cfg(target_os = "macos")]
fn resolve_editor_binary(editor_id: &str, binary: &str) -> Option<PathBuf> {
    for candidate in editor_command_candidates(editor_id, binary) {
        if let Some(path) = resolve_command_in_path(&candidate) {
            return Some(path);
        }
    }

    let common_paths = [
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/opt/local/bin",
        "/usr/bin",
    ];
    for candidate in editor_command_candidates(editor_id, binary) {
        for dir in common_paths {
            let path = Path::new(dir).join(&candidate);
            if path.is_file() {
                return Some(path);
            }
        }
    }

    let app_bundle_bins: [(&str, &[&str]); 11] = [
        (
            "cursor",
            &["/Applications/Cursor.app/Contents/Resources/app/bin/cursor"],
        ),
        (
            "vscode",
            &["/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code"],
        ),
        (
            "windsurf",
            &["/Applications/Windsurf.app/Contents/Resources/app/bin/windsurf"],
        ),
        (
            "antigravity",
            &["/Applications/Antigravity.app/Contents/Resources/app/bin/antigravity"],
        ),
        ("zed", &["/Applications/Zed.app/Contents/MacOS/zed"]),
        (
            "intellij",
            &["/Applications/IntelliJ IDEA.app/Contents/MacOS/idea"],
        ),
        (
            "webstorm",
            &["/Applications/WebStorm.app/Contents/MacOS/webstorm"],
        ),
        (
            "pycharm",
            &["/Applications/PyCharm.app/Contents/MacOS/pycharm"],
        ),
        ("clion", &["/Applications/CLion.app/Contents/MacOS/clion"]),
        (
            "goland",
            &["/Applications/GoLand.app/Contents/MacOS/goland"],
        ),
        (
            "rustrover",
            &["/Applications/RustRover.app/Contents/MacOS/rustrover"],
        ),
    ];
    if let Some((_, paths)) = app_bundle_bins.iter().find(|(id, _)| *id == editor_id) {
        for path in *paths {
            let candidate = PathBuf::from(path);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

#[cfg(not(target_os = "macos"))]
fn resolve_editor_binary(editor_id: &str, binary: &str) -> Option<PathBuf> {
    for candidate in editor_command_candidates(editor_id, binary) {
        if let Some(path) = resolve_command_in_path(&candidate) {
            return Some(path);
        }
    }
    None
}

fn resolve_workspace_target_dir(ship_dir: &Path, workspace: Option<&Workspace>) -> PathBuf {
    if let Some(entry) = workspace {
        if entry.is_worktree {
            if let Some(path) = entry.worktree_path.as_deref() {
                if !path.trim().is_empty() {
                    return PathBuf::from(path);
                }
            }
        }
    }

    ship_dir.parent().unwrap_or(ship_dir).to_path_buf()
}

fn normalize_terminal_provider(provider: Option<String>) -> String {
    provider
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("shell")
        .to_ascii_lowercase()
}

fn resolve_terminal_command(provider: &str) -> String {
    match provider {
        "claude" => "claude".to_string(),
        "codex" => "codex".to_string(),
        "gemini" => "gemini".to_string(),
        "shell" => std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string()),
        other => other.to_string(),
    }
}

fn friendly_terminal_spawn_error(provider: &str, command: &str, raw_error: &str) -> String {
    let normalized = raw_error.to_ascii_lowercase();
    if normalized.contains("no such file")
        || normalized.contains("not found")
        || normalized.contains("cannot find")
    {
        return format!(
            "Terminal provider '{}' is not installed or not on PATH (command: '{}').",
            provider, command
        );
    }
    format!(
        "Failed to start terminal provider '{}' (command: '{}'): {}",
        provider, command, raw_error
    )
}

fn load_terminal_session(
    state: &State<'_, AppState>,
    session_id: &str,
) -> Result<Arc<PtySession>, String> {
    let sessions = state
        .terminal_sessions
        .lock()
        .map_err(|_| "Terminal session registry lock poisoned".to_string())?;
    sessions
        .get(session_id)
        .cloned()
        .ok_or_else(|| format!("Terminal session '{}' not found", session_id))
}

fn clear_terminal_sessions(state: &State<'_, AppState>) -> Result<(), String> {
    let sessions = {
        let mut guard = state
            .terminal_sessions
            .lock()
            .map_err(|_| "Terminal session registry lock poisoned".to_string())?;
        guard
            .drain()
            .map(|(_, session)| session)
            .collect::<Vec<_>>()
    };
    for session in sessions {
        session.stop();
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn list_workspace_editors_cmd() -> Result<Vec<WorkspaceEditorInfo>, String> {
    let installed = SUPPORTED_WORKSPACE_EDITORS
        .iter()
        .filter_map(|(id, name, binary, _)| {
            resolve_editor_binary(id, binary).map(|path| WorkspaceEditorInfo {
                id: (*id).to_string(),
                name: (*name).to_string(),
                binary: path.to_string_lossy().to_string(),
            })
        })
        .collect::<Vec<_>>();

    Ok(installed)
}

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
    mode_id: Option<String>,
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
                active_mode: mode_id,
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
async fn set_workspace_mode_cmd(
    branch: String,
    mode_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        set_workspace_active_mode(&project_dir, &branch, mode_id.as_deref())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn delete_workspace_cmd(branch: String, state: State<'_, AppState>) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        delete_workspace(&project_dir, &branch).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn get_active_workspace_session_cmd(
    branch: String,
    state: State<'_, AppState>,
) -> Result<Option<WorkspaceSession>, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        get_active_workspace_session(&project_dir, &branch).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn list_workspace_sessions_cmd(
    branch: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<WorkspaceSession>, String> {
    let project_dir = get_active_dir(&state)?;
    let clamped_limit = limit.unwrap_or(25).clamp(1, 200);
    tauri::async_runtime::spawn_blocking(move || {
        list_workspace_sessions(&project_dir, branch.as_deref(), clamped_limit)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn get_workspace_provider_matrix_cmd(
    branch: String,
    mode_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<WorkspaceProviderMatrix, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        get_workspace_provider_matrix(&project_dir, &branch, mode_id.as_deref())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn repair_workspace_cmd(
    branch: String,
    dry_run: Option<bool>,
    state: State<'_, AppState>,
) -> Result<WorkspaceRepairReport, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        repair_workspace(&project_dir, &branch, dry_run.unwrap_or(true)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn start_workspace_session_cmd(
    branch: String,
    goal: Option<String>,
    mode_id: Option<String>,
    provider: Option<String>,
    state: State<'_, AppState>,
) -> Result<WorkspaceSession, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        start_workspace_session(&project_dir, &branch, goal, mode_id, provider)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn end_workspace_session_cmd(
    branch: String,
    summary: Option<String>,
    updated_feature_ids: Option<Vec<String>>,
    updated_spec_ids: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<WorkspaceSession, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let session = end_workspace_session(
            &project_dir,
            &branch,
            EndWorkspaceSessionRequest {
                summary,
                updated_feature_ids: updated_feature_ids.unwrap_or_default(),
                updated_spec_ids: updated_spec_ids.unwrap_or_default(),
            },
        )
        .map_err(|e| e.to_string())?;

        if !session.updated_feature_ids.is_empty() {
            let _ = ship_module_project::ops::feature::sync_feature_docs_after_session(
                &project_dir,
                &session.updated_feature_ids,
                session.summary.as_deref(),
            );
        }

        Ok(session)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn list_workspace_changes_cmd(
    branch: String,
    state: State<'_, AppState>,
) -> Result<Vec<WorkspaceFileChange>, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let workspace = get_workspace(&project_dir, &branch).map_err(|e| e.to_string())?;
        let target_dir = resolve_workspace_target_dir(&project_dir, workspace.as_ref());

        let output = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&target_dir)
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err("Failed to resolve workspace file changes".to_string());
        }

        let parsed = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return None;
                }

                let status = line.get(0..2)?.trim().to_string();
                let raw_path = line.get(3..)?.trim();
                if raw_path.is_empty() {
                    return None;
                }
                let normalized_path = raw_path
                    .rsplit(" -> ")
                    .next()
                    .unwrap_or(raw_path)
                    .trim()
                    .to_string();

                Some(WorkspaceFileChange {
                    status,
                    path: normalized_path,
                })
            })
            .collect::<Vec<_>>();

        Ok(parsed)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn open_workspace_editor_cmd(
    branch: String,
    editor: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let workspace = get_workspace(&project_dir, &branch).map_err(|e| e.to_string())?;
        let target_dir = resolve_workspace_target_dir(&project_dir, workspace.as_ref());

        let normalized_editor_id = normalize_workspace_editor_id(&editor).ok_or_else(|| {
            format!(
                "Unknown editor '{}'. Use one of: {}",
                editor,
                supported_workspace_editor_ids()
            )
        })?;

        let (_, _, binary, _) = SUPPORTED_WORKSPACE_EDITORS
            .iter()
            .find(|(id, _, _, _)| *id == normalized_editor_id)
            .ok_or_else(|| format!("Editor '{}' is not supported", normalized_editor_id))?;

        let resolved_binary =
            resolve_editor_binary(normalized_editor_id, binary).ok_or_else(|| {
                format!(
                    "Editor '{}' is not available on this machine",
                    normalized_editor_id
                )
            })?;

        std::process::Command::new(resolved_binary)
            .arg(target_dir)
            .spawn()
            .map_err(|e| e.to_string())?;

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn start_workspace_terminal_cmd(
    branch: String,
    provider: Option<String>,
    cols: Option<u16>,
    rows: Option<u16>,
    state: State<'_, AppState>,
) -> Result<WorkspaceTerminalSessionInfo, String> {
    state.perf.terminal_start_calls.fetch_add(1, Ordering::Relaxed);
    let started = Instant::now();
    let result: Result<WorkspaceTerminalSessionInfo, String> = async {
        let project_dir = get_active_dir(&state)?;
        let resolved_cols = cols.unwrap_or(120).clamp(40, 400);
        let resolved_rows = rows.unwrap_or(32).clamp(12, 240);
        let normalized_provider = normalize_terminal_provider(provider);
        let branch_for_spawn = branch.clone();
        let provider_for_spawn = normalized_provider.clone();
        let project_dir_for_spawn = project_dir.clone();

        let (session, activation_error) = tauri::async_runtime::spawn_blocking(
            move || -> Result<(Arc<PtySession>, Option<String>), String> {
                let workspace = get_workspace(&project_dir_for_spawn, &branch_for_spawn)
                    .map_err(|e| e.to_string())?;
                let target_dir =
                    resolve_workspace_target_dir(&project_dir_for_spawn, workspace.as_ref());

                let activation_error = activate_workspace(&project_dir_for_spawn, &branch_for_spawn)
                    .err()
                    .map(|e| e.to_string());

                let pty_system = native_pty_system();
                let pty_pair = pty_system
                    .openpty(PtySize {
                        rows: resolved_rows,
                        cols: resolved_cols,
                        pixel_width: 0,
                        pixel_height: 0,
                    })
                    .map_err(|e| e.to_string())?;

                let command = resolve_terminal_command(&provider_for_spawn);
                let mut cmd = CommandBuilder::new(command.as_str());
                cmd.cwd(&target_dir);
                let child = pty_pair.slave.spawn_command(cmd).map_err(|e| {
                    friendly_terminal_spawn_error(
                        &provider_for_spawn,
                        command.as_str(),
                        &e.to_string(),
                    )
                })?;
                let mut reader = pty_pair
                    .master
                    .try_clone_reader()
                    .map_err(|e| e.to_string())?;
                let writer = pty_pair.master.take_writer().map_err(|e| e.to_string())?;
                let (output_tx, output_rx) = mpsc::channel::<Vec<u8>>();
                let session = Arc::new(PtySession {
                    id: runtime::gen_nanoid(),
                    branch: branch_for_spawn,
                    provider: provider_for_spawn,
                    cwd: target_dir.to_string_lossy().to_string(),
                    cols: Mutex::new(resolved_cols),
                    rows: Mutex::new(resolved_rows),
                    master: Mutex::new(pty_pair.master),
                    writer: Mutex::new(writer),
                    child: Mutex::new(child),
                    output_rx: Mutex::new(output_rx),
                    reader_handle: Mutex::new(None),
                    closed: AtomicBool::new(false),
                    exit_code: Mutex::new(None),
                });

                let session_for_reader = Arc::clone(&session);
                let reader_handle = thread::spawn(move || {
                    let mut buffer = [0u8; 8192];
                    loop {
                        match reader.read(&mut buffer) {
                            Ok(0) => break,
                            Ok(read) => {
                                if output_tx.send(buffer[..read].to_vec()).is_err() {
                                    break;
                                }
                            }
                            Err(error) => {
                                if error.kind() == std::io::ErrorKind::Interrupted {
                                    continue;
                                }
                                break;
                            }
                        }
                    }
                    if session_for_reader.refresh_exit_state().is_err()
                        && !session_for_reader.is_closed()
                    {
                        session_for_reader.mark_closed(None);
                    }
                });
                if let Ok(mut handle) = session.reader_handle.lock() {
                    *handle = Some(reader_handle);
                }

                Ok((session, activation_error))
            },
        )
        .await
        .map_err(|e| e.to_string())??;

        let info = WorkspaceTerminalSessionInfo {
            session_id: session.id.clone(),
            branch: session.branch.clone(),
            provider: session.provider.clone(),
            cwd: session.cwd.clone(),
            cols: *session
                .cols
                .lock()
                .map_err(|_| "Terminal session size lock poisoned".to_string())?,
            rows: *session
                .rows
                .lock()
                .map_err(|_| "Terminal session size lock poisoned".to_string())?,
            activation_error,
        };

        let replaced_sessions = {
            let mut sessions = state
                .terminal_sessions
                .lock()
                .map_err(|_| "Terminal session registry lock poisoned".to_string())?;
            let replaced_ids = sessions
                .iter()
                .filter(|(_, existing)| existing.branch == info.branch)
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>();
            let mut removed = Vec::new();
            for id in replaced_ids {
                if let Some(existing) = sessions.remove(&id) {
                    removed.push(existing);
                }
            }
            sessions.insert(session.id.clone(), session);
            removed
        };
        for existing in replaced_sessions {
            existing.stop();
        }
        Ok(info)
    }
    .await;

    state.perf.terminal_start_last_micros.store(
        u128_to_u64_saturating(started.elapsed().as_micros()),
        Ordering::Relaxed,
    );
    if result.is_err() {
        state.perf.terminal_start_errors.fetch_add(1, Ordering::Relaxed);
    }
    result
}

#[tauri::command]
#[specta::specta]
fn read_workspace_terminal_cmd(
    session_id: String,
    max_bytes: Option<usize>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    state.perf.terminal_read_calls.fetch_add(1, Ordering::Relaxed);
    let started = Instant::now();
    let session = load_terminal_session(&state, &session_id)?;
    let _ = session.refresh_exit_state();
    let output = session.drain_output(max_bytes.unwrap_or(65_536).clamp(1, 262_144))?;
    state.perf.terminal_read_bytes.fetch_add(
        u128_to_u64_saturating(output.len() as u128),
        Ordering::Relaxed,
    );
    state.perf.terminal_last_read_micros.store(
        u128_to_u64_saturating(started.elapsed().as_micros()),
        Ordering::Relaxed,
    );

    if session.is_closed() && output.is_empty() {
        state.perf.terminal_read_errors.fetch_add(1, Ordering::Relaxed);
        let removed = {
            let mut sessions = state
                .terminal_sessions
                .lock()
                .map_err(|_| "Terminal session registry lock poisoned".to_string())?;
            sessions.remove(&session_id)
        };
        if let Some(closed_session) = removed {
            closed_session.stop();
        }
        return Err(match session.exit_code() {
            Some(code) => format!(
                "Terminal session '{}' closed (exit code {})",
                session_id, code
            ),
            None => format!("Terminal session '{}' closed", session_id),
        });
    }

    Ok(output)
}

#[tauri::command]
#[specta::specta]
fn write_workspace_terminal_cmd(
    session_id: String,
    input: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.perf.terminal_write_calls.fetch_add(1, Ordering::Relaxed);
    let started = Instant::now();
    let result = (|| -> Result<(), String> {
        let session = load_terminal_session(&state, &session_id)?;
        let _ = session.refresh_exit_state();
        if session.is_closed() {
            return Err(match session.exit_code() {
                Some(code) => format!(
                    "Terminal session '{}' closed (exit code {})",
                    session_id, code
                ),
                None => format!("Terminal session '{}' closed", session_id),
            });
        }
        session.write_input(&input)
    })();
    state.perf.terminal_write_last_micros.store(
        u128_to_u64_saturating(started.elapsed().as_micros()),
        Ordering::Relaxed,
    );
    if result.is_err() {
        state.perf.terminal_write_errors.fetch_add(1, Ordering::Relaxed);
    }
    result
}

#[tauri::command]
#[specta::specta]
fn resize_workspace_terminal_cmd(
    session_id: String,
    cols: u16,
    rows: u16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .perf
        .terminal_resize_calls
        .fetch_add(1, Ordering::Relaxed);
    let started = Instant::now();
    let result = (|| -> Result<(), String> {
        let session = load_terminal_session(&state, &session_id)?;
        let _ = session.refresh_exit_state();
        if session.is_closed() {
            return Err(match session.exit_code() {
                Some(code) => format!(
                    "Terminal session '{}' closed (exit code {})",
                    session_id, code
                ),
                None => format!("Terminal session '{}' closed", session_id),
            });
        }
        session.resize(cols.clamp(40, 400), rows.clamp(12, 240))
    })();
    state.perf.terminal_resize_last_micros.store(
        u128_to_u64_saturating(started.elapsed().as_micros()),
        Ordering::Relaxed,
    );
    if result.is_err() {
        state
            .perf
            .terminal_resize_errors
            .fetch_add(1, Ordering::Relaxed);
    }
    result
}

#[tauri::command]
#[specta::specta]
fn stop_workspace_terminal_cmd(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.perf.terminal_stop_calls.fetch_add(1, Ordering::Relaxed);
    let started = Instant::now();
    let result = (|| -> Result<(), String> {
        let session = {
            let mut sessions = state
                .terminal_sessions
                .lock()
                .map_err(|_| "Terminal session registry lock poisoned".to_string())?;
            sessions.remove(&session_id)
        }
        .ok_or_else(|| format!("Terminal session '{}' not found", session_id))?;
        session.stop();
        Ok(())
    })();
    state.perf.terminal_stop_last_micros.store(
        u128_to_u64_saturating(started.elapsed().as_micros()),
        Ordering::Relaxed,
    );
    if result.is_err() {
        state.perf.terminal_stop_errors.fetch_add(1, Ordering::Relaxed);
    }
    result
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
fn get_runtime_perf_cmd(state: State<AppState>) -> Result<RuntimePerfSnapshot, String> {
    Ok(RuntimePerfSnapshot {
        terminal_start_calls: state.perf.terminal_start_calls.load(Ordering::Relaxed),
        terminal_start_errors: state.perf.terminal_start_errors.load(Ordering::Relaxed),
        terminal_start_last_micros: state
            .perf
            .terminal_start_last_micros
            .load(Ordering::Relaxed),
        terminal_read_calls: state.perf.terminal_read_calls.load(Ordering::Relaxed),
        terminal_read_bytes: state.perf.terminal_read_bytes.load(Ordering::Relaxed),
        terminal_read_errors: state.perf.terminal_read_errors.load(Ordering::Relaxed),
        terminal_last_read_micros: state.perf.terminal_last_read_micros.load(Ordering::Relaxed),
        terminal_write_calls: state.perf.terminal_write_calls.load(Ordering::Relaxed),
        terminal_write_errors: state.perf.terminal_write_errors.load(Ordering::Relaxed),
        terminal_write_last_micros: state
            .perf
            .terminal_write_last_micros
            .load(Ordering::Relaxed),
        terminal_resize_calls: state.perf.terminal_resize_calls.load(Ordering::Relaxed),
        terminal_resize_errors: state.perf.terminal_resize_errors.load(Ordering::Relaxed),
        terminal_resize_last_micros: state
            .perf
            .terminal_resize_last_micros
            .load(Ordering::Relaxed),
        terminal_stop_calls: state.perf.terminal_stop_calls.load(Ordering::Relaxed),
        terminal_stop_errors: state.perf.terminal_stop_errors.load(Ordering::Relaxed),
        terminal_stop_last_micros: state.perf.terminal_stop_last_micros.load(Ordering::Relaxed),
        watcher_fs_events: state.perf.watcher_fs_events.load(Ordering::Relaxed),
        watcher_flushes: state.perf.watcher_flushes.load(Ordering::Relaxed),
        watcher_ingest_runs: state.perf.watcher_ingest_runs.load(Ordering::Relaxed),
        watcher_last_ingest_micros: state.perf.watcher_last_ingest_micros.load(Ordering::Relaxed),
    })
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
            list_workspace_editors_cmd,
            get_workspace_cmd,
            list_workspaces_cmd,
            sync_workspace_cmd,
            create_workspace_cmd,
            activate_workspace_cmd,
            set_workspace_mode_cmd,
            delete_workspace_cmd,
            get_active_workspace_session_cmd,
            list_workspace_sessions_cmd,
            get_workspace_provider_matrix_cmd,
            repair_workspace_cmd,
            start_workspace_session_cmd,
            end_workspace_session_cmd,
            list_workspace_changes_cmd,
            open_workspace_editor_cmd,
            start_workspace_terminal_cmd,
            read_workspace_terminal_cmd,
            write_workspace_terminal_cmd,
            resize_workspace_terminal_cmd,
            stop_workspace_terminal_cmd,
            transition_workspace_cmd,
            get_current_branch_cmd,
            // Log
            list_events_cmd,
            ingest_events_cmd,
            get_log,
            get_runtime_perf_cmd,
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
    use std::path::Path;

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

    #[test]
    fn terminal_provider_normalizes_and_defaults() {
        assert_eq!(normalize_terminal_provider(None), "shell");
        assert_eq!(
            normalize_terminal_provider(Some("  CoDeX  ".to_string())),
            "codex"
        );
        assert_eq!(normalize_terminal_provider(Some("".to_string())), "shell");
    }

    #[test]
    fn terminal_command_resolves_shell_and_named_providers() {
        assert_eq!(resolve_terminal_command("claude"), "claude");
        assert_eq!(resolve_terminal_command("codex"), "codex");
        assert_eq!(resolve_terminal_command("gemini"), "gemini");
        assert_eq!(resolve_terminal_command("custom-bin"), "custom-bin");

        let shell = resolve_terminal_command("shell");
        assert!(!shell.trim().is_empty());
    }

    #[test]
    fn terminal_spawn_error_is_human_readable_for_missing_binary() {
        let message = friendly_terminal_spawn_error("codex", "codex", "No such file or directory");
        assert!(message.contains("not installed or not on PATH"));
        assert!(message.contains("codex"));
    }

    #[test]
    fn workspace_target_dir_prefers_worktree_path() {
        let ship_dir = Path::new("/tmp/project/.ship");
        let workspace = Workspace {
            id: "ws_123".to_string(),
            branch: "feature/test".to_string(),
            workspace_type: WorkspaceType::Feature,
            status: WorkspaceStatus::Active,
            feature_id: None,
            spec_id: None,
            release_id: None,
            active_mode: None,
            providers: Vec::new(),
            resolved_at: "2026-03-06T00:00:00Z".parse().expect("valid timestamp"),
            is_worktree: true,
            worktree_path: Some("/tmp/worktrees/feature-test".to_string()),
            last_activated_at: None,
            context_hash: None,
            config_generation: 1,
            compiled_at: None,
            compile_error: None,
        };

        let resolved = resolve_workspace_target_dir(ship_dir, Some(&workspace));
        assert_eq!(resolved, Path::new("/tmp/worktrees/feature-test"));

        let fallback = resolve_workspace_target_dir(ship_dir, None);
        assert_eq!(fallback, Path::new("/tmp/project"));
    }
}
