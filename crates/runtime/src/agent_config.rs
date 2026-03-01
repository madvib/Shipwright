use crate::config::{McpServerConfig, get_config};
use crate::feature::FeatureAgentConfig;
use crate::permissions::{Permissions, get_permissions};
use crate::rule::{Rule, list_rules};
use crate::skill::{Skill, list_effective_skills};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::Path;

// ─── Resolved config ──────────────────────────────────────────────────────────

/// Fully resolved, in-memory agent configuration for the current branch/feature.
///
/// Not stored as a file. Computed by [`resolve_agent_config`] from:
/// 1. Project defaults (`ship.toml` + `agents/` directory)
/// 2. Active mode overrides
/// 3. Feature `[agent]` block (thin overrides)
///
/// A snapshot of key IDs is stored in the Workspace SQLite record for the UI.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AgentConfig {
    /// Active provider IDs (e.g. `["claude"]`). Inherited from project or overridden by feature.
    pub providers: Vec<String>,
    /// Model override — `None` means use the provider's default.
    pub model: Option<String>,
    /// USD spend cap per session. `None` = unlimited.
    pub max_cost_per_session: Option<f64>,
    /// Max agentic turns. `None` = unlimited.
    pub max_turns: Option<u32>,
    /// Resolved MCP servers (project list, filtered by mode + feature).
    pub mcp_servers: Vec<McpServerConfig>,
    /// Resolved skills (project + user effective list, filtered by feature).
    pub skills: Vec<Skill>,
    /// Rules from `agents/rules/` — always active, no filtering.
    pub rules: Vec<Rule>,
    /// Resolved permissions from `agents/permissions.toml`.
    pub permissions: Permissions,
    /// Active mode ID if one is set in `ship.toml`.
    pub active_mode: Option<String>,
}

// ─── Resolution ───────────────────────────────────────────────────────────────

/// Resolve the effective [`AgentConfig`] for the current project state.
///
/// Resolution order (highest wins):
/// 1. Project defaults (`ship.toml` + `agents/` directory)
/// 2. Active mode overrides (mode's `mcp_servers` filter, `prompt_id`)
/// 3. Feature `[agent]` block — thin overrides: `model`, `providers`, filtered server/skill IDs
///
/// Pass `None` for `feature_agent` when not on a feature branch.
pub fn resolve_agent_config(
    ship_dir: &Path,
    feature_agent: Option<&FeatureAgentConfig>,
) -> Result<AgentConfig> {
    let config = get_config(Some(ship_dir.to_path_buf()))?;

    // ── Providers ─────────────────────────────────────────────────────────────
    let providers = feature_agent
        .filter(|fa| !fa.providers.is_empty())
        .map(|fa| fa.providers.clone())
        .unwrap_or_else(|| config.providers.clone());

    // ── Active mode ───────────────────────────────────────────────────────────
    let active_mode = config.active_mode.clone();
    let mode = config
        .modes
        .iter()
        .find(|m| active_mode.as_deref() == Some(m.id.as_str()));

    // ── Model ─────────────────────────────────────────────────────────────────
    let model = feature_agent
        .and_then(|fa| fa.model.clone())
        .or_else(|| config.ai.as_ref().and_then(|ai| ai.model.clone()));

    // ── Cost / turns ──────────────────────────────────────────────────────────
    let max_cost_per_session = feature_agent.and_then(|fa| fa.max_cost_per_session);
    let max_turns: Option<u32> = None; // not in FeatureAgentConfig yet; reserved

    // ── MCP servers ───────────────────────────────────────────────────────────
    let mut mcp_servers = config.mcp_servers.clone();

    // Prioritize mcp.toml if it exists
    if let Ok(toml_servers) = crate::config::get_mcp_config(ship_dir) {
        if !toml_servers.is_empty() {
            // Merge or replace? User wants "single source of truth", so let's prefer toml_servers.
            // But we might want to combine them?
            // For now, let's prefer toml_servers and append if they have different IDs.
            for s in toml_servers {
                if let Some(existing) = mcp_servers.iter_mut().find(|matching| matching.id == s.id)
                {
                    *existing = s;
                } else {
                    mcp_servers.push(s);
                }
            }
        }
    }

    // Mode filter: if mode restricts servers, retain only allowed IDs.
    if let Some(m) = mode {
        if !m.mcp_servers.is_empty() {
            mcp_servers.retain(|s| m.mcp_servers.contains(&s.id));
        }
    }

    // Feature filter: if feature specifies server IDs, retain only those.
    if let Some(fa) = feature_agent {
        if !fa.mcp_servers.is_empty() {
            let ids: Vec<&str> = fa.mcp_servers.iter().map(|r| r.id.as_str()).collect();
            mcp_servers.retain(|s| ids.contains(&s.id.as_str()));
        }
    }

    // ── Skills ────────────────────────────────────────────────────────────────
    let all_skills = list_effective_skills(ship_dir)?;
    let skills = if let Some(fa) = feature_agent {
        if !fa.skills.is_empty() {
            let ids: Vec<&str> = fa.skills.iter().map(|r| r.id.as_str()).collect();
            all_skills
                .into_iter()
                .filter(|s| ids.contains(&s.id.as_str()))
                .collect()
        } else {
            all_skills
        }
    } else {
        all_skills
    };

    // ── Rules ─────────────────────────────────────────────────────────────────
    let rules = list_rules(ship_dir.to_path_buf())?;

    // ── Permissions ───────────────────────────────────────────────────────────
    let permissions = get_permissions(ship_dir.to_path_buf())?;

    Ok(AgentConfig {
        providers,
        model,
        max_cost_per_session,
        max_turns,
        mcp_servers,
        skills,
        rules,
        permissions,
        active_mode,
    })
}
