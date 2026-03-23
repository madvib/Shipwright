//! Agent profile application — merging agent config into the project library.

use anyhow::Result;
use compiler::{HookConfig, HookTrigger, PluginEntry, PluginsManifest, ProjectLibrary};
use std::path::Path;

use crate::agent_config::{AgentConfig, apply_agent_permissions};

// ── Agent → library ──────────────────────────────────────────────────────────

pub(crate) fn apply_agent_to_library(
    library: &mut ProjectLibrary,
    mode_id: &str,
    project_root: &Path,
) -> Result<()> {
    let Some(path) = find_agent_file(mode_id, project_root) else {
        return Ok(());
    };

    let profile = AgentConfig::load(&path)?;
    let ship_dir = project_root.join(".ship");

    // Permission overrides — pass ship_dir so preset sections from permissions.jsonc are resolved
    library.permissions =
        apply_agent_permissions(library.permissions.clone(), &profile, Some(&ship_dir));

    // Inline rules → append as a synthetic rule file
    if let Some(inline) = &profile.rules.inline {
        let trimmed = inline.trim();
        if !trimmed.is_empty() {
            library.rules.push(compiler::Rule {
                file_name: format!("{}.md", mode_id),
                content: trimmed.to_string(),
                always_apply: true,
                globs: vec![],
                description: None,
            });
        }
    }

    // If agent declares a provider list, inject a ModeConfig so resolve() applies it
    if !profile.meta.providers.is_empty() {
        library.modes.push(compiler::ModeConfig {
            id: mode_id.to_string(),
            name: profile.meta.name.clone(),
            target_agents: profile.meta.providers.clone(),
            mcp_servers: profile.mcp.servers.clone(),
            skills: profile.skills.refs.clone(),
            ..Default::default()
        });
    }

    // Plugins — convert agent's Vec<String> install list into PluginsManifest
    if !profile.plugins.install.is_empty() {
        library.plugins = PluginsManifest {
            install: profile
                .plugins
                .install
                .iter()
                .map(|id| PluginEntry {
                    id: id.clone(),
                    provider: "claude".to_string(),
                })
                .collect(),
            scope: profile.plugins.scope.clone(),
        };
    }

    // Hooks declared in [hooks] section of agent TOML
    if let Some(cmd) = &profile.hooks.stop {
        let id = format!("{}-stop", mode_id);
        if !library.hooks.iter().any(|h| h.id == id) {
            library.hooks.push(HookConfig {
                id,
                trigger: HookTrigger::Stop,
                command: cmd.clone(),
                matcher: None,
                cursor_event: None,
                gemini_event: None,
            });
        }
    }
    if let Some(cmd) = &profile.hooks.subagent_stop {
        let id = format!("{}-subagent-stop", mode_id);
        if !library.hooks.iter().any(|h| h.id == id) {
            library.hooks.push(HookConfig {
                id,
                trigger: HookTrigger::SubagentStop,
                command: cmd.clone(),
                matcher: None,
                cursor_event: None,
                gemini_event: None,
            });
        }
    }

    // Provider-specific settings pass-through (all providers)
    if let Some(v) = profile.provider_settings.get("claude") {
        library.claude_settings_extra = Some(v.clone());
    }
    if let Some(v) = profile.provider_settings.get("gemini") {
        library.gemini_settings_extra = Some(v.clone());
    }
    if let Some(v) = profile.provider_settings.get("codex") {
        library.codex_settings_extra = Some(v.clone());
    }
    if let Some(v) = profile.provider_settings.get("cursor") {
        library.cursor_settings_extra = Some(v.clone());
    }

    // Team agents from .ship/agents/teams/<provider>/*.md
    library.claude_team_agents = load_team_agents(project_root, "claude");

    Ok(())
}

// ── Team agents ──────────────────────────────────────────────────────────────

fn load_team_agents(project_root: &Path, provider_id: &str) -> Vec<(String, String)> {
    let teams_dir = project_root
        .join(".ship")
        .join("agents")
        .join("teams")
        .join(provider_id);
    if !teams_dir.exists() {
        return vec![];
    }
    let mut agents = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&teams_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md")
                && let (Some(name), Ok(content)) = (
                    path.file_name().map(|n| n.to_string_lossy().to_string()),
                    std::fs::read_to_string(&path),
                )
            {
                agents.push((name, content));
            }
        }
    }
    agents.sort_by(|a, b| a.0.cmp(&b.0));
    agents
}

// Re-use the canonical find_agent_file from profile.rs (supports JSONC + TOML, all compat paths).
use crate::profile::find_agent_file;

// ── Dep skill ref collection ─────────────────────────────────────────────────

/// Collect all skill refs declared in agent profiles and mode configs within
/// `library`. Only dep refs (those starting with `github.com/`) will be resolved
/// by the caller; local refs are included in the list but filtered in
/// [`resolve_dep_skills`](crate::dep_skills::resolve_dep_skills).
pub(crate) fn collect_all_skill_refs(library: &ProjectLibrary) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();

    // From agent profiles: [skills] refs = [...]
    for profile in &library.agent_profiles {
        for r in &profile.skills.refs {
            if !refs.contains(r) {
                refs.push(r.clone());
            }
        }
    }

    // From mode configs: skills = [...] filter list
    for mode in &library.modes {
        for r in &mode.skills {
            if !refs.contains(r) {
                refs.push(r.clone());
            }
        }
    }

    refs
}
