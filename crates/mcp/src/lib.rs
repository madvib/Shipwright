use anyhow::{Result, anyhow};
use ghost_issues;
use rmcp::schemars::{self, JsonSchema};
use rmcp::transport::stdio;
use rmcp::{
    ErrorData, RoleServer, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        AnnotateAble, CreateMessageRequestParams, Implementation, ListResourceTemplatesResult,
        ListResourcesResult, ProtocolVersion, RawResource, RawResourceTemplate,
        ReadResourceRequestParams, ReadResourceResult, ResourceContents, SamplingMessage,
        ServerCapabilities, ServerInfo,
    },
    service::Peer,
    tool, tool_handler, tool_router,
};
use runtime::project::{get_active_project_global, get_project_dir, set_active_project_global};
use runtime::{
    add_status, autodetect_providers, create_user_skill, delete_skill, delete_user_skill,
    disable_provider, enable_provider, get_config, get_effective_skill, list_effective_skills,
    list_events_since, list_models, list_providers, log_action_by, read_log, remove_status,
    set_category_committed, update_skill, update_user_skill,
};
use serde::Deserialize;
use ship_module_git::{install_hooks, on_post_checkout};
use ship_module_project::ops::adr::{create_adr, get_adr_by_id, list_adrs};
use ship_module_project::ops::feature::{
    create_feature, get_feature_by_id, list_features, update_feature_content,
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

// ─── Request Types ────────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct CreateIssueRequest {
    /// The title of the issue
    pub title: String,
    /// The detailed description of the issue
    pub description: String,
    /// Initial status: backlog (default), in-progress, blocked, or done
    pub status: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetIssueRequest {
    /// Issue filename (e.g. "my-feature.md")
    pub file_name: String,
    /// Status folder to look in. If omitted, all statuses are searched.
    pub status: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateIssueRequest {
    /// Issue filename (e.g. "my-feature.md")
    pub file_name: String,
    /// Current status folder
    pub status: String,
    /// New title (optional)
    pub title: Option<String>,
    /// New description (optional)
    pub description: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DeleteIssueRequest {
    /// Issue filename (e.g. "my-feature.md")
    pub file_name: String,
    /// Status folder the issue is in
    pub status: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct MoveIssueRequest {
    /// Issue filename (e.g. "my-feature.md")
    pub file_name: String,
    /// Current status
    pub from_status: String,
    /// Target status
    pub to_status: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct SearchIssuesRequest {
    /// Text to search for in issue titles and descriptions
    pub query: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateAdrRequest {
    /// Title of the architecture decision
    pub title: String,
    /// The decision content / reasoning
    pub decision: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct OpenProjectRequest {
    /// The absolute path to the project root
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TrackProjectRequest {
    /// The name of the project
    pub name: String,
    /// The absolute path to the project root
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TimeStartRequest {
    /// Issue filename (e.g. "my-feature.md")
    pub issue_file: String,
    /// Optional note for this session
    pub note: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TimeStopRequest {
    /// Optional note to attach to the completed entry
    pub note: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GenerateIssueRequest {
    /// Title or brief description of the issue to generate content for
    pub title: String,
    /// Optional extra context (e.g. related issues, tech stack)
    pub context: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GenerateAdrRequest {
    /// The problem or decision to address
    pub problem: String,
    /// Optional constraints or options already under consideration
    pub constraints: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct BrainstormRequest {
    /// Topic or area to brainstorm issues for
    pub topic: String,
    /// Number of issue suggestions to generate (default 5)
    pub count: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GitIncludeRequest {
    /// Category to change: issues, releases, features, specs, adrs, notes, agents, ship.toml, templates
    pub category: String,
    /// true = commit to git, false = local only (gitignored)
    pub commit: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct GitFeatureSyncRequest {
    /// Optional branch name. If omitted, resolves from `git branch --show-current`.
    pub branch: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateNoteRequest {
    /// Title of the note
    pub title: String,
    /// Optional markdown content
    pub content: Option<String>,
    /// Scope: project (default) or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListNotesRequest {
    /// Scope: project (default) or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetNoteRequest {
    /// Note filename (e.g. "session-summary.md")
    pub file_name: String,
    /// Scope: project (default) or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateNoteRequest {
    /// Note filename (e.g. "session-summary.md")
    pub file_name: String,
    /// Full replacement markdown content
    pub content: String,
    /// Scope: project (default) or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateSkillRequest {
    /// Stable skill id (e.g. "task-policy")
    pub id: String,
    /// Human-readable skill name
    pub name: String,
    /// Skill body content; supports $ARGUMENTS placeholder
    pub content: String,
    /// Scope: project (default) or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListSkillsRequest {
    /// Scope: effective (default), project, or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetSkillRequest {
    /// Skill id (without .md)
    pub id: String,
    /// Scope: effective (default), project, or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateSkillRequest {
    /// Skill id (without .md)
    pub id: String,
    /// Optional new display name
    pub name: Option<String>,
    /// Optional replacement content
    pub content: Option<String>,
    /// Scope: project (default) or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DeleteSkillRequest {
    /// Skill id (without .md)
    pub id: String,
    /// Scope: project (default) or user
    pub scope: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ManageStatusRequest {
    /// Action: "add" or "remove"
    pub action: String,
    /// Status name (e.g. "review", "testing")
    pub name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct GhostScanRequest {
    /// Directory to scan. Defaults to the project root (parent of .ship).
    pub dir: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GhostPromoteRequest {
    /// Relative file path of the ghost issue
    pub file: String,
    /// Line number of the ghost issue
    pub line: usize,
}

#[derive(Deserialize, JsonSchema)]
pub struct StatusNameRequest {
    /// Status name (e.g. "review", "testing")
    pub name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TimeLogRequest {
    /// Issue filename
    pub issue_file: String,
    /// Duration in minutes
    pub minutes: u64,
    /// Optional note
    pub note: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateSpecRequest {
    /// Title of the spec
    pub title: String,
    /// Initial markdown content (optional — defaults to a blank template)
    pub content: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetSpecRequest {
    /// Spec filename (e.g. "my-feature.md")
    pub file_name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateSpecRequest {
    /// Spec filename (e.g. "my-feature.md")
    pub file_name: String,
    /// Full replacement content
    pub content: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateReleaseRequest {
    /// Version label (e.g. "v0.1.0-alpha")
    pub version: String,
    /// Initial markdown content (optional — defaults to a scaffold)
    pub content: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetReleaseRequest {
    /// Release version/id (e.g. "v0.1.0-alpha")
    pub id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateReleaseRequest {
    /// Release version/id (e.g. "v0.1.0-alpha")
    pub id: String,
    /// Full replacement content
    pub content: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateFeatureRequest {
    /// Feature title
    pub title: String,
    /// Initial markdown content (optional — defaults to a scaffold)
    pub content: Option<String>,
    /// Linked release ID (optional)
    pub release_id: Option<String>,
    /// Linked spec ID (optional)
    pub spec_id: Option<String>,
    /// Linked git branch name (optional)
    pub branch: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetFeatureRequest {
    /// Feature ID (e.g. "agent-mode-ui")
    pub id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateFeatureRequest {
    /// Feature ID
    pub id: String,
    /// Full replacement content
    pub content: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct FeatureIdRequest {
    /// Feature ID
    pub id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetAdrRequest {
    /// ADR filename (e.g. "use-postgresql.json")
    pub file_name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListEventsRequest {
    /// Only return events where seq > since
    pub since: Option<u64>,
    /// Maximum number of events to return (default 100)
    pub limit: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ConnectProviderRequest {
    /// Provider ID to enable (claude, gemini, codex)
    pub provider_id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct DisconnectProviderRequest {
    /// Provider ID to disable (claude, gemini, codex)
    pub provider_id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListModelsRequest {
    /// Provider ID (claude, gemini, codex)
    pub provider_id: String,
}

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
        description = "Get full project context: name, statuses, open issues, releases, features, specs, ADRs, recent log, and recent events. Call this at the start of a session to understand the project without being told what exists."
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

        // Agent mode / workflow context
        out.push_str("## Agent Mode\n");
        if let Some(active_id) = config.active_mode.as_deref() {
            if let Some(mode) = config.modes.iter().find(|m| m.id == active_id) {
                out.push_str(&format!("- Active: {} ({})\n", mode.name, mode.id));
            } else {
                out.push_str(&format!(
                    "- Active: {} (not found in mode registry)\n",
                    active_id
                ));
            }
        } else {
            out.push_str("- Active: none\n");
        }
        if !config.modes.is_empty() {
            out.push_str("- Available:\n");
            for mode in &config.modes {
                out.push_str(&format!("  - {} ({})\n", mode.name, mode.id));
            }
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
                out.push_str(&format!(
                    "- [{}] {} ({})\n",
                    f.status, f.feature.metadata.title, f.id
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

    /// Create a new ADR
    #[tool(description = "Create a new Architecture Decision Record")]
    async fn create_adr(&self, Parameters(req): Parameters<CreateAdrRequest>) -> String {
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
        match create_spec(&project_dir, &req.title, content, None, None) {
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
    #[tool(description = "Create a new feature document in the active project")]
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
    #[tool(description = "Update the content of an existing feature")]
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
        description = "Connect (enable) an AI provider for this project by adding it to ship.toml providers list. Provider ID must be one of: claude, gemini, codex"
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
        description = "Disconnect (disable) an AI provider from this project by removing it from ship.toml providers list"
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

#[tool_handler]
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
                "Tools for managing Ship project issues, releases, features, specs, ADRs, \
                 event stream, and time tracking. Call open_project first if the project is \
                 not auto-detected. Use resources (ship://) to read documents without consuming tool calls."
                    .into(),
            ),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<RoleServer>,
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
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<RoleServer>,
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
        _context: rmcp::service::RequestContext<RoleServer>,
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
    eprintln!("Ship MCP Server v0.2.0 starting on stdio...");

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
    use runtime::{EventAction, EventEntity, list_events_since};
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
}
