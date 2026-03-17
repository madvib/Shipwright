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
    },

    // ── Auth (lazy — no account needed for core features) ────────────────────
    /// Authenticate with getship.dev
    Login,
    /// Sign out
    Logout,
    /// Show current identity
    Whoami,

    // ── Mode activation ───────────────────────────────────────────────────────
    /// Activate a mode for the current (or specified) directory
    Use {
        /// Mode ID or registry reference (e.g. rust-expert, @org/mode, https://...)
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

    /// List available modes
    Modes {
        /// Show only ~/.ship/modes/
        #[arg(long)]
        local: bool,
        /// Show only .ship/modes/
        #[arg(long)]
        project: bool,
        /// Show only cloud-saved modes (requires login)
        #[arg(long)]
        cloud: bool,
    },

    /// Manage mode definitions
    Mode {
        #[command(subcommand)]
        action: ModeCommands,
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

    // ── Import / Export ───────────────────────────────────────────────────────
    /// Detect and import existing provider configs into .ship/agents/
    Import {
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Export compiled output for a specific provider (alias for compile --provider)
    Export {
        /// Provider ID: claude, gemini, codex, cursor
        provider: String,
        /// Download all formats as a zip archive
        #[arg(long)]
        zip: bool,
    },

    // ── Cloud sync (account required) ─────────────────────────────────────────
    /// Sync modes with your Ship cloud library
    Sync {
        /// Download cloud modes to ~/.ship/modes/
        #[arg(long)]
        pull: bool,
        /// Upload local modes to cloud
        #[arg(long)]
        push: bool,
    },

    // ── Local server ──────────────────────────────────────────────────────────
    /// Manage the local Ship server (port 7701 — used by Ship Studio web app)
    Server {
        #[arg(long)]
        start: bool,
        #[arg(long)]
        stop: bool,
        #[arg(long)]
        status: bool,
        #[arg(long)]
        port: Option<u16>,
    },

    // ── Job coordination loop ─────────────────────────────────────────────────
    /// Claim the next pending job, create a worktree, and print a ready message
    Next {
        /// Override the worktrees root directory (default: ~/dev/ship-worktrees)
        #[arg(long)]
        worktrees_dir: Option<std::path::PathBuf>,
    },

    /// Reset a failed or stalled job so it can be re-claimed by `ship next`
    Retry {
        /// Job ID or unique prefix
        id: String,
        /// Override the worktrees root directory (default: ~/dev/ship-worktrees)
        #[arg(long)]
        worktrees_dir: Option<std::path::PathBuf>,
    },

    /// Run tests on the job branch; merge into current branch on pass
    Gate {
        /// Job ID or unique prefix
        id: String,
        /// Override the worktrees root directory (default: ~/dev/ship-worktrees)
        #[arg(long)]
        worktrees_dir: Option<std::path::PathBuf>,
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

    // ── Agent namespace (agent-facing; hidden from user help) ─────────────────
    /// Agent-facing commands (called from skills/scripts, not user-facing)
    #[command(hide = true)]
    Agent {
        #[command(subcommand)]
        action: AgentCommands,
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
pub enum AgentCommands {
    /// Append a timestamped log entry to .ship/agent.log
    Log {
        message: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ModeCommands {
    /// Create a new mode (project-local by default)
    Create {
        /// Mode ID (lowercase, hyphens — e.g. rust-expert)
        name: String,
        /// Create in ~/.ship/modes/ instead of .ship/modes/
        #[arg(long)]
        global: bool,
    },
    /// Open a mode in $EDITOR (or launch Ship Studio web app)
    Edit {
        name: String,
        /// Editor to use (defaults to $EDITOR)
        #[arg(long)]
        editor: Option<String>,
    },
    /// Delete a mode
    Delete {
        name: String,
    },
    /// Clone a mode under a new ID
    Clone {
        source: String,
        target: String,
    },
    /// Publish a mode to the Ship marketplace (requires account)
    Publish {
        name: String,
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
    /// Update all installed skills to their latest versions
    Update,
    /// Scaffold a new skill following the Agent Skills spec
    Create {
        id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    /// Validate a skill directory against the Agent Skills spec
    Validate {
        path: PathBuf,
    },
    /// Publish a skill to the Ship registry (requires account)
    Publish {
        path: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum McpCommands {
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
    /// Test MCP server connectivity
    Probe {
        /// Specific server to probe (omit to probe all)
        id: Option<String>,
    },
}
