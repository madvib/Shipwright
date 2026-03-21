use clap::Subcommand;

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
