use serde::{Deserialize, Serialize};
use specta::Type;

// ─── Data types ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogKind {
    Skill,
    McpServer,
}

/// A browsable catalog entry — either a community skill or an MCP server.
///
/// For alpha this is an embedded static list. In the future it can be supplemented
/// or replaced by an API call (agentskills.io, mcp.so, or a Ship-hosted registry).
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct CatalogEntry {
    /// Stable identifier (slug).
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: CatalogKind,
    pub tags: Vec<String>,
    /// Author or organisation name.
    pub author: Option<String>,
    /// Canonical URL for the skill/server (docs or registry page).
    pub source_url: Option<String>,
    /// Shell command to install the server (e.g. `npx -y @mcp/server-filesystem`).
    pub install_command: Option<String>,
    /// For MCP servers: the runtime command (e.g. `npx`).
    pub command: Option<String>,
    /// For MCP servers: default args (placeholders like `{path}` are replaced at install time).
    pub args: Vec<String>,
    /// For skills: the template content. `$ARGUMENTS` is the user input placeholder.
    pub skill_template: Option<String>,
}

// ─── Embedded catalog ─────────────────────────────────────────────────────────

struct StaticEntry {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    kind: CatalogKind,
    tags: &'static [&'static str],
    author: Option<&'static str>,
    source_url: Option<&'static str>,
    install_command: Option<&'static str>,
    command: Option<&'static str>,
    args: &'static [&'static str],
    skill_template: Option<&'static str>,
}

fn to_entry(s: &StaticEntry) -> CatalogEntry {
    CatalogEntry {
        id: s.id.to_string(),
        name: s.name.to_string(),
        description: s.description.to_string(),
        kind: s.kind.clone(),
        tags: s.tags.iter().map(|t| t.to_string()).collect(),
        author: s.author.map(|a| a.to_string()),
        source_url: s.source_url.map(|u| u.to_string()),
        install_command: s.install_command.map(|c| c.to_string()),
        command: s.command.map(|c| c.to_string()),
        args: s.args.iter().map(|a| a.to_string()).collect(),
        skill_template: s.skill_template.map(|t| t.to_string()),
    }
}

static EMBEDDED: &[StaticEntry] = &[
    // ── Skills ────────────────────────────────────────────────────────────────
    StaticEntry {
        id: "code-review",
        name: "Code Review",
        description: "Review code for correctness, security, performance, and style. Returns structured feedback.",
        kind: CatalogKind::Skill,
        tags: &["review", "quality", "security"],
        author: Some("community"),
        source_url: Some("https://agentskills.io"),
        install_command: None,
        command: None,
        args: &[],
        skill_template: Some(
            "Review the following code for correctness, security, performance, and style.\n\
             Return feedback as a markdown list grouped by severity (critical / major / minor).\n\n\
             Code to review:\n$ARGUMENTS",
        ),
    },
    StaticEntry {
        id: "test-generator",
        name: "Test Generator",
        description: "Generate unit tests for a given function or module. Supports multiple test frameworks.",
        kind: CatalogKind::Skill,
        tags: &["testing", "quality", "automation"],
        author: Some("community"),
        source_url: Some("https://agentskills.io"),
        install_command: None,
        command: None,
        args: &[],
        skill_template: Some(
            "Generate comprehensive unit tests for the following code.\n\
             Use the project's existing test framework and conventions.\n\
             Include happy path, edge cases, and error cases.\n\n\
             Code to test:\n$ARGUMENTS",
        ),
    },
    StaticEntry {
        id: "commit-message",
        name: "Commit Message",
        description: "Generate a conventional commit message from a diff or description of changes.",
        kind: CatalogKind::Skill,
        tags: &["git", "workflow", "conventions"],
        author: Some("community"),
        source_url: Some("https://agentskills.io"),
        install_command: None,
        command: None,
        args: &[],
        skill_template: Some(
            "Generate a conventional commit message for the following changes.\n\
             Format: <type>(<scope>): <description>\n\
             Types: feat, fix, docs, refactor, test, chore, perf, ci\n\
             Keep the subject line under 72 characters. Add a body if needed.\n\n\
             Changes:\n$ARGUMENTS",
        ),
    },
    StaticEntry {
        id: "debug-assistant",
        name: "Debug Assistant",
        description: "Walk through a bug systematically: hypothesis → reproduction → fix → verification.",
        kind: CatalogKind::Skill,
        tags: &["debugging", "troubleshooting"],
        author: Some("community"),
        source_url: Some("https://agentskills.io"),
        install_command: None,
        command: None,
        args: &[],
        skill_template: Some(
            "Debug the following issue step by step:\n\
             1. State your hypothesis about the root cause\n\
             2. Identify the smallest reproduction case\n\
             3. Propose a fix with rationale\n\
             4. Describe how to verify the fix\n\n\
             Issue:\n$ARGUMENTS",
        ),
    },
    StaticEntry {
        id: "doc-writer",
        name: "Documentation Writer",
        description: "Generate clear, accurate documentation for a function, module, or API.",
        kind: CatalogKind::Skill,
        tags: &["docs", "documentation", "api"],
        author: Some("community"),
        source_url: Some("https://agentskills.io"),
        install_command: None,
        command: None,
        args: &[],
        skill_template: Some(
            "Write comprehensive documentation for the following code.\n\
             Include: purpose, parameters, return values, errors, and usage examples.\n\
             Match the project's existing doc style.\n\n\
             Code:\n$ARGUMENTS",
        ),
    },
    StaticEntry {
        id: "refactor-plan",
        name: "Refactor Planner",
        description: "Produce a step-by-step refactoring plan with rationale, risk assessment, and rollback notes.",
        kind: CatalogKind::Skill,
        tags: &["refactoring", "architecture", "planning"],
        author: Some("community"),
        source_url: Some("https://agentskills.io"),
        install_command: None,
        command: None,
        args: &[],
        skill_template: Some(
            "Create a refactoring plan for the following code or system.\n\
             Include:\n\
             - Goal and motivation\n\
             - Step-by-step changes (each independently deployable)\n\
             - Risk assessment per step\n\
             - Rollback strategy\n\n\
             Target:\n$ARGUMENTS",
        ),
    },
    // ── MCP Servers ───────────────────────────────────────────────────────────
    StaticEntry {
        id: "mcp-filesystem",
        name: "Filesystem",
        description: "Read and write files on the local filesystem with configurable allowed paths.",
        kind: CatalogKind::McpServer,
        tags: &["filesystem", "files", "local", "official"],
        author: Some("Anthropic"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem"),
        install_command: Some("npx -y @modelcontextprotocol/server-filesystem"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-filesystem", "{path}"],
        skill_template: None,
    },
    StaticEntry {
        id: "mcp-github",
        name: "GitHub",
        description: "Interact with GitHub repositories, issues, pull requests, and actions via the GitHub API.",
        kind: CatalogKind::McpServer,
        tags: &["github", "git", "vcs", "official"],
        author: Some("Anthropic"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/github"),
        install_command: Some("npx -y @modelcontextprotocol/server-github"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-github"],
        skill_template: None,
    },
    StaticEntry {
        id: "mcp-postgres",
        name: "PostgreSQL",
        description: "Query and inspect a PostgreSQL database. Read-only by default for safety.",
        kind: CatalogKind::McpServer,
        tags: &["database", "postgres", "sql", "official"],
        author: Some("Anthropic"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/postgres"),
        install_command: Some("npx -y @modelcontextprotocol/server-postgres"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-postgres", "{connection_string}"],
        skill_template: None,
    },
    StaticEntry {
        id: "mcp-sqlite",
        name: "SQLite",
        description: "Query and inspect a local SQLite database file.",
        kind: CatalogKind::McpServer,
        tags: &["database", "sqlite", "sql", "official"],
        author: Some("Anthropic"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/sqlite"),
        install_command: Some("npx -y @modelcontextprotocol/server-sqlite"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-sqlite", "--db-path", "{db_path}"],
        skill_template: None,
    },
    StaticEntry {
        id: "mcp-fetch",
        name: "Fetch",
        description: "Fetch URLs and return their content. Supports HTML-to-markdown conversion.",
        kind: CatalogKind::McpServer,
        tags: &["web", "http", "fetch", "official"],
        author: Some("Anthropic"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/fetch"),
        install_command: Some("npx -y @modelcontextprotocol/server-fetch"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-fetch"],
        skill_template: None,
    },
    StaticEntry {
        id: "mcp-memory",
        name: "Memory",
        description: "Persistent key-value memory store for agent context across sessions.",
        kind: CatalogKind::McpServer,
        tags: &["memory", "context", "persistence", "official"],
        author: Some("Anthropic"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/memory"),
        install_command: Some("npx -y @modelcontextprotocol/server-memory"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-memory"],
        skill_template: None,
    },
    StaticEntry {
        id: "mcp-brave-search",
        name: "Brave Search",
        description: "Web search via the Brave Search API. Requires a Brave API key.",
        kind: CatalogKind::McpServer,
        tags: &["search", "web", "official"],
        author: Some("Anthropic"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/brave-search"),
        install_command: Some("npx -y @modelcontextprotocol/server-brave-search"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-brave-search"],
        skill_template: None,
    },
    StaticEntry {
        id: "mcp-puppeteer",
        name: "Puppeteer",
        description: "Browser automation and web scraping via Puppeteer. Requires Node.js.",
        kind: CatalogKind::McpServer,
        tags: &["browser", "automation", "scraping", "official"],
        author: Some("Anthropic"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/puppeteer"),
        install_command: Some("npx -y @modelcontextprotocol/server-puppeteer"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-puppeteer"],
        skill_template: None,
    },
    StaticEntry {
        id: "mcp-slack",
        name: "Slack",
        description: "Read channels, post messages, and list users in a Slack workspace.",
        kind: CatalogKind::McpServer,
        tags: &["slack", "communication", "official"],
        author: Some("Anthropic"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/slack"),
        install_command: Some("npx -y @modelcontextprotocol/server-slack"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-slack"],
        skill_template: None,
    },
    StaticEntry {
        id: "mcp-sequential-thinking",
        name: "Sequential Thinking",
        description: "Structured multi-step reasoning tool. Improves complex problem solving.",
        kind: CatalogKind::McpServer,
        tags: &["reasoning", "thinking", "official"],
        author: Some("Anthropic"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking"),
        install_command: Some("npx -y @modelcontextprotocol/server-sequential-thinking"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-sequential-thinking"],
        skill_template: None,
    },
];

// ─── API ──────────────────────────────────────────────────────────────────────

/// Return all embedded catalog entries.
pub fn list_catalog() -> Vec<CatalogEntry> {
    EMBEDDED.iter().map(to_entry).collect()
}

/// Return catalog entries filtered by kind.
pub fn list_catalog_by_kind(kind: CatalogKind) -> Vec<CatalogEntry> {
    EMBEDDED.iter().filter(|e| e.kind == kind).map(to_entry).collect()
}

/// Search catalog entries by tag or substring match in name/description.
pub fn search_catalog(query: &str) -> Vec<CatalogEntry> {
    let q = query.to_lowercase();
    EMBEDDED
        .iter()
        .filter(|e| {
            e.name.to_lowercase().contains(&q)
                || e.description.to_lowercase().contains(&q)
                || e.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .map(to_entry)
        .collect()
}
