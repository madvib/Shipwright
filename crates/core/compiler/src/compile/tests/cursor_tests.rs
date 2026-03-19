use crate::compile::{compile, get_provider, translate_to_cursor_permission};
use crate::compile::provider::ContextFile;
use crate::resolve::ResolvedConfig;
use crate::types::{HookConfig, HookTrigger, Permissions, ToolPermissions};

use super::fixtures::*;

// ── Cursor provider ───────────────────────────────────────────────────────────

#[test]
fn cursor_provider_exists() {
    let desc = get_provider("cursor").expect("cursor provider must be registered");
    assert_eq!(desc.name, "Cursor");
    assert_eq!(desc.mcp_key.as_str(), "mcpServers");
    assert!(!desc.emit_type_field, "Cursor does not emit type field for stdio");
    assert_eq!(desc.http_url_field, "url");
    assert_eq!(desc.mcp_config_path, Some(".cursor/mcp.json"));
}

#[test]
fn cursor_mcp_matches_claude_format() {
    let r = resolved(vec![make_server("github")]);
    let claude = compile(&r, "claude").unwrap();
    let cursor = compile(&r, "cursor").unwrap();

    let claude_keys: Vec<&str> = claude.mcp_servers.as_object().unwrap().keys().map(|k| k.as_str()).collect();
    let cursor_keys: Vec<&str> = cursor.mcp_servers.as_object().unwrap().keys().map(|k| k.as_str()).collect();
    assert_eq!(claude_keys[0], "ship");
    assert_eq!(cursor_keys[0], "ship");

    assert!(cursor.mcp_servers["github"].get("type").is_none());
    assert!(claude.mcp_servers["github"].get("type").is_none());
}

#[test]
fn cursor_skill_files_in_rules_dir() {
    let r = ResolvedConfig { skills: vec![make_skill("refactor")], ..resolved(vec![]) };
    let out = compile(&r, "cursor").unwrap();
    assert!(out.skill_files.contains_key(".cursor/skills/refactor/SKILL.md"));
}

#[test]
fn cursor_context_file_is_none() {
    let desc = get_provider("cursor").unwrap();
    assert_eq!(desc.context_file, ContextFile::None);
    assert_eq!(desc.context_file.file_name(), None);
}

#[test]
fn cursor_no_settings_patch() {
    let r = ResolvedConfig {
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["*".to_string()],
                deny: vec!["Bash(rm -rf *)".to_string()],
                ..Default::default()
            },
            ..Default::default()
        },
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    assert!(out.claude_settings_patch.is_none());
}

#[test]
fn cursor_is_valid_provider_in_normalize() {
    use crate::resolve::{FeatureOverrides, resolve};
    use crate::types::ProjectConfig;
    let feature = FeatureOverrides {
        providers: vec!["cursor".to_string()],
        ..Default::default()
    };
    let resolved = resolve(&ProjectConfig::default(), &[], &[], &Permissions::default(), &[], Some(&feature), None);
    assert_eq!(resolved.providers, vec!["cursor"]);
}

#[test]
fn cursor_mcp_config_path_in_output() {
    let r = resolved(vec![]);
    let out = compile(&r, "cursor").unwrap();
    assert_eq!(out.mcp_config_path.as_deref(), Some(".cursor/mcp.json"));
}

// ── Cursor hooks ──────────────────────────────────────────────────────────────

#[test]
fn cursor_pre_tool_emits_both_mcp_and_shell_events() {
    let r = ResolvedConfig {
        hooks: vec![make_hook(HookTrigger::PreToolUse, "ship check", Some("Bash"))],
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    let patch = out.cursor_hooks_patch.expect("cursor must emit hooks patch");
    assert!(patch["beforeMCPExecution"].is_array(),   "PreToolUse → beforeMCPExecution");
    assert!(patch["beforeShellExecution"].is_array(), "PreToolUse → beforeShellExecution");
}

#[test]
fn cursor_post_tool_emits_both_events() {
    let r = ResolvedConfig {
        hooks: vec![make_hook(HookTrigger::PostToolUse, "ship log", None)],
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    let patch = out.cursor_hooks_patch.unwrap();
    assert!(patch["afterMCPExecution"].is_array());
    assert!(patch["afterShellExecution"].is_array());
}

#[test]
fn cursor_stop_maps_to_session_end() {
    let r = ResolvedConfig {
        hooks: vec![make_hook(HookTrigger::Stop, "ship end", None)],
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    let patch = out.cursor_hooks_patch.unwrap();
    assert!(patch["sessionEnd"].is_array());
}

#[test]
fn cursor_unmapped_triggers_produce_no_patch() {
    let r = ResolvedConfig {
        hooks: vec![
            make_hook(HookTrigger::Notification, "cmd", None),
            make_hook(HookTrigger::SubagentStop, "cmd", None),
            make_hook(HookTrigger::PreCompact,   "cmd", None),
        ],
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    assert!(out.cursor_hooks_patch.is_none());
}

#[test]
fn cursor_hooks_not_emitted_for_other_providers() {
    let r = ResolvedConfig {
        hooks: vec![make_hook(HookTrigger::PreToolUse, "cmd", None)],
        ..resolved(vec![])
    };
    assert!(compile(&r, "claude").unwrap().cursor_hooks_patch.is_none());
    assert!(compile(&r, "gemini").unwrap().cursor_hooks_patch.is_none());
    assert!(compile(&r, "codex").unwrap().cursor_hooks_patch.is_none());
}

// ── Phase 4: Cursor new features ──────────────────────────────────────────────

/// Read(glob) and Write(glob) patterns must pass through as-is.
#[test]
fn cursor_read_glob_passes_through() {
    assert_eq!(
        translate_to_cursor_permission("Read(src/**/*.ts)"),
        Some("Read(src/**/*.ts)".to_string())
    );
    assert_eq!(
        translate_to_cursor_permission("Write(dist/*.js)"),
        Some("Write(dist/*.js)".to_string())
    );
    // Edit(glob) maps to Write(glob)
    assert_eq!(
        translate_to_cursor_permission("Edit(src/*.rs)"),
        Some("Write(src/*.rs)".to_string())
    );
}

#[test]
fn cursor_raw_event_bypasses_trigger_mapping() {
    use crate::types::HookConfig;
    let hook = HookConfig {
        id: "raw-hook".to_string(),
        trigger: HookTrigger::Notification, // normally unmapped for cursor
        matcher: None,
        command: "ship log".to_string(),
        cursor_event: Some("customEvent".to_string()),
        gemini_event: None,
    };
    let r = ResolvedConfig {
        hooks: vec![hook],
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    let patch = out.cursor_hooks_patch.expect("raw cursor_event must produce a patch");
    assert!(patch["customEvent"].is_array(), "raw event must appear under customEvent key");
    assert_eq!(patch["customEvent"][0]["command"], "ship log");
}

#[test]
fn cursor_environment_json_populated() {
    let env = serde_json::json!({ "NODE_ENV": "production", "API_URL": "https://api.example.com" });
    let r = ResolvedConfig {
        cursor_environment: Some(env.clone()),
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    let env_json = out.cursor_environment_json.expect("cursor_environment must be emitted");
    assert_eq!(env_json["NODE_ENV"], "production");
}

#[test]
fn cursor_environment_json_none_when_not_set() {
    let r = resolved(vec![]);
    let out = compile(&r, "cursor").unwrap();
    assert!(out.cursor_environment_json.is_none());
}

#[test]
fn cursor_mcp_env_file_emitted_for_stdio() {
    let mut s = make_server("my-server");
    s.cursor_env_file = Some(".env.local".to_string());
    let r = resolved(vec![s]);
    let out = compile(&r, "cursor").unwrap();
    assert_eq!(out.mcp_servers["my-server"]["envFile"], ".env.local");
}

#[test]
fn cursor_settings_extra_merged_into_cli_json() {
    let r = ResolvedConfig {
        cursor_settings_extra: Some(serde_json::json!({ "customSetting": true })),
        ..resolved(vec![])
    };
    let out = compile(&r, "cursor").unwrap();
    let cli_json = out.cursor_cli_permissions.expect("cursor_settings_extra must trigger cli.json");
    assert_eq!(cli_json["customSetting"], true);
}

#[test]
fn cursor_env_file_not_emitted_for_other_providers() {
    let mut s = make_server("my-server");
    s.cursor_env_file = Some(".env".to_string());
    let r = resolved(vec![s]);
    // Only cursor gets envFile; others should not have it.
    let claude_out = compile(&r, "claude").unwrap();
    assert!(claude_out.mcp_servers["my-server"].get("envFile").is_none());
    let gemini_out = compile(&r, "gemini").unwrap();
    assert!(gemini_out.mcp_servers["my-server"].get("envFile").is_none());
}
