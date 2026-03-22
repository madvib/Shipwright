use std::collections::HashMap;

use serde_json::Value as Json;

use crate::types::{HookConfig, HookTrigger, Permissions};

/// Build the `.claude/settings.json` patch from permissions, hooks, model, and provider settings.
///
/// Returns `None` when nothing needs to be written — i.e. when all permissions
/// are at their safe defaults and there are no hooks. This is intentional: Ship
/// must never write a settings file that silently restricts what Claude can do.
/// If no overrides are present, Claude Code's own defaults govern — which is the
/// safest and least surprising behaviour.
#[allow(clippy::too_many_arguments)]
pub fn build_claude_settings_patch(
    permissions: &Permissions,
    hooks: &[HookConfig],
    model: Option<&str>,
    extra: Option<&Json>,
    env: &std::collections::HashMap<String, String>,
    available_models: &[String],
    theme: Option<&str>,
    auto_updates: Option<bool>,
    include_co_authored_by: Option<bool>,
) -> Option<Json> {
    let has_perms = has_permission_overrides(permissions);
    let has_hooks = !hooks.is_empty();
    let _has_agent_limits = permissions.agent.max_cost_per_session.is_some()
        || permissions.agent.max_turns.is_some();
    let _has_model = model.is_some();
    let _has_extra = extra.is_some_and(|v: &Json| !v.is_null());
    let has_env = !env.is_empty();
    let has_available_models = !available_models.is_empty();
    let _has_theme = theme.is_some();
    let _has_auto_updates = auto_updates.is_some();
    let _has_co_authored = include_co_authored_by.is_some();

    let mut patch = serde_json::json!({});

    // Tool permissions — only emit when the user has deliberately configured them.
    // Claude Code interprets an explicit `allow` list as a strict allowlist, so we
    // only write it when the user has moved away from the "allow all" default.
    if has_perms {
        let mut perms = serde_json::json!({});
        // Only include allow if the user has a non-default allowlist.
        // Default ("*" or empty) → omit → Claude Code uses its own defaults.
        let non_default_allow = !(permissions.tools.allow.is_empty()
            || permissions.tools.allow.len() == 1 && permissions.tools.allow[0] == "*");
        if non_default_allow {
            perms["allow"] = serde_json::json!(permissions.tools.allow);
        }
        if !permissions.tools.ask.is_empty() {
            perms["ask"] = serde_json::json!(permissions.tools.ask);
        }
        if !permissions.tools.deny.is_empty() {
            perms["deny"] = serde_json::json!(permissions.tools.deny);
        }
        if let Some(ref mode) = permissions.default_mode {
            perms["defaultMode"] = serde_json::json!(mode);
        }
        if !permissions.additional_directories.is_empty() {
            perms["additionalDirectories"] = serde_json::json!(permissions.additional_directories);
        }
        patch["permissions"] = perms;
    } else {
        // Even without tool-level overrides, emit permissions block if defaultMode or additionalDirectories set
        let mut perms = serde_json::json!({});
        let mut has_extra_perms = false;
        if let Some(ref mode) = permissions.default_mode {
            perms["defaultMode"] = serde_json::json!(mode);
            has_extra_perms = true;
        }
        if !permissions.additional_directories.is_empty() {
            perms["additionalDirectories"] = serde_json::json!(permissions.additional_directories);
            has_extra_perms = true;
        }
        if has_extra_perms {
            patch["permissions"] = perms;
        }
    }

    // Hooks — grouped by trigger type, matching Claude Code's expected structure.
    if has_hooks {
        let mut by_trigger: HashMap<&str, Vec<Json>> = HashMap::new();
        for hook in hooks {
            let key = match hook.trigger {
                HookTrigger::PreToolUse => "PreToolUse",
                HookTrigger::PostToolUse => "PostToolUse",
                HookTrigger::Notification => "Notification",
                HookTrigger::Stop => "Stop",
                HookTrigger::SubagentStop => "SubagentStop",
                HookTrigger::PreCompact => "PreCompact",
            };
            let hook_obj = serde_json::json!({ "type": "command", "command": hook.command });
            let mut entry = serde_json::json!({ "hooks": [hook_obj] });
            if let Some(m) = &hook.matcher {
                entry["matcher"] = Json::String(m.clone());
            }
            by_trigger.entry(key).or_default().push(entry);
        }
        patch["hooks"] = serde_json::json!(by_trigger);
    }

    // Agent limits.
    if let Some(cost) = permissions.agent.max_cost_per_session {
        patch["maxCostPerSession"] = serde_json::json!(cost);
    }
    if let Some(turns) = permissions.agent.max_turns {
        patch["maxTurns"] = serde_json::json!(turns);
    }

    // Model override.
    if let Some(m) = model {
        patch["model"] = serde_json::json!(m);
    }

    // Environment variables — KEY=VALUE pairs in Claude's `env` field.
    if has_env {
        let env_obj: serde_json::Map<String, Json> = env
            .iter()
            .map(|(k, v)| (k.clone(), Json::String(v.clone())))
            .collect();
        patch["env"] = Json::Object(env_obj);
    }

    // Model allowlist — restricts the model picker in Claude Code.
    if has_available_models {
        patch["availableModels"] = serde_json::json!(available_models);
    }

    // Theme.
    if let Some(t) = theme {
        patch["theme"] = Json::String(t.to_string());
    }

    // Auto-updates.
    if let Some(au) = auto_updates {
        patch["autoUpdates"] = serde_json::json!(au);
    }

    // Include co-authored-by.
    if let Some(co) = include_co_authored_by {
        patch["includeCoAuthoredBy"] = serde_json::json!(co);
    }

    // Ship is the memory layer. Always disable Claude's built-in memories.
    patch["autoMemoryEnabled"] = serde_json::json!(false);

    // Extra provider-specific settings — pass through verbatim (must be last).
    // Source: `[provider_settings.claude]` in the active preset TOML.
    if let Some(extra_obj) = extra.and_then(|v| v.as_object()) {
        for (k, v) in extra_obj {
            patch[k] = v.clone();
        }
    }

    Some(patch)
}

/// Returns `true` when the permissions object contains any tool-level overrides
/// that deviate from "allow everything" defaults. Filesystem, command, network,
/// and agent limits are checked separately in the caller.
pub(super) fn has_permission_overrides(p: &Permissions) -> bool {
    let allow_is_default = p.tools.allow.is_empty()
        || (p.tools.allow.len() == 1 && p.tools.allow[0] == "*");
    !allow_is_default
        || !p.tools.ask.is_empty()
        || !p.tools.deny.is_empty()
        || p.default_mode.is_some()
        || !p.additional_directories.is_empty()
}
