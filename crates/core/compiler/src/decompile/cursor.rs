//! Cursor decompiler — parse `.cursor/mcp.json`, `.cursor/rules/*.mdc`,
//! `.cursor/hooks.json`, `.cursor/cli.json`, `.cursor/environment.json`
//! into a [`ProjectLibrary`].

use std::collections::HashMap;
use std::path::Path;

use serde_json::Value as Json;

use crate::types::{
    HookConfig, HookTrigger, McpServerConfig, McpServerType, Rule,
};
use crate::ProjectLibrary;

use super::cursor_helpers::{parse_cursor_cli, parse_mdc_frontmatter};
use super::json_string_array;

/// Parse Cursor native config files and produce a partial [`ProjectLibrary`].
///
/// Reads (if present):
/// - `.cursor/mcp.json` → MCP servers
/// - `.cursor/rules/*.mdc` → rules
/// - `.cursor/hooks.json` → hooks
/// - `.cursor/cli.json` → permissions, provider_defaults
/// - `.cursor/environment.json` → cursor_environment
pub fn decompile_cursor(project_root: &Path) -> ProjectLibrary {
    let mut library = ProjectLibrary::default();

    // ── .cursor/mcp.json ─────────────────────────────────────────────────────
    let mcp_path = project_root.join(".cursor").join("mcp.json");
    if let Ok(content) = std::fs::read_to_string(&mcp_path)
        && let Ok(json) = serde_json::from_str::<Json>(&content)
    {
        library.mcp_servers = parse_cursor_mcp(&json);
    }

    // ── .cursor/rules/*.mdc → rules ──────────────────────────────────────────
    let rules_dir = project_root.join(".cursor").join("rules");
    if rules_dir.is_dir() {
        library.rules = parse_cursor_rules(&rules_dir);
    }

    // ── .cursor/hooks.json → hooks ───────────────────────────────────────────
    let hooks_path = project_root.join(".cursor").join("hooks.json");
    if let Ok(content) = std::fs::read_to_string(&hooks_path)
        && let Ok(json) = serde_json::from_str::<Json>(&content)
    {
        library.hooks = parse_cursor_hooks(&json);
    }

    // ── .cursor/cli.json → permissions + provider_defaults ───────────────────
    let cli_path = project_root.join(".cursor").join("cli.json");
    if let Ok(content) = std::fs::read_to_string(&cli_path)
        && let Ok(json) = serde_json::from_str::<Json>(&content)
    {
        parse_cursor_cli(&mut library, &json);
    }

    // ── .cursor/environment.json → cursor_environment ────────────────────────
    let env_path = project_root.join(".cursor").join("environment.json");
    if let Ok(content) = std::fs::read_to_string(&env_path)
        && let Ok(json) = serde_json::from_str::<Json>(&content)
    {
        library.cursor_environment = Some(json);
    }

    library
}

// ── MCP parsing ──────────────────────────────────────────────────────────────

fn parse_cursor_mcp(json: &Json) -> Vec<McpServerConfig> {
    let servers_obj = json
        .get("mcpServers")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let mut servers = Vec::new();

    for (id, entry) in &servers_obj {
        let entry_obj = match entry.as_object() {
            Some(o) => o,
            None => continue,
        };

        let command = entry_obj
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let args = json_string_array(entry_obj.get("args"));

        let env: HashMap<String, String> = entry_obj
            .get("env")
            .and_then(|v| v.as_object())
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        let url = entry_obj
            .get("url")
            .and_then(|v| v.as_str())
            .map(String::from);

        let server_type = if url.is_some() && command.is_empty() {
            McpServerType::Sse
        } else {
            McpServerType::Stdio
        };

        let disabled = entry_obj
            .get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let env_file = entry_obj
            .get("envFile")
            .and_then(|v| v.as_str())
            .map(String::from);

        servers.push(McpServerConfig {
            id: id.clone(),
            name: id.clone(),
            command,
            args,
            env,
            scope: "project".to_string(),
            server_type,
            url,
            disabled,
            timeout_secs: None,
            codex_enabled_tools: vec![],
            codex_disabled_tools: vec![],
            gemini_trust: None,
            gemini_include_tools: vec![],
            gemini_exclude_tools: vec![],
            gemini_timeout_ms: None,
            cursor_env_file: env_file,
        });
    }

    servers
}

// ── Rules parsing (.cursor/rules/*.mdc) ──────────────────────────────────────

fn parse_cursor_rules(rules_dir: &Path) -> Vec<Rule> {
    let mut rules = Vec::new();

    let mut entries: Vec<_> = match std::fs::read_dir(rules_dir) {
        Ok(e) => e.flatten().collect(),
        Err(_) => return rules,
    };
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.extension().is_some_and(|e| e == "mdc" || e == "md") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };

        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let (frontmatter, body) = parse_mdc_frontmatter(&content);

        let trimmed = body.trim();
        if trimmed.is_empty() {
            continue;
        }

        let always_apply = frontmatter
            .get("alwaysApply")
            .map(|v| v == "true")
            .unwrap_or(true);

        let globs = frontmatter
            .get("globs")
            .map(|v| vec![v.clone()])
            .unwrap_or_default();

        let description = frontmatter
            .get("description")
            .map(|v| v.trim_matches('"').to_string());

        rules.push(Rule {
            file_name,
            content: trimmed.to_string(),
            always_apply,
            globs,
            description,
        });
    }

    rules
}

// ── Hooks parsing ────────────────────────────────────────────────────────────

fn parse_cursor_hooks(json: &Json) -> Vec<HookConfig> {
    let obj = match json.as_object() {
        Some(o) => o,
        None => return vec![],
    };

    let mut hooks = Vec::new();
    let mut counter = 0u32;

    for (event_name, entries) in obj {
        let trigger = match event_name.as_str() {
            "beforeMCPExecution" | "beforeShellExecution" => Some(HookTrigger::PreToolUse),
            "afterMCPExecution" | "afterShellExecution" => Some(HookTrigger::PostToolUse),
            "sessionEnd" => Some(HookTrigger::Stop),
            _ => None,
        };

        // Preserve the raw cursor event name for round-trip fidelity
        let cursor_event_name = event_name.clone();

        let entries_arr = match entries.as_array() {
            Some(a) => a,
            None => continue,
        };

        for entry in entries_arr {
            let entry_obj = match entry.as_object() {
                Some(o) => o,
                None => continue,
            };

            let command = match entry_obj.get("command").and_then(|v| v.as_str()) {
                Some(c) => c.to_string(),
                None => continue,
            };

            let matcher = entry_obj
                .get("matcher")
                .and_then(|v| v.as_str())
                .map(String::from);

            counter += 1;
            hooks.push(HookConfig {
                id: format!("cursor-imported-{counter}"),
                trigger: trigger.clone().unwrap_or(HookTrigger::PreToolUse),
                command,
                matcher,
                cursor_event: Some(cursor_event_name.clone()),
                gemini_event: None,
            });
        }
    }

    hooks
}
