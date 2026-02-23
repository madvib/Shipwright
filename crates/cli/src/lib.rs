use anyhow::Result;
use clap::{Parser, Subcommand};
use logic::{
    add_status, append_note, create_adr, create_issue, get_git_config, get_issue, get_project_dir,
    get_project_statuses, init_demo_project, init_project, is_category_committed, list_issues,
    log_action, migrate_json_config_file, migrate_yaml_issues, move_issue, remove_status,
    set_category_committed,
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
    Init {
        /// Directory to initialize (defaults to current directory)
        path: Option<PathBuf>,
    },
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
    /// Initialize a demo project with sample data (safe for testing)
    Demo {
        /// Directory to initialize (defaults to ./ship-demo)
        #[arg(default_value = "./ship-demo")]
        path: PathBuf,
    },
    /// Manage git commit settings for ship data
    Git {
        #[command(subcommand)]
        action: GitCommands,
    },
    /// Scan codebase for TODO/FIXME/HACK/BUG comments
    Ghost {
        #[command(subcommand)]
        action: GhostCommands,
    },
    /// Manage project configuration
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
    /// Track time spent on issues
    Time {
        #[command(subcommand)]
        action: TimeCommands,
    },
    /// Start the MCP server on stdio
    Mcp,
    /// Migrate legacy YAML issues and JSON config to TOML
    Migrate,
}

#[derive(Subcommand, Debug)]
pub enum GitCommands {
    /// Show what is and isn't committed to git
    Status,
    /// Include a category in git commits
    Include {
        /// One of: issues, adrs, log, config, plugins
        category: String,
    },
    /// Exclude a category from git commits (adds to .ship/.gitignore)
    Exclude {
        /// One of: issues, adrs, log, config, plugins
        category: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum GhostCommands {
    /// Scan the project root for ghost issues
    Scan {
        /// Directory to scan (defaults to project root)
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
    /// Show the report from the last scan
    Report,
    /// Promote a ghost issue to a real issue
    Promote {
        /// File path where the comment lives
        file: String,
        /// Line number of the comment
        line: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Manage issue statuses/categories
    Status {
        #[command(subcommand)]
        action: StatusCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum StatusCommands {
    /// List configured issue statuses
    List,
    /// Add a new status
    Add { name: String },
    /// Remove a status (does not delete existing issues)
    Remove { name: String },
}

#[derive(Subcommand, Debug)]
pub enum TimeCommands {
    /// Start a timer for an issue (provide the issue filename)
    Start {
        /// Issue filename (e.g. my-feature.md) or path
        issue: String,
        /// Optional note for this session
        #[arg(short, long)]
        note: Option<String>,
    },
    /// Stop the currently running timer
    Stop {
        /// Optional note to attach to the completed entry
        #[arg(short, long)]
        note: Option<String>,
    },
    /// Show the currently running timer
    Status,
    /// Manually log time for an issue
    Log {
        /// Issue filename
        issue: String,
        /// Duration in minutes
        minutes: u64,
        /// Optional note
        #[arg(short, long)]
        note: Option<String>,
    },
    /// List time entries (optionally filtered to an issue)
    List {
        /// Filter to a specific issue filename
        issue: Option<String>,
    },
    /// Generate a time report for the current project
    Report,
}

#[derive(Subcommand, Debug)]
pub enum IssueCommands {
    /// Create a new issue
    Create { title: String, description: String },
    /// List all issues
    List,
    /// Append a note to an issue (useful for implementation summaries)
    Note {
        /// Issue filename (e.g. my-feature.md)
        file_name: String,
        /// Status folder the issue is in (default: searches all)
        #[arg(short, long)]
        status: Option<String>,
        /// The note text to append
        note: String,
    },
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
        Some(Commands::Init { path: init_path }) => {
            let target = match init_path {
                Some(p) => std::fs::canonicalize(&p).unwrap_or_else(|_| env::current_dir().unwrap_or_default().join(&p)),
                None => env::current_dir()?,
            };
            let project_name = target
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "project".to_string());
            let ship_path = init_project(target.clone())?;
            logic::register_project(project_name, target)?;
            println!("Initialized and tracked Ship project in {}", ship_path.display());
            log_action(ship_path, "init", "Project initialized")?;
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
                IssueCommands::Note { file_name, status, note } => {
                    let statuses = get_project_statuses(Some(project_dir.clone()))?;
                    let search: Vec<String> = status
                        .map(|s| vec![s])
                        .unwrap_or(statuses);
                    let path = search.iter().find_map(|s| {
                        let p = project_dir.join("issues").join(s).join(&file_name);
                        p.exists().then_some(p)
                    });
                    match path {
                        Some(p) => {
                            append_note(p, &note)?;
                            println!("Note appended to {}", file_name);
                        }
                        None => eprintln!("Issue not found: {}", file_name),
                    }
                }
                IssueCommands::Move {
                    file_name,
                    from,
                    to,
                } => {
                    let issue_path = project_dir.join("issues").join(&from).join(&file_name);
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
        Some(Commands::Demo { path }) => {
            let abs = std::fs::canonicalize(&path).unwrap_or_else(|_| {
                env::current_dir().unwrap_or_default().join(&path)
            });
            let project_dir = init_demo_project(abs.clone())?;
            println!("Demo project ready at {}", project_dir.display());
            println!("Point Ship at it with: SHIP_DIR={} ship issue list", project_dir.display());
        }
        Some(Commands::Git { action }) => {
            let project_dir = get_project_dir(None)?;
            let git = get_git_config(&project_dir)?;
            match action {
                GitCommands::Status => {
                    let cats = ["issues", "adrs", "specs", "log.md", "config.toml", "templates", "plugins"];
                    println!("Ship git commit settings:");
                    for cat in cats {
                        let state = if is_category_committed(&git, cat) { "committed" } else { "local only" };
                        println!("  {:<14} {}", cat, state);
                    }
                    println!("\n.gitignore: {}", project_dir.join(".gitignore").display());
                }
                GitCommands::Include { category } => {
                    set_category_committed(&project_dir, &category, true)?;
                    println!("{} will now be committed to git.", category);
                    println!(".ship/.gitignore updated.");
                }
                GitCommands::Exclude { category } => {
                    set_category_committed(&project_dir, &category, false)?;
                    println!("{} will now be local only (gitignored).", category);
                    println!(".ship/.gitignore updated.");
                }
            }
        }
        Some(Commands::Ghost { action }) => {
            let project_dir = get_project_dir(None)?;
            match action {
                GhostCommands::Scan { dir } => {
                    let root = dir.unwrap_or_else(|| {
                        // project_dir is .ship/; go up one level to the repo root
                        project_dir.parent().unwrap_or(&project_dir).to_path_buf()
                    });
                    println!("Scanning {}...", root.display());
                    let result = ghost_issues::scan(&project_dir, &root)?;
                    let unpromoted = result.issues.iter().filter(|g| !g.promoted).count();
                    println!(
                        "Found {} ghost issue{} in {} file{}.",
                        unpromoted,
                        if unpromoted == 1 { "" } else { "s" },
                        {
                            let files: std::collections::HashSet<_> =
                                result.issues.iter().map(|g| &g.file).collect();
                            files.len()
                        },
                        if result.issues.len() == 1 { "" } else { "s" }
                    );
                    for g in result.issues.iter().filter(|g| !g.promoted).take(10) {
                        println!("  {}", g.display());
                    }
                    if unpromoted > 10 {
                        println!("  ... and {} more. Run `ship ghost report` for full list.", unpromoted - 10);
                    }
                }
                GhostCommands::Report => {
                    let report = ghost_issues::generate_report(&project_dir)?;
                    println!("{}", report);
                }
                GhostCommands::Promote { file, line } => {
                    let found = ghost_issues::mark_promoted(&project_dir, &file, line)?;
                    if found {
                        println!("Marked {}:{} as promoted.", file, line);
                        // Optionally create an issue
                        if let Ok(Some(scan)) = ghost_issues::load_last_scan(&project_dir) {
                            if let Some(g) = scan.issues.iter().find(|g| g.file == file && g.line == line) {
                                let title = g.suggested_title();
                                let desc = format!(
                                    "Promoted from `{}:{}` ({}).\n\nOriginal comment: {}",
                                    g.file, g.line, g.kind.as_str(), g.text.trim()
                                );
                                let path = create_issue(project_dir.clone(), &title, &desc, "backlog")?;
                                println!("Created issue: {}", path.display());
                                log_action(project_dir, "issue create", &format!("Ghost promoted: {}", title))?;
                            }
                        }
                    } else {
                        println!("Ghost issue not found at {}:{}. Run `ship ghost scan` first.", file, line);
                    }
                }
            }
        }
        Some(Commands::Config { action }) => {
            let project_dir = get_project_dir(None).ok();
            match action {
                ConfigCommands::Status { action } => match action {
                    StatusCommands::List => {
                        let statuses = get_project_statuses(project_dir)?;
                        println!("Issue statuses:");
                        for s in statuses {
                            println!("  - {}", s);
                        }
                    }
                    StatusCommands::Add { name } => {
                        add_status(project_dir, &name)?;
                        println!("Added status: {}", name.to_lowercase().replace(' ', "-"));
                    }
                    StatusCommands::Remove { name } => {
                        remove_status(project_dir, &name)?;
                        println!("Removed status: {}", name);
                    }
                },
            }
        }
        Some(Commands::Time { action }) => {
            let project_dir = get_project_dir(None)?;
            handle_time_command(action, &project_dir)?;
        }
        Some(Commands::Mcp) => {
            // Handled by the main unitary binary as it requires async
        }
        Some(Commands::Migrate) => {
            let project_dir = get_project_dir(None)?;
            let issues = migrate_yaml_issues(&project_dir)?;
            let config = migrate_json_config_file(&project_dir)?;
            println!(
                "Migration complete: {} issue{} converted to TOML{}.",
                issues,
                if issues == 1 { "" } else { "s" },
                if config { ", config.json → config.toml" } else { "" },
            );
        }
        None => {
            // This case should be handled by the caller to decide whether to show help or launch GUI
        }
    }

    Ok(())
}


fn handle_time_command(action: TimeCommands, project_dir: &PathBuf) -> Result<()> {
    use time_tracker::{format_duration, generate_report, get_active_timer, list_entries, log_time, start_timer, stop_timer};

    match action {
        TimeCommands::Start { issue, note } => {
            // issue can be a filename or a path — normalise to just the filename
            let issue_path = PathBuf::from(&issue);
            let issue_file = issue_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(issue.clone());

            // Try to read the issue title from the file
            let issue_title = {
                let path = if issue_path.is_absolute() {
                    issue_path.clone()
                } else {
                    // Search through statuses
                    let mut found = None;
                    for status in logic::ISSUE_STATUSES {
                        let p = project_dir.join("issues").join(status).join(&issue_file);
                        if p.exists() {
                            found = Some(p);
                            break;
                        }
                    }
                    found.unwrap_or(issue_path)
                };
                if path.exists() {
                    get_issue(path)
                        .ok()
                        .map(|i| i.metadata.title)
                        .unwrap_or_else(|| issue_file.clone())
                } else {
                    issue_file.clone()
                }
            };

            let timer = start_timer(project_dir, &issue_file, &issue_title, note)?;
            println!(
                "Timer started: {} ({})",
                timer.issue_title,
                timer.started_at.format("%H:%M")
            );
        }
        TimeCommands::Stop { note } => {
            let entry = stop_timer(project_dir, note)?;
            println!(
                "Timer stopped: {} — {}",
                entry.issue_title,
                format_duration(entry.duration_minutes)
            );
        }
        TimeCommands::Status => {
            match get_active_timer(project_dir)? {
                Some(t) => {
                    let elapsed = (chrono::Utc::now() - t.started_at)
                        .num_minutes()
                        .max(0) as u64;
                    println!(
                        "Running: {} (started {}, elapsed {})",
                        t.issue_title,
                        t.started_at.format("%H:%M"),
                        format_duration(elapsed)
                    );
                }
                None => println!("No timer running."),
            }
        }
        TimeCommands::Log {
            issue,
            minutes,
            note,
        } => {
            let issue_path = PathBuf::from(&issue);
            let issue_file = issue_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(issue.clone());
            let entry = log_time(project_dir, &issue_file, &issue_file, minutes, note)?;
            println!(
                "Logged {} for {}",
                format_duration(entry.duration_minutes),
                entry.issue_title
            );
        }
        TimeCommands::List { issue } => {
            let entries = list_entries(project_dir, issue.as_deref())?;
            if entries.is_empty() {
                println!("No time entries.");
            } else {
                for e in &entries {
                    println!(
                        "[{}] {} — {}{}",
                        e.started_at.format("%Y-%m-%d"),
                        e.issue_title,
                        format_duration(e.duration_minutes),
                        e.note
                            .as_deref()
                            .map(|n| format!(" ({})", n))
                            .unwrap_or_default()
                    );
                }
            }
        }
        TimeCommands::Report => {
            let report = generate_report(project_dir)?;
            println!("{}", report);
        }
    }
    Ok(())
}
