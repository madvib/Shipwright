use anyhow::{Result, anyhow};
use ghost_issues;
use rmcp::transport::stdio;
use rmcp::{
    ErrorData, RoleServer, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, tool::ToolCallContext, wrapper::Parameters},
    model::{
        AnnotateAble, CallToolRequestParams, CallToolResult, Content, CreateMessageRequestParams,
        Implementation, ListResourceTemplatesResult, ListResourcesResult, ListToolsResult,
        PaginatedRequestParams, ProtocolVersion, RawResource, RawResourceTemplate,
        ReadResourceRequestParams, ReadResourceResult, ResourceContents, SamplingMessage,
        ServerCapabilities, ServerInfo, Tool,
    },
    service::{Peer, RequestContext},
    tool, tool_router,
};
use runtime::project::{get_active_project_global, get_project_dir, set_active_project_global};
use runtime::{
    add_status, autodetect_providers, create_user_skill, delete_skill, delete_user_skill,
    disable_provider, enable_provider, get_active_mode, get_config, get_effective_skill,
    list_effective_skills, list_events_since, list_models, list_providers, log_action_by, read_log,
    remove_status, set_active_mode, set_category_committed, update_skill, update_user_skill,
    workspace::{
        CreateWorkspaceRequest as RuntimeCreateWorkspaceRequest,
        EndWorkspaceSessionRequest as RuntimeEndWorkspaceSessionRequest, WorkspaceType,
        activate_workspace as runtime_activate_workspace,
        create_workspace as runtime_create_workspace,
        end_workspace_session as runtime_end_workspace_session,
        get_active_workspace_session as runtime_get_active_workspace_session,
        get_workspace as runtime_get_workspace,
        get_workspace_provider_matrix as runtime_get_workspace_provider_matrix,
        list_workspace_sessions as runtime_list_workspace_sessions,
        list_workspaces as runtime_list_workspaces, repair_workspace as runtime_repair_workspace,
        set_workspace_active_mode, start_workspace_session as runtime_start_workspace_session,
        sync_workspace as runtime_sync_workspace,
    },
};
use ship_module_git::{install_hooks, on_post_checkout};
use ship_module_project::ops::adr::{create_adr, get_adr_by_id, list_adrs};
use ship_module_project::ops::feature::{
    create_feature, get_feature_by_id, list_features, sync_feature_docs_after_session,
    update_feature_content,
};
use ship_module_project::ops::issue::{
    create_issue, delete_issue, get_issue_by_id, list_issues, move_issue_with_from, update_issue,
};
use ship_module_project::ops::note::{
    create_note, get_note_by_id, list_notes, update_note_content,
};
use ship_module_project::ops::release::{
    create_release, get_release_by_id, list_releases, update_release_content,
};
use ship_module_project::ops::spec::{create_spec, get_spec_by_id, list_specs, update_spec};
use ship_module_project::{
    IssueEntry, IssueStatus, NoteScope, get_project_name, list_registered_projects,
};
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
        // Extended tools (issues, specs, releases, etc.) require a mode OR a project workspace.
        const CORE_TOOLS: &[&str] = &[
            // Project context
            "open_project",
            "get_project_info",
            // Planning
            "create_note",
            "create_feature",
            "get_feature",
            "update_feature",
            "log_decision",
            // Workspace
            "list_workspaces",
            "get_workspace",
            "get_workspace_provider_matrix",
            "activate_workspace",
            "create_workspace_tool",
            "list_modes",
            "set_mode",
            "sync_workspace",
            "repair_workspace",
            // Session
            "start_session",
            "end_session",
            "get_session_status",
            "log_progress",
        ];
        let normalized = Self::normalize_mode_tool_id(tool_name);
        CORE_TOOLS.contains(&normalized.as_str())
    }

    fn is_project_workspace_tool(tool_name: &str) -> bool {
        // Tools auto-unlocked when the active workspace is type=project.
        // These cover the PM layer: issues, specs, releases, session history, log.
        const PROJECT_TOOLS: &[&str] = &[
            "create_issue",
            "update_issue",
            "move_issue",
            "delete_issue",
            "search_issues",
            "create_spec",
            "update_spec",
            "create_release",
            "get_release",
            "update_release",
            "list_sessions",
            "get_log",
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

        // Project workspace auto-unlocks the PM tool surface without needing a mode.
        if Self::is_project_workspace_tool(tool_name) {
            let active_type =
                runtime::workspace::get_active_workspace_type(project_dir).unwrap_or(None);
            if matches!(active_type, Some(runtime::WorkspaceType::Project)) {
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
        // Activate a project workspace (ship workspace activate ship) to unlock PM tools,
        // or create a mode with active_tools set to unlock specific tools.
        Err(format!(
            "Tool '{}' is not in the core workflow surface. \
             Activate the project workspace ('ship') or a mode with this tool in its \
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

    /// Get full project context for an agent starting a new session
    #[tool(
        description = "Get full project context: active workspace, active session, mode, features, \
        releases, specs, ADRs, and recent activity. Call this at the start of every session to \
        understand the current state before taking any action."
    )]
    async fn get_project_info(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };

        let name = get_project_name(&project_dir);
        let config = get_config(Some(project_dir.clone())).unwrap_or_default();
        let statuses: Vec<String> = config.statuses.iter().map(|s| s.id.clone()).collect();

        let issues = list_issues(&project_dir).unwrap_or_default();
        let releases = list_releases(&project_dir).unwrap_or_default();
        let features = list_features(&project_dir).unwrap_or_default();
        let specs = list_specs(&project_dir).unwrap_or_default();
        let adrs = list_adrs(&project_dir).unwrap_or_default();

        let mut out = format!("# Project: {}\n\n", name);

        // ── Active workspace & session ────────────────────────────────────────
        out.push_str("## Current Context\n");
        let workspaces = runtime_list_workspaces(&project_dir).unwrap_or_default();
        let active_workspace = workspaces.iter().find(|w| {
            matches!(
                w.status,
                runtime::WorkspaceStatus::Active
            )
        });
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
                    out.push_str(&format!(
                        "- Session: ACTIVE (id: {})",
                        session.id
                    ));
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

        // Issue summary
        out.push_str("\n## Open Issues\n");
        let open: Vec<&IssueEntry> = issues
            .iter()
            .filter(|e| e.status != IssueStatus::Done)
            .collect();
        if open.is_empty() {
            out.push_str("No open issues.\n");
        } else {
            for status in &statuses {
                let in_status: Vec<_> = open
                    .iter()
                    .filter(|e| e.status.to_string() == *status)
                    .collect();
                if !in_status.is_empty() {
                    out.push_str(&format!("\n### {}\n", status));
                    for e in in_status {
                        out.push_str(&format!("- {} ({})\n", e.issue.metadata.title, e.file_name));
                    }
                }
            }
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
                let docs_flag = if !has_docs { " ⚠ missing ## Documentation" } else { "" };
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

    // ─── Issue Tools ──────────────────────────────────────────────────────────

    /// Create a new issue
    #[tool(description = "Create a new issue in the active project")]
    async fn create_issue(&self, Parameters(req): Parameters<CreateIssueRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let status_str = req.status.as_deref().unwrap_or("backlog");
        let status = status_str
            .parse::<IssueStatus>()
            .unwrap_or(IssueStatus::Backlog);
        match create_issue(
            &project_dir,
            &req.title,
            &req.description,
            status.clone(),
            None,
            None,
            None,
            None,
        ) {
            Ok(file) => format!("Created issue: {} ({})", file.file_name, status),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Update an issue's title or description
    #[tool(description = "Update the title or description of an existing issue")]
    async fn update_issue(&self, Parameters(req): Parameters<UpdateIssueRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let issues = list_issues(&project_dir).unwrap_or_default();
        let entry = issues
            .iter()
            .find(|e| e.status.to_string() == req.status && e.file_name == req.file_name);
        match entry {
            Some(entry) => {
                let mut issue = entry.issue.clone();
                if let Some(title) = req.title {
                    issue.metadata.title = title;
                }
                if let Some(desc) = req.description {
                    issue.description = desc;
                }
                match update_issue(&project_dir, &entry.id, issue) {
                    Ok(_) => format!("Updated: {}", req.file_name),
                    Err(e) => format!("Error: {}", e),
                }
            }
            None => format!(
                "Error: Issue not found in status {} with filename {}",
                req.status, req.file_name
            ),
        }
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

    /// Create a skill
    #[tool(
        description = "Create a skill. Scope can be 'project' (default) or 'user' for global/core skills."
    )]
    async fn create_skill(&self, Parameters(req): Parameters<CreateSkillRequest>) -> String {
        let scope = match parse_skill_write_scope(req.scope.as_deref()) {
            Ok(scope) => scope,
            Err(err) => return format!("Error: {}", err),
        };

        let created = match scope {
            SkillWriteScope::Project => {
                let project_dir = match self.get_effective_project_dir().await {
                    Ok(project_dir) => project_dir,
                    Err(err) => return err,
                };
                match runtime::create_skill(&project_dir, &req.id, &req.name, &req.content) {
                    Ok(skill) => {
                        log_action_by(
                            &project_dir,
                            "agent",
                            "skill create",
                            &format!("{} ({})", skill.id, skill.name),
                        )
                        .ok();
                        skill
                    }
                    Err(err) => return format!("Error: {}", err),
                }
            }
            SkillWriteScope::User => match create_user_skill(&req.id, &req.name, &req.content) {
                Ok(skill) => skill,
                Err(err) => return format!("Error: {}", err),
            },
        };

        format!("Created skill: {} ({})", created.id, created.name)
    }

    /// Update an existing skill
    #[tool(description = "Update a skill name/content in 'project' (default) or 'user' scope.")]
    async fn update_skill(&self, Parameters(req): Parameters<UpdateSkillRequest>) -> String {
        let scope = match parse_skill_write_scope(req.scope.as_deref()) {
            Ok(scope) => scope,
            Err(err) => return format!("Error: {}", err),
        };

        let updated = match scope {
            SkillWriteScope::Project => {
                let project_dir = match self.get_effective_project_dir().await {
                    Ok(project_dir) => project_dir,
                    Err(err) => return err,
                };
                match update_skill(
                    &project_dir,
                    &req.id,
                    req.name.as_deref(),
                    req.content.as_deref(),
                ) {
                    Ok(skill) => {
                        log_action_by(
                            &project_dir,
                            "agent",
                            "skill update",
                            &format!("{} ({})", skill.id, skill.name),
                        )
                        .ok();
                        skill
                    }
                    Err(err) => return format!("Error: {}", err),
                }
            }
            SkillWriteScope::User => {
                match update_user_skill(&req.id, req.name.as_deref(), req.content.as_deref()) {
                    Ok(skill) => skill,
                    Err(err) => return format!("Error: {}", err),
                }
            }
        };

        format!("Updated skill: {} ({})", updated.id, updated.name)
    }

    /// Delete a skill
    #[tool(description = "Delete a skill in 'project' (default) or 'user' scope.")]
    async fn delete_skill(&self, Parameters(req): Parameters<DeleteSkillRequest>) -> String {
        let scope = match parse_skill_write_scope(req.scope.as_deref()) {
            Ok(scope) => scope,
            Err(err) => return format!("Error: {}", err),
        };

        match scope {
            SkillWriteScope::Project => {
                let project_dir = match self.get_effective_project_dir().await {
                    Ok(project_dir) => project_dir,
                    Err(err) => return err,
                };
                match delete_skill(&project_dir, &req.id) {
                    Ok(()) => {
                        log_action_by(&project_dir, "agent", "skill delete", &req.id).ok();
                    }
                    Err(err) => return format!("Error: {}", err),
                }
            }
            SkillWriteScope::User => {
                if let Err(err) = delete_user_skill(&req.id) {
                    return format!("Error: {}", err);
                }
            }
        }

        format!("Deleted skill: {}", req.id)
    }

    /// Move an issue to a different status
    #[tool(description = "Move an issue from one status to another (e.g. backlog → in-progress)")]
    async fn move_issue(&self, Parameters(req): Parameters<MoveIssueRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let from_status = match req.from_status.parse::<IssueStatus>() {
            Ok(s) => s,
            Err(e) => return format!("Error: {}", e),
        };
        let to_status = match req.to_status.parse::<IssueStatus>() {
            Ok(s) => s,
            Err(e) => return format!("Error: {}", e),
        };
        match move_issue_with_from(&project_dir, &req.file_name, from_status, to_status) {
            Ok(_) => format!("{}: {} → {}", req.file_name, req.from_status, req.to_status),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Delete an issue
    #[tool(description = "Delete an issue permanently")]
    async fn delete_issue(&self, Parameters(req): Parameters<DeleteIssueRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match delete_issue(&project_dir, &req.file_name) {
            Ok(_) => format!("Deleted: {}", req.file_name),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Search issues by text
    #[tool(description = "Search issues by text in their title or description")]
    async fn search_issues(&self, Parameters(req): Parameters<SearchIssuesRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match list_issues(&project_dir) {
            Ok(entries) => {
                let query = req.query.to_lowercase();
                let matches: Vec<&IssueEntry> = entries
                    .iter()
                    .filter(|e| {
                        e.issue.metadata.title.to_lowercase().contains(&query)
                            || e.issue.description.to_lowercase().contains(&query)
                    })
                    .collect();
                if matches.is_empty() {
                    return format!("No issues matching '{}'", req.query);
                }
                let mut out = format!("Issues matching '{}':\n", req.query);
                for e in matches {
                    out.push_str(&format!(
                        "- [{}] {} ({})\n",
                        e.status, e.issue.metadata.title, e.file_name
                    ));
                }
                out
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    // ─── ADR Tools ────────────────────────────────────────────────────────────

    /// Log an architectural decision
    #[tool(
        description = "Log an architectural decision (ADR). Use when committing to a technical \
        approach, trade-off, or design choice that future contributors need to understand. \
        Captures the decision and reasoning in the project record."
    )]
    async fn log_decision(&self, Parameters(req): Parameters<LogDecisionRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match create_adr(&project_dir, &req.title, "", &req.decision, "proposed") {
            Ok(entry) => format!(
                "Logged decision '{}' (id: {})",
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
        let issues = list_specs(&project_dir).unwrap_or_default();
        let entry = issues.iter().find(|e| e.file_name == req.file_name);
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

    /// Get a release's details
    #[tool(description = "Get the details and content of a release by ID")]
    async fn get_release(&self, Parameters(req): Parameters<GetReleaseRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match get_release_by_id(&project_dir, &req.id) {
            Ok(release) => match serde_json::to_string_pretty(&release) {
                Ok(json) => json,
                Err(e) => format!("Error serializing release: {}", e),
            },
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

    /// Get a feature's details
    #[tool(description = "Get the details and content of a feature by ID")]
    async fn get_feature(&self, Parameters(req): Parameters<GetFeatureRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match get_feature_by_id(&project_dir, &req.id) {
            Ok(feature) => match serde_json::to_string_pretty(&feature) {
                Ok(json) => json,
                Err(e) => format!("Error serializing feature: {}", e),
            },
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

    // ─── ADR / Log Tools ──────────────────────────────────────────────────────

    /// Get recent project log entries
    #[tool(description = "Get the recent action log for the active project")]
    async fn get_log(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match read_log(&project_dir) {
            Ok(content) => {
                if content.trim().is_empty() || content.trim() == "# Project Log" {
                    "No log entries yet.".to_string()
                } else {
                    content
                }
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    /// List events from the append-only event stream
    #[tool(
        description = "List events from the append-only project event stream. Supports cursor-style reads via the since sequence."
    )]
    async fn list_events(&self, Parameters(req): Parameters<ListEventsRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let since = req.since.unwrap_or(0);
        let limit = req.limit.unwrap_or(100);
        match list_events_since(&project_dir, since, Some(limit)) {
            Ok(events) => {
                if events.is_empty() {
                    return "No events found.".to_string();
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
                out
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    // ─── Ghost Issues Tools ───────────────────────────────────────────────────

    /// Scan the codebase for TODO/FIXME/HACK/BUG comments
    #[tool(
        description = "Scan the project codebase for TODO, FIXME, HACK, and BUG comments and return a summary"
    )]
    async fn ghost_scan(&self, Parameters(req): Parameters<GhostScanRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        if let Err(e) = ensure_builtin_plugin_namespaces(&project_dir) {
            return format!("Error: {}", e);
        }
        let root = req
            .dir
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| project_dir.parent().unwrap_or(&project_dir).to_path_buf());

        match ghost_issues::scan(&project_dir, &root) {
            Ok(result) => {
                let unpromoted: Vec<_> = result.issues.iter().filter(|g| !g.promoted).collect();
                if unpromoted.is_empty() {
                    return "No ghost issues found.".to_string();
                }
                let mut out = format!(
                    "Found {} ghost issue{}:\n\n",
                    unpromoted.len(),
                    if unpromoted.len() == 1 { "" } else { "s" }
                );
                for g in &unpromoted {
                    out.push_str(&format!("- {}\n", g.display()));
                }
                out
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Promote a ghost issue to a real tracked issue
    #[tool(
        description = "Promote a ghost issue (TODO/FIXME comment) to a real tracked issue in the backlog"
    )]
    async fn ghost_promote(&self, Parameters(req): Parameters<GhostPromoteRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        if let Err(e) = ensure_builtin_plugin_namespaces(&project_dir) {
            return format!("Error: {}", e);
        }
        match ghost_issues::mark_promoted(&project_dir, &req.file, req.line) {
            Ok(true) => {
                if let Ok(Some(scan)) = ghost_issues::load_last_scan(&project_dir) {
                    if let Some(g) = scan
                        .issues
                        .iter()
                        .find(|g| g.file == req.file && g.line == req.line)
                    {
                        let title = g.suggested_title();
                        let desc = format!(
                            "Promoted from `{}:{}` ({}).\n\nOriginal comment: {}",
                            g.file,
                            g.line,
                            g.kind.as_str(),
                            g.text.trim()
                        );
                        match create_issue(
                            &project_dir,
                            &title,
                            &desc,
                            IssueStatus::Backlog,
                            None,
                            None,
                            None,
                            None,
                        ) {
                            Ok(file) => format!("Created issue: {}", file.file_name),
                            Err(e) => format!("Marked promoted but failed to create issue: {}", e),
                        }
                    } else {
                        "Marked as promoted.".to_string()
                    }
                } else {
                    "Marked as promoted.".to_string()
                }
            }
            Ok(false) => format!(
                "Ghost issue not found at {}:{}. Run ghost_scan first.",
                req.file, req.line
            ),
            Err(e) => format!("Error: {}", e),
        }
    }

    // ─── Status / Category Tools ─────────────────────────────────────────────

    /// Add or remove an issue status/category
    #[tool(
        description = "Add or remove an issue status/category. action: 'add' or 'remove'. Existing issues are not affected when removing."
    )]
    async fn manage_status(&self, Parameters(req): Parameters<ManageStatusRequest>) -> String {
        let project_dir = self.get_effective_project_dir().await.ok();
        match req.action.as_str() {
            "add" => match add_status(project_dir, &req.name) {
                Ok(_) => format!(
                    "Added status: {}",
                    req.name.to_lowercase().replace(' ', "-")
                ),
                Err(e) => format!("Error: {}", e),
            },
            "remove" => match remove_status(project_dir, &req.name) {
                Ok(_) => format!("Removed status: {}", req.name),
                Err(e) => format!("Error: {}", e),
            },
            _ => "Error: action must be 'add' or 'remove'".to_string(),
        }
    }

    // ─── AI Generation Tools ─────────────────────────────────────────────────

    /// Generate a detailed issue description from a title using AI
    #[tool(
        description = "Generate a detailed, actionable issue description from a title. Uses MCP sampling (Claude Code) or direct Anthropic API."
    )]
    async fn generate_issue_description(
        &self,
        peer: Peer<RoleServer>,
        Parameters(req): Parameters<GenerateIssueRequest>,
    ) -> String {
        let system = "You are a project management assistant. Generate clear, concise, actionable issue descriptions in markdown. Include: what needs to be done, why it matters, and acceptance criteria. Be specific but not verbose. 2-4 paragraphs max.";
        let prompt = match &req.context {
            Some(ctx) => format!(
                "Generate an issue description for:\n\nTitle: {}\n\nContext: {}",
                req.title, ctx
            ),
            None => format!("Generate an issue description for:\n\nTitle: {}", req.title),
        };
        self.generate_with_sampling(peer, system, &prompt, 800)
            .await
    }

    /// Generate an Architecture Decision Record from a problem statement
    #[tool(
        description = "Generate an ADR (Architecture Decision Record) from a problem statement using AI"
    )]
    async fn generate_adr(
        &self,
        peer: Peer<RoleServer>,
        Parameters(req): Parameters<GenerateAdrRequest>,
    ) -> String {
        let system = "You are a software architect. Generate a concise Architecture Decision Record. Format: state the context, decision, and consequences. Be direct and practical. Use markdown.";
        let prompt = match &req.constraints {
            Some(c) => format!(
                "Generate an ADR for:\n\nProblem: {}\n\nConstraints/Options: {}",
                req.problem, c
            ),
            None => format!("Generate an ADR for:\n\nProblem: {}", req.problem),
        };
        self.generate_with_sampling(peer, system, &prompt, 1000)
            .await
    }

    /// Brainstorm issue ideas for a topic
    #[tool(description = "Brainstorm a list of issue suggestions for a given topic using AI")]
    async fn brainstorm_issues(
        &self,
        peer: Peer<RoleServer>,
        Parameters(req): Parameters<BrainstormRequest>,
    ) -> String {
        let count = req.count.unwrap_or(5);
        let system = "You are a product and engineering planning assistant. Generate specific, actionable issue titles with one-sentence descriptions. Format as a numbered list.";
        let prompt = format!(
            "Brainstorm {} issue ideas for: {}\n\nFormat each as:\n1. **Title** — one sentence description",
            count, req.topic
        );
        self.generate_with_sampling(peer, system, &prompt, 600)
            .await
    }

    // ─── Git Config Tools ─────────────────────────────────────────────────────

    /// Update git commit settings for the active project
    #[tool(
        description = "Set whether a category (issues/releases/features/specs/adrs/notes/agents/ship.toml/templates) is committed to git or kept local. Updates .ship/.gitignore automatically."
    )]
    async fn git_config_set(&self, Parameters(req): Parameters<GitIncludeRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let known = [
            "issues",
            "releases",
            "features",
            "adrs",
            "specs",
            "notes",
            "agents",
            "ship.toml",
            "templates",
        ];
        if !known.contains(&req.category.as_str()) {
            return format!(
                "Unknown category '{}'. Use: {}",
                req.category,
                known.join(", ")
            );
        }
        match set_category_committed(&project_dir, &req.category, req.commit) {
            Ok(_) => format!(
                "{} is now {}",
                req.category,
                if req.commit {
                    "committed to git"
                } else {
                    "local only"
                }
            ),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Install Ship git hooks in this repository
    #[tool(
        description = "Install Ship's git hooks (including post-checkout) in the active project's repository"
    )]
    async fn git_hooks_install(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let Some(project_root) = project_dir.parent() else {
            return "Error: Could not resolve project root".to_string();
        };
        let git_dir = project_root.join(".git");
        if !git_dir.exists() {
            return format!("Error: No git repository found at {}", git_dir.display());
        }
        match install_hooks(&git_dir) {
            Ok(_) => format!("Installed git hooks in {}", git_dir.join("hooks").display()),
            Err(err) => format!("Error: {}", err),
        }
    }

    /// Regenerate feature-aware context files for a branch
    #[tool(description = "Regenerate branch-scoped CLAUDE.md and .mcp.json for a feature branch")]
    async fn git_feature_sync(&self, Parameters(req): Parameters<GitFeatureSyncRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let Some(project_root) = project_dir.parent() else {
            return "Error: Could not resolve project root".to_string();
        };
        let branch = match req.branch {
            Some(branch) if !branch.trim().is_empty() => branch,
            _ => match current_branch(project_root) {
                Ok(branch) => branch,
                Err(err) => return format!("Error: {}", err),
            },
        };
        match on_post_checkout(&project_dir, &branch, project_root) {
            Ok(_) => format!("Synced feature context for branch {}", branch),
            Err(err) => format!("Error: {}", err),
        }
    }

    // ─── Provider Tools ───────────────────────────────────────────────────────

    /// List all known AI providers and their installed/connected status
    #[tool(
        description = "List all known AI agent providers with installed (in PATH) and connected (enabled in project) status, version, and available models"
    )]
    async fn list_providers_tool(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match list_providers(&project_dir) {
            Ok(providers) => match serde_json::to_string_pretty(&providers) {
                Ok(json) => json,
                Err(e) => format!("Error serialising providers: {}", e),
            },
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Connect (enable) an AI provider for this project
    #[tool(
        description = "Connect (enable) an AI provider for this project by updating runtime settings in SQLite. Provider ID must be one of: claude, gemini, codex"
    )]
    async fn connect_provider(
        &self,
        Parameters(req): Parameters<ConnectProviderRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match enable_provider(&project_dir, &req.provider_id) {
            Ok(true) => format!("Connected provider: {}", req.provider_id),
            Ok(false) => format!("Provider '{}' is already connected.", req.provider_id),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Disconnect (disable) an AI provider from this project
    #[tool(
        description = "Disconnect (disable) an AI provider from this project by updating runtime settings in SQLite"
    )]
    async fn disconnect_provider(
        &self,
        Parameters(req): Parameters<DisconnectProviderRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match disable_provider(&project_dir, &req.provider_id) {
            Ok(true) => format!("Disconnected provider: {}", req.provider_id),
            Ok(false) => format!("Provider '{}' was not connected.", req.provider_id),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Detect installed providers in PATH and auto-connect them
    #[tool(
        description = "Detect which AI providers are installed in PATH and automatically connect them to this project"
    )]
    async fn detect_providers(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match autodetect_providers(&project_dir) {
            Ok(found) if found.is_empty() => "No new providers detected.".to_string(),
            Ok(found) => format!("Detected and connected: {}", found.join(", ")),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// List available models for a provider
    #[tool(
        description = "List available models for a provider with context window sizes and recommended model. Use for UI autocomplete and model selection."
    )]
    async fn list_models_tool(&self, Parameters(req): Parameters<ListModelsRequest>) -> String {
        match list_models(&req.provider_id) {
            Ok(models) => match serde_json::to_string_pretty(&models) {
                Ok(json) => json,
                Err(e) => format!("Error serialising models: {}", e),
            },
            Err(e) => format!("Error: {}", e),
        }
    }

    // ─── Mode / Workspace Control Plane Tools ──────────────────────────────

    /// List available modes plus active mode selection
    #[tool(
        description = "List available modes and the active mode. Modes are the control plane for tool/provider context."
    )]
    async fn list_modes(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(project_dir) => project_dir,
            Err(err) => return err,
        };
        match get_config(Some(project_dir)) {
            Ok(config) => {
                let payload = serde_json::json!({
                    "active_mode": config.active_mode,
                    "modes": config.modes,
                });
                serde_json::to_string_pretty(&payload).unwrap_or_else(|e| format!("Error: {}", e))
            }
            Err(err) => format!("Error: {}", err),
        }
    }

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

    /// List all workspaces in the project runtime
    #[tool(description = "List all workspaces in this project.")]
    async fn list_workspaces(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(project_dir) => project_dir,
            Err(err) => return err,
        };
        match runtime_list_workspaces(&project_dir) {
            Ok(workspaces) => serde_json::to_string_pretty(&workspaces)
                .unwrap_or_else(|e| format!("Error serializing workspaces: {}", e)),
            Err(err) => format!("Error: {}", err),
        }
    }

    /// Get one workspace by branch/id (defaults to current git branch)
    #[tool(
        description = "Get one workspace by branch/id. If branch is omitted, resolves from current git branch."
    )]
    async fn get_workspace(&self, Parameters(req): Parameters<GetWorkspaceRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(project_dir) => project_dir,
            Err(err) => return err,
        };
        let branch =
            match Self::resolve_workspace_branch_for_project(&project_dir, req.branch.as_deref()) {
                Ok(branch) => branch,
                Err(err) => return format!("Error: {}", err),
            };
        match runtime_get_workspace(&project_dir, &branch) {
            Ok(Some(workspace)) => serde_json::to_string_pretty(&workspace)
                .unwrap_or_else(|e| format!("Error serializing workspace: {}", e)),
            Ok(None) => format!("Workspace '{}' not found", branch),
            Err(err) => format!("Error: {}", err),
        }
    }

    /// Resolve provider policy matrix for a workspace/mode combination
    #[tool(
        description = "Resolve provider policy for a workspace (allowed providers, source, and resolution errors)."
    )]
    async fn get_workspace_provider_matrix(
        &self,
        Parameters(req): Parameters<WorkspaceProviderMatrixRequest>,
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
        match runtime_get_workspace_provider_matrix(&project_dir, &branch, req.mode_id.as_deref()) {
            Ok(matrix) => serde_json::to_string_pretty(&matrix)
                .unwrap_or_else(|e| format!("Error serializing provider matrix: {}", e)),
            Err(err) => format!("Error: {}", err),
        }
    }

    /// Create or update a workspace record
    #[tool(
        description = "Create or update a workspace runtime record (feature/refactor/experiment/hotfix)."
    )]
    async fn create_workspace_tool(
        &self,
        Parameters(req): Parameters<CreateWorkspaceToolRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(project_dir) => project_dir,
            Err(err) => return err,
        };

        let parsed_workspace_type = match req.workspace_type {
            Some(workspace_type) => match workspace_type.parse::<WorkspaceType>() {
                Ok(parsed) => Some(parsed),
                Err(err) => return format!("Error: {}", err),
            },
            None => None,
        };

        let workspace_request = RuntimeCreateWorkspaceRequest {
            branch: req.branch.clone(),
            workspace_type: parsed_workspace_type,
            status: None,
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
    async fn start_session(
        &self,
        Parameters(req): Parameters<StartSessionRequest>,
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
        get_project_info, bumps updated timestamps on touched features, and records the session \
        history. Call this when work is complete or paused for the day."
    )]
    async fn end_session(
        &self,
        Parameters(req): Parameters<EndSessionRequest>,
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

        // Emit a log event so this surfaces in get_project_info immediately.
        let summary = session.summary.as_deref().unwrap_or_default();
        let log_line = format!("session ended [{}]: {}", branch, summary);
        if let Err(e) = log_action_by(&project_dir, "agent", "session.end", &log_line) {
            eprintln!("[ship] warning: failed to log session end: {}", e);
        }

        // Touch feature timestamps for all updated features.
        for feature_id in &session.updated_feature_ids {
            match get_feature_by_id(&project_dir, feature_id) {
                Ok(entry) => {
                    if let Err(e) =
                        update_feature_content(&project_dir, feature_id, &entry.feature.body)
                    {
                        eprintln!(
                            "[ship] warning: failed to touch feature '{}': {}",
                            feature_id, e
                        );
                    }
                }
                Err(e) => eprintln!(
                    "[ship] warning: error reading feature '{}': {}",
                    feature_id, e
                ),
            }
        }

        serde_json::to_string_pretty(&session)
            .unwrap_or_else(|e| format!("Error serializing workspace session: {}", e))
    }

    /// Return active session (if any) for a branch
    #[tool(
        description = "Get the active session status for a workspace branch (or current git branch). \
        Returns session details including goal, mode, provider, and when it started."
    )]
    async fn get_session_status(
        &self,
        Parameters(req): Parameters<GetWorkspaceRequest>,
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
        match runtime_get_active_workspace_session(&project_dir, &branch) {
            Ok(Some(session)) => serde_json::to_string_pretty(&session)
                .unwrap_or_else(|e| format!("Error serializing workspace session: {}", e)),
            Ok(None) => format!("No active workspace session for '{}'", branch),
            Err(err) => format!("Error: {}", err),
        }
    }

    /// Log a progress note within the active session
    #[tool(
        description = "Record a progress note for the active session. Use mid-session to log \
        what you did, decisions made, or blockers encountered. Notes appear in the project log \
        and surface in get_project_info. Requires an active session (call start_session first)."
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
                )
            }
            Err(err) => return format!("Error checking session: {}", err),
            Ok(Some(_)) => {}
        }
        match log_action_by(&project_dir, "agent", "session.progress", &req.note) {
            Ok(()) => format!("Progress logged for session on '{}'.", branch),
            Err(e) => format!("Error logging progress: {}", e),
        }
    }

    /// List sessions, optionally filtered by branch
    #[tool(description = "List past workspace sessions, optionally filtered by workspace branch/id.")]
    async fn list_sessions(
        &self,
        Parameters(req): Parameters<ListSessionsRequest>,
    ) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(project_dir) => project_dir,
            Err(err) => return err,
        };
        let limit = req.limit.unwrap_or(50);
        let branch = req
            .branch
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        match runtime_list_workspace_sessions(&project_dir, branch, limit) {
            Ok(sessions) => serde_json::to_string_pretty(&sessions)
                .unwrap_or_else(|e| format!("Error serializing workspace sessions: {}", e)),
            Err(err) => format!("Error: {}", err),
        }
    }

    // ─── Time Tracking Tools ─────────────────────────────────────────────────

    /// Start a timer for an issue
    #[tool(description = "Start a time tracking timer for an issue")]
    async fn time_start(&self, Parameters(req): Parameters<TimeStartRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        if let Err(e) = ensure_builtin_plugin_namespaces(&project_dir) {
            return format!("Error: {}", e);
        }
        // Try to resolve title from issue file
        let issue_title = get_issue_by_id(&project_dir, &req.issue_file)
            .map(|entry| entry.issue.metadata.title)
            .unwrap_or_else(|_| req.issue_file.clone());
        match time_tracker::start_timer(&project_dir, &req.issue_file, &issue_title, req.note) {
            Ok(t) => format!(
                "Timer started: {} at {}",
                t.issue_title,
                t.started_at.format("%H:%M")
            ),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Stop the running timer
    #[tool(description = "Stop the currently running time tracking timer")]
    async fn time_stop(&self, Parameters(req): Parameters<TimeStopRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        if let Err(e) = ensure_builtin_plugin_namespaces(&project_dir) {
            return format!("Error: {}", e);
        }
        match time_tracker::stop_timer(&project_dir, req.note) {
            Ok(e) => format!(
                "Timer stopped: {} — {}",
                e.issue_title,
                time_tracker::format_duration(e.duration_minutes)
            ),
            Err(err) => format!("Error: {}", err),
        }
    }
}

impl ShipServer {
    /// Generate text via MCP sampling (peer.create_message).
    /// Requires the MCP client to support sampling (e.g. Claude Code).
    /// Returns an error message string if sampling is unavailable.
    async fn generate_with_sampling(
        &self,
        peer: Peer<RoleServer>,
        system: &str,
        prompt: &str,
        max_tokens: u32,
    ) -> String {
        if peer
            .peer_info()
            .map_or(false, |info| info.capabilities.sampling.is_some())
        {
            let params = CreateMessageRequestParams {
                messages: vec![SamplingMessage::user_text(prompt)],
                system_prompt: Some(system.to_string()),
                max_tokens,
                model_preferences: None,
                include_context: None,
                temperature: None,
                stop_sequences: None,
                metadata: None,
                tools: None,
                tool_choice: None,
                meta: None,
                task: None,
            };
            match peer.create_message(params).await {
                Ok(result) => {
                    return result
                        .message
                        .content
                        .first()
                        .and_then(|c| c.as_text())
                        .map(|t| t.text.clone())
                        .unwrap_or_else(|| "No content returned from sampling.".to_string());
                }
                Err(e) => return format!("Sampling failed: {}", e),
            }
        }
        "AI generation unavailable: the connected MCP client does not support sampling. Use Claude Code or another sampling-capable client.".to_string()
    }

    /// Resolve a `ship://` URI to its text content, or `None` if not found.
    async fn resolve_resource_uri(&self, uri: &str, dir: &PathBuf) -> Option<String> {
        // ship://issues
        if uri == "ship://issues" {
            let entries = list_issues(dir).ok()?;
            if entries.is_empty() {
                return Some("No issues found.".to_string());
            }
            let mut out = String::from("Issues:\n");
            for e in &entries {
                out.push_str(&format!(
                    "- [{}] {} ({})\n",
                    e.status, e.issue.metadata.title, e.file_name
                ));
            }
            return Some(out);
        }
        // ship://issues/{status}/{file}
        if let Some(rest) = uri.strip_prefix("ship://issues/") {
            let parts: Vec<&str> = rest.splitn(2, '/').collect();
            if parts.len() == 2 {
                let (status, file) = (parts[0], parts[1]);
                return get_issue_by_id(dir, file).ok().and_then(|entry| {
                    if entry.status.to_string() != status {
                        return None;
                    }
                    format!(
                        "Title: {}\nStatus: {}\nCreated: {}\nUpdated: {}\n\n{}",
                        entry.issue.metadata.title,
                        status,
                        entry.issue.metadata.created,
                        entry.issue.metadata.updated,
                        entry.issue.description
                    )
                    .into()
                });
            }
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

enum SkillWriteScope {
    Project,
    User,
}

fn parse_skill_write_scope(raw: Option<&str>) -> Result<SkillWriteScope> {
    match raw
        .unwrap_or("project")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "project" => Ok(SkillWriteScope::Project),
        "user" | "global" => Ok(SkillWriteScope::User),
        other => Err(anyhow!(
            "Invalid skill scope '{}'. Expected one of: project, user",
            other
        )),
    }
}

fn ensure_builtin_plugin_namespaces(project_dir: &PathBuf) -> Result<()> {
    let mut registry = runtime::PluginRegistry::new();
    registry.register_with_project(project_dir, Box::new(ghost_issues::GhostIssues))?;
    registry.register_with_project(project_dir, Box::new(time_tracker::TimeTracker))?;
    Ok(())
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
                 PLANNING: get_project_info → create_note / create_feature / update_feature / log_decision\n\
                 WORKSPACE: list_workspaces → activate_workspace → set_mode\n\
                 SESSION: start_session → (work) → log_progress → end_session\n\n\
                 By default only core workflow tools are visible. To access extended tools \
                 (issues, specs, releases, time tracking, etc.), activate a mode that includes \
                 them in its active_tools list. Call open_project first if the project is not \
                 auto-detected. Use resources (ship://) to read documents without a tool call."
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
                Some(runtime::WorkspaceType::Project)
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
            RawResource::new("ship://issues", "Issues").no_annotation(),
            RawResource::new("ship://features", "Features").no_annotation(),
            RawResource::new("ship://releases", "Releases").no_annotation(),
            RawResource::new("ship://specs", "Specs").no_annotation(),
            RawResource::new("ship://adrs", "ADRs").no_annotation(),
            RawResource::new("ship://notes", "Project Notes").no_annotation(),
            RawResource::new("ship://skills", "Skills").no_annotation(),
        ]))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        Ok(ListResourceTemplatesResult::with_all_items(vec![
            RawResourceTemplate {
                uri_template: "ship://issues/{status}/{file}".to_string(),
                name: "Issue".to_string(),
                title: Some("Issue by status and filename".to_string()),
                description: None,
                mime_type: Some("text/markdown".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://features/{file}".to_string(),
                name: "Feature".to_string(),
                title: None,
                description: None,
                mime_type: Some("text/markdown".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://releases/{file}".to_string(),
                name: "Release".to_string(),
                title: None,
                description: None,
                mime_type: Some("text/markdown".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://specs/{file}".to_string(),
                name: "Spec".to_string(),
                title: None,
                description: None,
                mime_type: Some("text/markdown".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://adrs/{file}".to_string(),
                name: "ADR".to_string(),
                title: None,
                description: None,
                mime_type: Some("text/markdown".to_string()),
                icons: None,
            }
            .no_annotation(),
            RawResourceTemplate {
                uri_template: "ship://notes/{file}".to_string(),
                name: "Note".to_string(),
                title: None,
                description: None,
                mime_type: Some("text/markdown".to_string()),
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

    #[tokio::test(flavor = "multi_thread")]
    async fn mcp_git_feature_sync_writes_context_files() {
        let tmp = tempdir().expect("tempdir");
        let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

        let server = ShipServer::new();
        *server.active_project.lock().await = Some(project_dir.clone());

        let feature_result = server
            .create_feature(Parameters(CreateFeatureRequest {
                title: "Auth flow".to_string(),
                content: Some("Ship auth.".to_string()),
                release_id: None,
                spec_id: None,
                branch: Some("feature/auth".to_string()),
            }))
            .await;
        assert!(
            feature_result.contains("Created feature:"),
            "unexpected feature response: {}",
            feature_result
        );

        let sync_result = server
            .git_feature_sync(Parameters(GitFeatureSyncRequest {
                branch: Some("feature/auth".to_string()),
            }))
            .await;
        assert!(
            sync_result.contains("Synced feature context"),
            "unexpected sync response: {}",
            sync_result
        );

        let claude_md = tmp.path().join("CLAUDE.md");
        let mcp_json = tmp.path().join(".mcp.json");
        assert!(claude_md.exists(), "CLAUDE.md should be written");
        assert!(mcp_json.exists(), ".mcp.json should be written");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn mcp_move_issue_rejects_from_status_mismatch() {
        let tmp = tempdir().expect("tempdir");
        let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

        let server = ShipServer::new();
        *server.active_project.lock().await = Some(project_dir.clone());

        let created = server
            .create_issue(Parameters(CreateIssueRequest {
                title: "Ops guard".to_string(),
                description: "ensure from-status check".to_string(),
                status: Some("backlog".to_string()),
            }))
            .await;
        assert!(
            created.contains("Created issue:"),
            "unexpected create response: {}",
            created
        );

        let issues = list_issues(&project_dir).expect("list issues");
        assert_eq!(issues.len(), 1);
        let file_name = issues[0].file_name.clone();

        let moved = server
            .move_issue(Parameters(MoveIssueRequest {
                file_name,
                from_status: "in-progress".to_string(),
                to_status: "done".to_string(),
            }))
            .await;

        assert!(
            moved.contains("Invalid status transition"),
            "unexpected move response: {}",
            moved
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
        ShipServer::enforce_mode_tool_gate(&project_dir, "get_workspace_provider_matrix")
            .expect("workspace provider matrix must remain control-plane allowed");
        ShipServer::enforce_mode_tool_gate(&project_dir, "repair_workspace")
            .expect("workspace repair must remain control-plane allowed");

        let blocked = ShipServer::enforce_mode_tool_gate(&project_dir, "create_issue")
            .expect_err("create_issue should be blocked");
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
            .get_workspace(Parameters(GetWorkspaceRequest {
                branch: Some("feature/mode-control-plane".to_string()),
            }))
            .await;
        assert!(
            fetched.contains("\"id\": \"feature-mode-control-plane\""),
            "unexpected get workspace response: {}",
            fetched
        );

        let sessions_before = server
            .list_sessions(Parameters(ListSessionsRequest {
                branch: Some("feature/mode-control-plane".to_string()),
                limit: Some(10),
            }))
            .await;
        assert_eq!(
            sessions_before.trim(),
            "[]",
            "expected no sessions before start, got {}",
            sessions_before
        );
    }
}
