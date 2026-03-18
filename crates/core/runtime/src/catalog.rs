use serde::{Deserialize, Serialize};
use specta::Type;

// ─── Data types ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogKind {
    Skill,
    McpServer,
}

/// Grouping category for catalog browsing.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogCategory {
    Database,
    Communication,
    Development,
    Search,
    Automation,
    Ai,
    Storage,
    Monitoring,
    Cloud,
}

/// A browsable catalog entry with rich metadata for UI display.
///
/// Curated by Ship. Entries are vetted for security, stability, and usefulness.
/// For alpha this is an embedded static list stored in the repo (not an external
/// registry). Community additions go through PR review.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct CatalogEntry {
    /// Stable identifier (slug).
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: CatalogKind,
    pub category: CatalogCategory,
    pub tags: Vec<String>,
    /// Emoji icon for UI display.
    pub icon: String,
    /// Whether this is an official first-party entry (MCP org or vendor-maintained).
    pub official: bool,
    /// Author or organisation name.
    pub author: Option<String>,
    /// SPDX license identifier.
    pub license: Option<String>,
    /// Canonical URL for the skill/server (docs or registry page).
    pub source_url: Option<String>,
    /// Shell command to install the server (e.g. `npx -y @mcp/server-filesystem`).
    pub install_command: Option<String>,
    /// For MCP servers: the runtime command (e.g. `npx`).
    pub command: Option<String>,
    /// For MCP servers: default args (placeholders like `{path}` are replaced at install time).
    pub args: Vec<String>,
    /// Environment variables the server requires (e.g. `["GITHUB_TOKEN", "BRAVE_API_KEY"]`).
    pub env_vars: Vec<String>,
}

// ─── Embedded catalog ─────────────────────────────────────────────────────────

struct StaticEntry {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    kind: CatalogKind,
    category: CatalogCategory,
    tags: &'static [&'static str],
    icon: &'static str,
    official: bool,
    author: Option<&'static str>,
    license: Option<&'static str>,
    source_url: Option<&'static str>,
    install_command: Option<&'static str>,
    command: Option<&'static str>,
    args: &'static [&'static str],
    env_vars: &'static [&'static str],
}

fn to_entry(s: &StaticEntry) -> CatalogEntry {
    CatalogEntry {
        id: s.id.to_string(),
        name: s.name.to_string(),
        description: s.description.to_string(),
        kind: s.kind.clone(),
        category: s.category.clone(),
        tags: s.tags.iter().map(|t| t.to_string()).collect(),
        icon: s.icon.to_string(),
        official: s.official,
        author: s.author.map(|a| a.to_string()),
        license: s.license.map(|l| l.to_string()),
        source_url: s.source_url.map(|u| u.to_string()),
        install_command: s.install_command.map(|c| c.to_string()),
        command: s.command.map(|c| c.to_string()),
        args: s.args.iter().map(|a| a.to_string()).collect(),
        env_vars: s.env_vars.iter().map(|e| e.to_string()).collect(),
    }
}

// ─── API ──────────────────────────────────────────────────────────────────────

/// Return all embedded catalog entries.
pub fn list_catalog() -> Vec<CatalogEntry> {
    EMBEDDED.iter().map(to_entry).collect()
}

/// Return catalog entries filtered by kind.
pub fn list_catalog_by_kind(kind: CatalogKind) -> Vec<CatalogEntry> {
    EMBEDDED
        .iter()
        .filter(|e| e.kind == kind)
        .map(to_entry)
        .collect()
}

/// Return catalog entries filtered by category.
pub fn list_catalog_by_category(category: CatalogCategory) -> Vec<CatalogEntry> {
    EMBEDDED
        .iter()
        .filter(|e| e.category == category)
        .map(to_entry)
        .collect()
}

/// Return only official (first-party) entries.
pub fn list_official() -> Vec<CatalogEntry> {
    EMBEDDED
        .iter()
        .filter(|e| e.official)
        .map(to_entry)
        .collect()
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
                || e.category_str().to_lowercase().contains(&q)
        })
        .map(to_entry)
        .collect()
}

impl StaticEntry {
    fn category_str(&self) -> &'static str {
        match &self.category {
            CatalogCategory::Database => "database",
            CatalogCategory::Communication => "communication",
            CatalogCategory::Development => "development",
            CatalogCategory::Search => "search",
            CatalogCategory::Automation => "automation",
            CatalogCategory::Ai => "ai",
            CatalogCategory::Storage => "storage",
            CatalogCategory::Monitoring => "monitoring",
            CatalogCategory::Cloud => "cloud",
        }
    }
}

// ─── Embedded catalog data ───────────────────────────────────────────────────
//
// Curated by Ship. Every entry is vetted: official source, known-good package,
// working install command. Community additions go through PR review.
//
// Categories: database, communication, development, search, automation, ai,
//             storage, monitoring, cloud

static EMBEDDED: &[StaticEntry] = &[
    // ── Official MCP servers (modelcontextprotocol/servers) ──────────────────
    StaticEntry {
        id: "mcp-filesystem",
        name: "Filesystem",
        description: "Read, write, and search files with configurable allowed paths.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Storage,
        tags: &["filesystem", "files", "local"],
        icon: "folder",
        official: true,
        author: Some("Anthropic"),
        license: Some("MIT"),
        source_url: Some(
            "https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem",
        ),
        install_command: Some("npx -y @modelcontextprotocol/server-filesystem"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-filesystem", "{path}"],
        env_vars: &[],
    },
    StaticEntry {
        id: "mcp-github",
        name: "GitHub",
        description: "Repos, issues, pull requests, and actions via the GitHub API.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Development,
        tags: &["github", "git", "vcs", "issues", "prs"],
        icon: "git-branch",
        official: true,
        author: Some("Anthropic"),
        license: Some("MIT"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/github"),
        install_command: Some("npx -y @modelcontextprotocol/server-github"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-github"],
        env_vars: &["GITHUB_TOKEN"],
    },
    StaticEntry {
        id: "mcp-git",
        name: "Git",
        description: "Read and search local Git repositories. Diffs, logs, branches, blame.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Development,
        tags: &["git", "vcs", "local", "diff"],
        icon: "git-commit",
        official: true,
        author: Some("Anthropic"),
        license: Some("MIT"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/git"),
        install_command: Some("npx -y @modelcontextprotocol/server-git"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-git", "--repository", "{path}"],
        env_vars: &[],
    },
    StaticEntry {
        id: "mcp-postgres",
        name: "PostgreSQL",
        description: "Query and inspect a PostgreSQL database. Read-only by default.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Database,
        tags: &["database", "postgres", "sql"],
        icon: "database",
        official: true,
        author: Some("Anthropic"),
        license: Some("MIT"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/postgres"),
        install_command: Some("npx -y @modelcontextprotocol/server-postgres"),
        command: Some("npx"),
        args: &[
            "-y",
            "@modelcontextprotocol/server-postgres",
            "{connection_string}",
        ],
        env_vars: &[],
    },
    StaticEntry {
        id: "mcp-sqlite",
        name: "SQLite",
        description: "Query and inspect a local SQLite database file.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Database,
        tags: &["database", "sqlite", "sql", "local"],
        icon: "database",
        official: true,
        author: Some("Anthropic"),
        license: Some("MIT"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/sqlite"),
        install_command: Some("npx -y @modelcontextprotocol/server-sqlite"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-sqlite", "--db-path", "{db_path}"],
        env_vars: &[],
    },
    StaticEntry {
        id: "mcp-fetch",
        name: "Fetch",
        description: "Fetch URLs and convert HTML to markdown. Robots.txt aware.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Search,
        tags: &["web", "http", "fetch", "scrape"],
        icon: "globe",
        official: true,
        author: Some("Anthropic"),
        license: Some("MIT"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/fetch"),
        install_command: Some("npx -y @modelcontextprotocol/server-fetch"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-fetch"],
        env_vars: &[],
    },
    StaticEntry {
        id: "mcp-memory",
        name: "Memory",
        description: "Knowledge graph-based persistent memory across sessions.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Ai,
        tags: &["memory", "context", "persistence", "knowledge-graph"],
        icon: "brain",
        official: true,
        author: Some("Anthropic"),
        license: Some("MIT"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/memory"),
        install_command: Some("npx -y @modelcontextprotocol/server-memory"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-memory"],
        env_vars: &[],
    },
    StaticEntry {
        id: "mcp-brave-search",
        name: "Brave Search",
        description: "Web and local search via the Brave Search API.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Search,
        tags: &["search", "web", "brave"],
        icon: "search",
        official: true,
        author: Some("Anthropic"),
        license: Some("MIT"),
        source_url: Some(
            "https://github.com/modelcontextprotocol/servers/tree/main/src/brave-search",
        ),
        install_command: Some("npx -y @modelcontextprotocol/server-brave-search"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-brave-search"],
        env_vars: &["BRAVE_API_KEY"],
    },
    StaticEntry {
        id: "mcp-puppeteer",
        name: "Puppeteer",
        description: "Browser automation via Puppeteer. Navigate, screenshot, interact.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Automation,
        tags: &["browser", "automation", "scraping", "testing"],
        icon: "monitor",
        official: true,
        author: Some("Anthropic"),
        license: Some("MIT"),
        source_url: Some(
            "https://github.com/modelcontextprotocol/servers/tree/main/src/puppeteer",
        ),
        install_command: Some("npx -y @modelcontextprotocol/server-puppeteer"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-puppeteer"],
        env_vars: &[],
    },
    StaticEntry {
        id: "mcp-slack",
        name: "Slack",
        description: "Read channels, post messages, and list users in a Slack workspace.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Communication,
        tags: &["slack", "messaging", "team"],
        icon: "message-square",
        official: true,
        author: Some("Anthropic"),
        license: Some("MIT"),
        source_url: Some("https://github.com/modelcontextprotocol/servers/tree/main/src/slack"),
        install_command: Some("npx -y @modelcontextprotocol/server-slack"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-slack"],
        env_vars: &["SLACK_BOT_TOKEN", "SLACK_TEAM_ID"],
    },
    StaticEntry {
        id: "mcp-sequential-thinking",
        name: "Sequential Thinking",
        description: "Structured multi-step reasoning. Improves complex problem solving.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Ai,
        tags: &["reasoning", "thinking", "chain-of-thought"],
        icon: "lightbulb",
        official: true,
        author: Some("Anthropic"),
        license: Some("MIT"),
        source_url: Some(
            "https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking",
        ),
        install_command: Some("npx -y @modelcontextprotocol/server-sequential-thinking"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-sequential-thinking"],
        env_vars: &[],
    },
    StaticEntry {
        id: "mcp-google-maps",
        name: "Google Maps",
        description: "Geocoding, directions, places, and elevation via Google Maps API.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Search,
        tags: &["maps", "geocoding", "directions", "places"],
        icon: "map-pin",
        official: true,
        author: Some("Anthropic"),
        license: Some("MIT"),
        source_url: Some(
            "https://github.com/modelcontextprotocol/servers/tree/main/src/google-maps",
        ),
        install_command: Some("npx -y @modelcontextprotocol/server-google-maps"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-google-maps"],
        env_vars: &["GOOGLE_MAPS_API_KEY"],
    },
    StaticEntry {
        id: "mcp-everything",
        name: "Everything",
        description: "Reference MCP server exercising all protocol features. For testing.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Development,
        tags: &["testing", "reference", "debug"],
        icon: "wrench",
        official: true,
        author: Some("Anthropic"),
        license: Some("MIT"),
        source_url: Some(
            "https://github.com/modelcontextprotocol/servers/tree/main/src/everything",
        ),
        install_command: Some("npx -y @modelcontextprotocol/server-everything"),
        command: Some("npx"),
        args: &["-y", "@modelcontextprotocol/server-everything"],
        env_vars: &[],
    },
    // ── First-party vendor MCP servers ──────────────────────────────────────
    StaticEntry {
        id: "mcp-sentry",
        name: "Sentry",
        description: "Search issues, view stack traces, and query error data from Sentry.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Monitoring,
        tags: &["sentry", "errors", "monitoring", "debugging"],
        icon: "alert-triangle",
        official: true,
        author: Some("Sentry"),
        license: Some("Apache-2.0"),
        source_url: Some("https://github.com/getsentry/sentry-mcp"),
        install_command: Some("npx -y @sentry/mcp-server"),
        command: Some("npx"),
        args: &["-y", "@sentry/mcp-server"],
        env_vars: &["SENTRY_AUTH_TOKEN"],
    },
    StaticEntry {
        id: "mcp-linear",
        name: "Linear",
        description: "Issues, projects, and cycles from Linear. Create, update, search.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Development,
        tags: &["linear", "issues", "project-management"],
        icon: "list-checks",
        official: true,
        author: Some("Linear"),
        license: Some("MIT"),
        source_url: Some("https://github.com/linear/linear-mcp-server"),
        install_command: Some("npx -y @linear/mcp-server"),
        command: Some("npx"),
        args: &["-y", "@linear/mcp-server"],
        env_vars: &["LINEAR_API_KEY"],
    },
    StaticEntry {
        id: "mcp-notion",
        name: "Notion",
        description: "Search, read, and create pages and databases in Notion.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Communication,
        tags: &["notion", "wiki", "documents", "knowledge-base"],
        icon: "file-text",
        official: true,
        author: Some("Notion"),
        license: Some("MIT"),
        source_url: Some("https://github.com/notionhq/notion-mcp-server"),
        install_command: Some("npx -y @notionhq/notion-mcp-server"),
        command: Some("npx"),
        args: &["-y", "@notionhq/notion-mcp-server"],
        env_vars: &["NOTION_API_KEY"],
    },
    StaticEntry {
        id: "mcp-stripe",
        name: "Stripe",
        description: "Payments, customers, subscriptions, and invoices via Stripe API.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Cloud,
        tags: &["stripe", "payments", "billing", "commerce"],
        icon: "credit-card",
        official: true,
        author: Some("Stripe"),
        license: Some("MIT"),
        source_url: Some("https://github.com/stripe/agent-toolkit"),
        install_command: Some("npx -y @stripe/mcp"),
        command: Some("npx"),
        args: &["-y", "@stripe/mcp"],
        env_vars: &["STRIPE_SECRET_KEY"],
    },
    StaticEntry {
        id: "mcp-cloudflare",
        name: "Cloudflare",
        description: "Workers, KV, R2, D1, and DNS management via Cloudflare API.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Cloud,
        tags: &["cloudflare", "workers", "cdn", "dns", "edge"],
        icon: "cloud",
        official: true,
        author: Some("Cloudflare"),
        license: Some("Apache-2.0"),
        source_url: Some("https://github.com/cloudflare/mcp-server-cloudflare"),
        install_command: Some("npx -y @cloudflare/mcp-server-cloudflare"),
        command: Some("npx"),
        args: &["-y", "@cloudflare/mcp-server-cloudflare"],
        env_vars: &["CLOUDFLARE_API_TOKEN"],
    },
    StaticEntry {
        id: "mcp-vercel",
        name: "Vercel",
        description: "Deployments, projects, domains, and environment variables on Vercel.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Cloud,
        tags: &["vercel", "deployments", "hosting", "next-js"],
        icon: "rocket",
        official: true,
        author: Some("Vercel"),
        license: Some("MIT"),
        source_url: Some("https://github.com/vercel/mcp-server"),
        install_command: Some("npx -y @vercel/mcp"),
        command: Some("npx"),
        args: &["-y", "@vercel/mcp"],
        env_vars: &["VERCEL_TOKEN"],
    },
    StaticEntry {
        id: "mcp-supabase",
        name: "Supabase",
        description: "Database, auth, storage, and edge functions via Supabase.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Database,
        tags: &["supabase", "postgres", "auth", "storage", "baas"],
        icon: "database",
        official: true,
        author: Some("Supabase"),
        license: Some("Apache-2.0"),
        source_url: Some("https://github.com/supabase-community/supabase-mcp"),
        install_command: Some("npx -y @supabase/mcp-server"),
        command: Some("npx"),
        args: &["-y", "@supabase/mcp-server"],
        env_vars: &["SUPABASE_URL", "SUPABASE_SERVICE_ROLE_KEY"],
    },
    StaticEntry {
        id: "mcp-neon",
        name: "Neon",
        description: "Serverless Postgres. Branch databases, run queries, manage projects.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Database,
        tags: &["neon", "postgres", "serverless", "database"],
        icon: "database",
        official: true,
        author: Some("Neon"),
        license: Some("MIT"),
        source_url: Some("https://github.com/neondatabase/mcp-server-neon"),
        install_command: Some("npx -y @neondatabase/mcp-server-neon"),
        command: Some("npx"),
        args: &["-y", "@neondatabase/mcp-server-neon"],
        env_vars: &["NEON_API_KEY"],
    },
    StaticEntry {
        id: "mcp-context7",
        name: "Context7",
        description: "Up-to-date library documentation. Query docs for any package.",
        kind: CatalogKind::McpServer,
        category: CatalogCategory::Development,
        tags: &["docs", "documentation", "libraries", "api-reference"],
        icon: "book-open",
        official: true,
        author: Some("Upstash"),
        license: Some("MIT"),
        source_url: Some("https://github.com/upstash/context7"),
        install_command: Some("npx -y @upstash/context7-mcp"),
        command: Some("npx"),
        args: &["-y", "@upstash/context7-mcp"],
        env_vars: &[],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_entries() {
        let entries = list_catalog();
        assert!(!entries.is_empty());
    }

    #[test]
    fn all_entries_have_required_metadata() {
        for entry in list_catalog() {
            assert!(!entry.id.is_empty(), "entry must have an id");
            assert!(!entry.name.is_empty(), "entry {} must have a name", entry.id);
            assert!(!entry.description.is_empty(), "entry {} must have a description", entry.id);
            assert!(!entry.icon.is_empty(), "entry {} must have an icon", entry.id);
        }
    }

    #[test]
    fn no_duplicate_ids() {
        let entries = list_catalog();
        let mut seen = std::collections::HashSet::new();
        for entry in &entries {
            assert!(seen.insert(&entry.id), "duplicate catalog id: {}", entry.id);
        }
    }

    #[test]
    fn official_entries_have_author_and_license() {
        for entry in list_catalog() {
            if entry.official {
                assert!(
                    entry.author.is_some(),
                    "official entry {} must have an author", entry.id,
                );
                assert!(
                    entry.license.is_some(),
                    "official entry {} must have a license", entry.id,
                );
            }
        }
    }

    #[test]
    fn mcp_servers_have_install_command() {
        for entry in list_catalog_by_kind(CatalogKind::McpServer) {
            assert!(
                entry.install_command.is_some(),
                "MCP server {} must have an install_command", entry.id,
            );
            assert!(
                entry.command.is_some(),
                "MCP server {} must have a command", entry.id,
            );
        }
    }

    #[test]
    fn filter_by_category() {
        let db = list_catalog_by_category(CatalogCategory::Database);
        assert!(!db.is_empty(), "should have database entries");
        for entry in &db {
            assert_eq!(entry.category, CatalogCategory::Database);
        }
    }

    #[test]
    fn list_official_returns_only_official() {
        let official = list_official();
        assert!(!official.is_empty());
        for entry in &official {
            assert!(entry.official, "{} should be official", entry.id);
        }
    }

    #[test]
    fn search_by_tag() {
        let results = search_catalog("postgres");
        assert!(!results.is_empty(), "search for 'postgres' should find entries");
    }

    #[test]
    fn search_by_category_name() {
        let results = search_catalog("database");
        assert!(!results.is_empty(), "search for 'database' should find entries");
    }
}
