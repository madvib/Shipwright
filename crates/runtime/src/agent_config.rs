use crate::config::{McpServerConfig, get_config};
#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct FeatureAgentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cost_per_session: Option<f64>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<String>,
}

use crate::permissions::{Permissions, get_permissions};
use crate::rule::{Rule, list_rules};
use crate::skill::{Skill, list_effective_skills};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashSet;
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

fn normalize_provider_ids(ids: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for raw in ids {
        let id = raw.trim().to_ascii_lowercase();
        if id.is_empty() {
            continue;
        }
        // Keep provider IDs aligned with registered provider descriptors.
        if !matches!(id.as_str(), "claude" | "gemini" | "codex") {
            continue;
        }
        if seen.insert(id.clone()) {
            normalized.push(id);
        }
    }

    normalized
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
    let feature_override_providers = feature_agent
        .filter(|fa| !fa.providers.is_empty())
        .map(|fa| normalize_provider_ids(&fa.providers))
        .unwrap_or_default();
    let providers = if !feature_override_providers.is_empty() {
        feature_override_providers
    } else {
        let project_providers = normalize_provider_ids(&config.providers);
        if project_providers.is_empty() {
            vec!["claude".to_string()]
        } else {
            project_providers
        }
    };

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
    if let Ok(toml_servers) = crate::config::get_mcp_config(ship_dir)
        && !toml_servers.is_empty()
    {
        // Merge or replace? User wants "single source of truth", so let's prefer toml_servers.
        // But we might want to combine them?
        // For now, let's prefer toml_servers and append if they have different IDs.
        for s in toml_servers {
            if let Some(existing) = mcp_servers.iter_mut().find(|matching| matching.id == s.id) {
                *existing = s;
            } else {
                mcp_servers.push(s);
            }
        }
    }

    // Mode filter: if mode restricts servers, retain only allowed IDs.
    if let Some(m) = mode
        && !m.mcp_servers.is_empty()
    {
        mcp_servers.retain(|s| m.mcp_servers.contains(&s.id));
    }

    // Feature filter: if feature specifies server IDs, retain only those.
    if let Some(fa) = feature_agent
        && !fa.mcp_servers.is_empty()
    {
        let ids: Vec<&str> = fa.mcp_servers.iter().map(|r| r.as_str()).collect();
        mcp_servers.retain(|s| ids.contains(&s.id.as_str()));
    }

    // ── Skills ────────────────────────────────────────────────────────────────
    let all_skills = list_effective_skills(ship_dir)?;
    let skills = if let Some(fa) = feature_agent {
        if !fa.skills.is_empty() {
            let ids: Vec<&str> = fa.skills.iter().map(|r| r.as_str()).collect();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AiConfig, McpServerType, ModeConfig, ProjectConfig, save_config};
    use crate::project::init_project;
    use crate::skill::create_skill;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn stdio_server(id: &str, command: &str) -> McpServerConfig {
        McpServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            command: command.to_string(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        }
    }

    #[test]
    fn resolve_agent_config_feature_overrides_providers_model_and_skill_filter() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;

        create_skill(&ship_dir, "__rt_alpha_skill__", "Alpha", "alpha body")?;
        create_skill(&ship_dir, "__rt_beta_skill__", "Beta", "beta body")?;

        let mut config = ProjectConfig::default();
        config.providers = vec!["claude".to_string(), "gemini".to_string()];
        config.ai = Some(AiConfig {
            provider: Some("claude".to_string()),
            model: Some("global-model".to_string()),
            cli_path: None,
        });
        config.mcp_servers = vec![stdio_server("github", "github-bin")];
        save_config(&config, Some(ship_dir.clone()))?;

        let feature_agent = FeatureAgentConfig {
            model: Some("feature-model".to_string()),
            max_cost_per_session: Some(4.2),
            mcp_servers: vec!["github".to_string()],
            skills: vec!["__rt_beta_skill__".to_string()],
            providers: vec!["codex".to_string()],
        };

        let resolved = resolve_agent_config(&ship_dir, Some(&feature_agent))?;
        assert_eq!(resolved.providers, vec!["codex".to_string()]);
        assert_eq!(resolved.model.as_deref(), Some("feature-model"));
        assert_eq!(resolved.max_cost_per_session, Some(4.2));
        assert_eq!(
            resolved
                .skills
                .iter()
                .map(|skill| skill.id.as_str())
                .collect::<Vec<_>>(),
            vec!["__rt_beta_skill__"]
        );
        Ok(())
    }

    #[test]
    fn resolve_agent_config_feature_skill_filter_ignores_missing_ids() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;

        create_skill(&ship_dir, "__rt_only_skill__", "Only", "only body")?;

        let feature_agent = FeatureAgentConfig {
            model: None,
            max_cost_per_session: None,
            mcp_servers: vec![],
            skills: vec![
                "__rt_missing_skill__".to_string(),
                "__rt_only_skill__".to_string(),
            ],
            providers: vec![],
        };

        let resolved = resolve_agent_config(&ship_dir, Some(&feature_agent))?;
        assert_eq!(
            resolved
                .skills
                .iter()
                .map(|skill| skill.id.as_str())
                .collect::<Vec<_>>(),
            vec!["__rt_only_skill__"]
        );
        Ok(())
    }

    #[test]
    fn resolve_agent_config_applies_mode_mcp_filter_without_feature_override() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;

        let mut config = ProjectConfig::default();
        config.mcp_servers = vec![
            stdio_server("github", "github-bin"),
            stdio_server("linear", "linear-bin"),
        ];
        config.modes = vec![ModeConfig {
            id: "planning".to_string(),
            name: "Planning".to_string(),
            mcp_servers: vec!["github".to_string()],
            ..Default::default()
        }];
        config.active_mode = Some("planning".to_string());
        save_config(&config, Some(ship_dir.clone()))?;

        let resolved = resolve_agent_config(&ship_dir, None)?;
        assert_eq!(resolved.mcp_servers.len(), 1);
        assert_eq!(resolved.mcp_servers[0].id, "github");
        Ok(())
    }

    #[test]
    fn resolve_agent_config_merges_mcp_toml_and_overrides_duplicate_ids() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;

        let mut config = ProjectConfig::default();
        config.mcp_servers = vec![stdio_server("github", "ship-toml-command")];
        save_config(&config, Some(ship_dir.clone()))?;

        let mut servers = HashMap::new();
        servers.insert(
            "github".to_string(),
            McpServerConfig {
                id: String::new(),
                name: "github".to_string(),
                command: "mcp-toml-command".to_string(),
                args: vec![],
                env: HashMap::new(),
                scope: "project".to_string(),
                server_type: McpServerType::Stdio,
                url: None,
                disabled: false,
                timeout_secs: None,
            },
        );
        servers.insert(
            "figma".to_string(),
            McpServerConfig {
                id: String::new(),
                name: "figma".to_string(),
                command: "figma-command".to_string(),
                args: vec![],
                env: HashMap::new(),
                scope: "project".to_string(),
                server_type: McpServerType::Stdio,
                url: None,
                disabled: false,
                timeout_secs: None,
            },
        );
        let mcp_toml = toml::to_string(&crate::config::McpConfig {
            mcp: crate::config::McpSection { servers },
        })?;
        std::fs::write(crate::project::mcp_config_path(&ship_dir), mcp_toml)?;

        let resolved = resolve_agent_config(&ship_dir, None)?;
        let github = resolved
            .mcp_servers
            .iter()
            .find(|server| server.id == "github")
            .expect("github server should exist");
        let figma = resolved
            .mcp_servers
            .iter()
            .find(|server| server.id == "figma")
            .expect("figma server should exist");

        assert_eq!(github.command, "mcp-toml-command");
        assert_eq!(figma.command, "figma-command");
        Ok(())
    }

    #[test]
    fn resolve_agent_config_mode_and_feature_mcp_filters_intersect() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;

        let mut config = ProjectConfig::default();
        config.mcp_servers = vec![
            stdio_server("github", "github-bin"),
            stdio_server("linear", "linear-bin"),
            stdio_server("figma", "figma-bin"),
        ];
        config.modes = vec![ModeConfig {
            id: "planning".to_string(),
            name: "Planning".to_string(),
            mcp_servers: vec!["github".to_string(), "linear".to_string()],
            ..Default::default()
        }];
        config.active_mode = Some("planning".to_string());
        save_config(&config, Some(ship_dir.clone()))?;

        let feature_agent = FeatureAgentConfig {
            model: None,
            max_cost_per_session: None,
            mcp_servers: vec!["linear".to_string(), "figma".to_string()],
            skills: vec![],
            providers: vec![],
        };

        let resolved = resolve_agent_config(&ship_dir, Some(&feature_agent))?;
        let ids: Vec<&str> = resolved
            .mcp_servers
            .iter()
            .map(|server| server.id.as_str())
            .collect();
        assert_eq!(ids, vec!["linear"]);
        Ok(())
    }

    #[test]
    fn resolve_agent_config_feature_without_provider_override_uses_project_providers() -> Result<()>
    {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;

        let mut config = ProjectConfig::default();
        config.providers = vec!["claude".to_string(), "gemini".to_string()];
        save_config(&config, Some(ship_dir.clone()))?;

        let feature_agent = FeatureAgentConfig {
            model: Some("feature-model".to_string()),
            max_cost_per_session: None,
            mcp_servers: vec![],
            skills: vec![],
            providers: vec![],
        };

        let resolved = resolve_agent_config(&ship_dir, Some(&feature_agent))?;
        assert_eq!(
            resolved.providers,
            vec!["claude".to_string(), "gemini".to_string()]
        );
        assert_eq!(resolved.model.as_deref(), Some("feature-model"));
        Ok(())
    }

    #[test]
    fn resolve_agent_config_feature_providers_are_normalized_and_unknown_filtered() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;

        let mut config = ProjectConfig::default();
        config.providers = vec!["gemini".to_string()];
        save_config(&config, Some(ship_dir.clone()))?;

        let feature_agent = FeatureAgentConfig {
            model: None,
            max_cost_per_session: None,
            mcp_servers: vec![],
            skills: vec![],
            providers: vec![
                " codex ".to_string(),
                "unknown-provider".to_string(),
                "CLAUDE".to_string(),
                "claude".to_string(),
            ],
        };

        let resolved = resolve_agent_config(&ship_dir, Some(&feature_agent))?;
        assert_eq!(
            resolved.providers,
            vec!["codex".to_string(), "claude".to_string()]
        );
        Ok(())
    }

    #[test]
    fn resolve_agent_config_invalid_feature_provider_override_falls_back_to_project() -> Result<()>
    {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;

        let mut config = ProjectConfig::default();
        config.providers = vec!["gemini".to_string(), "claude".to_string()];
        save_config(&config, Some(ship_dir.clone()))?;

        let feature_agent = FeatureAgentConfig {
            model: None,
            max_cost_per_session: None,
            mcp_servers: vec![],
            skills: vec![],
            providers: vec!["unknown-provider".to_string()],
        };

        let resolved = resolve_agent_config(&ship_dir, Some(&feature_agent))?;
        assert_eq!(
            resolved.providers,
            vec!["gemini".to_string(), "claude".to_string()]
        );
        Ok(())
    }

    #[test]
    fn resolve_agent_config_invalid_project_providers_fall_back_to_claude() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path().to_path_buf())?;

        let mut config = ProjectConfig::default();
        config.providers = vec!["unknown-provider".to_string(), "   ".to_string()];
        save_config(&config, Some(ship_dir.clone()))?;

        let resolved = resolve_agent_config(&ship_dir, None)?;
        assert_eq!(resolved.providers, vec!["claude".to_string()]);
        Ok(())
    }
}
