use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    /// List available agents
    List {
        /// Show only global agents (~/.ship/agents/)
        #[arg(long)]
        local: bool,
        /// Show only project agents (.ship/agents/)
        #[arg(long)]
        project: bool,
    },
    /// Create a new agent (project-local by default)
    Create {
        /// Agent ID (lowercase, hyphens — e.g. rust-expert)
        name: String,
        /// Create in ~/.ship/agents/ instead of .ship/agents/
        #[arg(long)]
        global: bool,
    },
    /// Open an agent in $EDITOR
    Edit {
        name: String,
        /// Editor to use (defaults to $EDITOR)
        #[arg(long)]
        editor: Option<String>,
    },
    /// Delete an agent
    Delete {
        name: String,
    },
    /// Clone an agent under a new ID
    Clone {
        source: String,
        target: String,
    },
    /// Append a timestamped log entry to .ship/agent.log (agent-facing)
    #[command(hide = true)]
    Log {
        message: String,
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
