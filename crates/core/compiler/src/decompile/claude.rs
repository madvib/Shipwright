//! Claude Code decompiler — parse `.claude/settings.json`, `CLAUDE.md`, `.mcp.json`
//! into a [`ProjectLibrary`].

use std::path::Path;

use serde_json::Value as Json;

use crate::ProjectLibrary;
use crate::types::{AgentLimits, HookConfig, HookTrigger, Permissions, Rule, ToolPermissions};

use super::claude_mcp::parse_mcp_json;
use super::json_string_array;

/// Parse Claude Code native config files and produce a partial [`ProjectLibrary`].
///
/// Reads (if present):
/// - `.claude/settings.json` → permissions, hooks, model, env, agent limits, provider_defaults
/// - `CLAUDE.md` → rules
/// - `.mcp.json` → mcp_servers
pub fn decompile_claude(project_root: &Path) -> ProjectLibrary {
    let mut library = ProjectLibrary::default();

    // ── .claude/settings.json ────────────────────────────────────────────────
    let settings_path = project_root.join(".claude").join("settings.json");
    if let Ok(content) = std::fs::read_to_string(&settings_path)
        && let Ok(json) = serde_json::from_str::<Json>(&content)
    {
        parse_claude_settings(&mut library, &json);
    }

    // ── CLAUDE.md ────────────────────────────────────────────────────────────
    let claude_md = project_root.join("CLAUDE.md");
    if let Ok(content) = std::fs::read_to_string(&claude_md) {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            library.rules.push(Rule {
                file_name: "CLAUDE.md".to_string(),
                content: trimmed.to_string(),
                always_apply: true,
                globs: vec![],
                description: None,
            });
        }
    }

    // ── .mcp.json ────────────────────────────────────────────────────────────
    let mcp_path = project_root.join(".mcp.json");
    if let Ok(content) = std::fs::read_to_string(&mcp_path)
        && let Ok(json) = serde_json::from_str::<Json>(&content)
    {
        library.mcp_servers = parse_mcp_json(&json);
    }

    library
}

// ── Settings parser ──────────────────────────────────────────────────────────

/// Known top-level keys in `.claude/settings.json` that map to structured Ship fields.
const KNOWN_SETTINGS_KEYS: &[&str] = &[
    "permissions",
    "hooks",
    "model",
    "env",
    "availableModels",
    "maxCostPerSession",
    "maxTurns",
    "theme",
    "autoUpdates",
    "includeCoAuthoredBy",
    "autoMemoryEnabled",
];

fn parse_claude_settings(library: &mut ProjectLibrary, settings: &Json) {
    let obj = match settings.as_object() {
        Some(o) => o,
        None => return,
    };

    // ── Permissions ──────────────────────────────────────────────────────────
    if let Some(perms) = obj.get("permissions") {
        library.permissions = parse_permissions(perms);
    }

    // ── Hooks ────────────────────────────────────────────────────────────────
    if let Some(hooks) = obj.get("hooks") {
        library.hooks = parse_hooks(hooks);
    }

    // ── Model ────────────────────────────────────────────────────────────────
    if let Some(Json::String(m)) = obj.get("model") {
        library.model = Some(m.clone());
    }

    // ── Env ──────────────────────────────────────────────────────────────────
    if let Some(Json::Object(env)) = obj.get("env") {
        for (k, v) in env {
            if let Json::String(s) = v {
                library.env.insert(k.clone(), s.clone());
            }
        }
    }

    // ── Available models ─────────────────────────────────────────────────────
    if let Some(Json::Array(models)) = obj.get("availableModels") {
        library.available_models = models
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }

    // ── Agent limits ─────────────────────────────────────────────────────────
    if let Some(cost) = obj.get("maxCostPerSession").and_then(|v| v.as_f64()) {
        library.permissions.agent.max_cost_per_session = Some(cost);
    }
    if let Some(turns) = obj.get("maxTurns").and_then(|v| v.as_u64()) {
        library.permissions.agent.max_turns = Some(turns as u32);
    }

    // ── Claude-specific scalars ──────────────────────────────────────────────
    if let Some(Json::String(t)) = obj.get("theme") {
        library.claude_theme = Some(t.clone());
    }
    if let Some(au) = obj.get("autoUpdates").and_then(|v| v.as_bool()) {
        library.claude_auto_updates = Some(au);
    }
    if let Some(co) = obj.get("includeCoAuthoredBy").and_then(|v| v.as_bool()) {
        library.claude_include_co_authored_by = Some(co);
    }

    // ── Provider defaults — everything we don't recognize ────────────────────
    let mut extra = serde_json::Map::new();
    for (k, v) in obj {
        if !KNOWN_SETTINGS_KEYS.contains(&k.as_str()) {
            extra.insert(k.clone(), v.clone());
        }
    }
    if !extra.is_empty() {
        library
            .provider_defaults
            .insert("claude".to_string(), Json::Object(extra));
    }
}

// ── Permission parsing ───────────────────────────────────────────────────────

fn parse_permissions(perms: &Json) -> Permissions {
    let obj = match perms.as_object() {
        Some(o) => o,
        None => return Permissions::default(),
    };

    let tools = ToolPermissions {
        allow: json_string_array(obj.get("allow")),
        ask: json_string_array(obj.get("ask")),
        deny: json_string_array(obj.get("deny")),
    };

    let default_mode = obj
        .get("defaultMode")
        .and_then(|v| v.as_str())
        .map(String::from);

    let additional_directories = json_string_array(obj.get("additionalDirectories"));

    Permissions {
        tools,
        default_mode,
        additional_directories,
        agent: AgentLimits::default(),
        ..Default::default()
    }
}

// ── Hook parsing ─────────────────────────────────────────────────────────────

fn parse_hooks(hooks_val: &Json) -> Vec<HookConfig> {
    let obj = match hooks_val.as_object() {
        Some(o) => o,
        None => return vec![],
    };

    let mut hooks = Vec::new();
    let mut counter = 0u32;

    for (trigger_name, entries) in obj {
        let trigger = match trigger_name.as_str() {
            "PreToolUse" => HookTrigger::PreToolUse,
            "PostToolUse" => HookTrigger::PostToolUse,
            "Notification" => HookTrigger::Notification,
            "Stop" => HookTrigger::Stop,
            "SubagentStop" => HookTrigger::SubagentStop,
            "PreCompact" => HookTrigger::PreCompact,
            _ => continue,
        };

        let entries_arr = match entries.as_array() {
            Some(a) => a,
            None => continue,
        };

        for entry in entries_arr {
            let entry_obj = match entry.as_object() {
                Some(o) => o,
                None => continue,
            };

            // Each entry has "hooks": [{"type": "command", "command": "..."}]
            let hook_list = match entry_obj.get("hooks").and_then(|v| v.as_array()) {
                Some(a) => a,
                None => continue,
            };

            let matcher = entry_obj
                .get("matcher")
                .and_then(|v| v.as_str())
                .map(String::from);

            for hook_item in hook_list {
                let command = match hook_item.get("command").and_then(|v| v.as_str()) {
                    Some(c) => c.to_string(),
                    None => continue,
                };

                counter += 1;
                hooks.push(HookConfig {
                    id: format!("imported-{counter}"),
                    trigger: trigger.clone(),
                    command,
                    matcher: matcher.clone(),
                    cursor_event: None,
                    gemini_event: None,
                });
            }
        }
    }

    hooks
}
