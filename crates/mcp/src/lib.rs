use anyhow::{Result, anyhow};
use rmcp::transport::stdio;
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use serde::Deserialize;
use std::path::PathBuf;
// Use schemars via rmcp to avoid version mismatch
use logic::{create_issue, get_project_dir, list_issues, register_project};
use rmcp::schemars::{self, JsonSchema};

#[derive(Deserialize, JsonSchema)]
pub struct CreateIssueRequest {
    /// The title of the issue
    pub title: String,
    /// The detailed description of the issue
    pub description: String,
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
            let path: PathBuf = path.clone();
            return Ok(path);
        }

        get_project_dir(None)
            .map_err(|e| format!("No active project and auto-detection failed: {}", e))
    }

    /// List all registered projects
    #[tool(description = "List all registered projects tracked by Ship")]
    fn list_projects(&self) -> String {
        match logic::list_registered_projects() {
            Ok(projects) => {
                if projects.is_empty() {
                    return "No projects registered. Use track_project to add one.".to_string();
                }
                let mut output = String::from("Registered Projects:\n");
                for p in projects {
                    output.push_str(&format!("- {} ({})\n", p.name, p.path.display()));
                }
                output
            }
            Err(e) => format!("Error listing projects: {}", e),
        }
    }

    /// Track a new project
    #[tool(description = "Start tracking a new project with Ship")]
    fn track_project(&self, Parameters(req): Parameters<TrackProjectRequest>) -> String {
        match register_project(req.name.clone(), PathBuf::from(req.path)) {
            Ok(_) => format!("Now tracking project: {}", req.name),
            Err(e) => format!("Error tracking project: {}", e),
        }
    }

    /// Set the active project for subsequent commands
    #[tool(description = "Set the active project for subsequent commands")]
    async fn open_project(&self, Parameters(req): Parameters<OpenProjectRequest>) -> String {
        let path = PathBuf::from(&req.path);
        match get_project_dir(Some(path.clone())) {
            Ok(ship_dir) => {
                let mut active = self.active_project.lock().await;
                *active = Some(ship_dir.clone());
                format!("Opened project at {}", ship_dir.display())
            }
            Err(e) => format!("Error opening project: {}", e),
        }
    }

    #[tool(description = "List all issues in the project")]
    async fn list_issues(&self) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(dir) => dir,
            Err(e) => return e,
        };

        match list_issues(project_dir) {
            Ok(issues) => {
                let mut output = String::from("Issues:\n");
                for (file_name, status) in issues {
                    output.push_str(&format!("- [{}] {}\n", status, file_name));
                }
                output
            }
            Err(e) => format!("Error listing issues: {}", e),
        }
    }

    /// Create a new issue
    #[tool(description = "Create a new issue")]
    async fn create_issue(&self, Parameters(req): Parameters<CreateIssueRequest>) -> String {
        let project_dir = match self.get_effective_project_dir().await {
            Ok(dir) => dir,
            Err(e) => return e,
        };

        // status defaults to backlog for new issues
        match create_issue(project_dir, &req.title, &req.description, "backlog") {
            Ok(file) => format!("Created issue: {}", file.display()),
            Err(e) => format!("Error creating issue: {}", e),
        }
    }
}

#[tool_handler]
impl ServerHandler for ShipServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "Ship Project Tracker".into(),
                version: "0.1.0".into(),
                ..Default::default()
            },
            instructions: Some("A tool for managing local project issues and ADRs".into()),
        }
    }
}

pub async fn run_server() -> Result<()> {
    let service = ShipServer::new();
    eprintln!("Ship MCP Server starting on stdio...");

    // In rmcp 0.16, serve returns a service that we must wait on
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
