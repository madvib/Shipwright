use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub use crate::commands::{AgentCommands, HookCommands, McpCommands, NetworkCommands, SkillCommands, VarsCommands};
#[cfg(feature = "unstable")]
pub use crate::commands::EventsCommands;

const AFTER_HELP: &str = "\x1b[1mDaily Workflow:\x1b[0m
  ship init              Start here — scaffold .ship/
  ship use <agent-id>    Activate an agent (compiles immediately)
  ship status            Show active agent
  ship compile           Re-compile after editing config

\x1b[1mPackages:\x1b[0m
  ship add <package>     Add a dependency
  ship install           Resolve all dependencies
  ship audit             Scan for hidden Unicode (security)
  ship publish           Share your package on the registry

\x1b[1mConfiguration:\x1b[0m
  ship agents create <n> Create an agent definition
  ship skills add <src>  Install a skill
  ship mcp add-stdio ... Register an MCP server
  ship config set k v    Set a user preference
  ship convert <source>  Convert provider configs to .ship/

\x1b[1mLearn More:\x1b[0m
  ship docs topics       List help topics
  ship docs <topic>      Detailed help for a topic
  https://getship.dev/docs";

#[derive(Parser, Debug)]
#[command(name = "ship")]
#[command(version = crate::build_version())]
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
        /// Extra skill IDs to inject at compile time (repeatable)
        #[arg(long = "with", value_name = "SKILL_ID")]
        with_skills: Vec<String>,
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
    #[command(name = "agents")]
    Agents {
        #[command(subcommand)]
        action: AgentCommands,
    },

    /// Manage agent skills (add, list, remove, create)
    #[command(name = "skills")]
    Skill {
        #[command(subcommand)]
        action: SkillCommands,
    },

    /// Read and write skill variable state (set, get, edit, append, reset)
    Vars {
        #[command(subcommand)]
        action: VarsCommands,
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
        /// Publish only this export (e.g. agents/skills/tdd, agents/profiles/red-green.toml)
        export_path: Option<String>,
        /// Preview what would be published without making any network requests
        #[arg(long)]
        dry_run: bool,
        /// Dist-tag for pre-release publishing (e.g. beta, next)
        #[arg(long)]
        tag: Option<String>,
    },

    /// Add a package dependency or import config from a Studio share link
    Add {
        /// Package path with optional version: github.com/owner/repo[@version]
        #[arg(required_unless_present = "from")]
        package: Option<String>,
        /// Import agent config from a Studio share link (MCP or JSON)
        #[arg(long, conflicts_with = "package")]
        from: Option<String>,
    },

    /// Scan files for hidden Unicode characters (prompt injection vectors)
    Audit {
        /// Path to scan (defaults to .ship/ in current directory)
        #[arg(long)]
        path: Option<PathBuf>,
        /// Emit findings as JSON array
        #[arg(long)]
        json: bool,
    },

    /// Convert provider config files (CLAUDE.md, .cursor/) into .ship/ format
    Convert {
        /// A getship.dev URL (e.g. https://getship.dev/p/<id>), local path, or provider config
        source: String,
    },

    // ── Inspection ───────────────────────────────────────────────────────────
    /// Query the project event log
    #[cfg(feature = "unstable")]
    Events {
        #[command(subcommand)]
        action: EventsCommands,
    },

    /// Show detailed help for a topic (run `ship docs topics` to list)
    Docs {
        /// Topic name (e.g. agents, skills, mcp, compile, providers)
        topic: Option<String>,
    },

    #[cfg(feature = "unstable")]
    #[command(hide = true)]
    Adrs,

    /// Provider hook integration (before-tool, after-tool, session-end)
    #[command(hide = true)]
    Hook {
        #[command(subcommand)]
        action: HookCommands,
    },

    /// Manage the Ship network daemon for cross-agent communication
    Network {
        #[command(subcommand)]
        action: NetworkCommands,
    },

    /// Launch Ship Studio -- visual IDE for skills and agents
    Studio {
        /// HTTP port for the Studio MCP server
        #[arg(long, default_value_t = 51741)]
        port: u16,
        /// Open the Studio in your default browser
        #[arg(long)]
        open: bool,
    },


    /// Real-time dashboard — workspaces, sessions, events (htop for Ship)
    #[cfg(feature = "unstable")]
    Top,

    /// Show help — same as --help
    Help,
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
