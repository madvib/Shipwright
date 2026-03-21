use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub use crate::commands::{AgentCommands, EventsCommands, JobCommands, McpCommands, SkillCommands};

const AFTER_HELP: &str = "\x1b[1mGetting Started:\x1b[0m
  ship init              Scaffold .ship/ in the current project
  ship agent create id   Create an agent definition
  ship use <agent-id>    Activate an agent (compiles immediately)
  ship compile           Re-compile after editing agent config

\x1b[1mLearn More:\x1b[0m
  ship help topics       List available help topics
  ship help <topic>      Show detailed help for a topic
  https://getship.dev/docs";

#[derive(Parser, Debug)]
#[command(name = "ship")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Agent configuration compiler — compose, compile, distribute")]
#[command(after_help = AFTER_HELP)]
#[command(disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // ── Setup ────────────────────────────────────────────────────────────────
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

    /// Authenticate with getship.dev
    Login,

    /// Sign out
    Logout,

    /// Show current identity
    Whoami,

    /// Read or write user preferences (~/.ship/config.toml)
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },

    // ── Daily Use ────────────────────────────────────────────────────────────
    /// Activate an agent for the current (or specified) directory
    Use {
        /// Agent ID or registry reference (e.g. rust-expert, @org/agent)
        agent_id: String,
        /// Bind to this path instead of the current directory
        #[arg(long)]
        path: Option<PathBuf>,
        /// Compile immediately after activating
        #[arg(long, default_value_t = true)]
        compile: bool,
    },

    /// Show the active agent and compilation status
    Status {
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Compile the active agent to provider-native config files
    Compile {
        /// Compile for a specific provider only (claude, gemini, codex, cursor)
        #[arg(long)]
        provider: Option<String>,
        /// Preview output without writing any files
        #[arg(long)]
        dry_run: bool,
        /// Recompile automatically when agent files change
        #[arg(long)]
        watch: bool,
        /// Path to project root (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Validate .ship/ config — checks TOML, skill refs, MCP fields, permissions
    Validate {
        /// Validate a single agent (omit to validate all)
        #[arg(long)]
        agent: Option<String>,
        /// Emit errors as JSON array instead of human-readable output
        #[arg(long)]
        json: bool,
        /// Path to project root (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    // ── Agent Configuration ──────────────────────────────────────────────────
    /// Manage agents (create, list, edit, delete, clone)
    Agent {
        #[command(subcommand)]
        action: AgentCommands,
    },

    /// Manage agent skills (add, list, remove, create)
    Skill {
        #[command(subcommand)]
        action: SkillCommands,
    },

    /// Manage MCP servers (serve, add, list, remove)
    Mcp {
        #[command(subcommand)]
        action: McpCommands,
    },

    // ── Publishing ───────────────────────────────────────────────────────────
    /// Install all dependencies declared in .ship/ship.toml
    Install {
        /// Fail if the lockfile would change rather than updating it
        #[arg(long)]
        frozen: bool,
        /// Skip install tracking (no network POST to registry)
        #[arg(long)]
        offline: bool,
    },

    /// Publish this package to the Ship registry
    Publish {
        /// Preview what would be published without making any network requests
        #[arg(long)]
        dry_run: bool,
        /// Dist-tag for pre-release publishing (e.g. beta, next)
        #[arg(long)]
        tag: Option<String>,
    },

    /// Add a package dependency to .ship/ship.toml and install it
    Add {
        /// Package path with optional version: github.com/owner/repo[@version]
        package: String,
    },

    /// Import an agent from a getship.dev URL, local path, or provider config
    Import {
        /// A getship.dev URL (e.g. https://getship.dev/p/<id>), local path, or provider config
        source: String,
    },

    // ── Inspection ───────────────────────────────────────────────────────────
    /// Query the project event log
    Events {
        #[command(subcommand)]
        action: EventsCommands,
    },

    /// Browse workflow state in a terminal UI (read-only)
    View,

    /// Show detailed help for a topic (run `ship help topics` to list)
    Help {
        /// Topic name (e.g. agents, skills, mcp, compile, providers)
        topic: Option<String>,
    },

    // ── Hidden / Internal ────────────────────────────────────────────────────
    #[command(hide = true)]
    Surface {
        /// Write output to docs/surface.md
        #[arg(long)]
        emit: bool,
        /// Diff against committed docs/surface.md; exit 1 if drift detected
        #[arg(long)]
        check: bool,
    },

    #[command(hide = true)]
    Job {
        #[command(subcommand)]
        action: JobCommands,
    },

    #[command(hide = true)]
    Adrs,

    #[command(hide = true)]
    Notes,

    #[command(hide = true)]
    Migrate,

    #[command(hide = true)]
    Diff {
        #[arg(long)]
        milestone: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Get a config value (e.g. ship config get terminal.program)
    Get {
        /// Dot-path key (e.g. terminal.program, dispatch.confirm, worktrees.dir)
        key: String,
    },
    /// Set a config value (e.g. ship config set terminal.program wt)
    Set {
        /// Dot-path key
        key: String,
        /// Value to set
        value: String,
    },
    /// List all set config values
    List,
    /// Show config file path
    Path,
}
