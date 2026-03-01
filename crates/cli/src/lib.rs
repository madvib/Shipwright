use anyhow::Result;
use clap::{Parser, Subcommand};
use runtime::{
    FeatureStatus, McpServerConfig, McpServerType, NoteScope, add_mcp_server, add_mode, add_status,
    autodetect_providers, backfill_issue_ids, create_adr, create_feature, create_issue,
    create_note, create_release, create_skill, create_spec, create_user_skill, delete_skill,
    delete_user_skill, disable_provider, enable_provider, feature_done, feature_start,
    find_release_path, get_active_mode, get_config, get_effective_skill, get_feature,
    get_feature_raw, get_git_config, get_global_dir, get_issue, get_note_raw, get_project_dir,
    get_project_statuses, get_release_raw, get_skill, get_spec_raw, get_user_skill,
    ingest_external_events, init_demo_project, init_project, is_category_committed,
    list_effective_skills, list_events_since, list_features, list_issues, list_mcp_servers,
    list_models, list_notes, list_providers, list_releases, list_skills, list_specs,
    list_user_skills, log_action, migrate_global_state, migrate_json_config_file,
    migrate_project_state, migrate_yaml_issues, move_issue, note_path_for_scope,
    remove_mcp_server, remove_mode, remove_status, set_active_mode, set_category_committed,
    update_feature, update_note, update_release, update_skill, update_user_skill,
};
use ship_module_git::{install_hooks, on_post_checkout, write_root_gitignore};
use std::env;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

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
    /// Manage MCP servers registered in ship.toml
    Mcp {
        #[command(subcommand)]
        action: McpCommands,
    },
    /// Manage AI agent providers (detect, connect, disconnect)
    Providers {
        #[command(subcommand)]
        action: ProviderCommands,
    },
    /// Migrate legacy YAML issues and JSON config to TOML
    #[command(hide = true)]
    Migrate,
}

#[derive(Subcommand, Debug)]
pub enum GitCommands {
    /// Show what is and isn't committed to git
    Status,
    /// Include a category in git commits
    Include {
        /// One of: issues, releases, features, specs, adrs, notes, agents, events.ndjson, ship.toml, templates
        category: String,
    },
    /// Exclude a category from git commits (adds to .ship/.gitignore)
    Exclude {
        /// One of: issues, releases, features, specs, adrs, notes, agents, events.ndjson, ship.toml, templates
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
    /// List MCP servers registered in ship.toml
    List,
    /// Add an HTTP/SSE MCP server
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
    /// Start the MCP stdio server (internal)
    #[command(hide = true)]
    Serve,
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
    Get { file_name: String },
    /// Replace release markdown content
    Update {
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
        /// Link this feature to a release filename
        #[arg(long)]
        release: Option<String>,
        /// Link this feature to a spec filename
        #[arg(long)]
        spec: Option<String>,
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
    Get { file_name: String },
    /// Replace feature markdown content
    Update {
        file_name: String,
        /// Full replacement content
        #[arg(short, long)]
        content: String,
    },
    /// Mark a feature as in-progress and link it to a branch
    Start {
        file_name: String,
        /// Git branch name to link (creates the branch if absent).
        /// Defaults to `feature/<file-name-without-.md>` when omitted.
        #[arg(long)]
        branch: Option<String>,
    },
    /// Check out the branch linked to a feature and regenerate agent config
    Switch { file_name: String },
    /// Mark a feature as implemented (done)
    Done { file_name: String },
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
            let tracked = match runtime::register_project(project_name, target.clone()) {
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
                    let issue_path = runtime::project::issues_dir(&project_dir)
                        .join(&from)
                        .join(&file_name);
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
                    let path = create_adr(project_dir.clone(), &title, &decision, "proposed")?;
                    println!("ADR created: {}", path.display());
                    log_action(
                        project_dir,
                        "adr create",
                        &format!("Created ADR: {}", title),
                    )?;
                }
                AdrCommands::List => {
                    let mut adrs = runtime::list_adrs(project_dir)?;
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
                    let path = runtime::find_adr_path(&project_dir, &file_name)?;
                    let content = std::fs::read_to_string(path)?;
                    println!("{}", content);
                }
                AdrCommands::Move { file_name, status } => {
                    let new_status = status
                        .parse::<runtime::AdrStatus>()
                        .map_err(|_| anyhow::anyhow!("Invalid ADR status"))?;
                    let new_path =
                        runtime::move_adr(project_dir.clone(), &file_name, new_status.clone())?;
                    println!(
                        "Moved {} to {} ({})",
                        file_name,
                        new_status,
                        new_path.display()
                    );
                    log_action(
                        project_dir,
                        "adr move",
                        &format!("Moved {} to {}", file_name, new_status),
                    )?;
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
                    NoteScope::Project => Some(get_project_dir(None)?),
                    NoteScope::User => None,
                };
                let body = content.unwrap_or_default();
                let path = create_note(scope, project_dir.clone(), &title, &body)?;
                println!("Note created: {}", path.display());
                if let Some(project_dir) = project_dir {
                    log_action(
                        project_dir,
                        "note create",
                        &format!("Created note: {}", title),
                    )?;
                }
            }
            NoteCommands::List { scope } => {
                let scope = parse_note_scope(&scope)?;
                let project_dir = match scope {
                    NoteScope::Project => Some(get_project_dir(None)?),
                    NoteScope::User => None,
                };
                let mut notes = list_notes(scope, project_dir)?;
                notes.sort_by(|a, b| b.updated.cmp(&a.updated));
                if notes.is_empty() {
                    println!("No notes found.");
                } else {
                    for note in notes {
                        println!("{} ({})", note.title, note.file_name);
                    }
                }
            }
            NoteCommands::Get { file_name, scope } => {
                let scope = parse_note_scope(&scope)?;
                let project_dir = match scope {
                    NoteScope::Project => Some(get_project_dir(None)?),
                    NoteScope::User => None,
                };
                let path = note_path_for_scope(scope, project_dir, &file_name)?;
                if !path.exists() {
                    anyhow::bail!("Note not found: {}", file_name);
                }
                println!("{}", get_note_raw(path)?);
            }
            NoteCommands::Update {
                file_name,
                content,
                scope,
            } => {
                let scope = parse_note_scope(&scope)?;
                let project_dir = match scope {
                    NoteScope::Project => Some(get_project_dir(None)?),
                    NoteScope::User => None,
                };
                let path = note_path_for_scope(scope, project_dir.clone(), &file_name)?;
                if !path.exists() {
                    anyhow::bail!("Note not found: {}", file_name);
                }
                update_note(path, &content)?;
                println!("Updated note: {}", file_name);
                if let Some(project_dir) = project_dir {
                    log_action(
                        project_dir,
                        "note update",
                        &format!("Updated note: {}", file_name),
                    )?;
                }
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
                        let project_dir = get_project_dir(None)?;
                        let skill = create_skill(&project_dir, &id, &name, &content)?;
                        log_action(
                            project_dir,
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
                        let project_dir = get_project_dir(None)?;
                        list_skills(&project_dir)?
                    }
                    SkillReadScope::User => list_user_skills()?,
                    SkillReadScope::Effective => {
                        let project_dir = get_project_dir(None)?;
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
                        let project_dir = get_project_dir(None)?;
                        get_skill(&project_dir, &id)?
                    }
                    SkillReadScope::User => get_user_skill(&id)?,
                    SkillReadScope::Effective => {
                        let project_dir = get_project_dir(None)?;
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
                        let project_dir = get_project_dir(None)?;
                        let updated =
                            update_skill(&project_dir, &id, name.as_deref(), content.as_deref())?;
                        log_action(
                            project_dir,
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
                        let project_dir = get_project_dir(None)?;
                        delete_skill(&project_dir, &id)?;
                        log_action(
                            project_dir,
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
            let project_dir = get_project_dir(None)?;
            match action {
                SpecCommands::Create { title, content } => {
                    let body = content.unwrap_or_default();
                    let path = create_spec(project_dir.clone(), &title, &body, "draft")?;
                    println!("Spec created: {}", path.display());
                    log_action(
                        project_dir,
                        "spec create",
                        &format!("Created spec: {}", title),
                    )?;
                }
                SpecCommands::List => {
                    let mut specs = list_specs(project_dir)?;
                    specs.sort_by(|a, b| b.updated.cmp(&a.updated));
                    if specs.is_empty() {
                        println!("No specs found.");
                    } else {
                        for spec in specs {
                            println!("[{}] {} ({})", spec.status, spec.title, spec.file_name);
                        }
                    }
                }
                SpecCommands::Get { file_name } => {
                    let path = runtime::project::specs_dir(&project_dir).join(&file_name);
                    if !path.exists() {
                        anyhow::bail!("Spec not found: {}", file_name);
                    }
                    let content = get_spec_raw(path)?;
                    println!("{}", content);
                }
            }
        }
        Some(Commands::Release { action }) => {
            let project_dir = get_project_dir(None)?;
            match action {
                ReleaseCommands::Create { version, content } => {
                    let body = content.unwrap_or_default();
                    let path = create_release(project_dir.clone(), &version, &body)?;
                    println!("Release created: {}", path.display());
                    log_action(
                        project_dir,
                        "release create",
                        &format!("Created release: {}", version),
                    )?;
                }
                ReleaseCommands::List => {
                    let mut releases = list_releases(project_dir)?;
                    releases.sort_by(|a, b| b.updated.cmp(&a.updated));
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
                    let path = find_release_path(&project_dir, &file_name)?;
                    let content = get_release_raw(path)?;
                    println!("{}", content);
                }
                ReleaseCommands::Update { file_name, content } => {
                    let path = find_release_path(&project_dir, &file_name)?;
                    update_release(path, &content)?;
                    println!("Updated release: {}", file_name);
                    log_action(
                        project_dir,
                        "release update",
                        &format!("Updated release: {}", file_name),
                    )?;
                }
            }
        }
        Some(Commands::Feature { action }) => {
            let project_dir = get_project_dir(None)?;
            match action {
                FeatureCommands::Create {
                    title,
                    content,
                    release,
                    spec,
                    branch,
                } => {
                    let body = content.unwrap_or_default();
                    let path = create_feature(
                        project_dir.clone(),
                        &title,
                        &body,
                        release.as_deref(),
                        spec.as_deref(),
                        branch.as_deref(),
                    )?;
                    println!("Feature created: {}", path.display());
                    log_action(
                        project_dir,
                        "feature create",
                        &format!("Created feature: {}", title),
                    )?;
                }
                FeatureCommands::List { status } => {
                    let status_filter = match status.as_deref() {
                        Some("planned") => Some(FeatureStatus::Planned),
                        Some("in-progress") => Some(FeatureStatus::InProgress),
                        Some("implemented") => Some(FeatureStatus::Implemented),
                        Some("deprecated") => Some(FeatureStatus::Deprecated),
                        Some(other) => anyhow::bail!(
                            "Unknown status: {}. Use: planned, in-progress, implemented, deprecated",
                            other
                        ),
                        None => None,
                    };
                    let mut features = list_features(project_dir, status_filter)?;
                    features.sort_by(|a, b| b.updated.cmp(&a.updated));
                    if features.is_empty() {
                        println!("No features found.");
                    } else {
                        for feature in features {
                            let release = feature.release_id.unwrap_or_else(|| "unassigned".into());
                            println!(
                                "[{}] {} ({}) release={}",
                                feature.status, feature.title, feature.file_name, release
                            );
                        }
                    }
                }
                FeatureCommands::Get { file_name } => {
                    let path = runtime::find_feature_path(&project_dir, &file_name)?;
                    let content = get_feature_raw(path)?;
                    println!("{}", content);
                }
                FeatureCommands::Update { file_name, content } => {
                    let path = runtime::find_feature_path(&project_dir, &file_name)?;
                    update_feature(path, &content)?;
                    println!("Updated feature: {}", file_name);
                    log_action(
                        project_dir,
                        "feature update",
                        &format!("Updated feature: {}", file_name),
                    )?;
                }
                FeatureCommands::Start { file_name, branch } => {
                    // Derive branch name from file name when not explicitly provided.
                    let branch = branch.unwrap_or_else(|| {
                        let base = file_name.trim_end_matches(".md");
                        format!("feature/{}", base)
                    });
                    // Create the branch if it doesn't exist
                    let branch_exists = ProcessCommand::new("git")
                        .args(["rev-parse", "--verify", &branch])
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false);
                    if !branch_exists {
                        let result = ProcessCommand::new("git")
                            .args(["checkout", "-b", &branch])
                            .status()?;
                        if !result.success() {
                            anyhow::bail!("Failed to create branch: {}", branch);
                        }
                    }
                    feature_start(project_dir.clone(), &file_name, &branch)?;
                    // Generate agent config for the new branch
                    let project_root = project_dir
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| project_dir.clone());
                    let _ = on_post_checkout(&project_dir, &branch, &project_root);
                    println!("Feature started: {} on branch {}", file_name, branch);
                    log_action(
                        project_dir,
                        "feature start",
                        &format!("Started feature: {} branch={}", file_name, branch),
                    )?;
                }
                FeatureCommands::Switch { file_name } => {
                    let path = runtime::find_feature_path(&project_dir, &file_name)?;
                    let feature = get_feature(path)?;
                    let branch = feature.metadata.branch.ok_or_else(|| {
                        anyhow::anyhow!(
                            "Feature has no linked branch — run 'ship feature start' first"
                        )
                    })?;
                    let result = ProcessCommand::new("git")
                        .args(["checkout", &branch])
                        .status()?;
                    if !result.success() {
                        anyhow::bail!("Failed to checkout branch: {}", branch);
                    }
                    let project_root = project_dir
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| project_dir.clone());
                    let _ = on_post_checkout(&project_dir, &branch, &project_root);
                    println!("Switched to feature: {} on branch {}", file_name, branch);
                }
                FeatureCommands::Done { file_name } => {
                    feature_done(project_dir.clone(), &file_name)?;
                    println!("Feature done: {}", file_name);
                    log_action(
                        project_dir,
                        "feature done",
                        &format!("Marked feature implemented: {}", file_name),
                    )?;
                }
            }
        }
        Some(Commands::Event { action }) => {
            let project_dir = get_project_dir(None)?;
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
            }
        }
        Some(Commands::Projects { action }) => match action {
            ProjectCommands::List => {
                let projects = runtime::list_registered_projects()?;
                for p in projects {
                    println!("- {} ({})", p.name, p.path.display());
                }
            }
            ProjectCommands::Track { name, path } => {
                runtime::register_project(name.clone(), path.clone())?;
                println!("Now tracking project: {} ({})", name, path.display());
            }
            ProjectCommands::Untrack { path } => {
                runtime::unregister_project(path.clone())?;
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
            let project_dir = get_project_dir(None)?;
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
                        "events.ndjson",
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
            let project_dir = get_project_dir(None)?;
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
                                let path =
                                    create_issue(project_dir.clone(), &title, &desc, "backlog")?;
                                println!("Created issue: {}", path.display());
                                log_action(
                                    project_dir,
                                    "issue create",
                                    &format!("Ghost promoted: {}", title),
                                )?;
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
            let project_dir = get_project_dir(None)?;
            ensure_builtin_plugin_namespaces(&project_dir)?;
            handle_time_command(action, &project_dir)?;
        }
        Some(Commands::Mcp { action }) => {
            match action {
                McpCommands::Serve => {
                    // Handled by the main unitary binary as it requires async
                }
                McpCommands::List => {
                    let project_dir = get_project_dir(None)?;
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
                McpCommands::Add {
                    id,
                    name,
                    url,
                    disabled,
                } => {
                    let project_dir = get_project_dir(None)?;
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
                McpCommands::AddStdio {
                    id,
                    name,
                    command,
                    args,
                } => {
                    let project_dir = get_project_dir(None)?;
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
                McpCommands::Remove { id } => {
                    let project_dir = get_project_dir(None)?;
                    remove_mcp_server(Some(project_dir), &id)?;
                    println!("Removed MCP server '{}'.", id);
                }
            }
        }
        Some(Commands::Providers { action }) => {
            let project_dir = get_project_dir(None)?;
            match action {
                ProviderCommands::List => {
                    let providers = list_providers(&project_dir)?;
                    println!("{:<12} {:<20} {:<10} {:<10} {}", "ID", "NAME", "INSTALLED", "CONNECTED", "VERSION");
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
        Some(Commands::Migrate) => {
            let project_dir = get_project_dir(None)?;
            let global_dir = get_global_dir()?;
            let global = migrate_global_state(&global_dir)?;
            let project = migrate_project_state(&project_dir)?;
            let issues = migrate_yaml_issues(&project_dir)?;
            let config = migrate_json_config_file(&project_dir)?;
            let ids = backfill_issue_ids(&project_dir)?;
            println!(
                "Migration complete:\n- file namespace copies: copied={} skipped={} conflicts={}\n- project DB: {} (applied {})\n- global DB: {} (applied {})\n- registry: {} -> {} entries (normalized {})\n- app_state paths normalized: {}\n- issue format: {} issue{} converted to TOML, {} ID{} backfilled{}.",
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
                issues,
                if issues == 1 { "" } else { "s" },
                ids,
                if ids == 1 { "" } else { "s" },
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
                    for status in runtime::ISSUE_STATUSES {
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
