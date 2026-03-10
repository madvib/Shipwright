use notify::{
    Config as NotifyConfig, Event as NotifyEvent, EventKind as NotifyEventKind, RecommendedWatcher,
    RecursiveMode, Watcher,
};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use runtime::config::{
    add_mcp_server, add_mode, generate_gitignore, get_active_mode, get_config,
    get_effective_config, list_mcp_servers, remove_mcp_server, remove_mode, save_config,
    set_active_mode, AiConfig, McpServerConfig, McpServerType, ModeConfig, ProjectConfig,
    ProjectDiscovery,
};
use runtime::project::{
    features_dir, get_active_project_global, get_project_dir, releases_dir,
    resolve_project_ship_dir, sanitize_file_name, set_active_project_global, SHIP_DIR_NAME,
};
use runtime::{
    activate_workspace, autodetect_providers, create_skill, create_user_skill, create_workspace,
    delete_skill, delete_user_skill, delete_workspace, detect_binary, detect_version,
    end_workspace_session, get_active_workspace_session, get_effective_skill, get_skill,
    get_user_skill, get_workspace, get_workspace_provider_matrix, ingest_external_events,
    install_skill_from_source, list_catalog, list_catalog_by_kind, list_effective_skills,
    list_events_since, list_models, list_providers, list_skills, list_user_skills,
    list_workspace_sessions, list_workspaces, log_action, read_log_entries, repair_workspace,
    resolve_agent_config, search_catalog, set_workspace_active_mode, start_workspace_session,
    sync_workspace, transition_workspace_status, update_skill, update_user_skill, AgentConfig,
    CatalogEntry, CatalogKind, CreateWorkspaceRequest, EndWorkspaceSessionRequest, EventRecord,
    LogEntry, ModelInfo, ProviderInfo, ShipWorkspaceKind, Skill, SkillInstallScope, Workspace,
    WorkspaceProviderMatrix, WorkspaceRepairReport, WorkspaceSession, WorkspaceStatus,
};
use serde::{Deserialize, Serialize};
use ship_module_project::{
    create_adr, create_feature, create_note, create_release_with_metadata, create_spec, delete_adr,
    delete_spec, feature_done, feature_start, get_adr_by_id, get_feature_by_id,
    get_feature_documentation, get_note_by_id, get_project_name, get_release_by_id, get_spec_by_id,
    init_project, list_adrs, list_features, list_notes, list_registered_projects, list_releases,
    list_specs, move_adr, move_spec, read_template, register_project, rename_project, update_adr,
    update_feature_content, update_feature_documentation, update_note_content, update_release,
    update_spec, AdrEntry, AdrStatus, FeatureDocStatus, FeatureEntry as ProjectFeatureEntry,
    NoteScope, ReleaseEntry as ProjectReleaseEntry, ReleaseStatus as ProjectReleaseStatus, Spec,
    SpecEntry, SpecStatus, ADR,
};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use tauri_specta::Event;

// ─── Typed Events ─────────────────────────────────────────────────────────────

/// Typed push events from the backend to the UI.
/// Each variant maps to a `{ type: "..." }` payload on the TypeScript side.
#[derive(Clone, Serialize, Type, tauri_specta::Event)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ShipEvent {
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

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct McpValidationIssue {
    pub level: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct McpValidationReport {
    pub ok: bool,
    pub checked_servers: usize,
    pub checked_provider_configs: usize,
    pub issues: Vec<McpValidationIssue>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ProviderImportReport {
    pub imported_mcp_servers: usize,
    pub imported_permissions: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct McpRegistryEntry {
    pub id: String,
    pub server_name: String,
    pub title: String,
    pub description: String,
    pub version: String,
    /// "stdio" | "http" | "sse"
    pub transport: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub url: Option<String>,
    pub required_env: Vec<String>,
    pub required_headers: Vec<String>,
    pub source_url: Option<String>,
    pub website_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct McpDiscoveredTool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct McpProbeServerReport {
    pub server_id: String,
    pub server_name: String,
    /// "stdio" | "http" | "sse"
    pub transport: String,
    pub ok: bool,
    /// "ready" | "needs-attention" | "disabled" | "partial"
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub discovered_tools: Vec<McpDiscoveredTool>,
    pub duration_ms: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct McpProbeReport {
    /// Unix epoch seconds.
    pub generated_at: String,
    pub checked_servers: usize,
    pub reachable_servers: usize,
    pub discovered_tools: usize,
    pub results: Vec<McpProbeServerReport>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct SkillToolHint {
    pub id: String,
    pub name: String,
    pub allowed_tools: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AgentDiscoveryCache {
    pub version: u32,
    /// Unix epoch seconds as string.
    pub updated_at: String,
    pub mcp_tools: HashMap<String, Vec<McpDiscoveredTool>>,
    pub shell_commands: Vec<String>,
    pub filesystem_paths: Vec<String>,
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
pub struct GitBranchInfo {
    pub name: String,
    pub current: bool,
    pub base_branch: String,
    pub ahead: u64,
    pub behind: u64,
    pub touched_files: usize,
    pub insertions: u64,
    pub deletions: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WorkspaceFileChange {
    pub status: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WorkspaceGitStatusSummary {
    pub branch: String,
    pub touched_files: usize,
    pub insertions: u64,
    pub deletions: u64,
    pub ahead: u64,
    pub behind: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct BranchFileChange {
    pub status: String,
    pub path: String,
    pub insertions: u64,
    pub deletions: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct BranchDetailSummary {
    pub branch: String,
    pub base_branch: String,
    pub ahead: u64,
    pub behind: u64,
    pub touched_files: usize,
    pub insertions: u64,
    pub deletions: u64,
    pub has_workspace: bool,
    pub changes: Vec<BranchFileChange>,
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
            return lines
                .collect::<Vec<_>>()
                .join("\n")
                .trim_start_matches('\n')
                .to_string();
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
        status: map_release_status_to_ui(entry.status).to_string(),
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

fn map_release_status_to_ui(status: ProjectReleaseStatus) -> &'static str {
    match status {
        ProjectReleaseStatus::Upcoming => "planned",
        ProjectReleaseStatus::Active => "active",
        ProjectReleaseStatus::Deprecated => "shipped",
    }
}

fn map_release_status_from_ui(
    status: Option<&str>,
) -> Result<Option<ProjectReleaseStatus>, String> {
    let Some(value) = status else {
        return Ok(None);
    };
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Ok(None);
    }
    let mapped = match normalized.as_str() {
        "planned" | "upcoming" => ProjectReleaseStatus::Upcoming,
        "active" => ProjectReleaseStatus::Active,
        "shipped" | "archived" | "deprecated" => ProjectReleaseStatus::Deprecated,
        _ => return Err(format!("Invalid release status: {}", value)),
    };
    Ok(Some(mapped))
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
            projects.push(ProjectDiscovery {
                name: entry.name,
                path: ship_path,
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
                    return Ok(Some(ProjectInfo {
                        name: project_display_name(&path),
                        path: path.to_string_lossy().to_string(),
                    }));
                }
            }
            Ok(None)
        }
        Some(path) => Ok(Some(ProjectInfo {
            name: project_display_name(path),
            path: path.to_string_lossy().to_string(),
        })),
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
    let display_name = project_display_name(&ship_path);
    let info = ProjectInfo {
        name: display_name.clone(),
        path: ship_path.to_string_lossy().to_string(),
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

    let display_name = project_display_name(&final_ship_path);
    let info = ProjectInfo {
        name: display_name.clone(),
        path: final_ship_path.to_string_lossy().to_string(),
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
            let display_name = project_display_name(&ship_path);
            let info = ProjectInfo {
                name: display_name,
                path: ship_path.to_string_lossy().to_string(),
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
            let display_name = project_display_name(&ship_path);
            let info = ProjectInfo {
                name: display_name.clone(),
                path: ship_path.to_string_lossy().to_string(),
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

    let display_name = project_display_name(&ship_path);
    let info = ProjectInfo {
        name: display_name.clone(),
        path: ship_path.to_string_lossy().to_string(),
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

    let info = ProjectInfo {
        name: display_name.clone(),
        path: ship_path.to_string_lossy().to_string(),
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
    app_handle: tauri::AppHandle,
    path: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<ProjectInfo, String> {
    let ship_path = ensure_ship_path(Path::new(&path));
    rename_project(ship_path.clone(), name).map_err(|e| e.to_string())?;

    if ship_path.exists() {
        *state.active_project.lock().unwrap() = Some(ship_path.clone());
    }

    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(ProjectInfo {
        name: project_display_name(&ship_path),
        path: ship_path.to_string_lossy().to_string(),
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
            config: bool,
            events_db: bool,
        }

        impl PendingWatchChanges {
            fn any(&self) -> bool {
                self.config || self.events_db
            }

            fn clear(&mut self) {
                *self = Self::default();
            }
        }

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

        // Only watch ship.toml for external config changes (non-recursive).
        if let Some(parent) = config_file.parent() {
            let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
        }

        // Watch the events DB directory for DB writes.
        if let Some(events_db) = events_db.as_ref() {
            if let Some(parent) = events_db.parent() {
                let _ = watcher.watch(parent, RecursiveMode::NonRecursive);
            }
        }

        let mut pending = PendingWatchChanges::default();
        let mut last_flush = Instant::now();

        let flush = |pending: &mut PendingWatchChanges| {
            if pending.config {
                let _ = ShipEvent::ConfigChanged.emit(&app_handle);
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
                    if matches!(event.kind, NotifyEventKind::Access(_)) {
                        continue;
                    }

                    let mut saw_relevant_change = false;
                    for path in event.paths {
                        if path == config_file {
                            pending.config = true;
                            saw_relevant_change = true;
                            continue;
                        }
                        if let Some(events_db) = events_db.as_ref() {
                            if path == *events_db {
                                pending.events_db = true;
                                saw_relevant_change = true;
                            }
                        }
                    }

                    if saw_relevant_change {
                        perf.watcher_fs_events.fetch_add(1, Ordering::Relaxed);
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
    app_handle: tauri::AppHandle,
    title: String,
    context: String,
    decision: String,
    state: State<AppState>,
) -> Result<AdrEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let entry = create_adr(&project_dir, &title, &context, &decision, "proposed")
        .map_err(|e| e.to_string())?;
    let _ = ShipEvent::AdrsChanged.emit(&app_handle);
    Ok(entry)
}

#[tauri::command]
#[specta::specta]
fn get_adr_cmd(id: String, state: State<AppState>) -> Result<AdrEntry, String> {
    let project_dir = get_active_dir(&state)?;
    get_adr_by_id(&project_dir, &id).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn update_adr_cmd(
    app_handle: tauri::AppHandle,
    id: String,
    adr: ADR,
    state: State<AppState>,
) -> Result<AdrEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let entry = update_adr(&project_dir, &id, adr).map_err(|e| e.to_string())?;
    let _ = ShipEvent::AdrsChanged.emit(&app_handle);
    Ok(entry)
}

#[tauri::command]
#[specta::specta]
fn move_adr_cmd(
    app_handle: tauri::AppHandle,
    id: String,
    new_status: String,
    state: State<AppState>,
) -> Result<AdrEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let status = new_status
        .parse::<AdrStatus>()
        .map_err(|_| format!("Invalid ADR status: {}", new_status))?;
    let entry = move_adr(&project_dir, &id, status).map_err(|e| e.to_string())?;
    let _ = ShipEvent::AdrsChanged.emit(&app_handle);
    Ok(entry)
}

#[tauri::command]
#[specta::specta]
fn delete_adr_cmd(
    app_handle: tauri::AppHandle,
    id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    delete_adr(&project_dir, &id).map_err(|e| e.to_string())?;
    let _ = ShipEvent::AdrsChanged.emit(&app_handle);
    Ok(())
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
    app_handle: tauri::AppHandle,
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
    let _ = ShipEvent::SpecsChanged.emit(&app_handle);
    Ok(entry)
}

#[tauri::command]
#[specta::specta]
fn update_spec_cmd(
    app_handle: tauri::AppHandle,
    id: String,
    spec: Spec,
    state: State<AppState>,
) -> Result<SpecEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let entry = update_spec(&project_dir, &id, spec).map_err(|e| e.to_string())?;
    log_action(
        &project_dir,
        "spec update",
        &format!("Updated Spec: {}", entry.file_name),
    )
    .ok();
    let _ = ShipEvent::SpecsChanged.emit(&app_handle);
    Ok(entry)
}

#[tauri::command]
#[specta::specta]
fn move_spec_cmd(
    app_handle: tauri::AppHandle,
    id: String,
    new_status: String,
    state: State<AppState>,
) -> Result<SpecEntry, String> {
    let project_dir = get_active_dir(&state)?;
    let status = new_status
        .parse::<SpecStatus>()
        .map_err(|_| format!("Invalid spec status: {}", new_status))?;
    let entry = move_spec(&project_dir, &id, status).map_err(|e| e.to_string())?;
    let _ = ShipEvent::SpecsChanged.emit(&app_handle);
    Ok(entry)
}

#[tauri::command]
#[specta::specta]
fn delete_spec_cmd(
    app_handle: tauri::AppHandle,
    id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    delete_spec(&project_dir, &id).map_err(|e| e.to_string())?;
    log_action(
        &project_dir,
        "spec delete",
        &format!("Deleted Spec: {}", id),
    )
    .ok();
    let _ = ShipEvent::SpecsChanged.emit(&app_handle);
    Ok(())
}

fn normalize_optional_branch(value: Option<&str>) -> Option<String> {
    value.and_then(|entry| {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn default_feature_workspace_branch(title: &str) -> String {
    let slug = sanitize_file_name(title);
    if slug.is_empty() {
        "feature/workspace".to_string()
    } else {
        format!("feature/{slug}")
    }
}

fn default_release_workspace_branch(version: &str) -> String {
    let slug = sanitize_file_name(version);
    if slug.is_empty() {
        "release/workspace".to_string()
    } else {
        format!("release/{slug}")
    }
}

fn auto_provision_workspace(
    project_dir: &Path,
    branch: &str,
    workspace_type: Option<ShipWorkspaceKind>,
    feature_id: Option<String>,
    release_id: Option<String>,
) -> Result<Workspace, String> {
    let git_root = project_dir.parent().unwrap_or(project_dir);
    let worktree_path = detect_existing_worktree_path(git_root, branch);
    create_workspace(
        project_dir,
        CreateWorkspaceRequest {
            branch: branch.to_string(),
            workspace_type,
            feature_id,
            release_id,
            is_worktree: Some(true),
            worktree_path,
            ..CreateWorkspaceRequest::default()
        },
    )
    .map_err(|e| e.to_string())
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
    app_handle: tauri::AppHandle,
    version: String,
    content: String,
    status: Option<String>,
    target_date: Option<String>,
    supported: Option<bool>,
    tags: Option<Vec<String>>,
    state: State<AppState>,
) -> Result<ReleaseDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let mapped_status = map_release_status_from_ui(status.as_deref())?;
    let normalized_target_date = target_date.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });
    let normalized_tags = tags
        .unwrap_or_default()
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    let entry = create_release_with_metadata(
        &project_dir,
        &version,
        &content,
        mapped_status,
        normalized_target_date,
        supported,
        normalized_tags,
    )
    .map_err(|e| e.to_string())?;
    let workspace_branch = default_release_workspace_branch(&entry.version);
    if let Err(err) = auto_provision_workspace(
        &project_dir,
        &workspace_branch,
        Some(ShipWorkspaceKind::Feature),
        None,
        Some(entry.id.clone()),
    ) {
        eprintln!(
            "[ship-ui] warning: release workspace auto-provision failed for {}: {}",
            workspace_branch, err
        );
    }
    log_action(
        &project_dir,
        "release create",
        &format!("Created Release: {}", version),
    )
    .ok();
    let _ = ShipEvent::ReleasesChanged.emit(&app_handle);
    Ok(map_release_document(&project_dir, &entry))
}

#[tauri::command]
#[specta::specta]
fn update_release_cmd(
    app_handle: tauri::AppHandle,
    file_name: String,
    content: String,
    version: Option<String>,
    status: Option<String>,
    target_date: Option<String>,
    supported: Option<bool>,
    tags: Option<Vec<String>>,
    state: State<AppState>,
) -> Result<ReleaseDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let id = file_name.trim_end_matches(".md");
    let release_entry = get_release_by_id(&project_dir, id).map_err(|e| e.to_string())?;
    let mut release = release_entry.release;

    if let Some(next_version) = version {
        let trimmed = next_version.trim();
        if !trimmed.is_empty() {
            release.metadata.version = trimmed.to_string();
        }
    }
    if let Some(next_status) = map_release_status_from_ui(status.as_deref())? {
        release.metadata.status = next_status;
    }
    if let Some(next_target_date) = target_date {
        let trimmed = next_target_date.trim();
        release.metadata.target_date = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }
    if let Some(next_supported) = supported {
        release.metadata.supported = Some(next_supported);
    }
    if let Some(next_tags) = tags {
        release.metadata.tags = next_tags
            .into_iter()
            .map(|tag| tag.trim().to_string())
            .filter(|tag| !tag.is_empty())
            .collect();
    }
    release.body = content.clone();

    let entry = update_release(&project_dir, id, release).map_err(|e| e.to_string())?;
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
    let _ = ShipEvent::ReleasesChanged.emit(&app_handle);
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
    app_handle: tauri::AppHandle,
    title: String,
    content: String,
    release: Option<String>,
    spec: Option<String>,
    branch: Option<String>,
    state: State<AppState>,
) -> Result<FeatureDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let workspace_branch = normalize_optional_branch(branch.as_deref())
        .unwrap_or_else(|| default_feature_workspace_branch(&title));
    let entry = create_feature(
        &project_dir,
        &title,
        &content,
        release.as_deref(),
        spec.as_deref(),
        Some(workspace_branch.as_str()),
    )
    .map_err(|e| e.to_string())?;
    if let Err(err) = auto_provision_workspace(
        &project_dir,
        &workspace_branch,
        Some(ShipWorkspaceKind::Feature),
        Some(entry.id.clone()),
        entry.feature.metadata.release_id.clone(),
    ) {
        eprintln!(
            "[ship-ui] warning: feature workspace auto-provision failed for {}: {}",
            workspace_branch, err
        );
    }
    log_action(
        &project_dir,
        "feature create",
        &format!("Created Feature: {}", title),
    )
    .ok();
    let _ = ShipEvent::FeaturesChanged.emit(&app_handle);
    Ok(map_feature_document(&project_dir, &entry))
}

#[tauri::command]
#[specta::specta]
fn update_feature_cmd(
    app_handle: tauri::AppHandle,
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
    let _ = ShipEvent::FeaturesChanged.emit(&app_handle);
    Ok(map_feature_document(&project_dir, &updated))
}

#[tauri::command]
#[specta::specta]
fn feature_start_cmd(
    app_handle: tauri::AppHandle,
    file_name: String,
    state: State<AppState>,
) -> Result<FeatureDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let id = file_name.trim_end_matches(".md");
    feature_start(&project_dir, id).map_err(|e| e.to_string())?;
    let _ = ShipEvent::FeaturesChanged.emit(&app_handle);
    let refreshed = get_feature_by_id(&project_dir, id).map_err(|e| e.to_string())?;
    Ok(map_feature_document(&project_dir, &refreshed))
}

#[tauri::command]
#[specta::specta]
fn feature_done_cmd(
    app_handle: tauri::AppHandle,
    file_name: String,
    state: State<AppState>,
) -> Result<FeatureDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let id = file_name.trim_end_matches(".md");
    feature_done(&project_dir, id).map_err(|e| e.to_string())?;
    let _ = ShipEvent::FeaturesChanged.emit(&app_handle);
    let refreshed = get_feature_by_id(&project_dir, id).map_err(|e| e.to_string())?;
    Ok(map_feature_document(&project_dir, &refreshed))
}

#[tauri::command]
#[specta::specta]
fn update_feature_documentation_cmd(
    app_handle: tauri::AppHandle,
    file_name: String,
    content: String,
    status: Option<String>,
    verify_now: Option<bool>,
    state: State<AppState>,
) -> Result<FeatureDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let id = file_name.trim_end_matches(".md");
    let docs_status = match status.filter(|value| !value.trim().is_empty()) {
        Some(value) => Some(
            value
                .parse::<FeatureDocStatus>()
                .map_err(|_| format!("Invalid feature docs status: {}", value))?,
        ),
        None => None,
    };
    update_feature_documentation(
        &project_dir,
        id,
        content,
        docs_status,
        verify_now.unwrap_or(false),
        Some("ui"),
    )
    .map_err(|e| e.to_string())?;
    let _ = ShipEvent::FeaturesChanged.emit(&app_handle);
    let refreshed = get_feature_by_id(&project_dir, id).map_err(|e| e.to_string())?;
    Ok(map_feature_document(&project_dir, &refreshed))
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
fn update_vision_cmd(
    app_handle: tauri::AppHandle,
    content: String,
    state: State<AppState>,
) -> Result<VisionDocument, String> {
    let project_dir = get_active_dir(&state)?;
    let vision_path = runtime::project::project_ns(&project_dir).join("vision.md");
    if let Some(parent) = vision_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&vision_path, &content).map_err(|e| e.to_string())?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
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
    app_handle: tauri::AppHandle,
    title: String,
    content: String,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<NoteDocument, String> {
    let (note_scope, project_dir) = resolve_note_scope_and_dir(&state, scope)?;
    let note = create_note(note_scope, project_dir.as_deref(), &title, &content)
        .map_err(|e| e.to_string())?;
    let _ = ShipEvent::NotesChanged.emit(&app_handle);
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
    app_handle: tauri::AppHandle,
    id: String,
    content: String,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<NoteDocument, String> {
    let (note_scope, project_dir) = resolve_note_scope_and_dir(&state, scope)?;
    let note = update_note_content(note_scope, project_dir.as_deref(), &id, &content)
        .map_err(|e| e.to_string())?;
    let _ = ShipEvent::NotesChanged.emit(&app_handle);
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
    app_handle: tauri::AppHandle,
    id: String,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let (note_scope, project_dir) = resolve_note_scope_and_dir(&state, scope)?;
    ship_module_project::delete_note(note_scope, project_dir.as_deref(), &id)
        .map_err(|e| e.to_string())?;
    let _ = ShipEvent::NotesChanged.emit(&app_handle);
    Ok(())
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
    app_handle: tauri::AppHandle,
    file_name: String,
    content: String,
    state: State<AppState>,
) -> Result<runtime::rule::Rule, String> {
    let project_dir = get_active_dir(&state)?;
    let rule =
        runtime::create_rule(project_dir, &file_name, &content).map_err(|e| e.to_string())?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(rule)
}

#[tauri::command]
#[specta::specta]
fn update_rule_cmd(
    app_handle: tauri::AppHandle,
    file_name: String,
    content: String,
    state: State<AppState>,
) -> Result<runtime::rule::Rule, String> {
    let project_dir = get_active_dir(&state)?;
    let rule =
        runtime::update_rule(project_dir, &file_name, &content).map_err(|e| e.to_string())?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(rule)
}

#[tauri::command]
#[specta::specta]
fn delete_rule_cmd(
    app_handle: tauri::AppHandle,
    file_name: String,
    state: State<AppState>,
) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    runtime::delete_rule(project_dir, &file_name).map_err(|e| e.to_string())?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(())
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
    app_handle: tauri::AppHandle,
    permissions: runtime::permissions::Permissions,
    state: State<AppState>,
) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    runtime::save_permissions(project_dir, &permissions).map_err(|e| e.to_string())?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(())
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

#[cfg(windows)]
fn resolve_shell_command() -> String {
    if let Ok(explicit) = std::env::var("SHIP_TERMINAL_SHELL") {
        let trimmed = explicit.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Some(path) = resolve_command_in_path("pwsh") {
        return path.to_string_lossy().to_string();
    }

    if let Some(path) = resolve_command_in_path("powershell") {
        return path.to_string_lossy().to_string();
    }

    std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
}

#[cfg(not(windows))]
fn resolve_shell_command() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
}

fn resolve_terminal_command(provider: &str) -> String {
    match provider {
        "claude" => "claude".to_string(),
        "codex" => "codex".to_string(),
        "gemini" => "gemini".to_string(),
        "shell" => resolve_shell_command(),
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
    environment_id: Option<String>,
    feature_id: Option<String>,
    spec_id: Option<String>,
    release_id: Option<String>,
    mode_id: Option<String>,
    is_worktree: Option<bool>,
    worktree_path: Option<String>,
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let parsed_workspace_type = workspace_type
            .as_deref()
            .map(|value| value.parse::<ShipWorkspaceKind>())
            .transpose()
            .map_err(|e| e.to_string())?;

        let git_root = project_dir.parent().unwrap_or(&project_dir).to_path_buf();
        let branch_key = branch.trim().to_string();
        let resolved_worktree_path = if is_worktree.unwrap_or(false) {
            let explicit = worktree_path.and_then(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            });

            explicit.or_else(|| detect_existing_worktree_path(&git_root, &branch_key))
        } else {
            None
        };

        create_workspace(
            &project_dir,
            CreateWorkspaceRequest {
                branch: branch_key,
                workspace_type: parsed_workspace_type,
                environment_id,
                feature_id,
                spec_id,
                release_id,
                active_mode: mode_id,
                is_worktree,
                worktree_path: resolved_worktree_path,
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
        collect_workspace_changes(&target_dir)
    })
    .await
    .map_err(|e| e.to_string())?
}

fn collect_workspace_changes(target_dir: &Path) -> Result<Vec<WorkspaceFileChange>, String> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(target_dir)
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
}

fn collect_workspace_loc_delta(target_dir: &Path) -> Result<(u64, u64), String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--numstat", "HEAD"])
        .current_dir(target_dir)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Ok((0, 0));
    }

    let mut insertions = 0_u64;
    let mut deletions = 0_u64;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut parts = line.split('\t');
        let added = parts.next().unwrap_or("0");
        let removed = parts.next().unwrap_or("0");
        if let Ok(value) = added.parse::<u64>() {
            insertions += value;
        }
        if let Ok(value) = removed.parse::<u64>() {
            deletions += value;
        }
    }

    Ok((insertions, deletions))
}

fn collect_workspace_ahead_behind(target_dir: &Path) -> (Option<String>, u64, u64) {
    let upstream_output = std::process::Command::new("git")
        .args([
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ])
        .current_dir(target_dir)
        .output();

    let upstream = match upstream_output {
        Ok(output) if output.status.success() => {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if value.is_empty() {
                None
            } else {
                Some(value)
            }
        }
        _ => None,
    };

    let Some(upstream_ref) = upstream.clone() else {
        return (None, 0, 0);
    };

    let graph = format!("{upstream_ref}...HEAD");
    let counts_output = std::process::Command::new("git")
        .args(["rev-list", "--left-right", "--count", &graph])
        .current_dir(target_dir)
        .output();

    let Ok(output) = counts_output else {
        return (upstream, 0, 0);
    };
    if !output.status.success() {
        return (upstream, 0, 0);
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let mut parts = raw.split_whitespace();
    let behind = parts
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let ahead = parts
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);

    (upstream, ahead, behind)
}

fn normalize_diff_path(raw: &str) -> String {
    raw.rsplit(" -> ")
        .next()
        .unwrap_or(raw)
        .trim()
        .trim_matches('"')
        .to_string()
}

fn parse_branch_name_status(line: &str) -> Option<(String, String)> {
    let mut parts = line.split('\t');
    let raw_status = parts.next()?.trim();
    let status = raw_status.chars().next()?.to_string();
    let path = parts.last()?.trim();
    if path.is_empty() {
        return None;
    }
    Some((status, normalize_diff_path(path)))
}

fn collect_branch_change_stats(
    git_root: &Path,
    base_branch: &str,
    branch: &str,
) -> Vec<BranchFileChange> {
    if branch == base_branch {
        return Vec::new();
    }

    let range = format!("{base_branch}...{branch}");
    let mut changes: HashMap<String, BranchFileChange> = HashMap::new();

    let numstat_output = std::process::Command::new("git")
        .args(["diff", "--numstat", "--find-renames", range.as_str()])
        .current_dir(git_root)
        .output();

    if let Ok(output) = numstat_output {
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let mut parts = line.split('\t');
                let added = parts.next().unwrap_or("0");
                let removed = parts.next().unwrap_or("0");
                let path_raw = parts.next().unwrap_or("").trim();
                if path_raw.is_empty() {
                    continue;
                }

                let path = normalize_diff_path(path_raw);
                let insertions = added.parse::<u64>().unwrap_or(0);
                let deletions = removed.parse::<u64>().unwrap_or(0);
                changes
                    .entry(path.clone())
                    .and_modify(|entry| {
                        entry.insertions = entry.insertions.saturating_add(insertions);
                        entry.deletions = entry.deletions.saturating_add(deletions);
                    })
                    .or_insert(BranchFileChange {
                        status: "M".to_string(),
                        path,
                        insertions,
                        deletions,
                    });
            }
        }
    }

    let status_output = std::process::Command::new("git")
        .args(["diff", "--name-status", "--find-renames", range.as_str()])
        .current_dir(git_root)
        .output();

    if let Ok(output) = status_output {
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                if let Some((status, path)) = parse_branch_name_status(line) {
                    changes
                        .entry(path.clone())
                        .and_modify(|entry| entry.status = status.clone())
                        .or_insert(BranchFileChange {
                            status,
                            path,
                            insertions: 0,
                            deletions: 0,
                        });
                }
            }
        }
    }

    let mut entries = changes.into_values().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    entries
}

#[tauri::command]
#[specta::specta]
async fn get_branch_detail_cmd(
    branch: String,
    state: State<'_, AppState>,
) -> Result<BranchDetailSummary, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let branch = branch.trim().to_string();
        if branch.is_empty() {
            return Err("Branch is required".to_string());
        }

        let git_root = project_dir.parent().unwrap_or(&project_dir).to_path_buf();
        let base_branch =
            resolve_default_compare_branch(&git_root).unwrap_or_else(|| "main".to_string());
        let (ahead, behind) = collect_branch_ahead_behind(&git_root, &base_branch, &branch);
        let (touched_files, insertions, deletions) =
            collect_branch_diff_totals(&git_root, &base_branch, &branch);
        let changes = collect_branch_change_stats(&git_root, &base_branch, &branch);
        let has_workspace = get_workspace(&project_dir, &branch)
            .map_err(|e| e.to_string())?
            .is_some();

        Ok(BranchDetailSummary {
            branch,
            base_branch,
            ahead,
            behind,
            touched_files,
            insertions,
            deletions,
            has_workspace,
            changes,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn get_branch_file_diff_cmd(
    branch: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let branch = branch.trim().to_string();
        let file_path = path.trim().to_string();
        if branch.is_empty() {
            return Err("Branch is required".to_string());
        }
        if file_path.is_empty() {
            return Err("Path is required".to_string());
        }

        let git_root = project_dir.parent().unwrap_or(&project_dir).to_path_buf();
        let base_branch =
            resolve_default_compare_branch(&git_root).unwrap_or_else(|| "main".to_string());
        let range = format!("{base_branch}...{branch}");

        let output = std::process::Command::new("git")
            .args([
                "diff",
                "--no-color",
                range.as_str(),
                "--",
                file_path.as_str(),
            ])
            .current_dir(&git_root)
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err("Failed to load branch file diff".to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
#[specta::specta]
async fn get_workspace_git_status_cmd(
    branch: String,
    state: State<'_, AppState>,
) -> Result<WorkspaceGitStatusSummary, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let workspace = get_workspace(&project_dir, &branch).map_err(|e| e.to_string())?;
        let target_dir = resolve_workspace_target_dir(&project_dir, workspace.as_ref());

        let output = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&target_dir)
            .output()
            .map_err(|e| e.to_string())?;
        let resolved_branch = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            branch
        };

        let changes = collect_workspace_changes(&target_dir)?;
        let (insertions, deletions) = collect_workspace_loc_delta(&target_dir)?;
        let (upstream, ahead, behind) = collect_workspace_ahead_behind(&target_dir);

        Ok(WorkspaceGitStatusSummary {
            branch: resolved_branch,
            touched_files: changes.len(),
            insertions,
            deletions,
            ahead,
            behind,
            upstream,
        })
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
    state
        .perf
        .terminal_start_calls
        .fetch_add(1, Ordering::Relaxed);
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

                let activation_error =
                    activate_workspace(&project_dir_for_spawn, &branch_for_spawn)
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
        state
            .perf
            .terminal_start_errors
            .fetch_add(1, Ordering::Relaxed);
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
    state
        .perf
        .terminal_read_calls
        .fetch_add(1, Ordering::Relaxed);
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
        state
            .perf
            .terminal_read_errors
            .fetch_add(1, Ordering::Relaxed);
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
    state
        .perf
        .terminal_write_calls
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
        session.write_input(&input)
    })();
    state.perf.terminal_write_last_micros.store(
        u128_to_u64_saturating(started.elapsed().as_micros()),
        Ordering::Relaxed,
    );
    if result.is_err() {
        state
            .perf
            .terminal_write_errors
            .fetch_add(1, Ordering::Relaxed);
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
    state
        .perf
        .terminal_stop_calls
        .fetch_add(1, Ordering::Relaxed);
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
        state
            .perf
            .terminal_stop_errors
            .fetch_add(1, Ordering::Relaxed);
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

fn resolve_default_compare_branch(git_root: &Path) -> Option<String> {
    let origin_head = std::process::Command::new("git")
        .args(["symbolic-ref", "--quiet", "refs/remotes/origin/HEAD"])
        .current_dir(git_root)
        .output()
        .ok();

    if let Some(output) = origin_head {
        if output.status.success() {
            let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Some(branch) = raw.strip_prefix("refs/remotes/origin/") {
                let trimmed = branch.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    for candidate in ["main", "master"] {
        let verify_ref = format!("refs/heads/{candidate}");
        let status = std::process::Command::new("git")
            .args(["show-ref", "--verify", "--quiet", verify_ref.as_str()])
            .current_dir(git_root)
            .status();
        if let Ok(exit) = status {
            if exit.success() {
                return Some(candidate.to_string());
            }
        }
    }

    let current = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(git_root)
        .output()
        .ok()?;
    if !current.status.success() {
        return None;
    }

    let branch = String::from_utf8_lossy(&current.stdout).trim().to_string();
    if branch.is_empty() || branch == "HEAD" {
        None
    } else {
        Some(branch)
    }
}

fn collect_branch_ahead_behind(git_root: &Path, base: &str, branch: &str) -> (u64, u64) {
    if branch == base {
        return (0, 0);
    }

    let graph = format!("{base}...{branch}");
    let output = std::process::Command::new("git")
        .args(["rev-list", "--left-right", "--count", graph.as_str()])
        .current_dir(git_root)
        .output();

    let Ok(output) = output else {
        return (0, 0);
    };
    if !output.status.success() {
        return (0, 0);
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let mut parts = raw.split_whitespace();
    let behind = parts
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let ahead = parts
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);

    (ahead, behind)
}

fn collect_branch_diff_totals(git_root: &Path, base: &str, branch: &str) -> (usize, u64, u64) {
    if branch == base {
        return (0, 0, 0);
    }

    let range = format!("{base}...{branch}");
    let output = std::process::Command::new("git")
        .args(["diff", "--numstat", range.as_str()])
        .current_dir(git_root)
        .output();

    let Ok(output) = output else {
        return (0, 0, 0);
    };
    if !output.status.success() {
        return (0, 0, 0);
    }

    let mut touched_files = 0usize;
    let mut insertions = 0u64;
    let mut deletions = 0u64;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut parts = line.split('\t');
        let added = parts.next().unwrap_or("0");
        let removed = parts.next().unwrap_or("0");
        let path = parts.next().unwrap_or("");
        if path.trim().is_empty() {
            continue;
        }

        touched_files = touched_files.saturating_add(1);
        if let Ok(value) = added.parse::<u64>() {
            insertions = insertions.saturating_add(value);
        }
        if let Ok(value) = removed.parse::<u64>() {
            deletions = deletions.saturating_add(value);
        }
    }

    (touched_files, insertions, deletions)
}

fn detect_existing_worktree_path(git_root: &Path, branch: &str) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(git_root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let mut current_path: Option<String> = None;
    let mut current_branch: Option<String> = None;

    for line in String::from_utf8_lossy(&output.stdout)
        .lines()
        .chain(std::iter::once(""))
    {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if current_branch.as_deref() == Some(branch) {
                return current_path;
            }
            current_path = None;
            current_branch = None;
            continue;
        }

        if let Some(path) = trimmed.strip_prefix("worktree ") {
            current_path = Some(path.trim().to_string());
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("branch ") {
            let normalized = value.trim().trim_start_matches("refs/heads/").to_string();
            current_branch = Some(normalized);
        }
    }

    None
}
#[tauri::command]
#[specta::specta]
async fn list_git_branches_cmd(state: State<'_, AppState>) -> Result<Vec<GitBranchInfo>, String> {
    let project_dir = get_active_dir(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let git_root = project_dir.parent().unwrap_or(&project_dir).to_path_buf();
        let base_branch =
            resolve_default_compare_branch(&git_root).unwrap_or_else(|| "main".to_string());
        let output = std::process::Command::new("git")
            .args([
                "for-each-ref",
                "--sort=-committerdate",
                "--format=%(refname:short)|%(HEAD)",
                "refs/heads",
            ])
            .current_dir(&git_root)
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err("Failed to list git branches".to_string());
        }

        let mut branches = Vec::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let mut parts = line.splitn(2, '|');
            let Some(name_raw) = parts.next() else {
                continue;
            };
            let name = name_raw.trim();
            if name.is_empty() {
                continue;
            }
            let head_marker = parts.next().unwrap_or("").trim();
            let (ahead, behind) = collect_branch_ahead_behind(&git_root, &base_branch, name);
            let (touched_files, insertions, deletions) =
                collect_branch_diff_totals(&git_root, &base_branch, name);
            branches.push(GitBranchInfo {
                name: name.to_string(),
                current: head_marker == "*",
                base_branch: base_branch.clone(),
                ahead,
                behind,
                touched_files,
                insertions,
                deletions,
            });
        }

        Ok(branches)
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
        watcher_last_ingest_micros: state
            .perf
            .watcher_last_ingest_micros
            .load(Ordering::Relaxed),
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
fn save_project_config(
    app_handle: tauri::AppHandle,
    config: ProjectConfig,
    state: State<AppState>,
) -> Result<(), String> {
    let project_dir = get_active_dir(&state)?;
    save_config(&config, Some(project_dir)).map_err(|e| e.to_string())?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(())
}

// ─── Commands: Settings ───────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
fn get_app_settings() -> Result<ProjectConfig, String> {
    get_config(None).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn save_app_settings(app_handle: tauri::AppHandle, config: ProjectConfig) -> Result<(), String> {
    save_config(&config, None).map_err(|e| e.to_string())?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(())
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
fn add_mode_cmd(
    app_handle: tauri::AppHandle,
    mode: ModeConfig,
    state: State<AppState>,
) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    add_mode(Some(dir), mode).map_err(|e| e.to_string())?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn remove_mode_cmd(
    app_handle: tauri::AppHandle,
    id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    remove_mode(Some(dir), &id).map_err(|e| e.to_string())?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn set_active_mode_cmd(
    app_handle: tauri::AppHandle,
    id: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    set_active_mode(Some(dir), id.as_deref()).map_err(|e| e.to_string())?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(())
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
fn add_mcp_server_cmd(
    app_handle: tauri::AppHandle,
    server: McpServerConfig,
    state: State<AppState>,
) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    add_mcp_server(Some(dir), server).map_err(|e| e.to_string())?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn remove_mcp_server_cmd(
    app_handle: tauri::AppHandle,
    id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    remove_mcp_server(Some(dir), &id).map_err(|e| e.to_string())?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(())
}

fn is_upper_snake_case(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_uppercase() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

fn command_resolves(command: &str) -> bool {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return false;
    }
    let command_name = trimmed.split_whitespace().next().unwrap_or(trimmed);
    if command_name.contains('/') || command_name.contains('\\') {
        Path::new(command_name).exists()
    } else {
        runtime::agent_export::detect_binary(command_name)
    }
}

fn parse_http_host_port(raw_url: &str) -> Option<(String, u16)> {
    let trimmed = raw_url.trim();
    let (scheme, rest) = trimmed.split_once("://")?;
    let default_port = match scheme.to_ascii_lowercase().as_str() {
        "http" => 80,
        "https" => 443,
        _ => return None,
    };
    let authority = rest
        .split('/')
        .next()
        .unwrap_or(rest)
        .split('?')
        .next()
        .unwrap_or(rest)
        .split('#')
        .next()
        .unwrap_or(rest);
    let host_port = authority.rsplit('@').next().unwrap_or(authority).trim();
    if host_port.is_empty() {
        return None;
    }
    if let Some(stripped) = host_port.strip_prefix('[') {
        let end = stripped.find(']')?;
        let host = stripped[..end].trim();
        if host.is_empty() {
            return None;
        }
        let remainder = &stripped[end + 1..];
        let port = if let Some(port_str) = remainder.strip_prefix(':') {
            port_str.parse::<u16>().ok()?
        } else {
            default_port
        };
        return Some((host.to_string(), port));
    }
    let mut split = host_port.splitn(2, ':');
    let host = split.next().unwrap_or("").trim();
    if host.is_empty() {
        return None;
    }
    let port = split
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(default_port);
    Some((host.to_string(), port))
}

fn now_epoch_secs_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn normalize_scope(scope: Option<&str>) -> &str {
    match scope.map(str::trim).map(str::to_ascii_lowercase) {
        Some(value) if value == "project" => "project",
        _ => "global",
    }
}

fn default_discovery_cache() -> AgentDiscoveryCache {
    AgentDiscoveryCache {
        version: 1,
        updated_at: now_epoch_secs_string(),
        mcp_tools: HashMap::new(),
        shell_commands: Vec::new(),
        filesystem_paths: Vec::new(),
    }
}

fn discovery_cache_path(
    scope: Option<&str>,
    project_dir: Option<&Path>,
) -> Result<PathBuf, String> {
    match normalize_scope(scope) {
        "project" => {
            let Some(project) = project_dir else {
                return Err("Project scope discovery requires an active project.".to_string());
            };
            Ok(project.join("agents").join("discovery-cache.json"))
        }
        _ => {
            let global = runtime::project::get_global_dir().map_err(|err| err.to_string())?;
            Ok(global.join("agents").join("discovery-cache.json"))
        }
    }
}

fn load_discovery_cache(path: &Path) -> AgentDiscoveryCache {
    let Ok(content) = fs::read_to_string(path) else {
        return default_discovery_cache();
    };
    serde_json::from_str::<AgentDiscoveryCache>(&content)
        .unwrap_or_else(|_| default_discovery_cache())
}

fn save_discovery_cache(path: &Path, cache: &AgentDiscoveryCache) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let content = serde_json::to_string_pretty(cache).map_err(|err| err.to_string())?;
    fs::write(path, content).map_err(|err| err.to_string())
}

fn discover_shell_commands(limit: usize) -> Vec<String> {
    let seeded = [
        "ship",
        "gh",
        "git",
        "bash",
        "sh",
        "zsh",
        "fish",
        "ls",
        "cat",
        "rg",
        "sed",
        "awk",
        "find",
        "xargs",
        "curl",
        "wget",
        "jq",
        "node",
        "pnpm",
        "npm",
        "yarn",
        "python",
        "python3",
        "pip",
        "uv",
        "uvx",
        "cargo",
        "rustc",
        "go",
        "docker",
        "kubectl",
        "terraform",
        "claude",
        "gemini",
        "codex",
    ];
    let mut commands = HashSet::<String>::new();
    for item in seeded {
        commands.insert(item.to_string());
    }

    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if !dir.exists() || !dir.is_dir() {
                continue;
            }
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                if commands.len() >= limit {
                    break;
                }
                let Ok(metadata) = entry.metadata() else {
                    continue;
                };
                if !metadata.is_file() {
                    continue;
                }
                #[cfg(unix)]
                if metadata.permissions().mode() & 0o111 == 0 {
                    continue;
                }
                let file_name = entry.file_name();
                let Some(name) = file_name.to_str().map(str::trim) else {
                    continue;
                };
                if name.is_empty() || name.starts_with('.') || name.contains(char::is_whitespace) {
                    continue;
                }
                commands.insert(name.to_string());
            }
            if commands.len() >= limit {
                break;
            }
        }
    }

    let mut sorted: Vec<String> = commands.into_iter().collect();
    sorted.sort_unstable();
    sorted.truncate(limit);
    sorted
}

fn discover_filesystem_paths(base: &Path, limit: usize) -> Vec<String> {
    let mut paths = HashSet::<String>::new();
    let seeded = [
        ".ship/**",
        "src/**",
        "docs/**",
        "tests/**",
        "scripts/**",
        ".github/**",
        "crates/**",
        "apps/**",
    ];
    for item in seeded {
        paths.insert(item.to_string());
    }

    if base.exists() && base.is_dir() {
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                if paths.len() >= limit {
                    break;
                }
                let Ok(metadata) = entry.metadata() else {
                    continue;
                };
                let file_name = entry.file_name();
                let Some(name) = file_name.to_str().map(str::trim) else {
                    continue;
                };
                if name.is_empty() {
                    continue;
                }
                if metadata.is_dir() {
                    paths.insert(format!("{}/**", name));
                } else {
                    paths.insert(name.to_string());
                }
            }
        }
    }

    let mut sorted: Vec<String> = paths.into_iter().collect();
    sorted.sort_unstable();
    sorted.truncate(limit);
    sorted
}

fn refresh_discovery_cache_data(cache: &mut AgentDiscoveryCache, base: &Path) {
    cache.version = 1;
    cache.updated_at = now_epoch_secs_string();
    cache.shell_commands = discover_shell_commands(500);
    cache.filesystem_paths = discover_filesystem_paths(base, 300);
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct SkillFrontmatterHints {
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "allowed-tools", default)]
    allowed_tools: Option<Vec<String>>,
}

fn parse_skill_tool_hint(skill_dir: &Path) -> Option<SkillToolHint> {
    let id = skill_dir.file_name()?.to_str()?.to_string();
    let path = skill_dir.join("SKILL.md");
    let raw = fs::read_to_string(path).ok()?;
    if !raw.starts_with("---\n") {
        return None;
    }
    let rest = &raw[4..];
    let end = rest.find("\n---")?;
    let yaml = &rest[..end];
    let hints = serde_yaml::from_str::<SkillFrontmatterHints>(yaml).ok()?;
    let allowed_tools = hints.allowed_tools.unwrap_or_default();
    if allowed_tools.is_empty() {
        return None;
    }
    Some(SkillToolHint {
        id: id.clone(),
        name: hints.name.unwrap_or(id),
        allowed_tools,
    })
}

fn list_skill_tool_hints_from_dir(root: &Path) -> Vec<SkillToolHint> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }
            parse_skill_tool_hint(&path)
        })
        .collect()
}

fn infer_probe_server_id(server: &McpServerConfig, index: usize) -> String {
    let trimmed = server.id.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }
    let fallback_name = server.name.trim();
    if !fallback_name.is_empty() {
        let mut out = String::with_capacity(fallback_name.len());
        let mut prev_dash = false;
        for ch in fallback_name.to_ascii_lowercase().chars() {
            if ch.is_ascii_alphanumeric() {
                out.push(ch);
                prev_dash = false;
            } else if !prev_dash {
                out.push('-');
                prev_dash = true;
            }
        }
        let slug = out.trim_matches('-').to_string();
        if !slug.is_empty() {
            return slug;
        }
    }
    format!("server-{}", index + 1)
}

fn summarize_mcp_stderr(raw: &str) -> Option<String> {
    let compact = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = compact.trim();
    if trimmed.is_empty() {
        return None;
    }
    let excerpt: String = trimmed.chars().take(220).collect();
    if trimmed.chars().count() > excerpt.chars().count() {
        Some(format!("{}…", excerpt))
    } else {
        Some(excerpt)
    }
}

fn write_mcp_frame(writer: &mut impl Write, value: &serde_json::Value) -> Result<(), String> {
    let body = serde_json::to_vec(value).map_err(|err| err.to_string())?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    writer
        .write_all(header.as_bytes())
        .map_err(|err| err.to_string())?;
    writer.write_all(&body).map_err(|err| err.to_string())?;
    writer.flush().map_err(|err| err.to_string())
}

fn read_mcp_frame(reader: &mut impl BufRead) -> Result<Option<serde_json::Value>, String> {
    let mut line = String::new();
    let mut content_length: Option<usize> = None;
    loop {
        line.clear();
        let read = reader.read_line(&mut line).map_err(|err| err.to_string())?;
        if read == 0 {
            return Ok(None);
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            if content_length.is_some() {
                break;
            }
            continue;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse::<usize>().ok();
            }
        }
    }

    let Some(length) = content_length else {
        return Err("Missing Content-Length header in MCP frame.".to_string());
    };
    let mut payload = vec![0u8; length];
    reader
        .read_exact(&mut payload)
        .map_err(|err| err.to_string())?;
    serde_json::from_slice::<serde_json::Value>(&payload)
        .map(Some)
        .map_err(|err| err.to_string())
}

fn wait_for_mcp_response(
    rx: &mpsc::Receiver<Result<serde_json::Value, String>>,
    request_id: i64,
    timeout: Duration,
) -> Result<serde_json::Value, String> {
    let deadline = Instant::now() + timeout;
    loop {
        let now = Instant::now();
        if now >= deadline {
            return Err(format!(
                "Timed out waiting for MCP response id {}.",
                request_id
            ));
        }
        let remaining = deadline.saturating_duration_since(now);
        let next = rx
            .recv_timeout(remaining)
            .map_err(|_| format!("Timed out waiting for MCP response id {}.", request_id))?;
        let value = next?;
        let matched_id = value
            .get("id")
            .and_then(|v| v.as_i64())
            .is_some_and(|id| id == request_id);
        if matched_id {
            return Ok(value);
        }
    }
}

fn probe_mcp_stdio_server(
    server: &McpServerConfig,
    server_id: &str,
    cwd: Option<&Path>,
) -> McpProbeServerReport {
    let start = Instant::now();
    let mut warnings: Vec<String> = Vec::new();

    let command_value = server.command.trim().to_string();
    if command_value.is_empty() {
        return McpProbeServerReport {
            server_id: server_id.to_string(),
            server_name: server.name.clone(),
            transport: "stdio".to_string(),
            ok: false,
            status: "needs-attention".to_string(),
            message: Some("Command is required for stdio MCP probing.".to_string()),
            warnings,
            discovered_tools: Vec::new(),
            duration_ms: start.elapsed().as_millis() as u64,
        };
    }

    let mut program = command_value.clone();
    let mut args = server.args.clone();
    if args.is_empty() {
        let split: Vec<String> = command_value
            .split_whitespace()
            .map(|part| part.to_string())
            .collect();
        if split.len() > 1 {
            program = split[0].clone();
            args = split[1..].to_vec();
            warnings.push(
                "Command includes inline args. Probe split this into binary + args.".to_string(),
            );
        }
    }

    let mut command = ProcessCommand::new(&program);
    command
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    for (key, value) in &server.env {
        if value.trim().is_empty() {
            continue;
        }
        command.env(key, value);
    }

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            return McpProbeServerReport {
                server_id: server_id.to_string(),
                server_name: server.name.clone(),
                transport: "stdio".to_string(),
                ok: false,
                status: "needs-attention".to_string(),
                message: Some(format!("Failed to start '{}': {}", program, err)),
                warnings,
                discovered_tools: Vec::new(),
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }
    };

    let stderr_handle = child.stderr.take().map(|mut stderr| {
        thread::spawn(move || {
            let mut output = String::new();
            let _ = stderr.read_to_string(&mut output);
            output
        })
    });
    let (frame_tx, frame_rx) = mpsc::channel::<Result<serde_json::Value, String>>();
    let frame_reader_handle = child.stdout.take().map(|stdout| {
        thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stdout);
            loop {
                match read_mcp_frame(&mut reader) {
                    Ok(Some(frame)) => {
                        if frame_tx.send(Ok(frame)).is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(err) => {
                        let _ = frame_tx.send(Err(err));
                        break;
                    }
                }
            }
        })
    });

    let timeout_secs = server.timeout_secs.unwrap_or(8).clamp(2, 30);
    let timeout = Duration::from_secs(timeout_secs as u64);

    let mut outcome = McpProbeServerReport {
        server_id: server_id.to_string(),
        server_name: server.name.clone(),
        transport: "stdio".to_string(),
        ok: false,
        status: "needs-attention".to_string(),
        message: None,
        warnings,
        discovered_tools: Vec::new(),
        duration_ms: 0,
    };

    if let Some(stdin) = child.stdin.as_mut() {
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "ship",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });
        if let Err(err) = write_mcp_frame(stdin, &init_request) {
            outcome.message = Some(format!("Failed to send initialize request: {}", err));
        } else {
            match wait_for_mcp_response(&frame_rx, 1, timeout) {
                Ok(init_response) => {
                    if let Some(error) = init_response.get("error") {
                        let message = error
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown initialize error");
                        outcome.message =
                            Some(format!("MCP initialize failed: {}", message.trim()));
                    } else {
                        let initialized = serde_json::json!({
                            "jsonrpc": "2.0",
                            "method": "notifications/initialized",
                            "params": {}
                        });
                        let tools_list = serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": 2,
                            "method": "tools/list",
                            "params": {}
                        });
                        if let Err(err) = write_mcp_frame(stdin, &initialized) {
                            outcome.message =
                                Some(format!("Failed to send initialized notification: {}", err));
                        } else if let Err(err) = write_mcp_frame(stdin, &tools_list) {
                            outcome.message =
                                Some(format!("Failed to request tools/list: {}", err));
                        } else {
                            match wait_for_mcp_response(&frame_rx, 2, timeout) {
                                Ok(tool_response) => {
                                    if let Some(error) = tool_response.get("error") {
                                        let message = error
                                            .get("message")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("Unknown tools/list error");
                                        outcome.status = "partial".to_string();
                                        outcome.ok = false;
                                        outcome.message = Some(format!(
                                            "tools/list not available: {}",
                                            message.trim()
                                        ));
                                    } else {
                                        let discovered = tool_response
                                            .get("result")
                                            .and_then(|result| result.get("tools"))
                                            .and_then(|tools| tools.as_array())
                                            .map(|tools| {
                                                tools
                                                    .iter()
                                                    .filter_map(|tool| {
                                                        let name = tool
                                                            .get("name")
                                                            .and_then(|value| value.as_str())
                                                            .map(str::trim)
                                                            .filter(|value| !value.is_empty())?
                                                            .to_string();
                                                        let description = tool
                                                            .get("description")
                                                            .and_then(|value| value.as_str())
                                                            .map(str::trim)
                                                            .filter(|value| !value.is_empty())
                                                            .map(str::to_string);
                                                        Some(McpDiscoveredTool {
                                                            name,
                                                            description,
                                                        })
                                                    })
                                                    .collect::<Vec<_>>()
                                            })
                                            .unwrap_or_default();
                                        outcome.discovered_tools = discovered;
                                        outcome.ok = true;
                                        outcome.status = "ready".to_string();
                                        outcome.message = Some(format!(
                                            "Discovered {} tool{} via runtime probe.",
                                            outcome.discovered_tools.len(),
                                            if outcome.discovered_tools.len() == 1 {
                                                ""
                                            } else {
                                                "s"
                                            }
                                        ));
                                    }
                                }
                                Err(err) => {
                                    outcome.message = Some(err);
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    outcome.message = Some(err);
                }
            }
        }
    } else {
        outcome.message = Some("MCP process stdin is unavailable.".to_string());
    }

    let _ = child.kill();
    let _ = child.wait();
    if let Some(handle) = frame_reader_handle {
        let _ = handle.join();
    }
    if let Some(handle) = stderr_handle {
        if let Ok(stderr_output) = handle.join() {
            if let Some(stderr_excerpt) = summarize_mcp_stderr(&stderr_output) {
                outcome.warnings.push(format!("stderr: {}", stderr_excerpt));
            }
        }
    }

    outcome.duration_ms = start.elapsed().as_millis() as u64;
    outcome
}

fn probe_mcp_network_server(
    server: &McpServerConfig,
    server_id: &str,
    transport: &str,
) -> McpProbeServerReport {
    let start = Instant::now();
    let mut report = McpProbeServerReport {
        server_id: server_id.to_string(),
        server_name: server.name.clone(),
        transport: transport.to_string(),
        ok: false,
        status: "needs-attention".to_string(),
        message: None,
        warnings: Vec::new(),
        discovered_tools: Vec::new(),
        duration_ms: 0,
    };
    let Some(url) = server
        .url
        .as_deref()
        .map(str::trim)
        .filter(|url| !url.is_empty())
    else {
        report.message = Some("URL is required for network MCP transports.".to_string());
        report.duration_ms = start.elapsed().as_millis() as u64;
        return report;
    };

    let Some((host, port)) = parse_http_host_port(url) else {
        report.message = Some(format!("URL appears invalid: {}", url));
        report.duration_ms = start.elapsed().as_millis() as u64;
        return report;
    };
    let address = format!("{}:{}", host, port);
    match address.to_socket_addrs() {
        Ok(mut addresses) => {
            if let Some(addr) = addresses.next() {
                if TcpStream::connect_timeout(&addr, Duration::from_millis(900)).is_ok() {
                    report.ok = true;
                    report.status = "partial".to_string();
                    report.message = Some(
                        "Endpoint is reachable. Runtime tool discovery currently supports stdio servers."
                            .to_string(),
                    );
                } else {
                    report.message = Some(format!("Endpoint is not reachable at {}.", address));
                }
            } else {
                report.message = Some(format!("No socket address resolved for {}.", address));
            }
        }
        Err(_) => {
            report.message = Some(format!("Host could not be resolved for {}.", address));
        }
    }
    report.duration_ms = start.elapsed().as_millis() as u64;
    report
}

fn validate_provider_configs(
    project_dir: Option<&Path>,
    issues: &mut Vec<McpValidationIssue>,
) -> usize {
    let Some(project_dir) = project_dir else {
        return 0;
    };

    let project_root = project_dir.parent().unwrap_or(project_dir);
    let home_dir = dirs::home_dir();
    let mut checked = 0usize;

    for provider in runtime::agent_export::PROVIDERS {
        let mut candidate_paths = Vec::new();
        candidate_paths.push(project_root.join(provider.project_config));
        if let Some(home) = home_dir.as_ref() {
            candidate_paths.push(home.join(provider.global_config));
        }

        let mut seen = HashSet::new();
        for path in candidate_paths {
            if !seen.insert(path.clone()) || !path.exists() {
                continue;
            }
            checked += 1;
            let raw = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(err) => {
                    issues.push(McpValidationIssue {
                        level: "error".to_string(),
                        code: "provider-config-read".to_string(),
                        message: format!("Could not read provider config: {}", err),
                        hint: Some("Check file permissions and path validity.".to_string()),
                        server_id: None,
                        provider_id: Some(provider.id.to_string()),
                        source_path: Some(path.to_string_lossy().to_string()),
                    });
                    continue;
                }
            };

            match provider.config_format {
                runtime::agent_export::ConfigFormat::Json => {
                    let root = match serde_json::from_str::<serde_json::Value>(&raw) {
                        Ok(value) => value,
                        Err(err) => {
                            issues.push(McpValidationIssue {
                                level: "error".to_string(),
                                code: "provider-config-json".to_string(),
                                message: format!("Invalid JSON: {}", err),
                                hint: Some("Fix JSON syntax before export/sync.".to_string()),
                                server_id: None,
                                provider_id: Some(provider.id.to_string()),
                                source_path: Some(path.to_string_lossy().to_string()),
                            });
                            continue;
                        }
                    };

                    if !root.is_object() {
                        issues.push(McpValidationIssue {
                            level: "warning".to_string(),
                            code: "provider-config-root".to_string(),
                            message: "JSON root is not an object.".to_string(),
                            hint: Some("Provider settings should be a JSON object.".to_string()),
                            server_id: None,
                            provider_id: Some(provider.id.to_string()),
                            source_path: Some(path.to_string_lossy().to_string()),
                        });
                        continue;
                    }

                    if let Some(mcp_value) = root.get(provider.mcp_key) {
                        if !mcp_value.is_object() {
                            issues.push(McpValidationIssue {
                                level: "warning".to_string(),
                                code: "provider-config-mcp-key".to_string(),
                                message: format!(
                                    "'{}' is present but not an object.",
                                    provider.mcp_key
                                ),
                                hint: Some(
                                    "Set MCP server collection to a JSON object keyed by server id."
                                        .to_string(),
                                ),
                                server_id: None,
                                provider_id: Some(provider.id.to_string()),
                                source_path: Some(path.to_string_lossy().to_string()),
                            });
                        }
                    }
                }
                runtime::agent_export::ConfigFormat::Toml => {
                    let root = match toml::from_str::<toml::Value>(&raw) {
                        Ok(value) => value,
                        Err(err) => {
                            issues.push(McpValidationIssue {
                                level: "error".to_string(),
                                code: "provider-config-toml".to_string(),
                                message: format!("Invalid TOML: {}", err),
                                hint: Some(
                                    "For Codex, ensure MCP section uses 'mcp_servers' (underscore)."
                                        .to_string(),
                                ),
                                server_id: None,
                                provider_id: Some(provider.id.to_string()),
                                source_path: Some(path.to_string_lossy().to_string()),
                            });
                            continue;
                        }
                    };

                    let Some(table) = root.as_table() else {
                        issues.push(McpValidationIssue {
                            level: "warning".to_string(),
                            code: "provider-config-root".to_string(),
                            message: "TOML root is not a table.".to_string(),
                            hint: Some("Provider settings should be a TOML table.".to_string()),
                            server_id: None,
                            provider_id: Some(provider.id.to_string()),
                            source_path: Some(path.to_string_lossy().to_string()),
                        });
                        continue;
                    };
                    if let Some(mcp_value) = table.get(provider.mcp_key) {
                        if !mcp_value.is_table() {
                            issues.push(McpValidationIssue {
                                level: "warning".to_string(),
                                code: "provider-config-mcp-key".to_string(),
                                message: format!(
                                    "'{}' is present but not a table.",
                                    provider.mcp_key
                                ),
                                hint: Some(
                                    "Set MCP servers as TOML tables keyed by server id."
                                        .to_string(),
                                ),
                                server_id: None,
                                provider_id: Some(provider.id.to_string()),
                                source_path: Some(path.to_string_lossy().to_string()),
                            });
                        }
                    }
                }
            }
        }
    }

    checked
}

#[tauri::command]
#[specta::specta]
fn validate_mcp_servers_cmd(
    servers: Vec<McpServerConfig>,
    state: State<AppState>,
) -> Result<McpValidationReport, String> {
    let project_dir = get_active_dir(&state).ok();
    let mut issues: Vec<McpValidationIssue> = Vec::new();
    let mut seen_ids = HashSet::new();

    for (index, server) in servers.iter().enumerate() {
        let trimmed_id = server.id.trim();
        let resolved_id = if trimmed_id.is_empty() {
            format!("server-{}", index + 1)
        } else {
            trimmed_id.to_string()
        };

        if trimmed_id.is_empty() {
            issues.push(McpValidationIssue {
                level: "error".to_string(),
                code: "server-id-missing".to_string(),
                message: "Server id is missing.".to_string(),
                hint: Some(
                    "Set a stable id (slug) for permission and mode references.".to_string(),
                ),
                server_id: Some(resolved_id.clone()),
                provider_id: None,
                source_path: None,
            });
        } else if !seen_ids.insert(trimmed_id.to_string()) {
            issues.push(McpValidationIssue {
                level: "warning".to_string(),
                code: "server-id-duplicate".to_string(),
                message: "Duplicate MCP server id detected.".to_string(),
                hint: Some("Use unique ids so exports and modes resolve correctly.".to_string()),
                server_id: Some(resolved_id.clone()),
                provider_id: None,
                source_path: None,
            });
        }

        if !matches!(server.scope.as_str(), "global" | "project" | "mode") {
            issues.push(McpValidationIssue {
                level: "warning".to_string(),
                code: "server-scope-unknown".to_string(),
                message: format!("Unexpected scope '{}'.", server.scope),
                hint: Some("Use one of: global, project, mode.".to_string()),
                server_id: Some(resolved_id.clone()),
                provider_id: None,
                source_path: None,
            });
        }

        if server.disabled {
            issues.push(McpValidationIssue {
                level: "info".to_string(),
                code: "server-disabled".to_string(),
                message: "Server is disabled and will not be started.".to_string(),
                hint: None,
                server_id: Some(resolved_id.clone()),
                provider_id: None,
                source_path: None,
            });
        }

        let bad_env = server
            .env
            .keys()
            .filter(|key| !is_upper_snake_case(key))
            .cloned()
            .collect::<Vec<_>>();
        if !bad_env.is_empty() {
            issues.push(McpValidationIssue {
                level: "warning".to_string(),
                code: "server-env-key-format".to_string(),
                message: format!(
                    "Env keys should be uppercase snake_case: {}",
                    bad_env.join(", ")
                ),
                hint: None,
                server_id: Some(resolved_id.clone()),
                provider_id: None,
                source_path: None,
            });
        }

        let empty_secret_env = server
            .env
            .iter()
            .filter(|(key, value)| {
                (key.contains("TOKEN")
                    || key.contains("KEY")
                    || key.contains("SECRET")
                    || key.contains("PASSWORD"))
                    && value.trim().is_empty()
            })
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        if !empty_secret_env.is_empty() {
            issues.push(McpValidationIssue {
                level: "info".to_string(),
                code: "server-env-secret-empty".to_string(),
                message: format!("Secret env vars are empty: {}", empty_secret_env.join(", ")),
                hint: Some("Set values before starting/exporting this server.".to_string()),
                server_id: Some(resolved_id.clone()),
                provider_id: None,
                source_path: None,
            });
        }

        match server.server_type {
            McpServerType::Stdio => {
                let command = server.command.trim();
                if command.is_empty() {
                    issues.push(McpValidationIssue {
                        level: "error".to_string(),
                        code: "server-command-missing".to_string(),
                        message: "Command is required for stdio transport.".to_string(),
                        hint: None,
                        server_id: Some(resolved_id.clone()),
                        provider_id: None,
                        source_path: None,
                    });
                } else {
                    let has_shell_operators = command.contains("&&")
                        || command.contains("||")
                        || command.contains(';')
                        || (command.contains('|') && !command.contains("://"));
                    if has_shell_operators {
                        issues.push(McpValidationIssue {
                            level: "warning".to_string(),
                            code: "server-command-shell-operators".to_string(),
                            message: "Command includes shell operators.".to_string(),
                            hint: Some(
                                "Split command and args to avoid parser/runtime issues."
                                    .to_string(),
                            ),
                            server_id: Some(resolved_id.clone()),
                            provider_id: None,
                            source_path: None,
                        });
                    }
                    if command.contains(' ') && server.args.is_empty() {
                        issues.push(McpValidationIssue {
                            level: "info".to_string(),
                            code: "server-command-split".to_string(),
                            message: "Command contains spaces and no args.".to_string(),
                            hint: Some(
                                "Prefer command as binary + separate args array.".to_string(),
                            ),
                            server_id: Some(resolved_id.clone()),
                            provider_id: None,
                            source_path: None,
                        });
                    }
                    if !command_resolves(command) {
                        issues.push(McpValidationIssue {
                            level: "warning".to_string(),
                            code: "server-command-not-found".to_string(),
                            message: format!("Command '{}' is not currently resolvable.", command),
                            hint: Some("Install the binary or update command/path.".to_string()),
                            server_id: Some(resolved_id.clone()),
                            provider_id: None,
                            source_path: None,
                        });
                    }
                }

                let unresolved_args = server
                    .args
                    .iter()
                    .filter(|arg| arg.starts_with('{') && arg.ends_with('}'))
                    .cloned()
                    .collect::<Vec<_>>();
                if !unresolved_args.is_empty() {
                    issues.push(McpValidationIssue {
                        level: "info".to_string(),
                        code: "server-args-placeholder".to_string(),
                        message: format!(
                            "Argument placeholders must be replaced: {}",
                            unresolved_args.join(", ")
                        ),
                        hint: None,
                        server_id: Some(resolved_id.clone()),
                        provider_id: None,
                        source_path: None,
                    });
                }
            }
            McpServerType::Sse | McpServerType::Http => {
                let Some(url) = server
                    .url
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                else {
                    issues.push(McpValidationIssue {
                        level: "error".to_string(),
                        code: "server-url-missing".to_string(),
                        message: "URL is required for HTTP/SSE transport.".to_string(),
                        hint: None,
                        server_id: Some(resolved_id.clone()),
                        provider_id: None,
                        source_path: None,
                    });
                    continue;
                };

                let Some((host, port)) = parse_http_host_port(url) else {
                    issues.push(McpValidationIssue {
                        level: "error".to_string(),
                        code: "server-url-invalid".to_string(),
                        message: format!("URL appears invalid: '{}'", url),
                        hint: Some(
                            "Use a fully-qualified http:// or https:// URL with host.".to_string(),
                        ),
                        server_id: Some(resolved_id.clone()),
                        provider_id: None,
                        source_path: None,
                    });
                    continue;
                };

                let address = format!("{}:{}", host, port);
                match address.to_socket_addrs() {
                    Ok(mut addrs) => {
                        if let Some(first) = addrs.next() {
                            let reachable =
                                TcpStream::connect_timeout(&first, Duration::from_millis(900))
                                    .is_ok();
                            if !reachable {
                                issues.push(McpValidationIssue {
                                    level: "warning".to_string(),
                                    code: "server-url-unreachable".to_string(),
                                    message: format!("Endpoint is not reachable at {}.", address),
                                    hint: Some(
                                        "Ensure the server is running and network/firewall rules allow access."
                                            .to_string(),
                                    ),
                                    server_id: Some(resolved_id.clone()),
                                    provider_id: None,
                                    source_path: None,
                                });
                            }
                        }
                    }
                    Err(_) => {
                        issues.push(McpValidationIssue {
                            level: "warning".to_string(),
                            code: "server-url-resolve-failed".to_string(),
                            message: format!("Host could not be resolved for {}.", address),
                            hint: Some("Check DNS/hostname and URL spelling.".to_string()),
                            server_id: Some(resolved_id.clone()),
                            provider_id: None,
                            source_path: None,
                        });
                    }
                }
            }
        }
    }

    let checked_provider_configs = validate_provider_configs(project_dir.as_deref(), &mut issues);
    let ok = !issues.iter().any(|issue| issue.level == "error");

    Ok(McpValidationReport {
        ok,
        checked_servers: servers.len(),
        checked_provider_configs,
        issues,
    })
}

#[tauri::command]
#[specta::specta]
async fn probe_mcp_servers_cmd(
    servers: Vec<McpServerConfig>,
    scope: Option<String>,
    state: State<'_, AppState>,
) -> Result<McpProbeReport, String> {
    let active_dir = get_active_dir(&state).ok();
    tokio::task::spawn_blocking(move || {
        let cwd = active_dir.as_ref().map(PathBuf::as_path);
        let mut results = Vec::with_capacity(servers.len());
        for (index, server) in servers.iter().enumerate() {
            let server_id = infer_probe_server_id(server, index);
            if server.disabled {
                results.push(McpProbeServerReport {
                    server_id: server_id.clone(),
                    server_name: server.name.clone(),
                    transport: match server.server_type {
                        McpServerType::Stdio => "stdio".to_string(),
                        McpServerType::Sse => "sse".to_string(),
                        McpServerType::Http => "http".to_string(),
                    },
                    ok: false,
                    status: "disabled".to_string(),
                    message: Some("Server is disabled in current config.".to_string()),
                    warnings: Vec::new(),
                    discovered_tools: Vec::new(),
                    duration_ms: 0,
                });
                continue;
            }

            let report = match server.server_type {
                McpServerType::Stdio => probe_mcp_stdio_server(server, &server_id, cwd),
                McpServerType::Sse => probe_mcp_network_server(server, &server_id, "sse"),
                McpServerType::Http => probe_mcp_network_server(server, &server_id, "http"),
            };
            results.push(report);
        }

        let reachable_servers = results.iter().filter(|row| row.ok).count();
        let discovered_tools = results
            .iter()
            .map(|row| row.discovered_tools.len())
            .sum::<usize>();
        let generated_at = now_epoch_secs_string();

        if let Ok(cache_path) = discovery_cache_path(scope.as_deref(), active_dir.as_deref()) {
            let mut cache = load_discovery_cache(&cache_path);
            for row in &results {
                if !row.discovered_tools.is_empty() {
                    cache
                        .mcp_tools
                        .insert(row.server_id.clone(), row.discovered_tools.clone());
                } else if row.status == "ready" {
                    cache.mcp_tools.insert(row.server_id.clone(), Vec::new());
                }
            }
            cache.updated_at = generated_at.clone();
            let _ = save_discovery_cache(&cache_path, &cache);
        }

        Ok(McpProbeReport {
            generated_at,
            checked_servers: servers.len(),
            reachable_servers,
            discovered_tools,
            results,
        })
    })
    .await
    .map_err(|err| err.to_string())?
}

#[tauri::command]
#[specta::specta]
fn get_agent_discovery_cache_cmd(
    scope: Option<String>,
    state: State<AppState>,
) -> Result<AgentDiscoveryCache, String> {
    let active_dir = get_active_dir(&state).ok();
    let path = discovery_cache_path(scope.as_deref(), active_dir.as_deref())?;
    Ok(load_discovery_cache(&path))
}

#[tauri::command]
#[specta::specta]
fn refresh_agent_discovery_cache_cmd(
    scope: Option<String>,
    state: State<AppState>,
) -> Result<AgentDiscoveryCache, String> {
    let active_dir = get_active_dir(&state).ok();
    let normalized_scope = normalize_scope(scope.as_deref()).to_string();
    let cache_path = discovery_cache_path(Some(&normalized_scope), active_dir.as_deref())?;
    let mut cache = load_discovery_cache(&cache_path);

    let base = if normalized_scope == "project" {
        let Some(dir) = active_dir.as_ref() else {
            return Err("Project scope discovery requires an active project.".to_string());
        };
        dir.parent().unwrap_or(dir).to_path_buf()
    } else {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
    };
    refresh_discovery_cache_data(&mut cache, &base);
    save_discovery_cache(&cache_path, &cache)?;
    Ok(cache)
}

#[tauri::command]
#[specta::specta]
fn list_skill_tool_hints_cmd(
    scope: Option<String>,
    state: State<AppState>,
) -> Result<Vec<SkillToolHint>, String> {
    let active_dir = get_active_dir(&state).ok();
    let scope_value = scope
        .as_deref()
        .map(str::trim)
        .unwrap_or("effective")
        .to_ascii_lowercase();

    let mut roots: Vec<PathBuf> = Vec::new();
    match scope_value.as_str() {
        "project" => {
            let Some(project_dir) = active_dir.as_ref() else {
                return Err("Project scope skill hints require an active project.".to_string());
            };
            roots.push(runtime::project::skills_dir(project_dir));
        }
        "user" | "global" => {
            roots.push(runtime::project::user_skills_dir());
        }
        _ => {
            if let Some(project_dir) = active_dir.as_ref() {
                roots.push(runtime::project::skills_dir(project_dir));
            }
            roots.push(runtime::project::user_skills_dir());
        }
    }

    let mut by_id = HashMap::<String, SkillToolHint>::new();
    for root in roots {
        for hint in list_skill_tool_hints_from_dir(&root) {
            by_id.entry(hint.id.clone()).or_insert(hint);
        }
    }
    let mut hints: Vec<SkillToolHint> = by_id.into_values().collect();
    hints.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(hints)
}

fn slugify_registry_server_id(input: &str) -> String {
    let tail = input.rsplit('/').next().unwrap_or(input);
    let mut out = String::with_capacity(tail.len());
    let mut prev_dash = false;
    for ch in tail.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.starts_with('-') {
        out.remove(0);
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "mcp-server".to_string()
    } else {
        out
    }
}

fn parse_registry_arg_values(args: Option<&Vec<serde_json::Value>>) -> Vec<String> {
    let Some(args) = args else {
        return Vec::new();
    };
    let mut values = Vec::new();
    for arg in args {
        let Some(arg_obj) = arg.as_object() else {
            continue;
        };
        if let Some(value) = arg_obj
            .get("value")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            values.push(value.to_string());
            continue;
        }
        if let Some(name) = arg_obj
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            values.push(name.to_string());
        }
    }
    values
}

fn parse_required_key_values(values: Option<&Vec<serde_json::Value>>) -> Vec<String> {
    let Some(values) = values else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for item in values {
        let Some(item_obj) = item.as_object() else {
            continue;
        };
        let required = item_obj
            .get("isRequired")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !required {
            continue;
        }
        if let Some(name) = item_obj
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            names.push(name.to_string());
        }
    }
    names
}

fn mcp_registry_entry_from_json(item: &serde_json::Value) -> Option<McpRegistryEntry> {
    let server = item.get("server")?.as_object()?;
    let server_name = server.get("name")?.as_str()?.trim().to_string();
    if server_name.is_empty() {
        return None;
    }

    let title = server
        .get("title")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(server_name.as_str())
        .to_string();
    let description = server
        .get("description")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("MCP server")
        .to_string();
    let version = server
        .get("version")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("latest")
        .to_string();
    let source_url = server
        .get("repository")
        .and_then(|v| v.as_object())
        .and_then(|repo| repo.get("url"))
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let website_url = server
        .get("websiteUrl")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let mut transport = "stdio".to_string();
    let mut command: Option<String> = None;
    let mut args = Vec::new();
    let mut url: Option<String> = None;
    let mut required_env = Vec::new();
    let mut required_headers = Vec::new();

    if let Some(packages) = server.get("packages").and_then(|v| v.as_array()) {
        let preferred = packages
            .iter()
            .find(|pkg| {
                pkg.get("transport")
                    .and_then(|t| t.as_object())
                    .and_then(|t| t.get("type"))
                    .and_then(|v| v.as_str())
                    == Some("stdio")
            })
            .or_else(|| packages.first());

        if let Some(package) = preferred.and_then(|v| v.as_object()) {
            let transport_type = package
                .get("transport")
                .and_then(|t| t.as_object())
                .and_then(|t| t.get("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("stdio");
            transport = match transport_type {
                "streamable-http" => "http".to_string(),
                "sse" => "sse".to_string(),
                _ => "stdio".to_string(),
            };
            if transport != "stdio" {
                url = package
                    .get("transport")
                    .and_then(|t| t.as_object())
                    .and_then(|t| t.get("url"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
            }

            let runtime_hint = package
                .get("runtimeHint")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string);
            let runtime_args = parse_registry_arg_values(
                package.get("runtimeArguments").and_then(|v| v.as_array()),
            );
            let package_args = parse_registry_arg_values(
                package.get("packageArguments").and_then(|v| v.as_array()),
            );
            required_env = parse_required_key_values(
                package
                    .get("environmentVariables")
                    .and_then(|v| v.as_array()),
            );

            if transport == "stdio" {
                command = runtime_hint.or_else(|| {
                    package
                        .get("registryType")
                        .and_then(|v| v.as_str())
                        .and_then(|kind| match kind {
                            "npm" => Some("npx".to_string()),
                            _ => None,
                        })
                });
                if !runtime_args.is_empty() {
                    args.extend(runtime_args);
                } else if command.as_deref() == Some("npx") {
                    if let Some(identifier) = package
                        .get("identifier")
                        .and_then(|v| v.as_str())
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                    {
                        let mut resolved = identifier.to_string();
                        let version_hint = package
                            .get("version")
                            .and_then(|v| v.as_str())
                            .map(str::trim)
                            .filter(|v| !v.is_empty());
                        if !resolved.contains('@') {
                            if let Some(version_hint) = version_hint {
                                resolved = format!("{}@{}", resolved, version_hint);
                            }
                        }
                        args.push("-y".to_string());
                        args.push(resolved);
                    }
                }
                args.extend(package_args);
            }
        }
    }

    if command.is_none() && url.is_none() {
        if let Some(remotes) = server.get("remotes").and_then(|v| v.as_array()) {
            if let Some(remote) = remotes.first().and_then(|v| v.as_object()) {
                let remote_type = remote
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("streamable-http");
                transport = if remote_type == "sse" {
                    "sse".to_string()
                } else {
                    "http".to_string()
                };
                url = remote
                    .get("url")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                required_headers =
                    parse_required_key_values(remote.get("headers").and_then(|v| v.as_array()));
            }
        }
    }

    Some(McpRegistryEntry {
        id: slugify_registry_server_id(&server_name),
        server_name,
        title,
        description,
        version,
        transport,
        command,
        args,
        url,
        required_env,
        required_headers,
        source_url,
        website_url,
    })
}

#[tauri::command]
#[specta::specta]
async fn search_mcp_registry_cmd(
    query: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<McpRegistryEntry>, String> {
    tokio::task::spawn_blocking(move || {
        let capped_limit = limit.unwrap_or(20).clamp(1, 100).to_string();
        let mut request = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(3))
            .timeout(Duration::from_secs(7))
            .build()
            .get("https://registry.modelcontextprotocol.io/v0.1/servers")
            .query("version", "latest")
            .query("limit", &capped_limit);

        if let Some(search) = query
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
        {
            request = request.query("search", &search);
        }

        let response = request.call().map_err(|err| err.to_string())?;
        let value: serde_json::Value = response.into_json().map_err(|err| err.to_string())?;
        let rows = value
            .get("servers")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut deduped = HashMap::<String, McpRegistryEntry>::new();
        for row in rows {
            let Some(entry) = mcp_registry_entry_from_json(&row) else {
                continue;
            };
            deduped.entry(entry.server_name.clone()).or_insert(entry);
        }

        let mut entries: Vec<McpRegistryEntry> = deduped.into_values().collect();
        entries.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        Ok(entries)
    })
    .await
    .map_err(|e| e.to_string())?
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
    app_handle: tauri::AppHandle,
    id: String,
    name: String,
    content: String,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<Skill, String> {
    let dir = get_active_dir(&state)?;
    let skill = match scope.as_deref() {
        Some("user") => create_user_skill(&id, &name, &content).map_err(|e| e.to_string()),
        _ => create_skill(&dir, &id, &name, &content).map_err(|e| e.to_string()),
    }?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(skill)
}

#[tauri::command]
#[specta::specta]
fn update_skill_cmd(
    app_handle: tauri::AppHandle,
    id: String,
    name: Option<String>,
    content: Option<String>,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<Skill, String> {
    let dir = get_active_dir(&state)?;
    let skill =
        match scope.as_deref() {
            Some("user") => update_user_skill(&id, name.as_deref(), content.as_deref())
                .map_err(|e| e.to_string()),
            _ => update_skill(&dir, &id, name.as_deref(), content.as_deref())
                .map_err(|e| e.to_string()),
        }?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(skill)
}

#[tauri::command]
#[specta::specta]
fn delete_skill_cmd(
    app_handle: tauri::AppHandle,
    id: String,
    scope: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let dir = get_active_dir(&state)?;
    match scope.as_deref() {
        Some("user") => delete_user_skill(&id).map_err(|e| e.to_string()),
        _ => delete_skill(&dir, &id).map_err(|e| e.to_string()),
    }?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(())
}

#[tauri::command]
#[specta::specta]
fn install_skill_from_source_cmd(
    app_handle: tauri::AppHandle,
    source: String,
    skill_id: String,
    git_ref: Option<String>,
    repo_path: Option<String>,
    scope: Option<String>,
    force: Option<bool>,
    state: State<AppState>,
) -> Result<Skill, String> {
    let active_dir = get_active_dir(&state).ok();
    let install_scope = match scope.as_deref() {
        Some("user") => SkillInstallScope::User,
        _ => SkillInstallScope::Project,
    };
    if matches!(install_scope, SkillInstallScope::Project) && active_dir.is_none() {
        return Err("Project scope install requires an active Ship project".to_string());
    }
    let installed = install_skill_from_source(
        active_dir.as_deref(),
        &source,
        &skill_id,
        git_ref.as_deref(),
        repo_path.as_deref(),
        install_scope,
        force.unwrap_or(false),
    )
    .map_err(|e| e.to_string())?;
    let _ = ShipEvent::ConfigChanged.emit(&app_handle);
    Ok(installed)
}

// ─── Commands: Agents / Providers ─────────────────────────────────────────────

/// List all supported agent providers with enabled + installed status and known models.
#[tauri::command]
#[specta::specta]
fn list_providers_cmd(state: State<AppState>) -> Result<Vec<ProviderInfo>, String> {
    if let Ok(dir) = get_active_dir(&state) {
        return list_providers(&dir).map_err(|e| e.to_string());
    }

    let enabled: HashSet<String> = get_config(None)
        .map(|config| config.providers.into_iter().collect())
        .unwrap_or_default();

    let providers = runtime::agent_export::PROVIDERS
        .iter()
        .map(|descriptor| {
            let installed = detect_binary(descriptor.binary);
            ProviderInfo {
                id: descriptor.id.to_string(),
                name: descriptor.name.to_string(),
                binary: descriptor.binary.to_string(),
                project_config: descriptor.project_config.to_string(),
                global_config: descriptor.global_config.to_string(),
                config_format: match descriptor.config_format {
                    runtime::agent_export::ConfigFormat::Json => "json".to_string(),
                    runtime::agent_export::ConfigFormat::Toml => "toml".to_string(),
                },
                prompt_output: match descriptor.prompt_output {
                    runtime::agent_export::PromptOutput::ClaudeMd => "claude-md".to_string(),
                    runtime::agent_export::PromptOutput::GeminiMd => "gemini-md".to_string(),
                    runtime::agent_export::PromptOutput::AgentsMd => "agents-md".to_string(),
                    runtime::agent_export::PromptOutput::None => "none".to_string(),
                },
                skills_output: match descriptor.skills_output {
                    runtime::agent_export::SkillsOutput::ClaudeSkills => {
                        "claude-skills".to_string()
                    }
                    runtime::agent_export::SkillsOutput::AgentSkills => "agent-skills".to_string(),
                    runtime::agent_export::SkillsOutput::CodexSkills => "codex-skills".to_string(),
                    runtime::agent_export::SkillsOutput::None => "none".to_string(),
                },
                enabled: enabled.contains(descriptor.id),
                installed,
                version: if installed {
                    detect_version(descriptor.binary)
                } else {
                    None
                },
                models: list_models(descriptor.id).unwrap_or_default(),
            }
        })
        .collect();
    Ok(providers)
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

#[tauri::command]
#[specta::specta]
async fn import_agent_config_cmd(
    target: String,
    include_permissions: Option<bool>,
    state: State<'_, AppState>,
) -> Result<ProviderImportReport, String> {
    let dir = get_active_dir(&state)?;
    tokio::task::spawn_blocking(move || {
        let imported_mcp_servers =
            runtime::agent_export::import_from_provider(&target, dir.clone())
                .map_err(|e| e.to_string())?;
        let imported_permissions = if include_permissions.unwrap_or(true) {
            runtime::agent_export::import_permissions_from_provider(&target, dir)
                .map_err(|e| e.to_string())?
        } else {
            false
        };
        Ok(ProviderImportReport {
            imported_mcp_servers,
            imported_permissions,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

// ─── Commands: AI ─────────────────────────────────────────────────────────────

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
            move_spec_cmd,
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
            feature_start_cmd,
            feature_done_cmd,
            update_feature_documentation_cmd,
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
            list_git_branches_cmd,
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
            get_workspace_git_status_cmd,
            get_branch_detail_cmd,
            get_branch_file_diff_cmd,
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
            validate_mcp_servers_cmd,
            probe_mcp_servers_cmd,
            get_agent_discovery_cache_cmd,
            refresh_agent_discovery_cache_cmd,
            search_mcp_registry_cmd,
            // Skills
            list_skills_cmd,
            list_skill_tool_hints_cmd,
            get_skill_cmd,
            create_skill_cmd,
            update_skill_cmd,
            delete_skill_cmd,
            install_skill_from_source_cmd,
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
            import_agent_config_cmd,
            // AI
            generate_adr_cmd,
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
    use std::fs;
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
            workspace_type: ShipWorkspaceKind::Feature,
            status: WorkspaceStatus::Active,
            environment_id: None,
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

    #[test]
    fn infer_probe_server_id_slugifies_name() {
        let server = McpServerConfig {
            id: "".to_string(),
            name: "GitHub MCP Server".to_string(),
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-github".to_string(),
            ],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        };
        let id = infer_probe_server_id(&server, 0);
        assert_eq!(id, "github-mcp-server");
    }

    #[test]
    fn parse_skill_tool_hint_reads_allowed_tools() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos().to_string())
            .unwrap_or_else(|_| "0".to_string());
        let root = std::env::temp_dir().join(format!("ship-skill-hint-{}", stamp));
        let skill_dir = root.join("docs-writer");
        fs::create_dir_all(&skill_dir).expect("create temp skill dir");
        let body = r#"---
name: docs-writer
description: docs helper
allowed-tools:
  - Read
  - mcp__github__issues_list
---

# Skill
"#;
        fs::write(skill_dir.join("SKILL.md"), body).expect("write skill");

        let hint = parse_skill_tool_hint(&skill_dir).expect("expected skill hint");
        assert_eq!(hint.id, "docs-writer");
        assert_eq!(hint.allowed_tools.len(), 2);
        assert!(hint.allowed_tools.contains(&"Read".to_string()));
        assert!(hint
            .allowed_tools
            .contains(&"mcp__github__issues_list".to_string()));

        let _ = fs::remove_dir_all(&root);
    }
}
