// ─── Provider registry ────────────────────────────────────────────────────────
// Authoritative path reference, support matrix, and compatibility dates:
// → crates/core/compiler/PROVIDERS.md
// Update that file before changing any ProviderDescriptor field.

/// How MCP servers are keyed in the target config file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpKey {
    /// `"mcpServers"` — Claude, Gemini
    McpServers,
    /// `"mcp_servers"` — Codex/OpenAI
    McpServersUnderscored,
    /// `"mcp"` — OpenCode
    Mcp,
}

impl McpKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::McpServers => "mcpServers",
            Self::McpServersUnderscored => "mcp_servers",
            Self::Mcp => "mcp",
        }
    }
}

/// Where the context / system-instructions file is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextFile {
    /// `CLAUDE.md` — Claude Code
    ClaudeMd,
    /// `GEMINI.md` — Gemini CLI
    GeminiMd,
    /// `AGENTS.md` — Codex, Roo, Amp, Goose
    AgentsMd,
    /// Provider does not use a context file
    None,
}

impl ContextFile {
    pub fn file_name(self) -> Option<&'static str> {
        match self {
            Self::ClaudeMd => Some("CLAUDE.md"),
            Self::GeminiMd => Some("GEMINI.md"),
            Self::AgentsMd => Some("AGENTS.md"),
            Self::None => None,
        }
    }
}

/// Where native skill files are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillsDir {
    /// `.claude/skills/<id>/SKILL.md`
    Claude,
    /// `.agents/skills/<id>/SKILL.md` (Gemini CLI, Cursor fallback, universal)
    Gemini,
    /// `.agents/skills/<id>/SKILL.md`
    Agents,
    /// `.cursor/skills/<id>/SKILL.md`
    Cursor,
    /// `.opencode/skills/<id>/SKILL.md`
    OpenCode,
    None,
}

impl SkillsDir {
    pub fn base_path(self) -> Option<&'static str> {
        match self {
            Self::Claude => Some(".claude/skills"),
            Self::Gemini => Some(".agents/skills"),
            Self::Agents => Some(".agents/skills"),
            Self::Cursor => Some(".cursor/skills"),
            Self::OpenCode => Some(".opencode/skills"),
            Self::None => None,
        }
    }
}

/// Where provider-native subagent definition files are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentsDir {
    /// `.claude/agents/<id>.md`
    Claude,
    /// `.gemini/agents/<id>.md`
    Gemini,
    /// `.codex/agents/<id>.toml`
    Codex,
    /// `.cursor/agents/<id>.md`
    Cursor,
    None,
}

impl AgentsDir {
    pub fn base_path(self) -> Option<&'static str> {
        match self {
            Self::Claude => Some(".claude/agents"),
            Self::Gemini => Some(".gemini/agents"),
            Self::Codex => Some(".codex/agents"),
            Self::Cursor => Some(".cursor/agents"),
            Self::None => None,
        }
    }

    pub fn ext(self) -> Option<&'static str> {
        match self {
            Self::Claude => Some("md"),
            Self::Gemini => Some("md"),
            Self::Codex => Some("toml"),
            Self::Cursor => Some("md"),
            Self::None => None,
        }
    }

    /// Build the project-relative path for an agent file: `<base>/<id>.<ext>`.
    pub fn agent_path(self, id: &str) -> Option<String> {
        let base = self.base_path()?;
        let ext = self.ext()?;
        Some(format!("{base}/{id}.{ext}"))
    }
}

#[derive(Debug, Clone)]
pub struct ProviderDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub mcp_key: McpKey,
    pub context_file: ContextFile,
    pub skills_dir: SkillsDir,
    pub agents_dir: AgentsDir,
    /// Whether to emit `"type"` field in MCP server entries.
    /// Claude and Cursor: false (no type field).
    /// Gemini and Codex: false — transport is inferred from field presence
    ///   (command → stdio, url → SSE, httpUrl → HTTP).
    pub emit_type_field: bool,
    /// Field name used for SSE transport URL entries ("url" for most providers).
    pub sse_url_field: &'static str,
    /// Field name used for streamable HTTP transport URL entries.
    /// Gemini uses "httpUrl"; others use "url".
    pub http_url_field: &'static str,
    /// Project-relative path where the MCP config file is written.
    /// `None` when the MCP config is merged into a larger settings file.
    pub mcp_config_path: Option<&'static str>,
}

pub(super) static PROVIDERS: &[ProviderDescriptor] = &[
    ProviderDescriptor {
        id: "claude",
        name: "Claude Code",
        mcp_key: McpKey::McpServers,
        context_file: ContextFile::ClaudeMd,
        skills_dir: SkillsDir::Claude,
        agents_dir: AgentsDir::Claude,
        emit_type_field: false,
        sse_url_field: "url",
        http_url_field: "url",
        // Project-level MCP for Claude Code lives in .mcp.json
        mcp_config_path: Some(".mcp.json"),
    },
    ProviderDescriptor {
        id: "gemini",
        name: "Gemini CLI",
        mcp_key: McpKey::McpServers,
        context_file: ContextFile::GeminiMd,
        skills_dir: SkillsDir::Gemini,
        agents_dir: AgentsDir::Gemini,
        // Source: https://geminicli.com/docs/tools/mcp-server/
        // No "type" field — transport inferred from field presence.
        emit_type_field: false,
        // SSE → "url", streamable HTTP → "httpUrl"
        sse_url_field: "url",
        http_url_field: "httpUrl",
        // MCP is nested under mcpServers inside settings.json (not a separate file)
        mcp_config_path: Some(".gemini/settings.json"),
    },
    ProviderDescriptor {
        id: "codex",
        name: "OpenAI Codex",
        // Source: https://developers.openai.com/codex/mcp
        // Codex MCP config is TOML ([mcp_servers.<name>] tables in ~/.codex/config.toml).
        // The mcp_key here reflects the TOML table key; the mcp_servers JSON output
        // is a known limitation — proper TOML serialisation is tracked as future work.
        mcp_key: McpKey::McpServersUnderscored,
        context_file: ContextFile::AgentsMd,
        skills_dir: SkillsDir::Agents,
        agents_dir: AgentsDir::Codex,
        // No "type" field in Codex TOML MCP entries either.
        emit_type_field: false,
        sse_url_field: "url",
        http_url_field: "url",
        mcp_config_path: Some(".codex/config.toml"),
    },
    ProviderDescriptor {
        id: "cursor",
        name: "Cursor",
        // Source: https://cursor.com/docs/context/skills
        mcp_key: McpKey::McpServers,
        // Cursor uses per-file .mdc rules in .cursor/rules/ — not a single context file
        context_file: ContextFile::None,
        skills_dir: SkillsDir::Cursor,
        agents_dir: AgentsDir::Cursor,
        emit_type_field: false,
        sse_url_field: "url",
        http_url_field: "url",
        mcp_config_path: Some(".cursor/mcp.json"),
    },
    ProviderDescriptor {
        id: "opencode",
        name: "OpenCode",
        // Source: https://opencode.ai/docs/config/
        // Config: opencode.json at project root. MCP servers nested under "mcp" key.
        // MCP uses type: "local"/"remote", command as array, "environment" not "env".
        mcp_key: McpKey::Mcp,
        context_file: ContextFile::AgentsMd,
        // Skills discovered from .opencode/skills/<name>/SKILL.md
        skills_dir: SkillsDir::OpenCode,
        // Agents defined inline in opencode.json "agent" object, not separate files.
        agents_dir: AgentsDir::None,
        emit_type_field: true, // OpenCode uses "type": "local"/"remote"
        sse_url_field: "url",
        http_url_field: "url",
        // MCP is nested inside opencode.json — no separate file.
        mcp_config_path: None,
    },
];

/// Feature support flags for a provider.
///
/// These govern what the compiler emits for a given provider:
/// - `supports_mcp` — provider reads an MCP server config file
/// - `supports_hooks` — provider supports session hooks (Stop, PreToolUse, etc.)
/// - `supports_tool_permissions` — provider supports allow/deny tool lists
/// - `supports_memory` — provider has persistent memory / context file
///
/// See SPEC.md §Provider Feature Matrix for the full per-provider table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderFeatureFlags {
    /// Does this provider read an MCP server config file?
    pub supports_mcp: bool,
    /// Does this provider support session hooks?
    pub supports_hooks: bool,
    /// Can you configure allow/deny tool permission lists for this provider?
    pub supports_tool_permissions: bool,
    /// Does this provider have a persistent memory / context file?
    pub supports_memory: bool,
}

impl ProviderDescriptor {
    /// Return the feature flags for this provider.
    pub fn feature_flags(&self) -> ProviderFeatureFlags {
        match self.id {
            "claude" => ProviderFeatureFlags {
                supports_mcp: true,
                supports_hooks: true,
                supports_tool_permissions: true,
                supports_memory: true,
            },
            "gemini" => ProviderFeatureFlags {
                supports_mcp: true,
                supports_hooks: true,
                supports_tool_permissions: true,
                supports_memory: true,
            },
            "codex" => ProviderFeatureFlags {
                supports_mcp: true,
                supports_hooks: false,
                supports_tool_permissions: false,
                supports_memory: true,
            },
            "cursor" => ProviderFeatureFlags {
                supports_mcp: true,
                supports_hooks: true,
                supports_tool_permissions: true,
                supports_memory: false,
            },
            "opencode" => ProviderFeatureFlags {
                supports_mcp: true,
                supports_hooks: false, // Not documented in OpenCode config reference
                supports_tool_permissions: true,
                supports_memory: true,
            },
            // Unknown providers: conservative defaults
            _ => ProviderFeatureFlags {
                supports_mcp: false,
                supports_hooks: false,
                supports_tool_permissions: false,
                supports_memory: false,
            },
        }
    }
}

pub fn get_provider(id: &str) -> Option<&'static ProviderDescriptor> {
    PROVIDERS.iter().find(|p| p.id == id)
}

pub fn list_providers() -> &'static [ProviderDescriptor] {
    PROVIDERS
}
