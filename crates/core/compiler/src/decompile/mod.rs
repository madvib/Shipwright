//! Decompile — reverse-compile provider-native configs into a [`ProjectLibrary`].
//!
//! Each provider module parses its native config files and produces a partial
//! [`ProjectLibrary`]. The caller merges results from multiple providers.

mod claude;
mod claude_mcp;
mod codex;
mod cursor;
mod cursor_helpers;
mod gemini;
mod gemini_policies;
mod opencode;
mod opencode_agents;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "opencode_tests.rs"]
mod opencode_tests;

#[cfg(test)]
#[path = "roundtrip_tests.rs"]
mod roundtrip_tests;

use std::path::Path;

use serde_json::Value as Json;

use crate::ProjectLibrary;

/// Extract a JSON array of strings from an optional JSON value.
/// Shared by multiple decompile modules.
pub(super) fn json_string_array(val: Option<&Json>) -> Vec<String> {
    val.and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

pub use claude::decompile_claude;
pub use codex::decompile_codex;
pub use cursor::decompile_cursor;
pub use gemini::decompile_gemini;
pub use opencode::decompile_opencode;

/// Which providers were detected in a project directory.
#[derive(Debug, Default)]
pub struct DetectedProviders {
    pub claude: bool,
    pub codex: bool,
    pub gemini: bool,
    pub cursor: bool,
    pub opencode: bool,
}

impl DetectedProviders {
    pub fn any(&self) -> bool {
        self.claude || self.codex || self.gemini || self.cursor || self.opencode
    }

    pub fn as_list(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.claude {
            v.push("claude");
        }
        if self.codex {
            v.push("codex");
        }
        if self.gemini {
            v.push("gemini");
        }
        if self.cursor {
            v.push("cursor");
        }
        if self.opencode {
            v.push("opencode");
        }
        v
    }
}

/// Detect which provider configs exist in a project directory.
pub fn detect_providers(project_root: &Path) -> DetectedProviders {
    DetectedProviders {
        claude: project_root.join(".claude").is_dir()
            || project_root.join("CLAUDE.md").exists()
            || project_root.join(".mcp.json").exists(),
        codex: project_root.join(".codex").is_dir()
            || project_root.join("AGENTS.md").exists(),
        gemini: project_root.join(".gemini").is_dir()
            || project_root.join("GEMINI.md").exists(),
        cursor: project_root.join(".cursor").is_dir(),
        opencode: project_root.join(".opencode").is_dir()
            || project_root.join("opencode.json").exists(),
    }
}

/// Decompile all detected providers in a directory into a single [`ProjectLibrary`].
///
/// MCP servers, rules, permissions, and hooks are merged. Provider-specific
/// settings go into `provider_defaults`. The caller should write the result
/// to `.ship/`.
pub fn decompile_all(project_root: &Path) -> ProjectLibrary {
    let detected = detect_providers(project_root);
    let mut library = ProjectLibrary::default();

    if detected.claude {
        let partial = decompile_claude(project_root);
        merge_into(&mut library, partial, "claude");
    }

    if detected.codex {
        let partial = decompile_codex(project_root);
        merge_into(&mut library, partial, "codex");
    }

    if detected.gemini {
        let partial = decompile_gemini(project_root);
        merge_into(&mut library, partial, "gemini");
    }

    if detected.cursor {
        let partial = decompile_cursor(project_root);
        merge_into(&mut library, partial, "cursor");
    }

    if detected.opencode {
        let partial = decompile_opencode(project_root);
        merge_into(&mut library, partial, "opencode");
    }

    library
}

/// Merge a provider-specific partial library into the accumulator.
fn merge_into(target: &mut ProjectLibrary, source: ProjectLibrary, provider_id: &str) {
    // MCP servers — deduplicate by id
    for server in source.mcp_servers {
        if !target.mcp_servers.iter().any(|s| s.id == server.id) {
            target.mcp_servers.push(server);
        }
    }

    // Rules — append (no dedup; different providers may have different context files)
    target.rules.extend(source.rules);

    // Permissions — merge tool lists (union)
    merge_permissions(&mut target.permissions, &source.permissions);

    // Hooks — append, dedup by id
    for hook in source.hooks {
        if !target.hooks.iter().any(|h| h.id == hook.id) {
            target.hooks.push(hook);
        }
    }

    // Agent profiles — append, dedup by id
    for profile in source.agent_profiles {
        if !target
            .agent_profiles
            .iter()
            .any(|p| p.profile.id == profile.profile.id)
        {
            target.agent_profiles.push(profile);
        }
    }

    // Env — merge (source wins on conflict)
    target.env.extend(source.env);

    // Provider defaults — merge in
    for (k, v) in source.provider_defaults {
        target.provider_defaults.insert(k, v);
    }

    // Scalar fields — take from source if set
    if source.model.is_some() {
        target.model = source.model;
    }
    if !source.available_models.is_empty() {
        target.available_models = source.available_models;
    }

    // Claude-specific fields
    if provider_id == "claude" {
        target.claude_settings_extra = source.claude_settings_extra;
        target.claude_theme = source.claude_theme;
        target.claude_auto_updates = source.claude_auto_updates;
        target.claude_include_co_authored_by = source.claude_include_co_authored_by;
        target.claude_team_agents = source.claude_team_agents;
    }

    // Codex-specific fields
    if provider_id == "codex" {
        target.codex_settings_extra = source.codex_settings_extra;
        target.codex_sandbox = source.codex_sandbox;
        target.codex_approval_policy = source.codex_approval_policy;
        target.codex_reasoning_effort = source.codex_reasoning_effort;
        target.codex_max_threads = source.codex_max_threads;
        target.codex_max_depth = source.codex_max_depth;
        target.codex_job_max_runtime_seconds = source.codex_job_max_runtime_seconds;
        target.codex_shell_env_policy = source.codex_shell_env_policy;
        target.codex_notify = source.codex_notify;
    }

    // Gemini-specific fields
    if provider_id == "gemini" {
        target.gemini_settings_extra = source.gemini_settings_extra;
        target.gemini_default_approval_mode = source.gemini_default_approval_mode;
        target.gemini_max_session_turns = source.gemini_max_session_turns;
        target.gemini_disable_yolo_mode = source.gemini_disable_yolo_mode;
        target.gemini_disable_always_allow = source.gemini_disable_always_allow;
        target.gemini_tools_sandbox = source.gemini_tools_sandbox;
    }

    // Cursor-specific fields
    if provider_id == "cursor" {
        target.cursor_settings_extra = source.cursor_settings_extra;
        target.cursor_environment = source.cursor_environment;
    }
}

/// Merge source permissions into target. Appends tool lists rather than replacing.
fn merge_permissions(
    target: &mut crate::types::Permissions,
    source: &crate::types::Permissions,
) {
    // Tool lists — append unique entries
    for p in &source.tools.allow {
        if !target.tools.allow.contains(p) {
            target.tools.allow.push(p.clone());
        }
    }
    for p in &source.tools.deny {
        if !target.tools.deny.contains(p) {
            target.tools.deny.push(p.clone());
        }
    }
    for p in &source.tools.ask {
        if !target.tools.ask.contains(p) {
            target.tools.ask.push(p.clone());
        }
    }
    for d in &source.additional_directories {
        if !target.additional_directories.contains(d) {
            target.additional_directories.push(d.clone());
        }
    }
    if target.default_mode.is_none() {
        target.default_mode = source.default_mode.clone();
    }
    if target.agent.max_cost_per_session.is_none() {
        target.agent.max_cost_per_session = source.agent.max_cost_per_session;
    }
    if target.agent.max_turns.is_none() {
        target.agent.max_turns = source.agent.max_turns;
    }
}
