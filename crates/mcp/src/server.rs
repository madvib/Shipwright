use anyhow::{Result, anyhow};
use rmcp::transport::stdio;
use rmcp::{
    ErrorData, RoleServer, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, tool::ToolCallContext, wrapper::Parameters},
    model::{
        AnnotateAble, CallToolRequestParams, CallToolResult, Content, Implementation,
        ListResourceTemplatesResult, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
        ProtocolVersion, RawResource, RawResourceTemplate, ReadResourceRequestParams,
        ReadResourceResult, ResourceContents, ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
    tool, tool_router,
};
use runtime::project::{get_active_project_global, get_project_dir, set_active_project_global};
use runtime::{
    get_active_mode, get_config, get_effective_skill, list_effective_skills, list_events_since,
    list_models, list_providers, read_log, set_active_mode,
    workspace::{
        CreateWorkspaceRequest as RuntimeCreateWorkspaceRequest,
        EndWorkspaceSessionRequest as RuntimeEndWorkspaceSessionRequest, ShipWorkspaceKind,
        activate_workspace as runtime_activate_workspace,
        create_workspace as runtime_create_workspace,
        end_workspace_session as runtime_end_workspace_session,
        get_active_workspace_session as runtime_get_active_workspace_session,
        get_workspace as runtime_get_workspace,
        get_workspace_provider_matrix as runtime_get_workspace_provider_matrix,
        list_workspace_sessions as runtime_list_workspace_sessions,
        list_workspaces as runtime_list_workspaces,
        record_workspace_session_progress as runtime_record_workspace_session_progress,
        repair_workspace as runtime_repair_workspace, set_workspace_active_mode,
        start_workspace_session as runtime_start_workspace_session,
        sync_workspace as runtime_sync_workspace,
    },
};
use ship_module_project::ops::adr::{create_adr, get_adr_by_id, list_adrs};
use ship_module_project::ops::feature::{
    create_feature, get_feature_by_id, list_features, sync_feature_docs_after_session,
    update_feature_content,
};
use ship_module_project::ops::note::{
    create_note, get_note_by_id, list_notes, update_note_content,
};
use ship_module_project::ops::release::{
    create_release, get_release_by_id, list_releases, update_release_content,
};
use ship_module_project::ops::spec::{create_spec, get_spec_by_id, list_specs, update_spec};
use ship_module_project::{NoteScope, get_project_name, list_registered_projects};
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use crate::requests::*;

// ─── Server ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ShipServer {
    tool_router: ToolRouter<Self>,
    pub active_project: std::sync::Arc<tokio::sync::Mutex<Option<PathBuf>>>,
}

#[tool_router]
impl ShipServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            active_project: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    async fn get_effective_project_dir(&self) -> Result<PathBuf, String> {
        let active = self.active_project.lock().await;
        if let Some(ref path) = *active {
            return Ok(path.clone());
        }
        drop(active);

        if let Ok(project_dir) = get_project_dir(None) {
            return Ok(project_dir);
        }

        if let Ok(Some(global_active)) = get_active_project_global() {
            if let Ok(project_dir) = get_project_dir(Some(global_active.clone())) {
                let mut active = self.active_project.lock().await;
                *active = Some(project_dir.clone());
                return Ok(project_dir);
            }
        }

        if let Ok(registry) = list_registered_projects() {
            if registry.len() == 1 {
                if let Ok(project_dir) = get_project_dir(Some(registry[0].path.clone())) {
                    let mut active = self.active_project.lock().await;
                    *active = Some(project_dir.clone());
                    return Ok(project_dir);
                }
            }
        }

        get_project_dir(None).map_err(|e| {
            format!(
                "No active project and auto-detection failed: {}. Checked process cwd, global active project, and registered projects.",
                e
            )
        })
    }

    fn normalize_mode_tool_id(raw: &str) -> String {
        let mut normalized = raw.trim().to_ascii_lowercase().replace('-', "_");
        if let Some(stripped) = normalized.strip_prefix("ship_") {
            normalized = stripped.to_string();
        }
        if let Some(stripped) = normalized.strip_suffix("_tool") {
            normalized = stripped.to_string();
        }
        normalized
    }

    fn is_core_tool(tool_name: &str) -> bool {
        // Core workflow tools always available regardless of active mode.
        // These cover the three-stage Ship workflow: Planning → Workspace → Session.
        // Extended tools (specs, releases, etc.) require a mode OR a service workspace.
        const CORE_TOOLS: &[&str] = &[
            // Project context
            "open_project",
            // Planning
            "create_note",
            "create_feature",
            "update_feature",
            "create_adr",
            // Workspace
            "activate_workspace",
            "create_workspace",
            "set_mode",
            "sync_workspace",
            "repair_workspace",
            // Session
            "start_session",
            "end_session",
            "log_progress",
        ];
        let normalized = Self::normalize_mode_tool_id(tool_name);
        CORE_TOOLS.contains(&normalized.as_str())
    }

    fn is_project_workspace_tool(tool_name: &str) -> bool {
        // Tools auto-unlocked when the active workspace is type=service.
        // These cover PM mutation flows (read surfaces should prefer resources).
        const PROJECT_TOOLS: &[&str] = &[
            "create_spec",
            "update_spec",
            "create_release",
            "update_release",
        ];
        let normalized = Self::normalize_mode_tool_id(tool_name);
        PROJECT_TOOLS.contains(&normalized.as_str())
    }

    fn mode_allows_tool(tool_name: &str, active_tools: &[String]) -> bool {
        if active_tools.is_empty() {
            return true;
        }

        let normalized_tool = Self::normalize_mode_tool_id(tool_name);
        active_tools
            .iter()
            .map(|tool| Self::normalize_mode_tool_id(tool))
            .any(|allowed| allowed == normalized_tool)
    }

    fn enforce_mode_tool_gate(project_dir: &PathBuf, tool_name: &str) -> Result<(), String> {
        if Self::is_core_tool(tool_name) {
            return Ok(());
        }

        // Service workspace auto-unlocks the PM tool surface without needing a mode.
        if Self::is_project_workspace_tool(tool_name) {
            let active_type =
                runtime::workspace::get_active_workspace_type(project_dir).unwrap_or(None);
            if matches!(active_type, Some(runtime::ShipWorkspaceKind::Service)) {
                return Ok(());
            }
        }

        let active_mode = get_active_mode(Some(project_dir.clone())).map_err(|e| e.to_string())?;

        if let Some(ref mode) = active_mode {
            if Self::mode_allows_tool(tool_name, &mode.active_tools) {
                return Ok(());
            }
            let allowed = if mode.active_tools.is_empty() {
                "all tools".to_string()
            } else {
                mode.active_tools.join(", ")
            };
            return Err(format!(
                "Tool '{}' blocked by active mode '{}' (allowed: {}).",
                tool_name, mode.id, allowed
            ));
        }

        // No mode active: only core tools are available.
        // Activate a service workspace (`ship` branch) to unlock PM tools,
        // or create a mode with active_tools set to unlock specific tools.
        Err(format!(
            "Tool '{}' is not in the core workflow surface. \
             Activate the service workspace ('ship') or a mode with this tool in its \
             active_tools list to use it.",
            tool_name
        ))
    }

    fn resolve_workspace_branch_for_project(
        project_dir: &PathBuf,
        branch: Option<&str>,
    ) -> Result<String, String> {
        if let Some(branch) = branch {
            let trimmed = branch.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
        let Some(project_root) = project_dir.parent() else {
            return Err("Error: Could not resolve project root".to_string());
        };
        current_branch(project_root).map_err(|e| e.to_string())
    }

    // ─── Project Tools ────────────────────────────────────────────────────────

    /// Set the active project for subsequent commands
    #[tool(description = "Set the active project for subsequent MCP tool calls")]
    async fn open_project(&self, Parameters(req): Parameters<OpenProjectRequest>) -> String {
        let path = PathBuf::from(&req.path);
        match get_project_dir(Some(path.clone())) {
            Ok(ship_dir) => {
                let mut active = self.active_project.lock().await;
                *active = Some(ship_dir.clone());
                drop(active);

                if let Err(e) = set_active_project_global(ship_dir.clone()) {
                    return format!(
                        "Opened project at {} (warning: failed to persist global active project: {})",
                        ship_dir.display(),
                        e
                    );
                }

                format!("Opened project at {}", ship_dir.display())
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Build full project context snapshot used by resources.
    async fn get_project_info(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };

        let name = get_project_name(&project_dir);
        let config = get_config(Some(project_dir.clone())).unwrap_or_default();

        let releases = list_releases(&project_dir).unwrap_or_default();
        let features = list_features(&project_dir).unwrap_or_default();
        let specs = list_specs(&project_dir).unwrap_or_default();
        let adrs = list_adrs(&project_dir).unwrap_or_default();

        let mut out = format!("# Project: {}\n\n", name);

        // ── Active workspace & session ────────────────────────────────────────
        out.push_str("## Current Context\n");
        let workspaces = runtime_list_workspaces(&project_dir).unwrap_or_default();
        let active_workspace = workspaces
            .iter()
            .find(|w| matches!(w.status, runtime::WorkspaceStatus::Active));
        if let Some(ws) = active_workspace {
            out.push_str(&format!(
                "- Workspace: {} [{:?}]",
                ws.branch, ws.workspace_type
            ));
            if let Some(ref mode) = ws.active_mode {
                out.push_str(&format!(" mode={}", mode));
            }
            if let Some(ref fid) = ws.feature_id {
                out.push_str(&format!(" feature={}", fid));
            }
            out.push('\n');

            // Active session for this workspace
            match runtime_get_active_workspace_session(&project_dir, &ws.branch) {
                Ok(Some(session)) => {
                    out.push_str(&format!("- Session: ACTIVE (id: {})", session.id));
                    if let Some(ref goal) = session.goal {
                        out.push_str(&format!(" — goal: {}", goal));
                    }
                    if let Some(ref provider) = session.primary_provider {
                        out.push_str(&format!(" [{}]", provider));
                    }
                    out.push('\n');
                }
                Ok(None) => out.push_str("- Session: none (call start_session to begin)\n"),
                Err(_) => {}
            }
        } else {
            out.push_str("- Workspace: none active (call activate_workspace to begin)\n");
        }

        // Agent mode
        if let Some(active_id) = config.active_mode.as_deref() {
            if let Some(mode) = config.modes.iter().find(|m| m.id == active_id) {
                out.push_str(&format!("- Mode: {} ({})\n", mode.name, mode.id));
            }
        } else if !config.modes.is_empty() {
            out.push_str("- Mode: none (available: ");
            let names: Vec<_> = config.modes.iter().map(|m| m.id.as_str()).collect();
            out.push_str(&names.join(", "));
            out.push_str(")\n");
        }

        // Releases
        out.push_str("\n## Releases\n");
        if releases.is_empty() {
            out.push_str("No releases.\n");
        } else {
            for r in &releases {
                out.push_str(&format!("- [{}] {} ({})\n", r.status, r.version, r.id));
            }
        }

        // Features
        out.push_str("\n## Features\n");
        if features.is_empty() {
            out.push_str("No features.\n");
        } else {
            for f in &features {
                let has_docs = f.feature.body.contains("## Documentation");
                let docs_flag = if !has_docs {
                    " ⚠ missing ## Documentation"
                } else {
                    ""
                };
                out.push_str(&format!(
                    "- [{}] {} ({}){}\n",
                    f.status, f.feature.metadata.title, f.id, docs_flag
                ));
            }
        }

        // Specs
        out.push_str("\n## Specs\n");
        if specs.is_empty() {
            out.push_str("No specs.\n");
        } else {
            for s in &specs {
                out.push_str(&format!("- {} ({})\n", s.spec.metadata.title, s.file_name));
            }
        }

        // ADRs
        out.push_str("\n## ADRs\n");
        if adrs.is_empty() {
            out.push_str("No ADRs.\n");
        } else {
            for a in &adrs {
                out.push_str(&format!(
                    "- [{}] {} ({})\n",
                    a.status, a.adr.metadata.title, a.id
                ));
            }
        }

        // Recent log (last 10 lines)
        if let Ok(log) = read_log(&project_dir) {
            let recent: Vec<&str> = log
                .lines()
                .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
                .rev()
                .take(10)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            if !recent.is_empty() {
                out.push_str("\n## Recent Activity\n");
                for line in recent {
                    out.push_str(&format!("{}\n", line));
                }
            }
        }

        if let Ok(events) = list_events_since(&project_dir, 0, Some(10)) {
            if !events.is_empty() {
                out.push_str("\n## Recent Events\n");
                for e in events {
                    let details = e
                        .details
                        .as_ref()
                        .map(|d| format!(" — {}", d))
                        .unwrap_or_default();
                    out.push_str(&format!(
                        "- #{} {} [{}] {:?}.{:?} {}{}\n",
                        e.seq,
                        e.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        e.actor,
                        e.entity,
                        e.action,
                        e.subject,
                        details
                    ));
                }
            }
        }

        out
    }

    // ─── Notes Tools ────────────────────────────────────────────────────────

    /// Create a standalone note (project- or user-scoped)
    #[tool(description = "Create a standalone note. Scope can be 'project' or 'user'.")]
    async fn create_note(&self, Parameters(req): Parameters<CreateNoteRequest>) -> String {
        let scope = match parse_note_scope(req.scope.as_deref()) {
            Ok(scope) => scope,
            Err(err) => return format!("Error: {}", err),
        };
        let project_dir = match scope {
            NoteScope::Project => match self.get_effective_project_dir().await {
                Ok(project_dir) => Some(project_dir),
                Err(err) => return err,
            },
            NoteScope::User => None,
        };
        let body = req.content.unwrap_or_default();
        match create_note(scope, project_dir.as_deref(), &req.title, &body) {
            Ok(note) => format!("Created note: {} (id: {})", note.title, note.id),
            Err(e) => format!("Error creating note: {}", e),
        }
    }

    /// Update a standalone note
    #[tool(description = "Replace a note's markdown content by filename.")]
    async fn update_note(&self, Parameters(req): Parameters<UpdateNoteRequest>) -> String {
        let scope = match parse_note_scope(req.scope.as_deref()) {
            Ok(scope) => scope,
            Err(err) => return format!("Error: {}", err),
        };
        let project_dir = match scope {
            NoteScope::Project => match self.get_effective_project_dir().await {
                Ok(project_dir) => Some(project_dir),
                Err(err) => return err,
            },
            NoteScope::User => None,
        };
        match update_note_content(scope, project_dir.as_deref(), &req.file_name, &req.content) {
            Ok(note) => format!("Updated note: {}", note.title),
            Err(e) => format!("Error updating note: {}", e),
        }
    }

    // ─── ADR Tools ────────────────────────────────────────────────────────────

    /// Create a new Architecture Decision Record
    #[tool(
        description = "Create a new Architecture Decision Record (ADR). Use when committing to a \
        technical approach, trade-off, or design choice that future contributors need to understand. \
        Captures the decision and reasoning in the project record."
    )]
    async fn create_adr(&self, Parameters(req): Parameters<LogDecisionRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match create_adr(&project_dir, &req.title, "", &req.decision, "proposed") {
            Ok(entry) => format!(
                "Created ADR '{}' (id: {})",
                entry.adr.metadata.title, entry.id
            ),
            Err(e) => format!("Error: {}", e),
        }
    }

    // ─── Spec Tools ───────────────────────────────────────────────────────────

    /// Create a new spec
    #[tool(description = "Create a new spec document in the active project")]
    async fn create_spec(&self, Parameters(req): Parameters<CreateSpecRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let content = req.content.as_deref().unwrap_or("");
        match create_spec(&project_dir, &req.title, content, req.workspace.as_deref()) {
            Ok(file) => format!("Created spec: {}", file.file_name),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Update a spec's content
    #[tool(description = "Update the content of an existing spec")]
    async fn update_spec(&self, Parameters(req): Parameters<UpdateSpecRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let specs = list_specs(&project_dir).unwrap_or_default();
        let entry = specs.iter().find(|e| e.file_name == req.file_name);
        match entry {
            Some(entry) => {
                let mut spec = entry.spec.clone();
                spec.body = req.content;
                match update_spec(&project_dir, &entry.id, spec) {
                    Ok(_) => format!("Updated spec: {}", req.file_name),
                    Err(e) => format!("Error: {}", e),
                }
            }
            None => format!("Error: Spec not found with filename {}", req.file_name),
        }
    }

    // ─── Release Tools ────────────────────────────────────────────────────────

    /// Create a new release
    #[tool(description = "Create a new release document in the active project")]
    async fn create_release(&self, Parameters(req): Parameters<CreateReleaseRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let content = req.content.as_deref().unwrap_or("");
        match create_release(&project_dir, &req.version, content) {
            Ok(release) => format!(
                "Created release: {} ({})",
                release.release.metadata.version, release.id
            ),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Update a release's content
    #[tool(description = "Update the content of an existing release")]
    async fn update_release(&self, Parameters(req): Parameters<UpdateReleaseRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match update_release_content(&project_dir, &req.id, &req.content) {
            Ok(release) => format!(
                "Updated release: {} ({})",
                release.release.metadata.version, req.id
            ),
            Err(e) => format!("Error: {}", e),
        }
    }

    // ─── Feature Tools ────────────────────────────────────────────────────────

    /// Create a new feature
    #[tool(
        description = "Create a new feature document. Features are the primary planning artifact — \
        they capture intent and evolve into living documentation. Structure the body with \
        '## Intent' (what this is, why it exists, how it should behave) and \
        '## Documentation' (how it actually works once built). \
        Link to a release, spec, or branch as needed."
    )]
    async fn create_feature(&self, Parameters(req): Parameters<CreateFeatureRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let content = req.content.as_deref().unwrap_or("");
        match create_feature(
            &project_dir,
            &req.title,
            content,
            req.release_id.as_deref(),
            req.spec_id.as_deref(),
            req.branch.as_deref(),
        ) {
            Ok(feature) => format!(
                "Created feature: {} ({})",
                feature.feature.metadata.title, feature.id
            ),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Update a feature's content
    #[tool(
        description = "Update the content of an existing feature. Use this to refine intent, \
        add implementation notes, or update the '## Documentation' section as the feature is built. \
        Features should always have '## Intent' (planning north star) and \
        '## Documentation' (how it works) sections. Pass the full replacement body."
    )]
    async fn update_feature(&self, Parameters(req): Parameters<UpdateFeatureRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match update_feature_content(&project_dir, &req.id, &req.content) {
            Ok(feature) => format!(
                "Updated feature: {} ({})",
                feature.feature.metadata.title, req.id
            ),
            Err(e) => format!("Error: {}", e),
        }
    }

    // ─── Mode / Workspace Control Plane Tools ──────────────────────────────

    /// Activate a mode or clear active mode
    #[tool(
        description = "Activate a mode by id, or clear active mode by passing null/omitting id."
    )]
    async fn set_mode(&self, Parameters(req): Parameters<SetModeRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(project_dir) => project_dir,
            Err(err) => return err,
        };
        match set_active_mode(Some(project_dir), req.id.as_deref()) {
            Ok(()) => match req.id {
                Some(id) => format!("Active mode set to '{}'", id),
                None => "Active mode cleared".to_string(),
            },
            Err(err) => format!("Error: {}", err),
        }
    }

    /// Create or update a workspace record
    #[tool(description = "Create or update a workspace runtime record (feature/patch/service).")]
    async fn create_workspace_tool(
        &self,
        Parameters(req): Parameters<CreateWorkspaceToolRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(project_dir) => project_dir,
            Err(err) => return err,
        };

        let parsed_workspace_type = match req.workspace_type {
            Some(workspace_type) => match workspace_type.parse::<ShipWorkspaceKind>() {
                Ok(parsed) => Some(parsed),
                Err(err) => return format!("Error: {}", err),
            },
            None => None,
        };

        let workspace_request = RuntimeCreateWorkspaceRequest {
            branch: req.branch.clone(),
            workspace_type: parsed_workspace_type,
            status: None,
            environment_id: req.environment_id,
            feature_id: req.feature_id,
            spec_id: req.spec_id,
            release_id: req.release_id,
            active_mode: req.mode_id,
            providers: None,
            is_worktree: req.is_worktree,
            worktree_path: req.worktree_path,
            context_hash: None,
        };

        let workspace = match runtime_create_workspace(&project_dir, workspace_request) {
            Ok(workspace) => workspace,
            Err(err) => return format!("Error: {}", err),
        };

        let workspace = if req.activate.unwrap_or(false) {
            match runtime_activate_workspace(&project_dir, &workspace.branch) {
                Ok(active) => active,
                Err(err) => return format!("Error: {}", err),
            }
        } else {
            workspace
        };

        serde_json::to_string_pretty(&workspace)
            .unwrap_or_else(|e| format!("Error serializing workspace: {}", e))
    }

    /// Activate workspace and optionally apply a mode override
    #[tool(description = "Activate a workspace by branch/id and optionally set its mode override.")]
    async fn activate_workspace(
        &self,
        Parameters(req): Parameters<ActivateWorkspaceRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(project_dir) => project_dir,
            Err(err) => return err,
        };
        let mut workspace = match runtime_activate_workspace(&project_dir, &req.branch) {
            Ok(workspace) => workspace,
            Err(err) => return format!("Error: {}", err),
        };
        if let Some(mode_id) = req.mode_id.as_deref() {
            workspace = match set_workspace_active_mode(&project_dir, &req.branch, Some(mode_id)) {
                Ok(workspace) => workspace,
                Err(err) => return format!("Error: {}", err),
            };
        }
        serde_json::to_string_pretty(&workspace)
            .unwrap_or_else(|e| format!("Error serializing workspace: {}", e))
    }

    /// Sync workspace state with current branch context
    #[tool(description = "Sync the workspace for a branch/id (or current git branch if omitted).")]
    async fn sync_workspace(&self, Parameters(req): Parameters<SyncWorkspaceRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(project_dir) => project_dir,
            Err(err) => return err,
        };
        let branch =
            match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) {
                Ok(branch) => branch,
                Err(err) => return format!("Error: {}", err),
            };
        match runtime_sync_workspace(&project_dir, &branch) {
            Ok(workspace) => serde_json::to_string_pretty(&workspace)
                .unwrap_or_else(|e| format!("Error serializing workspace: {}", e)),
            Err(err) => format!("Error: {}", err),
        }
    }

    /// Repair workspace compile/config drift and report actions taken
    #[tool(
        description = "Repair workspace compile/config drift. Defaults to dry-run unless dry_run=false."
    )]
    async fn repair_workspace(
        &self,
        Parameters(req): Parameters<RepairWorkspaceRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(project_dir) => project_dir,
            Err(err) => return err,
        };
        let branch =
            match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) {
                Ok(branch) => branch,
                Err(err) => return format!("Error: {}", err),
            };
        match runtime_repair_workspace(&project_dir, &branch, req.dry_run.unwrap_or(true)) {
            Ok(report) => serde_json::to_string_pretty(&report)
                .unwrap_or_else(|e| format!("Error serializing workspace repair report: {}", e)),
            Err(err) => format!("Error: {}", err),
        }
    }

    /// Start a session on the active workspace (compiles provider context)
    #[tool(
        description = "Start a workspace session for the active compiled context and selected provider."
    )]
    async fn start_session(&self, Parameters(req): Parameters<StartSessionRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(project_dir) => project_dir,
            Err(err) => return err,
        };
        let branch =
            match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) {
                Ok(branch) => branch,
                Err(err) => return format!("Error: {}", err),
            };
        match runtime_start_workspace_session(
            &project_dir,
            &branch,
            req.goal,
            req.mode_id,
            req.provider_id,
        ) {
            Ok(session) => serde_json::to_string_pretty(&session)
                .unwrap_or_else(|e| format!("Error serializing workspace session: {}", e)),
            Err(err) => format!("Error: {}", err),
        }
    }

    /// End the active session, record what was accomplished, and update feature metadata
    #[tool(
        description = "End the active workspace session. Provide a summary of what was accomplished \
        and the IDs of any features that were updated. This emits a session-end event visible in \
        get_project_info and records the session \
        history. Call this when work is complete or paused for the day."
    )]
    async fn end_session(&self, Parameters(req): Parameters<EndSessionRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(project_dir) => project_dir,
            Err(err) => return err,
        };
        let branch =
            match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) {
                Ok(branch) => branch,
                Err(err) => return format!("Error: {}", err),
            };

        let updated_feature_ids = req.updated_feature_ids.unwrap_or_default();

        let end_req = RuntimeEndWorkspaceSessionRequest {
            summary: req.summary,
            updated_feature_ids,
            updated_spec_ids: req.updated_spec_ids.unwrap_or_default(),
        };
        let session = match runtime_end_workspace_session(&project_dir, &branch, end_req) {
            Ok(session) => session,
            Err(err) => return format!("Error: {}", err),
        };

        if !session.updated_feature_ids.is_empty() {
            let _ = sync_feature_docs_after_session(
                &project_dir,
                &session.updated_feature_ids,
                session.summary.as_deref(),
            );
        }

        serde_json::to_string_pretty(&session)
            .unwrap_or_else(|e| format!("Error serializing workspace session: {}", e))
    }

    /// Log a progress note within the active session
    #[tool(
        description = "Record a progress note for the active session. Use mid-session to log \
        what you did, decisions made, or blockers encountered. Notes are recorded in the unified \
        workspace session event stream. Requires an active session (call start_session first)."
    )]
    async fn log_progress(&self, Parameters(req): Parameters<LogProgressRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(project_dir) => project_dir,
            Err(err) => return err,
        };
        let branch =
            match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) {
                Ok(branch) => branch,
                Err(err) => return format!("Error: {}", err),
            };
        match runtime_get_active_workspace_session(&project_dir, &branch) {
            Ok(None) => {
                return format!(
                    "No active session for '{}'. Call start_session first.",
                    branch
                );
            }
            Err(err) => return format!("Error checking session: {}", err),
            Ok(Some(_)) => {}
        }
        match runtime_record_workspace_session_progress(&project_dir, &branch, &req.note) {
            Ok(()) => format!("Progress logged for session on '{}'.", branch),
            Err(e) => format!("Error logging progress: {}", e),
        }
    }
}

impl ShipServer {
    /// Resolve a `ship://` URI to its text content, or `None` if not found.
    async fn resolve_resource_uri(&self, uri: &str, dir: &PathBuf) -> Option<String> {
        // ship://project_info
        if uri == "ship://project_info" {
            return Some(self.get_project_info().await);
        }
        // ship://features
        if uri == "ship://features" {
            let entries = list_features(&dir).ok()?;
            if entries.is_empty() {
                return Some("No features found.".to_string());
            }
            let mut out = String::from("Features:\n");
            for f in &entries {
                out.push_str(&format!(
                    "- [{}] {} ({})\n",
                    f.status, f.feature.metadata.title, f.id
                ));
            }
            return Some(out);
        }
        // ship://features/{id}
        if let Some(id) = uri.strip_prefix("ship://features/") {
            return get_feature_by_id(&dir, id)
                .ok()
                .and_then(|e| e.feature.to_markdown().ok());
        }
        // ship://releases
        if uri == "ship://releases" {
            let entries = list_releases(&dir).ok()?;
            if entries.is_empty() {
                return Some("No releases found.".to_string());
            }
            let mut out = String::from("Releases:\n");
            for r in &entries {
                out.push_str(&format!(
                    "- [{}] {} ({})\n",
                    r.status, r.release.metadata.version, r.id
                ));
            }
            return Some(out);
        }
        // ship://releases/{id}
        if let Some(id) = uri.strip_prefix("ship://releases/") {
            return get_release_by_id(&dir, id)
                .ok()
                .and_then(|e| e.release.to_markdown().ok());
        }
        // ship://specs
        if uri == "ship://specs" {
            let entries = list_specs(dir).ok()?;
            if entries.is_empty() {
                return Some("No specs found.".to_string());
            }
            let mut out = String::from("Specs:\n");
            for s in &entries {
                out.push_str(&format!("- {} ({})\n", s.spec.metadata.title, s.file_name));
            }
            return Some(out);
        }
        // ship://specs/{file}
        if let Some(file) = uri.strip_prefix("ship://specs/") {
            return get_spec_by_id(dir, file)
                .ok()
                .and_then(|entry| entry.spec.to_markdown().ok());
        }
        // ship://adrs
        if uri == "ship://adrs" {
            let entries = list_adrs(dir).ok()?;
            if entries.is_empty() {
                return Some("No ADRs found.".to_string());
            }
            let mut out = String::from("ADRs:\n");
            for a in &entries {
                out.push_str(&format!(
                    "- [{}] {} ({})\n",
                    a.status, a.adr.metadata.title, a.file_name
                ));
            }
            return Some(out);
        }
        // ship://adrs/{id}
        if let Some(id) = uri.strip_prefix("ship://adrs/") {
            return get_adr_by_id(dir, id).ok().map(|entry| {
                format!(
                    "Title: {}\nStatus: {}\nDate: {}\n\n## Context\n\n{}\n\n## Decision\n\n{}",
                    entry.adr.metadata.title,
                    entry.status,
                    entry.adr.metadata.date,
                    entry.adr.context,
                    entry.adr.decision
                )
            });
        }
        // ship://notes
        if uri == "ship://notes" {
            let entries = list_notes(NoteScope::Project, Some(&dir)).ok()?;
            if entries.is_empty() {
                return Some("No notes found.".to_string());
            }
            let mut out = String::from("Notes:\n");
            for entry in entries {
                out.push_str(&format!("- {} ({})\n", entry.title, entry.id));
            }
            return Some(out);
        }

        // ship://notes/{file}
        if let Some(id) = uri.strip_prefix("ship://notes/") {
            let note = get_note_by_id(NoteScope::Project, Some(&dir), id).ok()?;
            return Some(note.content);
        }
        // ship://skills
        if uri == "ship://skills" {
            let entries = list_effective_skills(dir).ok()?;
            if entries.is_empty() {
                return Some("No skills found.".to_string());
            }
            let mut out = String::from("Skills:\n");
            for s in &entries {
                out.push_str(&format!("- {} ({})\n", s.id, s.name));
            }
            return Some(out);
        }
        // ship://skills/{id}
        if let Some(id) = uri.strip_prefix("ship://skills/") {
            return get_effective_skill(dir, id).ok().map(|s| s.content);
        }
        // ship://log
        if uri == "ship://log" {
            return match read_log(dir) {
                Ok(content) if content.trim().is_empty() || content.trim() == "# Project Log" => {
                    Some("No log entries yet.".to_string())
                }
                Ok(content) => Some(content),
                Err(err) => Some(format!("Error: {}", err)),
            };
        }
        // ship://events
        if uri == "ship://events" {
            return render_events_resource(dir, 0, 100);
        }
        // ship://events/{since}
        if let Some(since) = uri.strip_prefix("ship://events/") {
            let Ok(since) = since.parse::<u64>() else {
                return Some(format!("Error: invalid event sequence '{}'", since));
            };
            return render_events_resource(dir, since, 100);
        }
        // ship://workspaces
        if uri == "ship://workspaces" {
            return match runtime_list_workspaces(dir) {
                Ok(workspaces) => serde_json::to_string_pretty(&workspaces).ok(),
                Err(err) => Some(format!("Error: {}", err)),
            };
        }
        // ship://workspaces/{branch}/provider-matrix
        if let Some(rest) = uri.strip_prefix("ship://workspaces/")
            && let Some(branch) = rest.strip_suffix("/provider-matrix")
        {
            return match runtime_get_workspace_provider_matrix(dir, branch, None) {
                Ok(matrix) => serde_json::to_string_pretty(&matrix).ok(),
                Err(err) => Some(format!("Error: {}", err)),
            };
        }
        // ship://workspaces/{branch}/session
        if let Some(rest) = uri.strip_prefix("ship://workspaces/")
            && let Some(branch) = rest.strip_suffix("/session")
        {
            return match runtime_get_active_workspace_session(dir, branch) {
                Ok(Some(session)) => serde_json::to_string_pretty(&session).ok(),
                Ok(None) => Some(format!("No active workspace session for '{}'", branch)),
                Err(err) => Some(format!("Error: {}", err)),
            };
        }
        // ship://workspaces/{branch}
        if let Some(branch) = uri.strip_prefix("ship://workspaces/") {
            return match runtime_get_workspace(dir, branch) {
                Ok(Some(workspace)) => serde_json::to_string_pretty(&workspace).ok(),
                Ok(None) => Some(format!("Workspace '{}' not found", branch)),
                Err(err) => Some(format!("Error: {}", err)),
            };
        }
        // ship://sessions
        if uri == "ship://sessions" {
            return match runtime_list_workspace_sessions(dir, None, 50) {
                Ok(sessions) => serde_json::to_string_pretty(&sessions).ok(),
                Err(err) => Some(format!("Error: {}", err)),
            };
        }
        // ship://sessions/{workspace}
        if let Some(workspace) = uri.strip_prefix("ship://sessions/") {
            return match runtime_list_workspace_sessions(dir, Some(workspace), 50) {
                Ok(sessions) => serde_json::to_string_pretty(&sessions).ok(),
                Err(err) => Some(format!("Error: {}", err)),
            };
        }
        // ship://modes
        if uri == "ship://modes" {
            return match get_config(Some(dir.clone())) {
                Ok(config) => {
                    let payload = serde_json::json!({
                        "active_mode": config.active_mode,
                        "modes": config.modes,
                    });
                    serde_json::to_string_pretty(&payload).ok()
                }
                Err(err) => Some(format!("Error: {}", err)),
            };
        }
        // ship://providers
        if uri == "ship://providers" {
            return match list_providers(dir) {
                Ok(providers) => serde_json::to_string_pretty(&providers).ok(),
                Err(err) => Some(format!("Error: {}", err)),
            };
        }
        // ship://providers/{id}/models
        if let Some(rest) = uri.strip_prefix("ship://providers/")
            && let Some(provider_id) = rest.strip_suffix("/models")
        {
            return match list_models(provider_id) {
                Ok(models) => serde_json::to_string_pretty(&models).ok(),
                Err(err) => Some(format!("Error: {}", err)),
            };
        }
        None
    }
}

fn current_branch(project_root: &std::path::Path) -> Result<String> {
    let output = ProcessCommand::new("git")
        .args(["branch", "--show-current"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("Failed to determine current git branch");
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        anyhow::bail!("Current HEAD is detached; cannot map to a feature branch");
    }
    Ok(branch)
}

fn parse_note_scope(raw: Option<&str>) -> Result<NoteScope> {
    raw.unwrap_or("project").parse::<NoteScope>()
}

fn render_events_resource(project_dir: &PathBuf, since: u64, limit: usize) -> Option<String> {
    match list_events_since(project_dir, since, Some(limit)) {
        Ok(events) => {
            if events.is_empty() {
                return Some("No events found.".to_string());
            }
            let mut out = String::from("Events:\n");
            for e in events {
                let details = e
                    .details
                    .as_ref()
                    .map(|d| format!(" — {}", d))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "- #{} {} [{}] {:?}.{:?} {}{}\n",
                    e.seq,
                    e.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    e.actor,
                    e.entity,
                    e.action,
                    e.subject,
                    details
                ));
            }
            Some(out)
        }
        Err(err) => Some(format!("Error: {}", err)),
    }
}

impl ServerHandler for ShipServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: "Ship Project Tracker".into(),
                version: "0.2.0".into(),
                ..Default::default()
            },
            instructions: Some(
                "Ship project intelligence — three-stage workflow:\n\n\
                 PLANNING: get_project_info → create_note / create_feature / update_feature / create_adr\n\
                 WORKSPACE: list_workspaces → activate_workspace → set_mode\n\
                 SESSION: start_session → (work) → log_progress → end_session\n\n\
                 By default only core workflow tools are visible. To access extended tools \
                 (specs, releases, etc.), activate a mode that includes \
                 them in its active_tools list. Call open_project first if the project is not \
                 auto-detected. Use resources (ship://) for read-heavy workflows."
                    .into(),
            ),
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let tool_name = request.name.to_string();

        if let Ok(project_dir) = self.get_effective_project_dir().await
            && let Err(message) = Self::enforce_mode_tool_gate(&project_dir, &tool_name)
        {
            return Ok(CallToolResult::error(vec![Content::text(message)]));
        }

        self.tool_router
            .call(ToolCallContext::new(self, request, context))
            .await
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let all_tools = self.tool_router.list_all();

        // Filter tool list to match what enforce_mode_tool_gate allows.
        // Agents should only see tools they can actually call.
        let visible = if let Ok(project_dir) = self.get_effective_project_dir().await {
            let active_mode = get_active_mode(Some(project_dir.clone())).unwrap_or(None);
            let in_project_workspace = matches!(
                runtime::workspace::get_active_workspace_type(&project_dir).unwrap_or(None),
                Some(runtime::ShipWorkspaceKind::Service)
            );
            all_tools
                .into_iter()
                .filter(|tool| {
                    let name = tool.name.as_ref();
                    if Self::is_core_tool(name) {
                        return true;
                    }
                    if in_project_workspace && Self::is_project_workspace_tool(name) {
                        return true;
                    }
                    if let Some(ref mode) = active_mode {
                        Self::mode_allows_tool(name, &mode.active_tools)
                    } else {
                        false
                    }
                })
                .collect()
        } else {
            // No project resolved — show core tools only.
            all_tools
                .into_iter()
                .filter(|tool| Self::is_core_tool(tool.name.as_ref()))
                .collect()
        };

        Ok(ListToolsResult::with_all_items(visible))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool_router.get(name).cloned()
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult::with_all_items(vec![
            RawResource::new("ship://project_info", "Project Info").no_annotation(),
            RawResource::new("ship://features", "Features").no_annotation(),
            RawResource::new("ship://releases", "Releases").no_annotation(),
            RawResource::new("ship://specs", "Specs").no_annotation(),
            RawResource::new("ship://adrs", "ADRs").no_annotation(),
            RawResource::new("ship://notes", "Project Notes").no_annotation(),
            RawResource::new("ship://skills", "Skills").no_annotation(),
            RawResource::new("ship://workspaces", "Workspaces").no_annotation(),
            RawResource::new("ship://sessions", "Workspace Sessions").no_annotation(),
            RawResource::new("ship://modes", "Modes").no_annotation(),
            RawResource::new("ship://providers", "Providers").no_annotation(),
            RawResource::new("ship://log", "Project Log").no_annotation(),
            RawResource::new("ship://events", "Event Stream").no_annotation(),
        ]))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        Ok(ListResourceTemplatesResult::with_all_items(vec![
            RawResourceTemplate {
                uri_template: "ship://features/{id}".to_string(),
                name: "Feature".to_string(),
                title: None,
                description: None,
                mime_type: Some("text/markdown".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://releases/{id}".to_string(),
                name: "Release".to_string(),
                title: None,
                description: None,
                mime_type: Some("text/markdown".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://specs/{id}".to_string(),
                name: "Spec".to_string(),
                title: None,
                description: None,
                mime_type: Some("text/markdown".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://adrs/{id}".to_string(),
                name: "ADR".to_string(),
                title: None,
                description: None,
                mime_type: Some("text/markdown".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://notes/{id}".to_string(),
                name: "Note".to_string(),
                title: None,
                description: None,
                mime_type: Some("text/markdown".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://workspaces/{branch}".to_string(),
                name: "Workspace".to_string(),
                title: None,
                description: None,
                mime_type: Some("application/json".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://workspaces/{branch}/provider-matrix".to_string(),
                name: "Workspace Provider Matrix".to_string(),
                title: None,
                description: None,
                mime_type: Some("application/json".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://workspaces/{branch}/session".to_string(),
                name: "Workspace Active Session".to_string(),
                title: None,
                description: None,
                mime_type: Some("application/json".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://sessions/{workspace}".to_string(),
                name: "Workspace Sessions".to_string(),
                title: None,
                description: None,
                mime_type: Some("application/json".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://providers/{id}/models".to_string(),
                name: "Provider Models".to_string(),
                title: None,
                description: None,
                mime_type: Some("application/json".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://events/{since}".to_string(),
                name: "Event Stream Since Seq".to_string(),
                title: None,
                description: None,
                mime_type: Some("text/plain".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://skills/{id}".to_string(),
                name: "Skill".to_string(),
                title: None,
                description: None,
                mime_type: Some("text/markdown".to_string()),
                icons: None,
            }
            .no_annotation(),
        ]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        let Ok(dir) = self.get_effective_project_dir().await else {
            return Err(ErrorData::internal_error("No active project", None));
        };
        match self.resolve_resource_uri(&request.uri, &dir).await {
            Some(text) => Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(text, &request.uri)],
            }),
            None => Err(ErrorData::resource_not_found(
                format!("Resource not found: {}", request.uri),
                None,
            )),
        }
    }
}

pub async fn run_server() -> Result<()> {
    let service = ShipServer::new();

    let running = service
        .serve(stdio())
        .await
        .map_err(|e| anyhow!("MCP Server initialization error: {:?}", e))?;

    running
        .waiting()
        .await
        .map_err(|e| anyhow!("MCP Server runtime error: {:?}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime::{EventAction, EventEntity, ModeConfig, add_mode, list_events_since};
    use ship_module_project::init_project;
    use tempfile::tempdir;

    #[tokio::test(flavor = "multi_thread")]
    async fn mcp_release_feature_flow_emits_events() {
        let tmp = tempdir().expect("tempdir");
        let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

        let server = ShipServer::new();
        *server.active_project.lock().await = Some(project_dir.clone());

        let release_result = server
            .create_release(Parameters(CreateReleaseRequest {
                version: "v0.3.0-alpha".to_string(),
                content: None,
            }))
            .await;
        assert!(
            release_result.contains("Created release:"),
            "unexpected release response: {}",
            release_result
        );

        let releases = list_releases(&project_dir).expect("list releases");
        assert_eq!(releases.len(), 1);
        let release_id = releases[0].id.clone();

        let feature_result = server
            .create_feature(Parameters(CreateFeatureRequest {
                title: "Filesystem Routing Migration".to_string(),
                content: None,
                release_id: Some(release_id),
                spec_id: None,
                branch: Some("feature/fs-routing".to_string()),
            }))
            .await;
        assert!(
            feature_result.contains("Created feature:"),
            "unexpected feature response: {}",
            feature_result
        );

        let features = list_features(&project_dir).expect("list features");
        assert_eq!(features.len(), 1);
        let feature_entry = get_feature_by_id(&project_dir, &features[0].id).expect("get feature");
        assert_eq!(
            feature_entry.feature.metadata.branch.as_deref(),
            Some("feature/fs-routing")
        );

        let events = list_events_since(&project_dir, 0, Some(200)).expect("list events");
        assert!(
            events
                .iter()
                .any(|event| event.entity == EventEntity::Release
                    && event.action == EventAction::Create),
            "missing Release.Create event"
        );
        assert!(
            events
                .iter()
                .any(|event| event.entity == EventEntity::Feature
                    && event.action == EventAction::Create),
            "missing Feature.Create event"
        );
    }

    #[test]
    fn mode_gate_normalizes_and_blocks_disallowed_tools() {
        let tmp = tempdir().expect("tempdir");
        let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

        add_mode(
            Some(project_dir.clone()),
            ModeConfig {
                id: "mode-gate-test".to_string(),
                name: "Mode Gate Test".to_string(),
                active_tools: vec!["ship_list_notes".to_string()],
                ..Default::default()
            },
        )
        .expect("add mode");
        set_active_mode(Some(project_dir.clone()), Some("mode-gate-test"))
            .expect("set active mode");

        ShipServer::enforce_mode_tool_gate(&project_dir, "list_notes").expect("list_notes allowed");
        ShipServer::enforce_mode_tool_gate(&project_dir, "ship_list_notes_tool")
            .expect("prefixed note tool allowed");
        ShipServer::enforce_mode_tool_gate(&project_dir, "create_workspace_tool")
            .expect("create workspace must remain control-plane allowed");
        ShipServer::enforce_mode_tool_gate(&project_dir, "repair_workspace")
            .expect("workspace repair must remain control-plane allowed");

        let blocked = ShipServer::enforce_mode_tool_gate(&project_dir, "update_note")
            .expect_err("update_note should be blocked");
        assert!(
            blocked.contains("blocked by active mode"),
            "unexpected mode gate message: {}",
            blocked
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn mcp_workspace_control_plane_round_trip() {
        let tmp = tempdir().expect("tempdir");
        let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

        let server = ShipServer::new();
        *server.active_project.lock().await = Some(project_dir.clone());

        let created = server
            .create_workspace_tool(Parameters(CreateWorkspaceToolRequest {
                branch: "feature/mode-control-plane".to_string(),
                workspace_type: Some("feature".to_string()),
                environment_id: None,
                feature_id: None,
                spec_id: None,
                release_id: None,
                mode_id: None,
                is_worktree: Some(false),
                worktree_path: None,
                activate: Some(true),
            }))
            .await;
        assert!(
            created.contains("\"branch\": \"feature/mode-control-plane\""),
            "unexpected create workspace response: {}",
            created
        );

        let fetched = server
            .resolve_resource_uri("ship://workspaces/feature/mode-control-plane", &project_dir)
            .await
            .expect("workspace resource");
        assert!(
            fetched.contains("\"id\": \"feature-mode-control-plane\""),
            "unexpected get workspace response: {}",
            fetched
        );

        let sessions_before = server
            .resolve_resource_uri("ship://sessions/feature/mode-control-plane", &project_dir)
            .await;
        let sessions_before = sessions_before.expect("sessions resource");
        assert_eq!(
            sessions_before.trim(),
            "[]",
            "expected no sessions before start, got {}",
            sessions_before
        );
    }
}
