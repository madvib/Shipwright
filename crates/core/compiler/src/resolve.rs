use std::collections::HashSet;

use crate::types::{
    HookConfig, McpServerConfig, ModeConfig, Permissions, ProjectConfig, Rule, Skill,
};

/// Thin overrides that a feature branch can apply on top of project defaults.
/// Maps to the `[agent]` block in a feature's TOML frontmatter.
#[derive(Debug, Clone, Default)]
pub struct FeatureOverrides {
    pub model: Option<String>,
    pub max_cost_per_session: Option<f64>,
    /// Restrict to these MCP server IDs (empty = no restriction)
    pub mcp_servers: Vec<String>,
    /// Restrict to these skill IDs (empty = no restriction)
    pub skills: Vec<String>,
    /// Override provider list (empty = use project default)
    pub providers: Vec<String>,
}

/// Fully resolved, in-memory agent configuration.
///
/// Produced by [`resolve`] from pre-loaded project data.
/// No filesystem access occurs during resolution.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub providers: Vec<String>,
    pub model: Option<String>,
    pub max_cost_per_session: Option<f64>,
    pub max_turns: Option<u32>,
    pub mcp_servers: Vec<McpServerConfig>,
    pub skills: Vec<Skill>,
    pub rules: Vec<Rule>,
    pub permissions: Permissions,
    pub hooks: Vec<HookConfig>,
    pub active_mode: Option<String>,
}

/// Resolve the effective agent config from pre-loaded project data.
///
/// Resolution order (highest wins):
/// 1. Project defaults (`ship.toml` types + `agents/` content)
/// 2. Active mode filter (restricts servers/skills/rules)
/// 3. Feature overrides (model, providers, additional server/skill filter)
/// A self-contained library of agent config assets loaded from the `agents/`
/// directory. This is the primary input for the compiler in the new config model:
/// ship.toml holds identity only; the library holds everything the compiler needs.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProjectLibrary {
    /// Mode definitions from `agents/modes/*.toml`.
    #[serde(default)]
    pub modes: Vec<ModeConfig>,
    /// Active mode for this resolve (e.g. workspace override).
    #[serde(default)]
    pub active_mode: Option<String>,
    /// MCP server definitions from `agents/mcp.toml`.
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
    /// Skills from `agents/skills/`.
    #[serde(default)]
    pub skills: Vec<Skill>,
    /// Rules from `agents/rules/` or `agents/rules.md`.
    #[serde(default)]
    pub rules: Vec<Rule>,
    /// Permissions from `agents/permissions.toml`.
    #[serde(default)]
    pub permissions: Permissions,
    /// Hooks from `agents/hooks.toml`.
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
}

/// Resolve a [`ProjectLibrary`] directly — the new-model entry point.
/// No `ProjectConfig` or filesystem access required.
pub fn resolve_library(
    library: &ProjectLibrary,
    feature: Option<&FeatureOverrides>,
    active_mode_override: Option<&str>,
) -> ResolvedConfig {
    let config = ProjectConfig {
        modes: library.modes.clone(),
        active_mode: library.active_mode.clone(),
        mcp_servers: library.mcp_servers.clone(),
        ..Default::default()
    };
    resolve(
        &config,
        &library.skills,
        &library.rules,
        &library.permissions,
        &library.hooks,
        feature,
        active_mode_override,
    )
}

pub fn resolve(
    config: &ProjectConfig,
    skills: &[Skill],
    rules: &[Rule],
    permissions: &Permissions,
    hooks: &[HookConfig],
    feature: Option<&FeatureOverrides>,
    active_mode_override: Option<&str>,
) -> ResolvedConfig {
    // ── Active mode (resolved first — needed for provider target_agents) ──────
    let override_mode = active_mode_override
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .filter(|v| config.modes.iter().any(|m| m.id == *v))
        .map(str::to_string);
    let active_mode = override_mode.or_else(|| config.active_mode.clone());
    let mode = active_mode
        .as_deref()
        .and_then(|id| config.modes.iter().find(|m| m.id == id));

    // ── Providers ─────────────────────────────────────────────────────────────
    // Resolution priority (highest wins):
    // 1. Feature/workspace explicit providers — propagate even if empty (unknown
    //    providers → empty rather than silent fallback).
    // 2. Mode target_agents — when a mode is active and specifies target agents.
    // 3. Project-level providers.
    // 4. Default: ["claude"].
    let feature_has_explicit = feature.map_or(false, |f| !f.providers.is_empty());
    let feature_providers = feature
        .filter(|f| !f.providers.is_empty())
        .map(|f| normalize_providers(&f.providers))
        .unwrap_or_default();

    let providers = if feature_has_explicit {
        feature_providers
    } else {
        let mode_providers = mode
            .filter(|m| !m.target_agents.is_empty())
            .map(|m| normalize_providers(&m.target_agents))
            .unwrap_or_default();
        if !mode_providers.is_empty() {
            mode_providers
        } else {
            let project = normalize_providers(&config.providers);
            if project.is_empty() {
                vec!["claude".to_string()]
            } else {
                project
            }
        }
    };

    // ── Model ─────────────────────────────────────────────────────────────────
    let model = feature
        .and_then(|f| f.model.clone())
        .or_else(|| config.ai.as_ref().and_then(|ai| ai.model.clone()));

    // ── Cost / turns ──────────────────────────────────────────────────────────
    let max_cost_per_session = feature.and_then(|f| f.max_cost_per_session);
    let max_turns: Option<u32> = None;

    // ── MCP servers ───────────────────────────────────────────────────────────
    let mut mcp_servers = config.mcp_servers.clone();
    apply_mode_server_filter(&mut mcp_servers, mode);
    if let Some(f) = feature {
        apply_feature_server_filter(&mut mcp_servers, f);
    }

    // ── Skills ────────────────────────────────────────────────────────────────
    let mut resolved_skills = skills.to_vec();
    apply_mode_skill_filter(&mut resolved_skills, mode);
    if let Some(f) = feature {
        apply_feature_skill_filter(&mut resolved_skills, f);
    }

    // ── Rules ─────────────────────────────────────────────────────────────────
    let mut resolved_rules = rules.to_vec();
    if let Some(m) = mode {
        if !m.rules.is_empty() {
            let allowed: HashSet<String> =
                m.rules.iter().map(|id| normalize_rule_id(id)).collect();
            resolved_rules
                .retain(|rule| allowed.contains(&normalize_rule_id(&rule.file_name)));
        }
    }

    ResolvedConfig {
        providers,
        model,
        max_cost_per_session,
        max_turns,
        mcp_servers,
        skills: resolved_skills,
        rules: resolved_rules,
        permissions: permissions.clone(),
        hooks: hooks.to_vec(),
        active_mode,
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn normalize_providers(ids: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for raw in ids {
        let id = raw.trim().to_ascii_lowercase();
        if id.is_empty() {
            continue;
        }
        if !matches!(id.as_str(), "claude" | "gemini" | "codex" | "cursor") {
            continue;
        }
        if seen.insert(id.clone()) {
            out.push(id);
        }
    }
    out
}

fn normalize_rule_id(id: &str) -> String {
    let normalized = id.trim().trim_end_matches(".md");
    if let Some((prefix, rest)) = normalized.split_once('-')
        && !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_digit())
    {
        return rest.to_string();
    }
    normalized.to_string()
}

fn apply_mode_server_filter(servers: &mut Vec<McpServerConfig>, mode: Option<&ModeConfig>) {
    if let Some(m) = mode {
        if !m.mcp_servers.is_empty() {
            servers.retain(|s| m.mcp_servers.contains(&s.id));
        }
    }
}

fn apply_feature_server_filter(servers: &mut Vec<McpServerConfig>, feature: &FeatureOverrides) {
    if !feature.mcp_servers.is_empty() {
        let ids: Vec<&str> = feature.mcp_servers.iter().map(String::as_str).collect();
        servers.retain(|s| ids.contains(&s.id.as_str()));
    }
}

fn apply_mode_skill_filter(skills: &mut Vec<Skill>, mode: Option<&ModeConfig>) {
    if let Some(m) = mode {
        if !m.skills.is_empty() {
            skills.retain(|s| m.skills.contains(&s.id));
        }
    }
}

fn apply_feature_skill_filter(skills: &mut Vec<Skill>, feature: &FeatureOverrides) {
    if !feature.skills.is_empty() {
        let ids: Vec<&str> = feature.skills.iter().map(String::as_str).collect();
        skills.retain(|s| ids.contains(&s.id.as_str()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ModeConfig, ProjectConfig};

    fn make_skill(id: &str) -> Skill {
        Skill {
            id: id.to_string(),
            name: id.to_string(),
            description: None,
            version: None,
            author: None,
            content: String::new(),
            source: Default::default(),
        }
    }

    fn make_server(id: &str) -> McpServerConfig {
        McpServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            command: "cmd".to_string(),
            args: vec![],
            env: Default::default(),
            scope: "project".to_string(),
            server_type: Default::default(),
            url: None,
            disabled: false,
            timeout_secs: None,
        }
    }

    #[test]
    fn default_provider_is_claude() {
        let config = ProjectConfig::default();
        let resolved = resolve(&config, &[], &[], &Permissions::default(), &[], None, None);
        assert_eq!(resolved.providers, vec!["claude"]);
    }

    #[test]
    fn feature_overrides_providers() {
        let config = ProjectConfig {
            providers: vec!["gemini".to_string()],
            ..Default::default()
        };
        let feature = FeatureOverrides {
            providers: vec!["codex".to_string()],
            ..Default::default()
        };
        let resolved = resolve(
            &config,
            &[],
            &[],
            &Permissions::default(),
            &[],
            Some(&feature),
            None,
        );
        assert_eq!(resolved.providers, vec!["codex"]);
    }

    #[test]
    fn mode_filters_servers_and_skills() {
        let config = ProjectConfig {
            mcp_servers: vec![make_server("github"), make_server("linear")],
            modes: vec![ModeConfig {
                id: "planning".to_string(),
                name: "Planning".to_string(),
                mcp_servers: vec!["github".to_string()],
                skills: vec!["alpha".to_string()],
                ..Default::default()
            }],
            active_mode: Some("planning".to_string()),
            ..Default::default()
        };
        let skills = vec![make_skill("alpha"), make_skill("beta")];
        let resolved = resolve(&config, &skills, &[], &Permissions::default(), &[], None, None);
        assert_eq!(resolved.mcp_servers.len(), 1);
        assert_eq!(resolved.mcp_servers[0].id, "github");
        assert_eq!(resolved.skills.len(), 1);
        assert_eq!(resolved.skills[0].id, "alpha");
    }

    #[test]
    fn workspace_mode_override_takes_precedence() {
        let config = ProjectConfig {
            modes: vec![
                ModeConfig {
                    id: "planning".to_string(),
                    name: "Planning".to_string(),
                    skills: vec!["plan-skill".to_string()],
                    ..Default::default()
                },
                ModeConfig {
                    id: "code".to_string(),
                    name: "Code".to_string(),
                    skills: vec!["code-skill".to_string()],
                    ..Default::default()
                },
            ],
            active_mode: Some("planning".to_string()),
            ..Default::default()
        };
        let skills = vec![make_skill("plan-skill"), make_skill("code-skill")];

        let planning = resolve(&config, &skills, &[], &Permissions::default(), &[], None, None);
        assert_eq!(planning.active_mode.as_deref(), Some("planning"));

        let code = resolve(
            &config,
            &skills,
            &[],
            &Permissions::default(),
            &[],
            None,
            Some("code"),
        );
        assert_eq!(code.active_mode.as_deref(), Some("code"));
        assert_eq!(code.skills[0].id, "code-skill");
    }
}
