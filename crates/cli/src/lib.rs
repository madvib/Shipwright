use anyhow::Result;
use clap::{Parser, Subcommand};
use logic::{
    create_adr, create_issue, get_project_dir, init_project, list_issues, log_action, move_issue,
};
use std::env;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "ship")]
#[command(about = "A project-aware task and ADR tracker", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new project
    Init,
    /// Manage project issues
    Issue {
        #[command(subcommand)]
        action: IssueCommands,
    },
    /// Manage architecture decisions
    Adr {
        #[command(subcommand)]
        action: AdrCommands,
    },
    /// Manage tracked projects
    Projects {
        #[command(subcommand)]
        action: ProjectCommands,
    },
    /// Start the MCP server on stdio
    Mcp,
}

#[derive(Subcommand, Debug)]
pub enum IssueCommands {
    /// Create a new issue
    Create { title: String, description: String },
    /// List all issues
    List,
    /// Move an issue to a new status
    Move {
        file_name: String,
        from: String,
        to: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum AdrCommands {
    /// Create a new ADR
    Create { title: String, decision: String },
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    /// List all tracked projects
    List,
    /// Start tracking a project
    Track { name: String, path: PathBuf },
    /// Stop tracking a project
    Untrack { path: PathBuf },
}

pub fn handle_cli(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Init) => {
            let current_dir = env::current_dir()?;
            let path = init_project(current_dir.clone())?;
            let project_name = current_dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "New Project".to_string());

            logic::register_project(project_name, current_dir)?;
            println!("Initialized and tracked Ship project in {}", path.display());
            log_action(path, "init", "Project initialized")?;
        }
        Some(Commands::Issue { action }) => {
            let project_dir = get_project_dir(None)?;
            match action {
                IssueCommands::Create { title, description } => {
                    let path = create_issue(project_dir.clone(), &title, &description, "backlog")?;
                    println!("Issue created: {}", path.display());
                    log_action(
                        project_dir,
                        "issue create",
                        &format!("Created issue: {}", title),
                    )?;
                }
                IssueCommands::List => {
                    let issues = list_issues(project_dir)?;
                    for (file, status) in issues {
                        println!("[{}] {}", status, file);
                    }
                }
                IssueCommands::Move {
                    file_name,
                    from,
                    to,
                } => {
                    let issue_path = project_dir.join("Issues").join(&from).join(&file_name);
                    move_issue(project_dir.clone(), issue_path, &from, &to)?;
                    println!("Moved {} from {} to {}", file_name, from, to);
                    log_action(
                        project_dir,
                        "issue move",
                        &format!("Moved {} to {}", file_name, to),
                    )?;
                }
            }
        }
        Some(Commands::Adr { action }) => {
            let project_dir = get_project_dir(None)?;
            match action {
                AdrCommands::Create { title, decision } => {
                    let path = create_adr(project_dir.clone(), &title, &decision, "accepted")?;
                    println!("ADR created: {}", path.display());
                    log_action(
                        project_dir,
                        "adr create",
                        &format!("Created ADR: {}", title),
                    )?;
                }
            }
        }
        Some(Commands::Projects { action }) => match action {
            ProjectCommands::List => {
                let projects = logic::list_registered_projects()?;
                for p in projects {
                    println!("- {} ({})", p.name, p.path.display());
                }
            }
            ProjectCommands::Track { name, path } => {
                logic::register_project(name.clone(), path.clone())?;
                println!("Now tracking project: {} ({})", name, path.display());
            }
            ProjectCommands::Untrack { path } => {
                logic::unregister_project(path.clone())?;
                println!("Stopped tracking project: {}", path.display());
            }
        },
        Some(Commands::Mcp) => {
            // Handled by the main unitary binary as it requires async
        }
        None => {
            // This case should be handled by the caller to decide whether to show help or launch GUI
        }
    }

    Ok(())
}
