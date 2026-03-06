use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "ship")]
#[command(version = env!("SHIP_VERSION_STRING"))]
#[command(
    about = "Workspace-first AI-native software lifecycle CLI",
    long_about = None
)]
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
    /// Manage MCP servers registered in .ship/agents/mcp.toml. Runs the server if no subcommand is provided.
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
    /// Developer-only maintenance and migration operations
    #[command(hide = true)]
    Dev {
        #[command(subcommand)]
        action: DevCommands,
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
    /// List MCP servers registered in .ship/agents/mcp.toml
    List,
    /// Export MCP server registry to an AI client's config file
    Export {
        /// Target AI client: claude, codex, or gemini
        #[arg(short, long)]
        target: String,
    },
    /// Import MCP server registry from an AI client's config file
    Import {
        /// Provider ID: claude, codex, or gemini
        provider: String,
    },
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
    /// Import provider-managed state back into Ship (MCP servers + permissions)
    Import {
        /// Provider ID (claude, gemini, codex). If omitted, imports all connected providers.
        id: Option<String>,
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
    /// Install a skill from a Git source (GitHub shorthand, URL, or local path)
    Install {
        /// Source repo: owner/repo, git URL, or local path
        source: String,
        /// Skill ID (directory name containing SKILL.md)
        id: String,
        /// Git ref to clone
        #[arg(long, default_value = "main")]
        git_ref: String,
        /// Subpath in repo to search
        #[arg(long, default_value = ".")]
        repo_path: String,
        /// Scope: user (default) or project
        #[arg(long, default_value = "user")]
        scope: String,
        /// Overwrite destination if already installed
        #[arg(long, default_value_t = false)]
        force: bool,
    },
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
        /// Workspace branch/id. Defaults to the active workspace.
        #[arg(long)]
        workspace: Option<String>,
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
    Switch {
        branch: String,
        /// Optional workspace mode override to apply while this workspace is active
        #[arg(long)]
        mode: Option<String>,
    },
    /// Create/update a workspace runtime record (git checkout optional)
    Create {
        branch: String,
        /// Optional workspace type: feature | refactor | experiment | hotfix
        #[arg(long = "type")]
        workspace_type: Option<String>,
        /// Link this workspace to a feature id
        #[arg(long)]
        feature: Option<String>,
        /// Title for an auto-created feature when --type feature is used without --feature
        #[arg(long)]
        feature_title: Option<String>,
        /// Link this workspace to a spec id
        #[arg(long)]
        spec: Option<String>,
        /// Link this workspace to a release id
        #[arg(long)]
        release: Option<String>,
        /// Optional workspace mode override for this branch workspace
        #[arg(long)]
        mode: Option<String>,
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
    /// Manage execution sessions within a long-lived workspace
    Session {
        #[command(subcommand)]
        action: WorkspaceSessionCommands,
    },
    /// Open a workspace in an installed IDE (cursor, vscode, zed)
    Open {
        /// Branch workspace key (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
        /// IDE/editor id (cursor, vscode, zed). Auto-selects when omitted.
        #[arg(long)]
        editor: Option<String>,
    },
    /// Mark a workspace as archived
    Archive { branch: String },
}

#[derive(Subcommand, Debug)]
pub enum WorkspaceSessionCommands {
    /// Start a new session in a workspace (defaults to current branch workspace)
    Start {
        /// Branch workspace key (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
        /// Session goal/intention
        #[arg(long)]
        goal: Option<String>,
        /// Optional mode override for this session/workspace
        #[arg(long)]
        mode: Option<String>,
        /// Primary provider to compile/export for this session
        #[arg(long)]
        provider: Option<String>,
    },
    /// End the active session in a workspace (defaults to current branch workspace)
    End {
        /// Branch workspace key (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
        /// Session summary to fold back into planning artifacts
        #[arg(long)]
        summary: Option<String>,
        /// Feature IDs updated during the session
        #[arg(long = "updated-feature")]
        updated_feature: Vec<String>,
        /// Spec IDs updated during the session
        #[arg(long = "updated-spec")]
        updated_spec: Vec<String>,
    },
    /// Show the active session for a workspace (defaults to current branch workspace)
    Status {
        /// Branch workspace key (defaults to current git branch)
        #[arg(long)]
        branch: Option<String>,
    },
    /// List recent sessions (workspace-filtered when branch is provided)
    List {
        /// Branch workspace key
        #[arg(long)]
        branch: Option<String>,
        /// Max sessions to return
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
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

#[derive(Subcommand, Debug)]
pub enum DevCommands {
    /// Migrate legacy YAML issues and JSON config to TOML
    Migrate {
        /// Re-run startup markdown imports even if already marked complete
        #[arg(long, default_value_t = false)]
        force: bool,
    },
}
