use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "ship")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Agent configuration studio — compose, compile, distribute")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scaffold .ship/ in the current project, or configure ~/.ship/ globally
    Init {
        /// Configure ~/.ship/ identity and defaults instead of current project
        #[arg(long)]
        global: bool,
        /// Default provider target (claude, gemini, codex, cursor)
        #[arg(long)]
        provider: Option<String>,
        /// Overwrite existing .ship/ configuration
        #[arg(long)]
        force: bool,
    },

    // ── Auth (lazy — no account needed for core features) ────────────────────
    /// Authenticate with getship.dev
    Login,
    /// Sign out
    Logout,
    /// Show current identity
    Whoami,

    // ── Agent profile activation ──────────────────────────────────────────────
    /// Activate an agent profile for the current (or specified) directory
    Use {
        /// Agent profile ID or registry reference (e.g. rust-expert, @org/profile, https://...)
        mode: String,
        /// Bind to this path instead of the current directory
        #[arg(long)]
        path: Option<PathBuf>,
        /// Compile immediately after activating
        #[arg(long, default_value_t = true)]
        compile: bool,
    },

    /// Show the active mode and compilation status for the current directory
    Status {
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// List available agent profiles
    AgentProfiles {
        /// Show only ~/.ship/modes/
        #[arg(long)]
        local: bool,
        /// Show only .ship/modes/
        #[arg(long)]
        project: bool,
        /// Show only cloud-saved profiles (requires login)
        #[arg(long)]
        cloud: bool,
    },

    /// Manage agent profile definitions
    AgentProfile {
        #[command(subcommand)]
        action: AgentProfileCommands,
    },

    // ── Compilation ───────────────────────────────────────────────────────────
    /// Compile the active mode to provider-native config files
    Compile {
        /// Compile for a specific provider only (claude, gemini, codex, cursor)
        #[arg(long)]
        provider: Option<String>,
        /// Preview output without writing any files
        #[arg(long)]
        dry_run: bool,
        /// Recompile automatically when mode or agent files change
        #[arg(long)]
        watch: bool,
        /// Path to project root (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    // ── Skills ────────────────────────────────────────────────────────────────
    /// Manage agent skills
    Skill {
        #[command(subcommand)]
        action: SkillCommands,
    },

    // ── MCP servers ───────────────────────────────────────────────────────────
    /// Manage MCP servers
    Mcp {
        #[command(subcommand)]
        action: McpCommands,
    },

    // ── Registry ─────────────────────────────────────────────────────────────
    /// Install all dependencies declared in .ship/ship.toml
    Install {
        /// Fail if the lockfile would change rather than updating it
        #[arg(long)]
        frozen: bool,
    },

    /// Add a package dependency to .ship/ship.toml and install it
    Add {
        /// Package path with optional version: github.com/owner/repo[@version]
        package: String,
    },

    // ── Import / Export ───────────────────────────────────────────────────────
    /// Import a profile from a getship.dev URL, local path, or provider config
    Import {
        /// A getship.dev URL (e.g. https://getship.dev/p/<id>), local path, or provider config directory
        source: String,
    },

    /// Export compiled output for a specific provider (alias for compile --provider)
    Export {
        /// Provider ID: claude, gemini, codex, cursor
        provider: String,
        /// Download all formats as a zip archive
        #[arg(long)]
        zip: bool,
    },

    // ── Profile sync (account required) ──────────────────────────────────────
    /// Push and pull agent profiles from the Ship cloud
    Profile {
        #[command(subcommand)]
        action: ProfileSyncCommands,
    },

    // ── Job queue ─────────────────────────────────────────────────────────────
    /// Manage the agent job queue
    Job {
        #[command(subcommand)]
        action: JobCommands,
    },

    // ── Project visibility ────────────────────────────────────────────────────
    /// List architecture decision records in the current project
    Adrs,

    /// List notes in the current project
    Notes,

    /// Migrate notes and ADRs from old ship.db to platform.db
    Migrate,

    // ── Validation ────────────────────────────────────────────────────────────
    /// Validate .ship/ config before compile — checks TOML, skill refs, MCP fields, permissions
    Validate {
        /// Validate a single profile (omit to validate all)
        #[arg(long)]
        profile: Option<String>,
        /// Emit errors as JSON array instead of human-readable output
        #[arg(long)]
        json: bool,
        /// Path to project root (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    // ── Capability diff ───────────────────────────────────────────────────────
    /// Show capability progress delta for the active milestone
    Diff {
        /// Milestone target id. If omitted, uses the first active milestone found.
        #[arg(long)]
        milestone: Option<String>,
    },

    // ── Event log ─────────────────────────────────────────────────────────────
    /// Query the project event log
    Events {
        #[command(subcommand)]
        action: EventsCommands,
    },

    // ── Agent namespace (agent-facing; hidden from user help) ─────────────────
    /// Agent-facing commands (called from skills/scripts, not user-facing)
    #[command(hide = true)]
    Agent {
        #[command(subcommand)]
        action: AgentCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProfileSyncCommands {
    /// Upload .ship/agents/profiles/ to the Ship API (requires login)
    Push,
    /// Download profiles from the Ship API (requires login)
    Pull {
        /// Pull only the named profile
        name: Option<String>,
        /// Overwrite existing profiles without prompting
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum JobCommands {
    /// Create a new job
    Create {
        /// Job kind/category (e.g. feature, infra, test, spec)
        #[arg(long, default_value = "feature")]
        kind: String,
        /// Human-readable job title
        title: String,
        /// Milestone group (e.g. "M1: Auth & Server")
        #[arg(long)]
        milestone: Option<String>,
        /// Optional description
        #[arg(long)]
        description: Option<String>,
        /// Linked branch
        #[arg(long)]
        branch: Option<String>,
    },
    /// List jobs
    List {
        /// Filter by status (pending, running, done, blocked)
        #[arg(long)]
        status: Option<String>,
        /// Filter by branch
        #[arg(long)]
        branch: Option<String>,
        /// Filter by milestone
        #[arg(long)]
        milestone: Option<String>,
    },
    /// Update a job's status
    Update {
        /// Job ID prefix (unique prefix is sufficient)
        id: String,
        /// New status: pending, running, done, blocked
        status: String,
    },
    /// Mark a job complete: stage files in job scope, commit, set status=complete
    Done {
        /// Job ID or unique prefix
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum EventsCommands {
    /// List events from the project event log
    List {
        /// Show events since this timestamp (ISO 8601) or relative (e.g. "1h", "24h")
        #[arg(long)]
        since: Option<String>,
        /// Filter by actor
        #[arg(long)]
        actor: Option<String>,
        /// Filter by entity type (workspace, session, note, etc.)
        #[arg(long)]
        entity: Option<String>,
        /// Filter by action (create, update, delete, etc.)
        #[arg(long)]
        action: Option<String>,
        /// Maximum number of events to show (default: 50)
        #[arg(long, default_value = "50")]
        limit: u32,
        /// Output as JSON array instead of table
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum AgentCommands {
    /// Append a timestamped log entry to .ship/agent.log
    Log {
        message: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum AgentProfileCommands {
    /// Create a new agent profile (project-local by default)
    Create {
        /// Profile ID (lowercase, hyphens — e.g. rust-expert)
        name: String,
        /// Create in ~/.ship/modes/ instead of .ship/modes/
        #[arg(long)]
        global: bool,
    },
    /// Open an agent profile in $EDITOR (or launch Ship Studio web app)
    Edit {
        name: String,
        /// Editor to use (defaults to $EDITOR)
        #[arg(long)]
        editor: Option<String>,
    },
    /// Delete an agent profile
    Delete {
        name: String,
    },
    /// Clone an agent profile under a new ID
    Clone {
        source: String,
        target: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum SkillCommands {
    /// Install a skill from the registry or a local path
    Add {
        /// Skill ID, registry reference, local path, or GitHub URL
        source: String,
        /// Skill ID to install (required when repo has multiple skills)
        #[arg(long)]
        skill: Option<String>,
        /// Install to ~/.ship/skills/ instead of .ship/agents/skills/
        #[arg(long)]
        global: bool,
    },
    /// List installed skills
    List,
    /// Remove a skill
    Remove {
        id: String,
        #[arg(long)]
        global: bool,
    },
    /// Scaffold a new skill following the Agent Skills spec
    Create {
        id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum McpCommands {
    /// Run the Ship MCP server (stdio by default; --http for HTTP daemon)
    Serve {
        /// Serve over HTTP instead of stdio
        #[arg(long)]
        http: bool,
        /// HTTP port (requires --http, default: 3000)
        #[arg(long, default_value = "3000")]
        port: u16,
    },

    /// Register an MCP server (HTTP/SSE transport)
    Add {
        /// Stable server ID
        id: String,
        /// Human-readable name (defaults to id)
        #[arg(long)]
        name: Option<String>,
        /// Server URL (required for HTTP/SSE transport)
        #[arg(long)]
        url: Option<String>,
        /// Register to ~/.ship/mcp/ instead of .ship/agents/mcp.toml
        #[arg(long)]
        global: bool,
    },
    /// Register a stdio MCP server
    AddStdio {
        id: String,
        command: String,
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        global: bool,
    },
    /// List configured MCP servers
    List,
    /// Remove an MCP server
    Remove {
        id: String,
    },
}
