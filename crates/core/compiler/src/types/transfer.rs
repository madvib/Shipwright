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
}

/// Agent configuration within a transfer bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AgentBundle {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<serde_json::Value>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PullAgent {
    pub profile: PullProfile,
    pub skills: Vec<PullSkill>,
    #[serde(rename = "mcpServers")]
    pub mcp_servers: Vec<PullMcpServer>,
    pub rules: Vec<PullRule>,
    pub hooks: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<serde_json::Value>,
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
