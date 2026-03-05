#![allow(dead_code)]
use anyhow::Result;
use clap::{Parser, Subcommand};
use runtime::project::{get_global_dir, get_project_dir};
use runtime::{
    CreateWorkspaceRequest, McpServerConfig, McpServerType, WorkspaceStatus, WorkspaceType,
    activate_workspace, add_mcp_server, add_mode, add_status, autodetect_providers, create_skill,
    create_user_skill, create_workspace, delete_skill, delete_user_skill, disable_provider,
    enable_provider, get_active_mode, get_config, get_effective_skill, get_git_config,
    get_project_statuses, get_skill, get_user_skill, ingest_external_events, is_category_committed,
    list_effective_skills, list_events_since, list_mcp_servers, list_models, list_providers,
    list_skills, list_user_skills, list_workspaces, log_action, migrate_global_state,
    migrate_json_config_file, migrate_project_state, remove_mcp_server, remove_mode, remove_status,
    set_active_mode, set_category_committed, sync_workspace, transition_workspace_status,
    update_skill, update_user_skill,
};
use ship_module_git::{install_hooks, on_post_checkout, write_root_gitignore};
use ship_module_project::ops::adr::{create_adr, find_adr_path, list_adrs, move_adr};
use ship_module_project::ops::feature::{
    create_feature, feature_done, feature_start, get_feature_by_id, list_features, update_feature,
};
use ship_module_project::ops::issue::{
    create_issue, get_issue_by_id, list_issues, move_issue_with_from,
};
use ship_module_project::ops::note::{
    create_note, get_note_by_id, list_notes, update_note_content,
};
use ship_module_project::ops::release::{
    create_release, get_release_by_id, list_releases, update_release,
};
use ship_module_project::ops::spec::{create_spec, get_spec_by_id, list_specs};
use ship_module_project::{
    ADR, AdrStatus, FeatureStatus, ISSUE_STATUSES, IssueStatus, NoteScope, import_adrs_from_files,
    import_features_from_files, import_issues_from_files, import_notes_from_files,
    import_releases_from_files, import_specs_from_files, init_demo_project, init_project,
    list_registered_projects, register_project, rename_project, unregister_project,
};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

#[derive(Parser, Debug)]
#[command(name = "ship")]
#[command(version)]
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
    /// Manage notes
    Note {
        #[command(subcommand)]
        action: NoteCommands,
    },
    /// Manage agent skills
    Skill {
        #[command(subcommand)]
        action: SkillCommands,
    },
    /// Manage specs
    Spec {
        #[command(subcommand)]
        action: SpecCommands,
    },
    /// Manage releases
    Release {
        #[command(subcommand)]
        action: ReleaseCommands,
    },
    /// Manage features
    Feature {
        #[command(subcommand)]
        action: FeatureCommands,
    },
    /// Manage workspace lifecycle state
    Workspace {
        #[command(subcommand)]
        action: WorkspaceCommands,
    },
    /// Inspect the project event stream
    Event {
        #[command(subcommand)]
        action: EventCommands,
    },
    /// Manage tracked projects
    Projects {
        #[command(subcommand)]
        action: ProjectCommands,
    },
    /// Initialize a demo project with sample data (safe for testing)
    #[command(hide = true)]
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
    #[command(hide = true)]
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
    #[command(hide = true)]
    Time {
        #[command(subcommand)]
        action: TimeCommands,
    },
    /// Manage workflow modes
    Mode {
        #[command(subcommand)]
        action: ModeCommands,
    },
    /// Manage MCP servers registered in ship.toml. Runs the server if no subcommand is provided.
    Mcp {
        #[command(subcommand)]
        action: Option<McpCommands>,
    },
    /// Manage AI agent providers (detect, connect, disconnect)
    Providers {
        #[command(subcommand)]
        action: ProviderCommands,
    },
    /// Run diagnostics on the Ship environment
    Doctor,
    /// Show version information
    Version,
    /// Migrate legacy YAML issues and JSON config to TOML
    #[command(hide = true)]
    Migrate {
        /// Re-run startup markdown imports even if already marked complete
        #[arg(long, default_value_t = false)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum GitCommands {
    /// Show what is and isn't committed to git
    Status,
    /// Include a category in git commits
    Include {
        /// One of: issues, releases, features, specs, adrs, notes, agents, ship.toml, templates
        category: String,
    },
    /// Exclude a category from git commits (adds to .ship/.gitignore)
    Exclude {
        /// One of: issues, releases, features, specs, adrs, notes, agents, ship.toml, templates
        category: String,
    },
    /// Install git hooks for feature-aware checkout
    InstallHooks,
    /// Hook entrypoint: regenerate agent context after checkout
    PostCheckout {
        /// Previous HEAD (hook arg $1) or branch name when called manually
        old: Option<String>,
        /// New HEAD (hook arg $2)
        new: Option<String>,
        /// Checkout mode flag (hook arg $3)
        flag: Option<String>,
    },
    /// Manually regenerate CLAUDE.md + .mcp.json for the current branch
    Sync,
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
    /// Export MCP server registry to an AI client's config file
    Export {
        /// Target AI client: claude, codex, or gemini
        #[arg(short, long)]
        target: String,
    },
    /// Show current AI provider configuration
    Ai,
}

#[derive(Subcommand, Debug)]
pub enum ModeCommands {
    /// List all defined modes
    List,
    /// Add a new mode
    Add {
        /// Mode ID (e.g. planning, dev, review)
        id: String,
        /// Display name
        name: String,
    },
    /// Remove a mode by ID
    Remove { id: String },
    /// Set the active mode
    Set { id: String },
    /// Clear the active mode (use all tools)
    Clear,
    /// Show the currently active mode
    Get,
}

#[derive(Subcommand, Debug)]
pub enum McpCommands {
    /// Start the MCP stdio server (explicitly)
    Serve,
    /// List MCP servers registered in ship.toml
    List,
    Add {
        /// Stable server ID (used in feature frontmatter)
        id: String,
        /// Human-readable name
        name: String,
        /// Server URL (for SSE or HTTP transport)
        url: String,
        /// Register but do not start
        #[arg(long)]
        disabled: bool,
    },
    /// Add a stdio MCP server
    AddStdio {
        /// Stable server ID
        id: String,
        /// Human-readable name
        name: String,
        /// Binary to run
        command: String,
        /// Arguments to pass to the binary
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Remove an MCP server by ID
    Remove { id: String },
}

#[derive(Subcommand, Debug)]
pub enum ProviderCommands {
    /// List all known providers and their status
    List,
    /// Connect (enable) a provider for this project
    Connect {
        /// Provider ID (claude, gemini, codex)
        id: String,
    },
    /// Disconnect (disable) a provider from this project
    Disconnect {
        /// Provider ID (claude, gemini, codex)
        id: String,
    },
    /// Detect installed providers in PATH and auto-connect them
    Detect,
    /// List available models for a provider
    Models {
        /// Provider ID (claude, gemini, codex)
        id: String,
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
    /// List ADRs
    List,
    /// Print an ADR's markdown content
    Get { file_name: String },
    /// Move an ADR to a new status
    Move { file_name: String, status: String },
}

#[derive(Subcommand, Debug)]
pub enum NoteCommands {
    /// Create a new note
    Create {
        title: String,
        /// Optional initial markdown content
        #[arg(short, long)]
        content: Option<String>,
        /// Scope: project (default) or user
        #[arg(long, default_value = "project")]
        scope: String,
    },
    /// List notes
    List {
        /// Scope: project (default) or user
        #[arg(long, default_value = "project")]
        scope: String,
    },
    /// Print a note document's markdown content
    Get {
        file_name: String,
        /// Scope: project (default) or user
        #[arg(long, default_value = "project")]
        scope: String,
    },
    /// Replace note markdown content
    Update {
        file_name: String,
        /// Full replacement content
        #[arg(short, long)]
        content: String,
        /// Scope: project (default) or user
        #[arg(long, default_value = "project")]
        scope: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum SkillCommands {
    /// Create a new skill
    Create {
        id: String,
        name: String,
        /// Skill body. Use $ARGUMENTS placeholder for slash command args.
        #[arg(short, long)]
        content: String,
        /// Scope: project (default) or user
        #[arg(long, default_value = "project")]
        scope: String,
    },
    /// List skills
    List {
        /// Scope: effective (default), project, or user
        #[arg(long, default_value = "effective")]
        scope: String,
    },
    /// Print a skill's markdown content
    Get {
        id: String,
        /// Scope: effective (default), project, or user
        #[arg(long, default_value = "effective")]
        scope: String,
    },
    /// Update an existing skill
    Update {
        id: String,
        /// Optional new display name
        #[arg(long)]
        name: Option<String>,
        /// Optional replacement content
        #[arg(short, long)]
        content: Option<String>,
        /// Scope: project (default) or user
        #[arg(long, default_value = "project")]
        scope: String,
    },
    /// Delete a skill
    Delete {
        id: String,
        /// Scope: project (default) or user
        #[arg(long, default_value = "project")]
        scope: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum SpecCommands {
    /// Create a new spec
    Create {
        title: String,
        /// Optional initial content (defaults to scaffold)
        #[arg(short, long)]
        content: Option<String>,
    },
    /// List spec documents
    List,
    /// Print a spec document's markdown content
    Get { file_name: String },
}

#[derive(Subcommand, Debug)]
pub enum ReleaseCommands {
    /// Create a new release
    Create {
        version: String,
        /// Optional initial content (defaults to scaffold)
        #[arg(short, long)]
        content: Option<String>,
    },
    /// List release documents
    List,
    /// Print a release document's markdown content
    Get {
        /// Release version or filename
        file_name: String,
    },
    /// Replace release markdown content
    Update {
        /// Release version or filename
        file_name: String,
        /// Full replacement content
        #[arg(short, long)]
        content: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum FeatureCommands {
    /// Create a new feature
    Create {
        title: String,
        /// Optional initial content (defaults to scaffold)
        #[arg(short, long)]
        content: Option<String>,
        /// Link this feature to a release ID
        #[arg(long)]
        release_id: Option<String>,
        /// Link this feature to a spec ID
        #[arg(long)]
        spec_id: Option<String>,
        /// Link this feature to a git branch name
        #[arg(long)]
        branch: Option<String>,
    },
    /// List feature documents
    List {
        /// Filter by status: planned, in-progress, implemented, deprecated
        #[arg(long)]
        status: Option<String>,
    },
    /// Print a feature document's markdown content
    Get {
        /// Feature ID (e.g. my-feature)
        id: String,
    },
    /// Replace feature markdown content
    Update {
        /// Feature ID
        id: String,
        /// Full replacement content
        #[arg(short, long)]
        content: String,
    },
    /// Mark a feature as in-progress
    Start {
        /// Feature ID
        id: String,
        /// Branch name to create/checkout (defaults to feature/{id})
        #[arg(short, long)]
        branch: Option<String>,
    },
    /// Mark a feature as implemented (done)
    Done {
        /// Feature ID
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum WorkspaceCommands {
    /// List workspace records and status
    List,
    /// Reconcile a branch into active workspace state
    Sync {
        /// Branch to sync (defaults to current branch)
        #[arg(short, long)]
        branch: Option<String>,
    },
    /// Checkout an existing branch and activate it
    Switch { branch: String },
    /// Create/update a workspace runtime record (git checkout optional)
    Create {
        branch: String,
        /// Optional workspace type: feature | refactor | experiment | hotfix
        #[arg(long = "type")]
        workspace_type: Option<String>,
        /// Link this workspace to a feature id
        #[arg(long)]
        feature: Option<String>,
        /// Link this workspace to a spec id
        #[arg(long)]
        spec: Option<String>,
        /// Link this workspace to a release id
        #[arg(long)]
        release: Option<String>,
        /// Mark workspace active immediately
        #[arg(long, default_value_t = false)]
        activate: bool,
        /// Also create/switch the git branch and then sync active state
        #[arg(long, default_value_t = false)]
        checkout: bool,
        /// Use a git worktree for this workspace
        #[arg(long, default_value_t = false)]
        worktree: bool,
        /// Path for the worktree (defaults to ../{branch})
        #[arg(long)]
        worktree_path: Option<String>,
    },
    /// Mark a workspace as archived
    Archive { branch: String },
}

#[derive(Subcommand, Debug)]
pub enum EventCommands {
    /// List events from the append-only event stream
    List {
        /// Only include events with seq greater than this value
        #[arg(long, default_value_t = 0)]
        since: u64,
        /// Maximum number of events to show
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Scan tracked files and emit events for external filesystem changes
    Ingest,
    /// Export events from SQLite to NDJSON
    Export {
        /// Destination path (defaults to .ship/events.ndjson)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    /// List all tracked projects
    List,
    /// Start tracking a project
    Track { name: String, path: PathBuf },
    /// Rename a tracked project without changing its path
    Rename { path: PathBuf, name: String },
    /// Stop tracking a project
    Untrack { path: PathBuf },
}

pub fn handle_cli(cli: Cli) -> Result<()> {
    let _ = ensure_user_notes_imported_once(false, false);
    if let Ok(project_dir) = get_project_dir(None) {
        let _ = ensure_project_imported_once(&project_dir, false, false);
    }

    match cli.command {
        Some(Commands::Init { path: init_path }) => {
            let target = match init_path {
                Some(p) => std::fs::canonicalize(&p)
                    .unwrap_or_else(|_| env::current_dir().unwrap_or_default().join(&p)),
                None => env::current_dir()?,
            };
            let project_name = target
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "project".to_string());
            let ship_path = init_project(target.clone())?;
            if let Err(err) = install_hooks(&target.join(".git")) {
                eprintln!(
                    "[ship] warning: failed to install git hooks in {}: {}",
                    target.join(".git").display(),
                    err
                );
            }
            if let Err(err) = write_root_gitignore(&target) {
                eprintln!("[ship] warning: failed to update root .gitignore: {}", err);
            }
            let tracked = match register_project(project_name, target.clone()) {
                Ok(()) => true,
                Err(err) => {
                    eprintln!(
                        "[ship] warning: initialized project but failed to register globally: {}",
                        err
                    );
                    eprintln!(
                        "[ship] run `ship projects track {} {}` later to add it to the global registry",
                        target
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "project".to_string()),
                        target.display()
                    );
                    false
                }
            };
            if tracked {
                println!(
                    "Initialized and tracked Ship project in {}",
                    ship_path.display()
                );
            } else {
                println!("Initialized Ship project in {}", ship_path.display());
            }
            // Auto-detect installed providers and enable them
            match autodetect_providers(&target) {
                Ok(found) if !found.is_empty() => {
                    println!("Detected and connected providers: {}", found.join(", "));
                }
                Ok(_) => {}
                Err(err) => {
                    eprintln!("[ship] warning: provider detection failed: {}", err);
                }
            }
        }
        Some(Commands::Issue { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                IssueCommands::Create { title, description } => {
                    let issue = create_issue(
                        &project_dir,
                        &title,
                        &description,
                        IssueStatus::Backlog,
                        None,
                        None,
                        None,
                        None,
                    )?;
                    println!("Issue created: {} ({})", issue.file_name, issue.id);
                }
                IssueCommands::List => {
                    let issues = list_issues(&project_dir)?;
                    for issue in issues {
                        println!("[{}] {}", issue.status, issue.file_name);
                    }
                }
                IssueCommands::Move {
                    file_name,
                    from,
                    to,
                } => {
                    let from_status = from
                        .parse::<IssueStatus>()
                        .map_err(|_| anyhow::anyhow!("Invalid issue status: {}", from))?;
                    let to_status = to
                        .parse::<IssueStatus>()
                        .map_err(|_| anyhow::anyhow!("Invalid issue status: {}", to))?;
                    move_issue_with_from(&project_dir, &file_name, from_status, to_status)?;
                    println!("Moved {} from {} to {}", file_name, from, to);
                }
            }
        }
        Some(Commands::Adr { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                AdrCommands::Create { title, decision } => {
                    let entry = create_adr(&project_dir, &title, "", &decision, "proposed")?;
                    println!("ADR created: {} (id: {})", title, entry.id);
                }
                AdrCommands::List => {
                    let mut adrs = list_adrs(&project_dir)?;
                    adrs.sort_by(|a, b| b.file_name.cmp(&a.file_name));
                    if adrs.is_empty() {
                        println!("No ADRs found.");
                    } else {
                        for adr in adrs {
                            println!(
                                "[{}] {} ({})",
                                adr.status, adr.adr.metadata.title, adr.file_name
                            );
                        }
                    }
                }
                AdrCommands::Get { file_name } => {
                    let path = find_adr_path(&project_dir, &file_name)?;
                    let content = std::fs::read_to_string(path)?;
                    println!("{}", content);
                }
                AdrCommands::Move { file_name, status } => {
                    let new_status = status
                        .parse::<AdrStatus>()
                        .map_err(|_| anyhow::anyhow!("Invalid ADR status: {}", status))?;
                    // Find the ADR by reading the file and extracting its id.
                    let path = find_adr_path(&project_dir, &file_name)?;
                    let content = std::fs::read_to_string(&path)?;
                    let adr = ADR::from_markdown(&content)
                        .map_err(|_| anyhow::anyhow!("Could not parse ADR file: {}", file_name))?;
                    let entry = move_adr(&project_dir, &adr.metadata.id, new_status.clone())?;
                    println!("Moved {} to {} (id: {})", file_name, new_status, entry.id);
                }
            }
        }
        Some(Commands::Note { action }) => match action {
            NoteCommands::Create {
                title,
                content,
                scope,
            } => {
                let scope = parse_note_scope(&scope)?;
                let project_dir = match scope {
                    NoteScope::Project => Some(get_project_dir_cli()?),
                    NoteScope::User => None,
                };
                let body = content.unwrap_or_default();
                let note = create_note(scope, project_dir.as_deref(), &title, &body)?;
                println!("Note created: {} (id: {})", note.title, note.id);
            }
            NoteCommands::List { scope } => {
                let scope = parse_note_scope(&scope)?;
                let project_dir = match scope {
                    NoteScope::Project => Some(get_project_dir_cli()?),
                    NoteScope::User => None,
                };
                let notes = list_notes(scope, project_dir.as_deref())?;
                if notes.is_empty() {
                    println!("No notes found.");
                } else {
                    for note in notes {
                        println!("{} ({})", note.title, note.id);
                    }
                }
            }
            NoteCommands::Get { file_name, scope } => {
                let scope = parse_note_scope(&scope)?;
                let project_dir = match scope {
                    NoteScope::Project => Some(get_project_dir_cli()?),
                    NoteScope::User => None,
                };
                let note = get_note_by_id(scope, project_dir.as_deref(), &file_name)?;
                println!("{}", note.content);
            }
            NoteCommands::Update {
                file_name,
                content,
                scope,
            } => {
                let scope = parse_note_scope(&scope)?;
                let project_dir = match scope {
                    NoteScope::Project => Some(get_project_dir_cli()?),
                    NoteScope::User => None,
                };
                let note =
                    update_note_content(scope, project_dir.as_deref(), &file_name, &content)?;
                println!("Updated note: {}", note.title);
            }
        },
        Some(Commands::Skill { action }) => match action {
            SkillCommands::Create {
                id,
                name,
                content,
                scope,
            } => {
                let scope = parse_skill_write_scope(&scope)?;
                let skill = match scope {
                    SkillWriteScope::Project => {
                        let project_dir = get_project_dir_cli()?;
                        let skill = create_skill(&project_dir, &id, &name, &content)?;
                        log_action(
                            &project_dir,
                            "skill create",
                            &format!("Created skill: {}", id),
                        )
                        .ok();
                        skill
                    }
                    SkillWriteScope::User => create_user_skill(&id, &name, &content)?,
                };
                println!("Skill created: {} ({})", skill.id, skill.name);
            }
            SkillCommands::List { scope } => {
                let scope = parse_skill_read_scope(&scope)?;
                let skills = match scope {
                    SkillReadScope::Project => {
                        let project_dir = get_project_dir_cli()?;
                        list_skills(&project_dir)?
                    }
                    SkillReadScope::User => list_user_skills()?,
                    SkillReadScope::Effective => {
                        let project_dir = get_project_dir_cli()?;
                        list_effective_skills(&project_dir)?
                    }
                };
                if skills.is_empty() {
                    println!("No skills found.");
                } else {
                    for skill in skills {
                        println!("{} ({})", skill.id, skill.name);
                    }
                }
            }
            SkillCommands::Get { id, scope } => {
                let scope = parse_skill_read_scope(&scope)?;
                let skill = match scope {
                    SkillReadScope::Project => {
                        let project_dir = get_project_dir_cli()?;
                        let skill = get_skill(&project_dir, &id)?;
                        log_action(&project_dir, "skill get", &format!("Got skill: {}", id)).ok();
                        skill
                    }
                    SkillReadScope::User => get_user_skill(&id)?,
                    SkillReadScope::Effective => {
                        let project_dir = get_project_dir_cli()?;
                        get_effective_skill(&project_dir, &id)?
                    }
                };
                println!("{}", skill.content);
            }
            SkillCommands::Update {
                id,
                name,
                content,
                scope,
            } => {
                let scope = parse_skill_write_scope(&scope)?;
                let updated = match scope {
                    SkillWriteScope::Project => {
                        let project_dir = get_project_dir_cli()?;
                        let updated =
                            update_skill(&project_dir, &id, name.as_deref(), content.as_deref())?;
                        log_action(
                            &project_dir,
                            "skill update",
                            &format!("Updated skill: {}", id),
                        )
                        .ok();
                        updated
                    }
                    SkillWriteScope::User => {
                        update_user_skill(&id, name.as_deref(), content.as_deref())?
                    }
                };
                println!("Updated skill: {} ({})", updated.id, updated.name);
            }
            SkillCommands::Delete { id, scope } => {
                let scope = parse_skill_write_scope(&scope)?;
                match scope {
                    SkillWriteScope::Project => {
                        let project_dir = get_project_dir_cli()?;
                        delete_skill(&project_dir, &id)?;
                        log_action(
                            &project_dir,
                            "skill delete",
                            &format!("Deleted skill: {}", id),
                        )
                        .ok();
                    }
                    SkillWriteScope::User => delete_user_skill(&id)?,
                }
                println!("Deleted skill: {}", id);
            }
        },
        Some(Commands::Spec { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                SpecCommands::Create { title, content } => {
                    let body = content.unwrap_or_default();
                    let spec = create_spec(&project_dir, &title, &body, None, None)?;
                    println!("Spec created: {} ({})", spec.file_name, spec.id);
                }
                SpecCommands::List => {
                    let mut specs = list_specs(&project_dir)?;
                    specs.sort_by(|a, b| b.spec.metadata.updated.cmp(&a.spec.metadata.updated));
                    if specs.is_empty() {
                        println!("No specs found.");
                    } else {
                        for spec in specs {
                            println!(
                                "[{}] {} ({})",
                                spec.status, spec.spec.metadata.title, spec.file_name
                            );
                        }
                    }
                }
                SpecCommands::Get { file_name } => {
                    let spec = get_spec_by_id(&project_dir, &file_name)?;
                    println!("{}", spec.spec.to_markdown()?);
                }
            }
        }
        Some(Commands::Release { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                ReleaseCommands::Create { version, content } => {
                    let body = content.unwrap_or_default();
                    let entry = create_release(&project_dir, &version, &body)?;
                    println!("Release created: {}", entry.path);
                }
                ReleaseCommands::List => {
                    let releases = list_releases(&project_dir)?;
                    if releases.is_empty() {
                        println!("No releases found.");
                    } else {
                        for release in releases {
                            println!(
                                "[{}] {} ({})",
                                release.status, release.version, release.file_name
                            );
                        }
                    }
                }
                ReleaseCommands::Get { file_name } => {
                    let version = file_name.trim_end_matches(".md");
                    let entry = get_release_by_id(&project_dir, version)
                        .map_err(|_| anyhow::anyhow!("Release not found: {}", file_name))?;
                    let release_path = {
                        let primary =
                            runtime::project::releases_dir(&project_dir).join(&entry.file_name);
                        if primary.exists() {
                            primary
                        } else {
                            let legacy = runtime::project::upcoming_releases_dir(&project_dir)
                                .join(&entry.file_name);
                            if legacy.exists() {
                                legacy
                            } else {
                                anyhow::bail!("Release file not found: {}", entry.file_name);
                            }
                        }
                    };
                    let content = std::fs::read_to_string(release_path)?;
                    println!("{}", content);
                }
                ReleaseCommands::Update { file_name, content } => {
                    let version = file_name.trim_end_matches(".md");
                    let mut entry = get_release_by_id(&project_dir, version)?;
                    entry.release.body = content;
                    update_release(&project_dir, version, entry.release)?;
                    println!("Updated release: {}", file_name);
                }
            }
        }
        Some(Commands::Feature { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                FeatureCommands::Create {
                    title,
                    content,
                    release_id,
                    spec_id,
                    branch,
                } => {
                    let body = content.unwrap_or_default();
                    let entry = create_feature(
                        &project_dir,
                        &title,
                        &body,
                        release_id.as_deref(),
                        spec_id.as_deref(),
                        branch.as_deref(),
                    )?;
                    println!("Feature created: {}", entry.path);
                }
                FeatureCommands::List { status } => {
                    let features = list_features(&project_dir)?;
                    let filtered: Vec<_> = if let Some(s) = status {
                        let target_status = s.parse::<FeatureStatus>().unwrap_or_default();
                        features
                            .into_iter()
                            .filter(|f| f.status == target_status)
                            .collect()
                    } else {
                        features
                    };

                    if filtered.is_empty() {
                        println!("No features found.");
                    } else {
                        for entry in filtered {
                            println!(
                                "[{}] {} ({}) id={}",
                                entry.status,
                                entry.feature.metadata.title,
                                entry.file_name,
                                entry.id
                            );
                        }
                    }
                }
                FeatureCommands::Get { id } => {
                    let entry = get_feature_by_id(&project_dir, &id)?;
                    println!("{}", entry.feature.to_markdown()?);
                }
                FeatureCommands::Update { id, content } => {
                    let mut entry = get_feature_by_id(&project_dir, &id)?;
                    entry.feature.body = content;
                    update_feature(&project_dir, &id, entry.feature)?;
                    println!("Updated feature: {}", id);
                }
                FeatureCommands::Start { id, branch } => {
                    let mut entry = get_feature_by_id(&project_dir, &id)?;
                    let branch_name = branch.unwrap_or_else(|| {
                        let base =
                            runtime::project::sanitize_file_name(&entry.feature.metadata.title);
                        format!("feature/{}", base)
                    });

                    // Create the branch if it doesn't exist
                    let branch_exists = std::process::Command::new("git")
                        .args(["rev-parse", "--verify", &branch_name])
                        .current_dir(&project_dir)
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false);

                    if !branch_exists {
                        let result = std::process::Command::new("git")
                            .args(["checkout", "-b", &branch_name])
                            .current_dir(&project_dir)
                            .status()?;
                        if !result.success() {
                            anyhow::bail!("Failed to create branch: {}", branch_name);
                        }
                    } else {
                        let result = std::process::Command::new("git")
                            .args(["checkout", &branch_name])
                            .current_dir(&project_dir)
                            .status()?;
                        if !result.success() {
                            anyhow::bail!("Failed to checkout branch: {}", branch_name);
                        }
                    }

                    entry.feature.metadata.branch = Some(branch_name);
                    update_feature(&project_dir, &id, entry.feature)?;
                    feature_start(&project_dir, &id)?;
                    println!("Feature started: {}", id);
                }
                FeatureCommands::Done { id } => {
                    feature_done(&project_dir, &id)?;
                    println!("Feature marked as implemented: {}", id);
                }
            }
        }
        Some(Commands::Workspace { action }) => {
            let project_dir = get_project_dir_cli()?;
            let project_root = project_dir.parent().unwrap_or(&project_dir).to_path_buf();
            match action {
                WorkspaceCommands::List => {
                    let workspaces = list_workspaces(&project_dir)?;
                    if workspaces.is_empty() {
                        println!("No workspaces found.");
                    } else {
                        for workspace in workspaces {
                            println!(
                                "[{}] {} ({}){}",
                                workspace.status,
                                workspace.branch,
                                workspace.workspace_type,
                                workspace
                                    .feature_id
                                    .as_ref()
                                    .map(|id| format!(" feature={}", id))
                                    .unwrap_or_default()
                            );
                        }
                    }
                }
                WorkspaceCommands::Sync { branch } => {
                    let branch = match branch {
                        Some(value) => value,
                        None => current_branch(&project_root)?,
                    };
                    let workspace = sync_workspace(&project_dir, &branch)?;
                    println!(
                        "Workspace synced: {} [{}]",
                        workspace.branch, workspace.status
                    );
                }
                WorkspaceCommands::Switch { branch } => {
                    let result = ProcessCommand::new("git")
                        .args(["checkout", &branch])
                        .current_dir(&project_root)
                        .status()?;
                    if !result.success() {
                        anyhow::bail!("Failed to checkout branch: {}", branch);
                    }
                    let workspace = activate_workspace(&project_dir, &branch)?;
                    println!(
                        "Workspace active: {} [{}]",
                        workspace.branch, workspace.status
                    );
                }
                WorkspaceCommands::Create {
                    branch,
                    workspace_type,
                    feature,
                    spec,
                    release,
                    activate,
                    checkout,
                    worktree,
                    worktree_path,
                } => {
                    if worktree && checkout {
                        anyhow::bail!("--worktree and --checkout cannot be used together");
                    }
                    if worktree_path.is_some() && !worktree {
                        anyhow::bail!("--worktree-path requires --worktree");
                    }

                    let parsed_workspace_type = workspace_type
                        .as_deref()
                        .map(str::parse::<WorkspaceType>)
                        .transpose()?;
                    let resolved_worktree_path = if worktree {
                        Some(worktree_path.unwrap_or_else(|| {
                            let b = branch
                                .trim_start_matches("feature/")
                                .trim_start_matches("hotfix/");
                            format!("../{}", b)
                        }))
                    } else {
                        None
                    };

                    if worktree {
                        let path = resolved_worktree_path
                            .as_deref()
                            .ok_or_else(|| anyhow::anyhow!("Worktree path resolution failed"))?;
                        let exists = ProcessCommand::new("git")
                            .args(["rev-parse", "--verify", &branch])
                            .current_dir(&project_root)
                            .output()
                            .map(|output| output.status.success())
                            .unwrap_or(false);

                        let mut args = vec!["worktree", "add"];
                        if !exists {
                            args.push("-b");
                            args.push(&branch);
                            args.push(path);
                        } else {
                            args.push(path);
                            args.push(&branch);
                        }

                        let status = ProcessCommand::new("git")
                            .args(args)
                            .current_dir(&project_root)
                            .status()?;
                        if !status.success() {
                            if !exists {
                                let _ = ProcessCommand::new("git")
                                    .args(["branch", "-D", &branch])
                                    .current_dir(&project_root)
                                    .status();
                            }
                            anyhow::bail!("Failed to create git worktree: {}", branch);
                        }
                    } else if checkout {
                        let exists = ProcessCommand::new("git")
                            .args(["rev-parse", "--verify", &branch])
                            .current_dir(&project_root)
                            .output()
                            .map(|output| output.status.success())
                            .unwrap_or(false);
                        let checkout_status = if exists {
                            ProcessCommand::new("git")
                                .args(["checkout", &branch])
                                .current_dir(&project_root)
                                .status()?
                        } else {
                            ProcessCommand::new("git")
                                .args(["checkout", "-b", &branch])
                                .current_dir(&project_root)
                                .status()?
                        };
                        if !checkout_status.success() {
                            anyhow::bail!("Failed to create/switch branch: {}", branch);
                        }
                    }

                    let desired_status = if activate && !checkout && !worktree {
                        Some(WorkspaceStatus::Active)
                    } else {
                        None
                    };
                    let mut workspace = create_workspace(
                        &project_dir,
                        CreateWorkspaceRequest {
                            branch: branch.clone(),
                            workspace_type: parsed_workspace_type,
                            status: desired_status,
                            feature_id: feature,
                            spec_id: spec,
                            release_id: release,
                            is_worktree: Some(worktree),
                            worktree_path: resolved_worktree_path,
                            ..CreateWorkspaceRequest::default()
                        },
                    )?;

                    if worktree || checkout {
                        workspace = sync_workspace(&project_dir, &branch)?;
                    } else if activate {
                        workspace = activate_workspace(&project_dir, &branch)?;
                    }

                    println!(
                        "Workspace {}: {} [{}]",
                        if workspace.status == WorkspaceStatus::Active {
                            "active"
                        } else {
                            "created"
                        },
                        workspace.branch,
                        workspace.status
                    );
                }
                WorkspaceCommands::Archive { branch } => {
                    let workspace = transition_workspace_status(
                        &project_dir,
                        &branch,
                        WorkspaceStatus::Archived,
                    )?;
                    println!(
                        "Workspace archived: {} [{}]",
                        workspace.branch, workspace.status
                    );
                }
            }
        }
        Some(Commands::Event { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                EventCommands::List { since, limit } => {
                    let events = list_events_since(&project_dir, since, Some(limit))?;
                    if events.is_empty() {
                        println!("No events found.");
                    } else {
                        for e in events {
                            let details = e
                                .details
                                .as_ref()
                                .map(|d| format!(" — {}", d))
                                .unwrap_or_default();
                            println!(
                                "#{:04} {} [{}] {:?}.{:?} {}{}",
                                e.seq,
                                e.timestamp.format("%Y-%m-%d %H:%M:%S"),
                                e.actor,
                                e.entity,
                                e.action,
                                e.subject,
                                details
                            );
                        }
                    }
                }
                EventCommands::Ingest => {
                    let events = ingest_external_events(&project_dir)?;
                    if events.is_empty() {
                        println!("No external filesystem changes detected.");
                    } else {
                        println!("Ingested {} filesystem event(s).", events.len());
                        for e in events {
                            let details = e
                                .details
                                .as_ref()
                                .map(|d| format!(" — {}", d))
                                .unwrap_or_default();
                            println!(
                                "#{:04} {} [{}] {:?}.{:?} {}{}",
                                e.seq,
                                e.timestamp.format("%Y-%m-%d %H:%M:%S"),
                                e.actor,
                                e.entity,
                                e.action,
                                e.subject,
                                details
                            );
                        }
                    }
                }
                EventCommands::Export { output } => {
                    let output_path =
                        output.unwrap_or_else(|| project_dir.join(runtime::EVENTS_FILE_NAME));
                    let exported = runtime::export_events_ndjson(&project_dir, &output_path)?;
                    println!(
                        "Exported {} event{} to {}",
                        exported,
                        if exported == 1 { "" } else { "s" },
                        output_path.display()
                    );
                }
            }
        }
        Some(Commands::Projects { action }) => match action {
            ProjectCommands::List => {
                let projects = list_registered_projects()?;
                for p in projects {
                    println!("- {} ({})", p.name, p.path.display());
                }
            }
            ProjectCommands::Track { name, path } => {
                register_project(name.clone(), path.clone())?;
                println!("Now tracking project: {} ({})", name, path.display());
            }
            ProjectCommands::Rename { path, name } => {
                rename_project(path.clone(), name.clone())?;
                println!("Renamed project at {} to {}", path.display(), name);
            }
            ProjectCommands::Untrack { path } => {
                unregister_project(path.clone())?;
                println!("Stopped tracking project: {}", path.display());
            }
        },
        Some(Commands::Demo { path }) => {
            let abs = std::fs::canonicalize(&path)
                .unwrap_or_else(|_| env::current_dir().unwrap_or_default().join(&path));
            let project_dir = init_demo_project(abs.clone())?;
            println!("Demo project ready at {}", project_dir.display());
            println!(
                "Point Ship at it with: SHIP_DIR={} ship issue list",
                project_dir.display()
            );
        }
        Some(Commands::Git { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                GitCommands::Status => {
                    let git = get_git_config(&project_dir)?;
                    let cats = [
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
                    println!("Ship git commit settings:");
                    for cat in cats {
                        let state = if is_category_committed(&git, cat) {
                            "committed"
                        } else {
                            "local only"
                        };
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
                GitCommands::InstallHooks => {
                    let project_root = project_dir
                        .parent()
                        .ok_or_else(|| anyhow::anyhow!("Could not resolve project root"))?;
                    install_hooks(&project_root.join(".git"))?;
                    println!(
                        "Installed git hooks in {}",
                        project_root.join(".git/hooks").display()
                    );
                }
                GitCommands::PostCheckout { old, new, flag } => {
                    // Use CWD as project_root so worktrees write CLAUDE.md to the
                    // worktree directory, not the main repo root.
                    let cwd = env::current_dir()?;
                    let old_ref = old
                        .or_else(|| env::var("SHIP_GIT_OLD_REF").ok())
                        .or_else(|| env::var("GIT_OLD_REF").ok());
                    let _new_ref = new
                        .or_else(|| env::var("SHIP_GIT_NEW_REF").ok())
                        .or_else(|| env::var("GIT_NEW_REF").ok());
                    let _checkout_flag = flag
                        .or_else(|| env::var("SHIP_GIT_CHECKOUT_FLAG").ok())
                        .or_else(|| env::var("GIT_CHECKOUT_FLAG").ok());

                    let branch = if _new_ref.is_none() && _checkout_flag.is_none() {
                        old_ref
                    } else {
                        None
                    }
                    .or_else(|| env::var("SHIP_GIT_BRANCH").ok())
                    .unwrap_or(current_branch(&cwd)?);

                    on_post_checkout(&project_dir, &branch, &cwd)?;
                }
                GitCommands::Sync => {
                    let cwd = env::current_dir()?;
                    let branch = current_branch(&cwd)?;
                    on_post_checkout(&project_dir, &branch, &cwd)?;
                }
            }
        }
        Some(Commands::Ghost { action }) => {
            let project_dir = get_project_dir_cli()?;
            ensure_builtin_plugin_namespaces(&project_dir)?;
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
                        println!(
                            "  ... and {} more. Run `ship ghost report` for full list.",
                            unpromoted - 10
                        );
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
                            if let Some(g) = scan
                                .issues
                                .iter()
                                .find(|g| g.file == file && g.line == line)
                            {
                                let title = g.suggested_title();
                                let desc = format!(
                                    "Promoted from `{}:{}` ({}).\n\nOriginal comment: {}",
                                    g.file,
                                    g.line,
                                    g.kind.as_str(),
                                    g.text.trim()
                                );
                                let path = create_issue(
                                    &project_dir,
                                    &title,
                                    &desc,
                                    IssueStatus::Backlog,
                                    None,
                                    None,
                                    None,
                                    None,
                                )?;
                                println!("Created issue: {}", path.file_name);
                            }
                        }
                    } else {
                        println!(
                            "Ghost issue not found at {}:{}. Run `ship ghost scan` first.",
                            file, line
                        );
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
                        if let Some(p_dir) = project_dir.as_ref() {
                            log_action(&p_dir, "config status add", &name)?;
                        }
                        add_status(project_dir, &name)?;
                        println!("Added status: {}", name.to_lowercase().replace(' ', "-"));
                    }
                    StatusCommands::Remove { name } => {
                        remove_status(project_dir, &name)?;
                        println!("Removed status: {}", name);
                    }
                },
                ConfigCommands::Export { target } => {
                    let dir = project_dir.ok_or_else(|| {
                        anyhow::anyhow!("No Ship project found in current directory")
                    })?;
                    runtime::agent_export::export_to(dir, &target)?;
                    println!("Exported MCP server registry to {} config.", target);
                }
                ConfigCommands::Ai => {
                    let cfg = get_config(project_dir)?;
                    let ai = cfg.ai.unwrap_or_default();
                    println!("AI provider : {}", ai.effective_provider());
                    if let Some(path) = &ai.cli_path {
                        println!("CLI path    : {}", path);
                    } else {
                        println!("CLI path    : (default — uses provider name on PATH)");
                    }
                }
            }
        }
        Some(Commands::Mode { action }) => {
            let project_dir = get_project_dir(None).ok();
            match action {
                ModeCommands::List => {
                    let cfg = get_config(project_dir.clone())?;
                    let modes = cfg.modes;
                    let active_id = cfg.active_mode.as_deref().unwrap_or("");
                    if modes.is_empty() {
                        println!("No modes configured.");
                    } else {
                        for m in &modes {
                            let marker = if m.id == active_id { " *" } else { "" };
                            println!("  {}{} — {}", m.id, marker, m.name);
                        }
                    }
                }
                ModeCommands::Add { id, name } => {
                    let mode = runtime::ModeConfig {
                        id: id.clone(),
                        name: name.clone(),
                        ..Default::default()
                    };
                    add_mode(project_dir, mode)?;
                    println!("Mode added: {} ({})", id, name);
                }
                ModeCommands::Remove { id } => {
                    remove_mode(project_dir, &id)?;
                    println!("Mode removed: {}", id);
                }
                ModeCommands::Set { id } => {
                    set_active_mode(project_dir, Some(&id))?;
                    println!("Active mode set to: {}", id);
                }
                ModeCommands::Clear => {
                    set_active_mode(project_dir, None)?;
                    println!("Active mode cleared (all tools available).");
                }
                ModeCommands::Get => match get_active_mode(project_dir)? {
                    Some(m) => println!("Active mode: {} ({})", m.id, m.name),
                    None => println!("No active mode set."),
                },
            }
        }
        Some(Commands::Time { action }) => {
            let project_dir = get_project_dir_cli()?;
            ensure_builtin_plugin_namespaces(&project_dir)?;
            handle_time_command(action, &project_dir)?;
        }
        Some(Commands::Mcp { action }) => {
            match action {
                None | Some(McpCommands::Serve) => {
                    // Handled by the main unitary binary as it requires async
                }
                Some(McpCommands::List) => {
                    let project_dir = get_project_dir_cli()?;
                    let servers = list_mcp_servers(Some(project_dir))?;
                    if servers.is_empty() {
                        println!("No MCP servers configured. Add one with `ship mcp add`.");
                    } else {
                        for s in &servers {
                            let transport = match &s.server_type {
                                McpServerType::Stdio => format!("stdio:{}", s.command),
                                McpServerType::Sse => {
                                    format!("sse:{}", s.url.as_deref().unwrap_or("?"))
                                }
                                McpServerType::Http => {
                                    format!("http:{}", s.url.as_deref().unwrap_or("?"))
                                }
                            };
                            let status = if s.disabled { " [disabled]" } else { "" };
                            println!("{} — {} ({}){}", s.id, s.name, transport, status);
                        }
                    }
                }
                Some(McpCommands::Add {
                    id,
                    name,
                    url,
                    disabled,
                }) => {
                    let project_dir = get_project_dir_cli()?;
                    let server = McpServerConfig {
                        id: id.clone(),
                        name,
                        command: String::new(),
                        args: vec![],
                        env: Default::default(),
                        scope: "project".to_string(),
                        server_type: McpServerType::Http,
                        url: Some(url),
                        disabled,
                        timeout_secs: None,
                    };
                    add_mcp_server(Some(project_dir), server)?;
                    println!("Added MCP server '{}'.", id);
                }
                Some(McpCommands::AddStdio {
                    id,
                    name,
                    command,
                    args,
                }) => {
                    let project_dir = get_project_dir_cli()?;
                    let server = McpServerConfig {
                        id: id.clone(),
                        name,
                        command,
                        args,
                        env: Default::default(),
                        scope: "project".to_string(),
                        server_type: McpServerType::Stdio,
                        url: None,
                        disabled: false,
                        timeout_secs: None,
                    };
                    add_mcp_server(Some(project_dir), server)?;
                    println!("Added stdio MCP server '{}'.", id);
                }
                Some(McpCommands::Remove { id }) => {
                    let project_dir = get_project_dir_cli()?;
                    remove_mcp_server(Some(project_dir), &id)?;
                    println!("Removed MCP server '{}'.", id);
                }
            }
        }
        Some(Commands::Providers { action }) => {
            let project_dir = get_project_dir_cli()?;
            match action {
                ProviderCommands::List => {
                    let providers = list_providers(&project_dir)?;
                    println!(
                        "{:<12} {:<20} {:<10} {:<10} {}",
                        "ID", "NAME", "INSTALLED", "CONNECTED", "VERSION"
                    );
                    println!("{}", "-".repeat(70));
                    for p in providers {
                        println!(
                            "{:<12} {:<20} {:<10} {:<10} {}",
                            p.id,
                            p.name,
                            if p.installed { "yes" } else { "no" },
                            if p.enabled { "yes" } else { "no" },
                            p.version.as_deref().unwrap_or("-"),
                        );
                    }
                }
                ProviderCommands::Connect { id } => {
                    if enable_provider(&project_dir, &id)? {
                        println!("Connected provider: {}", id);
                    } else {
                        println!("Provider '{}' is already connected.", id);
                    }
                }
                ProviderCommands::Disconnect { id } => {
                    if disable_provider(&project_dir, &id)? {
                        println!("Disconnected provider: {}", id);
                    } else {
                        println!("Provider '{}' was not connected.", id);
                    }
                }
                ProviderCommands::Detect => {
                    let found = autodetect_providers(&project_dir)?;
                    if found.is_empty() {
                        println!("No new providers detected.");
                    } else {
                        println!("Detected and connected: {}", found.join(", "));
                    }
                }
                ProviderCommands::Models { id } => {
                    let models = list_models(&id)?;
                    println!("{:<30} {:<14} {}", "MODEL", "CONTEXT", "");
                    println!("{}", "-".repeat(60));
                    for m in models {
                        println!(
                            "{:<30} {:<14} {}",
                            m.id,
                            format!("{}k", m.context_window / 1000),
                            if m.recommended { "(recommended)" } else { "" },
                        );
                    }
                }
            }
        }
        Some(Commands::Doctor) => {
            handle_doctor_command()?;
        }
        Some(Commands::Version) => {
            let version = env!("CARGO_PKG_VERSION");
            let git_hash = option_env!("SHIP_GIT_SHA").unwrap_or("unknown");
            let build_time = option_env!("SHIP_BUILD_TIMESTAMP").unwrap_or("unknown");
            println!("ship version {} ({})", version, git_hash);
            println!("built at {}", build_time);
        }
        Some(Commands::Migrate { force }) => {
            let project_dir = get_project_dir_cli()?;
            let global_dir = get_global_dir()?;
            let global = migrate_global_state(&global_dir)?;
            let project = migrate_project_state(&project_dir)?;
            let issues = import_issues_from_files(&project_dir)?;
            let specs = import_specs_from_files(&project_dir)?;
            let config = migrate_json_config_file(&project_dir)?;
            let cleared_project_markers = runtime::clear_project_migration_meta(&project_dir)?;
            let cleared_global_markers = runtime::clear_global_migration_meta()?;
            ensure_user_notes_imported_once(true, true)?;
            ensure_project_imported_once(&project_dir, true, true)?;
            println!(
                "Migration complete{}:\n- file namespace copies: copied={} skipped={} conflicts={}\n- project DB: {} (applied {})\n- global DB: {} (applied {})\n- registry: {} -> {} entries (normalized {})\n- app_state paths normalized: {}\n- startup import markers reset: {} project marker{}, {} global marker{}\n- imported docs: {} issue{}, {} spec{}{}.",
                if force { " (forced)" } else { "" },
                project.files.copied_files,
                project.files.skipped_identical_files,
                project.files.conflict_files,
                project.db.db_path.display(),
                project.db.applied_migrations,
                global.db.db_path.display(),
                global.db.applied_migrations,
                global.registry_entries_before,
                global.registry_entries_after,
                global.normalized_paths,
                global.app_state_paths_normalized,
                cleared_project_markers,
                if cleared_project_markers == 1 {
                    ""
                } else {
                    "s"
                },
                cleared_global_markers,
                if cleared_global_markers == 1 { "" } else { "s" },
                issues,
                if issues == 1 { "" } else { "s" },
                specs,
                if specs == 1 { "" } else { "s" },
                if config {
                    ", config.json → ship.toml"
                } else {
                    ""
                },
            );
        }
        None => {
            // This case should be handled by the caller to decide whether to show help or launch GUI
        }
    }

    Ok(())
}

fn handle_doctor_command() -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let git_hash = option_env!("SHIP_GIT_SHA").unwrap_or("unknown");
    println!("Checking Ship environment (v{} - {})...", version, git_hash);

    // 1. Check project directory
    match get_project_dir(None) {
        Ok(dir) => println!("✓ Project directory found: {}", dir.display()),
        Err(e) => println!("✗ Project directory not found: {}", e),
    }

    // 2. Check global directory
    match get_global_dir() {
        Ok(dir) => println!("✓ Global directory found: {}", dir.display()),
        Err(e) => println!("✗ Global directory not found: {}", e),
    }

    // 3. Check configuration
    match get_config(None) {
        Ok(_) => println!("✓ Configuration is valid"),
        Err(e) => println!("✗ Configuration error: {}", e),
    }

    // 4. Check for binary in PATH
    let current_exe = std::env::current_exe().unwrap_or_default();
    println!("✓ Current executable: {}", current_exe.display());

    // 5. Check AI providers
    if let Ok(dir) = get_project_dir(None) {
        if let Ok(providers) = list_providers(&dir) {
            let connected = providers.iter().filter(|p| p.enabled).count();
            let installed = providers.iter().filter(|p| p.installed).count();
            println!(
                "✓ AI Providers: {} connected, {} installed (out of {} supported)",
                connected,
                installed,
                providers.len()
            );

            if connected == 0 {
                println!(
                    "  ⚠ No AI providers connected. Connect one with `ship providers connect <id>`"
                );
            }

            for p in providers.iter().filter(|p| p.enabled) {
                if p.installed {
                    let version = p.version.as_deref().unwrap_or("unknown version");
                    println!(
                        "✓ Provider '{}' is connected and installed ({})",
                        p.id, version
                    );
                } else {
                    println!(
                        "  ⚠ Provider '{}' is connected but binary '{}' was not found in PATH",
                        p.id, p.binary
                    );
                }
            }
        }
    }

    // 6. Check MCP servers
    if let Ok(dir) = get_project_dir(None) {
        if let Ok(servers) = list_mcp_servers(Some(dir)) {
            let ship_mcp = servers.iter().find(|s| s.id == "ship");
            match ship_mcp {
                Some(s) => {
                    if s.command == "ship" && s.args.len() >= 1 && s.args[0] == "mcp" {
                        println!("✓ Shipwright MCP server is correctly registered");
                    } else {
                        println!(
                            "  ⚠ Shipwright MCP server registration looks outdated or customized: {} {:?}",
                            s.command, s.args
                        );
                    }
                }
                None => println!("  ⚠ Shipwright MCP server 'ship' is not registered in ship.toml"),
            }
        }
    }

    Ok(())
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

fn get_project_dir_cli() -> Result<PathBuf> {
    get_project_dir(None)
}

fn ensure_project_imported_once(project_dir: &Path, force: bool, strict: bool) -> Result<()> {
    run_project_import(project_dir, "adr", "ADRs", force, strict, || {
        import_adrs_from_files(project_dir)
    })?;
    run_project_import(
        project_dir,
        "note_project",
        "project notes",
        force,
        strict,
        || import_notes_from_files(NoteScope::Project, Some(project_dir)),
    )?;
    run_project_import(project_dir, "feature", "features", force, strict, || {
        import_features_from_files(project_dir)
    })?;
    run_project_import(project_dir, "release", "releases", force, strict, || {
        import_releases_from_files(project_dir)
    })?;
    Ok(())
}

fn ensure_user_notes_imported_once(force: bool, strict: bool) -> Result<()> {
    if !force && runtime::migration_meta_complete_global("note_user")? {
        return Ok(());
    }

    match import_notes_from_files(NoteScope::User, None) {
        Ok(count) => {
            runtime::mark_migration_meta_complete_global("note_user", count)?;
            if count > 0 {
                println!(
                    "[ship] Imported {} global notes from files to SQLite",
                    count
                );
            }
            Ok(())
        }
        Err(err) if strict => Err(err),
        Err(err) => {
            eprintln!(
                "[ship] warning: failed to import global notes from files: {}",
                err
            );
            Ok(())
        }
    }
}

fn run_project_import<F>(
    project_dir: &Path,
    entity_type: &str,
    label: &str,
    force: bool,
    strict: bool,
    importer: F,
) -> Result<()>
where
    F: FnOnce() -> Result<usize>,
{
    if !force && runtime::migration_meta_complete_project(project_dir, entity_type)? {
        return Ok(());
    }

    match importer() {
        Ok(count) => {
            runtime::mark_migration_meta_complete_project(project_dir, entity_type, count)?;
            if count > 0 {
                println!("[ship] Imported {} {} from files to SQLite", count, label);
            }
            Ok(())
        }
        Err(err) if strict => Err(err),
        Err(err) => {
            eprintln!(
                "[ship] warning: failed to import {} from files: {}",
                label, err
            );
            Ok(())
        }
    }
}

fn parse_note_scope(raw: &str) -> Result<NoteScope> {
    raw.parse::<NoteScope>()
}

enum SkillReadScope {
    Project,
    User,
    Effective,
}

enum SkillWriteScope {
    Project,
    User,
}

fn parse_skill_read_scope(raw: &str) -> Result<SkillReadScope> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "project" => Ok(SkillReadScope::Project),
        "user" | "global" => Ok(SkillReadScope::User),
        "effective" => Ok(SkillReadScope::Effective),
        other => anyhow::bail!(
            "Invalid skill scope '{}'. Expected one of: project, user, effective",
            other
        ),
    }
}

fn parse_skill_write_scope(raw: &str) -> Result<SkillWriteScope> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "project" => Ok(SkillWriteScope::Project),
        "user" | "global" => Ok(SkillWriteScope::User),
        other => anyhow::bail!(
            "Invalid skill scope '{}'. Expected one of: project, user",
            other
        ),
    }
}

fn ensure_builtin_plugin_namespaces(project_dir: &PathBuf) -> Result<()> {
    let mut registry = runtime::PluginRegistry::new();
    registry.register_with_project(project_dir, Box::new(ghost_issues::GhostIssues))?;
    registry.register_with_project(project_dir, Box::new(time_tracker::TimeTracker))?;
    Ok(())
}

fn handle_time_command(action: TimeCommands, project_dir: &PathBuf) -> Result<()> {
    use time_tracker::{
        format_duration, generate_report, get_active_timer, list_entries, log_time, start_timer,
        stop_timer,
    };

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
                    for status in ISSUE_STATUSES {
                        let p = runtime::project::issues_dir(project_dir)
                            .join(status)
                            .join(&issue_file);
                        if p.exists() {
                            found = Some(p);
                            break;
                        }
                    }
                    found.unwrap_or(issue_path)
                };
                if path.exists() {
                    get_issue_by_id(project_dir, &issue_file)
                        .ok()
                        .map(|i| i.issue.metadata.title)
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
        TimeCommands::Status => match get_active_timer(project_dir)? {
            Some(t) => {
                let elapsed = (chrono::Utc::now() - t.started_at).num_minutes().max(0) as u64;
                println!(
                    "Running: {} (started {}, elapsed {})",
                    t.issue_title,
                    t.started_at.format("%H:%M"),
                    format_duration(elapsed)
                );
            }
            None => println!("No timer running."),
        },
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_feature_file(ship_dir: &Path, id: &str, title: &str, file_name: &str) -> Result<()> {
        let path = runtime::project::features_dir(ship_dir)
            .join("planned")
            .join(file_name);
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(
            path,
            format!(
                "+++\nid = \"{}\"\ntitle = \"{}\"\ncreated = \"2026-01-01T00:00:00Z\"\nupdated = \"2026-01-01T00:00:00Z\"\ntags = []\n+++\n\nbody\n",
                id, title
            ),
        )?;
        Ok(())
    }

    #[test]
    fn ensure_project_imported_once_skips_after_marker_and_force_reimports() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;

        write_feature_file(
            &project_dir,
            "feature-startup-1",
            "Startup Import One",
            "startup-import-one.md",
        )?;

        ensure_project_imported_once(&project_dir, false, true)?;
        assert!(runtime::migration_meta_complete_project(
            &project_dir,
            "feature"
        )?);
        assert_eq!(list_features(&project_dir)?.len(), 1);

        write_feature_file(
            &project_dir,
            "feature-startup-2",
            "Startup Import Two",
            "startup-import-two.md",
        )?;

        // Marker is already set, so regular startup import should skip re-scan.
        ensure_project_imported_once(&project_dir, false, true)?;
        assert_eq!(list_features(&project_dir)?.len(), 1);

        ensure_project_imported_once(&project_dir, true, true)?;
        assert_eq!(list_features(&project_dir)?.len(), 2);
        Ok(())
    }

    #[test]
    fn cli_parses_projects_rename_subcommand() {
        let cli = Cli::try_parse_from(["ship", "projects", "rename", "/tmp/project", "ship-core"])
            .expect("projects rename should parse");

        match cli.command {
            Some(Commands::Projects {
                action: ProjectCommands::Rename { path, name },
            }) => {
                assert_eq!(path, PathBuf::from("/tmp/project"));
                assert_eq!(name, "ship-core");
            }
            other => panic!("unexpected parse result: {:?}", other),
        }
    }
}
