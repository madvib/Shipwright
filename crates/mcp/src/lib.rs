use anyhow::{Result, anyhow};
use ghost_issues;
use logic::{
    add_status, append_note, create_adr, create_issue, create_spec, delete_issue, get_adr,
    get_config, get_git_config, get_issue, get_project_dir, get_project_name,
    get_project_statuses, get_spec_raw, is_category_committed, list_adrs, list_issues,
    list_issues_full, list_registered_projects, list_specs, log_action, log_action_by, move_issue,
    read_log, register_project, remove_status, set_category_committed, update_spec,
};
use rmcp::transport::stdio;
use rmcp::{
    RoleServer, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CreateMessageRequestParams, Implementation, ProtocolVersion, SamplingMessage,
        ServerCapabilities, ServerInfo,
    },
    service::Peer,
    tool, tool_handler, tool_router,
};
use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;
use std::path::PathBuf;

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
    /// Category to change: issues, adrs, log, config, plugins
    pub category: String,
    /// true = commit to git, false = local only (gitignored)
    pub commit: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct AppendNoteRequest {
    /// Issue filename (e.g. "my-feature.md")
    pub file_name: String,
    /// Status folder the issue is in. If omitted, all statuses are searched.
    pub status: Option<String>,
    /// The note text to append (markdown supported)
    pub note: String,
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
pub struct GetAdrRequest {
    /// ADR filename (e.g. "use-postgresql.json")
    pub file_name: String,
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
        get_project_dir(None)
            .map_err(|e| format!("No active project and auto-detection failed: {}", e))
    }

    // ─── Project Tools ────────────────────────────────────────────────────────

    /// List all registered projects
    #[tool(description = "List all registered projects tracked by Ship")]
    fn list_projects(&self) -> String {
        match list_registered_projects() {
            Ok(projects) => {
                if projects.is_empty() {
                    return "No projects registered. Use track_project to add one.".to_string();
                }
                let mut out = String::from("Registered Projects:\n");
                for p in projects {
                    out.push_str(&format!("- {} ({})\n", p.name, p.path.display()));
                }
                out
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Track a new project
    #[tool(description = "Start tracking a new project with Ship")]
    fn track_project(&self, Parameters(req): Parameters<TrackProjectRequest>) -> String {
        match register_project(req.name.clone(), PathBuf::from(req.path)) {
            Ok(_) => format!("Now tracking project: {}", req.name),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Set the active project for subsequent commands
    #[tool(description = "Set the active project for subsequent MCP tool calls")]
    async fn open_project(&self, Parameters(req): Parameters<OpenProjectRequest>) -> String {
        let path = PathBuf::from(&req.path);
        match get_project_dir(Some(path.clone())) {
            Ok(ship_dir) => {
                let mut active = self.active_project.lock().await;
                *active = Some(ship_dir.clone());
                format!("Opened project at {}", ship_dir.display())
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Get stats for the active project
    #[tool(description = "Get an overview of the active project: issue counts by status, ADR count, project name")]
    async fn get_project_stats(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let name = get_project_name(&project_dir);
        match list_issues(project_dir.clone()) {
            Ok(issues) => {
                let mut counts: std::collections::HashMap<String, u32> =
                    std::collections::HashMap::new();
                for (_, status) in &issues {
                    *counts.entry(status.clone()).or_insert(0) += 1;
                }
                let statuses =
                    get_project_statuses(Some(project_dir.clone())).unwrap_or_default();
                let adrs = list_adrs(project_dir).map(|a| a.len()).unwrap_or(0);
                let mut out = format!("Project: {}\n", name);
                out.push_str(&format!("Total issues: {}\n", issues.len()));
                for status in &statuses {
                    out.push_str(&format!(
                        "  {}: {}\n",
                        status,
                        counts.get(status).unwrap_or(&0)
                    ));
                }
                out.push_str(&format!("ADRs: {}\n", adrs));
                out
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Get full project context for an agent starting a new session
    #[tool(description = "Get full project context: name, statuses, open issues, specs, ADRs, and recent log. Call this at the start of a session to understand the project without being told what exists.")]
    async fn get_project_info(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };

        let name = get_project_name(&project_dir);
        let config = get_config(Some(project_dir.clone())).unwrap_or_default();
        let statuses: Vec<String> = config.statuses.iter().map(|s| s.id.clone()).collect();

        let issues = list_issues_full(project_dir.clone()).unwrap_or_default();
        let specs = list_specs(project_dir.clone()).unwrap_or_default();
        let adrs = list_adrs(project_dir.clone()).unwrap_or_default();

        let mut out = format!("# Project: {}\n\n", name);

        // Issue summary
        out.push_str("## Open Issues\n");
        let open: Vec<_> = issues.iter().filter(|e| e.status != "done").collect();
        if open.is_empty() {
            out.push_str("No open issues.\n");
        } else {
            for status in &statuses {
                let in_status: Vec<_> = open.iter().filter(|e| &e.status == status).collect();
                if !in_status.is_empty() {
                    out.push_str(&format!("\n### {}\n", status));
                    for e in in_status {
                        out.push_str(&format!("- {} ({})\n", e.issue.metadata.title, e.file_name));
                    }
                }
            }
        }

        // Specs
        out.push_str("\n## Specs\n");
        if specs.is_empty() {
            out.push_str("No specs.\n");
        } else {
            for s in &specs {
                out.push_str(&format!("- {} ({})\n", s.title, s.file_name));
            }
        }

        // ADRs
        out.push_str("\n## ADRs\n");
        if adrs.is_empty() {
            out.push_str("No ADRs.\n");
        } else {
            for a in &adrs {
                out.push_str(&format!("- [{}] {} ({})\n", a.adr.metadata.status, a.adr.metadata.title, a.file_name));
            }
        }

        // Recent log (last 10 lines)
        if let Ok(log) = read_log(project_dir.clone()) {
            let recent: Vec<&str> = log.lines()
                .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
                .rev().take(10).collect::<Vec<_>>().into_iter().rev().collect();
            if !recent.is_empty() {
                out.push_str("\n## Recent Activity\n");
                for line in recent {
                    out.push_str(&format!("{}\n", line));
                }
            }
        }

        out
    }

    // ─── Issue Tools ──────────────────────────────────────────────────────────

    /// List all issues in the project
    #[tool(description = "List all issues in the active project with their statuses")]
    async fn list_issues(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match list_issues_full(project_dir) {
            Ok(entries) => {
                if entries.is_empty() {
                    return "No issues found.".to_string();
                }
                let mut out = String::from("Issues:\n");
                for e in entries {
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

    /// Get a specific issue by filename
    #[tool(description = "Get the full content of a specific issue by filename")]
    async fn get_issue(&self, Parameters(req): Parameters<GetIssueRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };

        // Search through provided status or all configured statuses
        let configured = get_project_statuses(Some(project_dir.clone())).unwrap_or_default();
        let statuses: Vec<String> = if let Some(s) = req.status {
            vec![s]
        } else {
            configured
        };

        for status in &statuses {
            let path = project_dir
                .join("issues")
                .join(status)
                .join(&req.file_name);
            if path.exists() {
                return match get_issue(path) {
                    Ok(issue) => format!(
                        "Title: {}\nStatus: {}\nCreated: {}\nUpdated: {}\n\n{}",
                        issue.metadata.title,
                        status,
                        issue.metadata.created.format("%Y-%m-%d %H:%M"),
                        issue.metadata.updated.format("%Y-%m-%d %H:%M"),
                        issue.description
                    ),
                    Err(e) => format!("Error reading issue: {}", e),
                };
            }
        }
        format!("Issue not found: {}", req.file_name)
    }

    /// Create a new issue
    #[tool(description = "Create a new issue in the active project")]
    async fn create_issue(&self, Parameters(req): Parameters<CreateIssueRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let status = req.status.as_deref().unwrap_or("backlog");
        match create_issue(project_dir.clone(), &req.title, &req.description, status) {
            Ok(file) => {
                let fname = file.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                log_action_by(project_dir, "agent", "issue create", &format!("{} ({})", fname, status)).ok();
                format!("Created issue: {} ({})", fname, status)
            }
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
        let path = project_dir
            .join("issues")
            .join(&req.status)
            .join(&req.file_name);
        match get_issue(path.clone()) {
            Ok(mut issue) => {
                if let Some(title) = req.title {
                    issue.metadata.title = title;
                }
                if let Some(desc) = req.description {
                    issue.description = desc;
                }
                match logic::update_issue(path, issue) {
                    Ok(_) => format!("Updated: {}", req.file_name),
                    Err(e) => format!("Error: {}", e),
                }
            }
            Err(e) => format!("Error reading issue: {}", e),
        }
    }

    /// Append a note to an issue without rewriting it
    #[tool(description = "Append a note or implementation summary to an issue. Use this when closing work to record what changed.")]
    async fn append_to_issue(&self, Parameters(req): Parameters<AppendNoteRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let configured = get_project_statuses(Some(project_dir.clone())).unwrap_or_default();
        let statuses: Vec<String> = if let Some(s) = req.status {
            vec![s]
        } else {
            configured
        };
        for status in &statuses {
            let path = project_dir
                .join("issues")
                .join(status)
                .join(&req.file_name);
            if path.exists() {
                return match append_note(path, &req.note) {
                    Ok(_) => format!("Note appended to {}", req.file_name),
                    Err(e) => format!("Error: {}", e),
                };
            }
        }
        format!("Issue not found: {}", req.file_name)
    }

    /// Move an issue to a different status
    #[tool(description = "Move an issue from one status to another (e.g. backlog → in-progress)")]
    async fn move_issue(&self, Parameters(req): Parameters<MoveIssueRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let path = project_dir
            .join("issues")
            .join(&req.from_status)
            .join(&req.file_name);
        match move_issue(project_dir.clone(), path, &req.from_status, &req.to_status) {
            Ok(_) => {
                log_action_by(project_dir, "agent", "issue move",
                    &format!("{}: {} → {}", req.file_name, req.from_status, req.to_status)).ok();
                format!("{}: {} → {}", req.file_name, req.from_status, req.to_status)
            }
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
        let path = project_dir
            .join("issues")
            .join(&req.status)
            .join(&req.file_name);
        match delete_issue(path) {
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
        match list_issues_full(project_dir) {
            Ok(entries) => {
                let query = req.query.to_lowercase();
                let matches: Vec<_> = entries
                    .into_iter()
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

    /// List all ADRs
    #[tool(description = "List all Architecture Decision Records in the active project")]
    async fn list_adrs(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match list_adrs(project_dir) {
            Ok(adrs) => {
                if adrs.is_empty() {
                    return "No ADRs found.".to_string();
                }
                let mut out = String::from("ADRs:\n");
                for a in adrs {
                    out.push_str(&format!(
                        "- [{}] {} ({})\n",
                        a.adr.metadata.status, a.adr.metadata.title, a.file_name
                    ));
                }
                out
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Create a new ADR
    #[tool(description = "Create a new Architecture Decision Record")]
    async fn create_adr(&self, Parameters(req): Parameters<CreateAdrRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match create_adr(project_dir, &req.title, &req.decision, "accepted") {
            Ok(file) => format!(
                "Created ADR: {}",
                file.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
            ),
            Err(e) => format!("Error: {}", e),
        }
    }

    // ─── Spec Tools ───────────────────────────────────────────────────────────

    /// List all specs
    #[tool(description = "List all specs in the active project")]
    async fn list_specs(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match list_specs(project_dir) {
            Ok(specs) => {
                if specs.is_empty() {
                    return "No specs found.".to_string();
                }
                let mut out = String::from("Specs:\n");
                for s in specs {
                    out.push_str(&format!("- {} ({})\n", s.title, s.file_name));
                }
                out
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Get the full content of a spec
    #[tool(description = "Get the full markdown content of a spec by filename")]
    async fn get_spec(&self, Parameters(req): Parameters<GetSpecRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let path = project_dir.join("specs").join(&req.file_name);
        match get_spec_raw(path) {
            Ok(content) => content,
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Create a new spec
    #[tool(description = "Create a new spec document in the active project")]
    async fn create_spec(&self, Parameters(req): Parameters<CreateSpecRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let content = req.content.as_deref().unwrap_or("");
        match create_spec(project_dir, &req.title, content) {
            Ok(file) => format!(
                "Created spec: {}",
                file.file_name().and_then(|n| n.to_str()).unwrap_or("unknown")
            ),
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
        let path = project_dir.join("specs").join(&req.file_name);
        match update_spec(path, &req.content) {
            Ok(_) => format!("Updated spec: {}", req.file_name),
            Err(e) => format!("Error: {}", e),
        }
    }

    // ─── ADR / Log Tools ──────────────────────────────────────────────────────

    /// Get the full content of an ADR
    #[tool(description = "Get the full content of a specific ADR by filename")]
    async fn get_adr(&self, Parameters(req): Parameters<GetAdrRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let path = project_dir.join("adrs").join(&req.file_name);
        match get_adr(path) {
            Ok(adr) => format!(
                "Title: {}\nStatus: {}\nDate: {}\n\n{}",
                adr.metadata.title,
                adr.metadata.status,
                adr.metadata.date,
                adr.body,
            ),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Get recent project log entries
    #[tool(description = "Get the recent action log for the active project")]
    async fn get_log(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match read_log(project_dir) {
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

    // ─── Ghost Issues Tools ───────────────────────────────────────────────────

    /// Scan the codebase for TODO/FIXME/HACK/BUG comments
    #[tool(description = "Scan the project codebase for TODO, FIXME, HACK, and BUG comments and return a summary")]
    async fn ghost_scan(&self, Parameters(req): Parameters<GhostScanRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
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

    /// Get the ghost issues report from the last scan
    #[tool(description = "Get the Markdown ghost issues report from the last scan")]
    async fn ghost_report(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match ghost_issues::generate_report(&project_dir) {
            Ok(r) => r,
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Promote a ghost issue to a real tracked issue
    #[tool(description = "Promote a ghost issue (TODO/FIXME comment) to a real tracked issue in the backlog")]
    async fn ghost_promote(&self, Parameters(req): Parameters<GhostPromoteRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
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
                        match create_issue(project_dir.clone(), &title, &desc, "backlog") {
                            Ok(path) => {
                                log_action(
                                    project_dir,
                                    "issue create",
                                    &format!("Ghost promoted: {}", title),
                                )
                                .ok();
                                format!(
                                    "Created issue: {}",
                                    path.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("unknown")
                                )
                            }
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

    /// List configured issue statuses/categories for the active project
    #[tool(description = "List the configured issue statuses (categories) for the active project")]
    async fn list_statuses(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match get_project_statuses(Some(project_dir)) {
            Ok(statuses) => {
                let mut out = String::from("Issue statuses:\n");
                for s in statuses {
                    out.push_str(&format!("- {}\n", s));
                }
                out
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Add a new issue status/category
    #[tool(description = "Add a new issue status/category to the project")]
    async fn add_status(&self, Parameters(req): Parameters<StatusNameRequest>) -> String {
        let project_dir = self.get_effective_project_dir().await.ok();
        match add_status(project_dir, &req.name) {
            Ok(_) => format!("Added status: {}", req.name.to_lowercase().replace(' ', "-")),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Remove an issue status/category
    #[tool(description = "Remove an issue status/category from the project (existing issues are not deleted)")]
    async fn remove_status(&self, Parameters(req): Parameters<StatusNameRequest>) -> String {
        let project_dir = self.get_effective_project_dir().await.ok();
        match remove_status(project_dir, &req.name) {
            Ok(_) => format!("Removed status: {}", req.name),
            Err(e) => format!("Error: {}", e),
        }
    }

    // ─── AI Generation Tools ─────────────────────────────────────────────────

    /// Generate a detailed issue description from a title using AI
    #[tool(description = "Generate a detailed, actionable issue description from a title. Uses MCP sampling (Claude Code) or direct Anthropic API.")]
    async fn generate_issue_description(
        &self,
        peer: Peer<RoleServer>,
        Parameters(req): Parameters<GenerateIssueRequest>,
    ) -> String {
        let system = "You are a project management assistant. Generate clear, concise, actionable issue descriptions in markdown. Include: what needs to be done, why it matters, and acceptance criteria. Be specific but not verbose. 2-4 paragraphs max.";
        let prompt = match &req.context {
            Some(ctx) => format!("Generate an issue description for:\n\nTitle: {}\n\nContext: {}", req.title, ctx),
            None => format!("Generate an issue description for:\n\nTitle: {}", req.title),
        };
        self.generate_with_sampling(peer, system, &prompt, 800).await
    }

    /// Generate an Architecture Decision Record from a problem statement
    #[tool(description = "Generate an ADR (Architecture Decision Record) from a problem statement using AI")]
    async fn generate_adr(
        &self,
        peer: Peer<RoleServer>,
        Parameters(req): Parameters<GenerateAdrRequest>,
    ) -> String {
        let system = "You are a software architect. Generate a concise Architecture Decision Record. Format: state the context, decision, and consequences. Be direct and practical. Use markdown.";
        let prompt = match &req.constraints {
            Some(c) => format!("Generate an ADR for:\n\nProblem: {}\n\nConstraints/Options: {}", req.problem, c),
            None => format!("Generate an ADR for:\n\nProblem: {}", req.problem),
        };
        self.generate_with_sampling(peer, system, &prompt, 1000).await
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
        self.generate_with_sampling(peer, system, &prompt, 600).await
    }

    // ─── Git Config Tools ─────────────────────────────────────────────────────

    /// Get the current git commit settings for the active project
    #[tool(description = "Get which Ship data categories are committed to git vs kept local")]
    async fn git_config_get(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match get_git_config(&project_dir) {
            Ok(git) => {
                let cats = ["issues", "adrs", "specs", "log.md", "config.toml", "templates", "plugins"];
                let mut out = String::from("Git commit settings:\n");
                for cat in cats {
                    let state = if is_category_committed(&git, cat) { "committed" } else { "local only" };
                    out.push_str(&format!("  {:<12} {}\n", cat, state));
                }
                out
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Update git commit settings for the active project
    #[tool(description = "Set whether a category (issues/adrs/log/config/plugins) is committed to git or kept local. Updates .ship/.gitignore automatically.")]
    async fn git_config_set(&self, Parameters(req): Parameters<GitIncludeRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        let known = ["issues", "adrs", "specs", "log.md", "config.toml", "templates", "plugins"];
        if !known.contains(&req.category.as_str()) {
            return format!("Unknown category '{}'. Use: {}", req.category, known.join(", "));
        }
        match set_category_committed(&project_dir, &req.category, req.commit) {
            Ok(_) => format!("{} is now {}", req.category, if req.commit { "committed to git" } else { "local only" }),
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
        // Try to resolve title from issue file
        let issue_title = {
            let mut title = req.issue_file.clone();
            let statuses =
                get_project_statuses(Some(project_dir.clone())).unwrap_or_default();
            for status in &statuses {
                let p = project_dir
                    .join("issues")
                    .join(status)
                    .join(&req.issue_file);
                if p.exists() {
                    if let Ok(issue) = get_issue(p) {
                        title = issue.metadata.title;
                    }
                    break;
                }
            }
            title
        };
        match time_tracker::start_timer(&project_dir, &req.issue_file, &issue_title, req.note) {
            Ok(t) => format!("Timer started: {} at {}", t.issue_title, t.started_at.format("%H:%M")),
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
        match time_tracker::stop_timer(&project_dir, req.note) {
            Ok(e) => format!(
                "Timer stopped: {} — {}",
                e.issue_title,
                time_tracker::format_duration(e.duration_minutes)
            ),
            Err(err) => format!("Error: {}", err),
        }
    }

    /// Get the active timer status
    #[tool(description = "Check if a time tracking timer is currently running")]
    async fn time_status(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match time_tracker::get_active_timer(&project_dir) {
            Ok(Some(t)) => {
                let elapsed = (chrono::Utc::now() - t.started_at)
                    .num_minutes()
                    .max(0) as u64;
                format!(
                    "Running: {} (started {}, elapsed {})",
                    t.issue_title,
                    t.started_at.format("%H:%M"),
                    time_tracker::format_duration(elapsed)
                )
            }
            Ok(None) => "No timer running.".to_string(),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Generate a time report
    #[tool(description = "Generate a Markdown time report for the active project")]
    async fn time_report(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(d) => d,
            Err(e) => return e,
        };
        match time_tracker::generate_report(&project_dir) {
            Ok(report) => report,
            Err(e) => format!("Error: {}", e),
        }
    }
}

impl ShipServer {
    /// Generate text via MCP sampling (peer.create_message) if supported,
    /// falling back to direct Anthropic API call if available, otherwise error.
    async fn generate_with_sampling(
        &self,
        peer: Peer<RoleServer>,
        system: &str,
        prompt: &str,
        max_tokens: u32,
    ) -> String {
        // Try MCP sampling first
        if peer.peer_info().map_or(false, |info| info.capabilities.sampling.is_some()) {
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
                Err(e) => {
                    // Fall through to API fallback
                    eprintln!("[ship:ai] Sampling failed: {}, trying API fallback", e);
                }
            }
        }

        // Fallback: direct Anthropic API via config
        let project_dir = self.get_effective_project_dir().await.ok();
        let ai_config = project_dir
            .as_deref()
            .and_then(|d| get_config(Some(d.to_path_buf())).ok())
            .and_then(|c| c.ai)
            .unwrap_or_default();

        match ai_config.resolve_api_key() {
            Some(key) => call_anthropic_api(&key, ai_config.effective_model(), system, prompt, max_tokens).await,
            None => "AI generation unavailable: MCP sampling not supported by client and no ANTHROPIC_API_KEY configured. Set it in global config or as an environment variable.".to_string(),
        }
    }
}

async fn call_anthropic_api(
    api_key: &str,
    model: &str,
    system: &str,
    prompt: &str,
    max_tokens: u32,
) -> String {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": model,
        "max_tokens": max_tokens,
        "system": system,
        "messages": [{"role": "user", "content": prompt}]
    });

    match client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => json["content"][0]["text"]
                        .as_str()
                        .unwrap_or("Empty response")
                        .to_string(),
                    Err(e) => format!("Failed to parse API response: {}", e),
                }
            } else {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                format!("Anthropic API error {}: {}", status, body)
            }
        }
        Err(e) => format!("HTTP request failed: {}", e),
    }
}

#[tool_handler]
impl ServerHandler for ShipServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "Ship Project Tracker".into(),
                version: "0.2.0".into(),
                ..Default::default()
            },
            instructions: Some(
                "Tools for managing Ship project issues, ADRs, and time tracking. \
                 Call open_project first if the project is not auto-detected."
                    .into(),
            ),
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
