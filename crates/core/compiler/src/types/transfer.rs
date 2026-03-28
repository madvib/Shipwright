use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Push: Studio → CLI ──────────────────────────────────────────────────

/// Bundle sent from Studio to the local CLI via MCP push_bundle tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TransferBundle {
    pub agent: AgentBundle,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(default)]
    pub skills: HashMap<String, SkillBundle>,
    /// Rule file content keyed by filename (e.g. "code-style.md" → content).
    #[serde(default)]
    pub rules: HashMap<String, String>,
}

/// Agent configuration within a transfer bundle.
/// Every field the agent.schema.json defines is represented here.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AgentBundle {
    // ── agent section ───────────────────────────────────────────────
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Vec<String>>,

    // ── top-level fields ────────────────────────────────────────────
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available_models: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_limits: Option<serde_json::Value>,

    // ── refs (IDs only — content in bundle.skills / bundle.rules) ──
    /// Skill IDs to activate.
    #[serde(default)]
    pub skill_refs: Vec<String>,
    /// Rule file IDs to reference (e.g. "code-style").
    #[serde(default)]
    pub rule_refs: Vec<String>,
    /// Inline rules text (appended after file-based rules).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules_inline: Option<String>,
    /// MCP server names from .ship/mcp.jsonc.
    #[serde(default)]
    pub mcp_servers: Vec<String>,

    // ── structured sections (passed through as JSON) ────────────────
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugins: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_settings: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Vec<serde_json::Value>>,
}

/// Skill files within a transfer bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SkillBundle {
    pub files: HashMap<String, String>,
}

// ── Pull: CLI → Studio ──────────────────────────────────────────────────

/// Response from pull_agents MCP tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PullResponse {
    pub agents: Vec<PullAgent>,
}

/// Resolved agent as returned by pull_agents.
/// Every schema field is present — nothing dropped on pull.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PullAgent {
    pub profile: PullProfile,
    pub skills: Vec<PullSkill>,
    #[serde(rename = "mcpServers")]
    pub mcp_servers: Vec<PullMcpServer>,
    pub rules: Vec<PullRule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules_inline: Option<String>,
    pub hooks: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available_models: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_limits: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugins: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_settings: Option<serde_json::Value>,
    /// "project" (from .ship/) or "library" (from ~/.ship/).
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "project".into()
}

/// Agent profile metadata as returned by pull_agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PullProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub providers: Vec<String>,
    pub version: String,
}

/// Skill with content as returned by pull_agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PullSkill {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub content: String,
    pub source: String,
    /// Canonical storage key from `stable-id:` frontmatter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stable_id: Option<String>,
    /// Tags from frontmatter `tags: [a, b]`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Authors from frontmatter `authors: [ship]`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    /// Raw `assets/vars.json` content (unparsed JSON). `None` if absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vars_schema: Option<serde_json::Value>,
    /// All files in the skill directory, relative paths (e.g. `["SKILL.md", "assets/vars.json"]`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    /// Content of reference docs keyed by relative path (e.g. `"references/docs/index.md"` -> content).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub reference_docs: HashMap<String, String>,
    /// Raw `evals/evals.json` content. `None` if absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evals: Option<serde_json::Value>,
}

/// MCP server reference as returned by pull_agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PullMcpServer {
    pub name: String,
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Rule with content as returned by pull_agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PullRule {
    pub file_name: String,
    pub content: String,
}

// ── List: CLI → Studio ──────────────────────────────────────────────────

/// Response from list_local_agents MCP tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ListAgentsResponse {
    pub agents: Vec<String>,
}
