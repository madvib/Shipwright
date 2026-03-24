//! Round-trip tests: compile → write → decompile → assert key fields preserved.
//!
//! These tests verify that compiling a ProjectLibrary to provider-native configs
//! and then decompiling back produces equivalent results for fields we support.

use std::io::Write as _;

use serde_json::json;
use tempfile::TempDir;

use crate::decompile::{
    decompile_claude, decompile_codex, decompile_cursor, decompile_gemini, decompile_opencode,
};
use crate::resolve::ProjectLibrary;
use crate::types::{
    HookConfig, HookTrigger, McpServerConfig, McpServerType, Permissions, Rule, ToolPermissions,
};
use crate::{compile, resolve_library};

fn write_file(dir: &TempDir, path: &str, content: &str) {
    let full = dir.path().join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = std::fs::File::create(&full).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

fn test_library() -> ProjectLibrary {
    ProjectLibrary {
        mcp_servers: vec![
            McpServerConfig {
                id: "ship".to_string(),
                name: "ship".to_string(),
                command: "ship".to_string(),
                args: vec!["mcp".to_string(), "serve".to_string()],
                env: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("SHIP_PROJECT".to_string(), "/tmp/test".to_string());
                    m
                },
                scope: "project".to_string(),
                server_type: McpServerType::Stdio,
                url: None,
                disabled: false,
                timeout_secs: None,
                codex_enabled_tools: vec![],
                codex_disabled_tools: vec![],
                gemini_trust: None,
                gemini_include_tools: vec![],
                gemini_exclude_tools: vec![],
                gemini_timeout_ms: None,
                cursor_env_file: None,
            },
        ],
        rules: vec![Rule {
            file_name: "engineering.md".to_string(),
            content: "# Engineering\n\nWrite tests first.".to_string(),
            always_apply: true,
            globs: vec![],
            description: None,
        }],
        permissions: Permissions {
            tools: ToolPermissions {
                allow: vec!["Read".to_string(), "Bash(git *)".to_string()],
                deny: vec!["Bash(rm -rf *)".to_string()],
                ask: vec![],
            },
            default_mode: Some("acceptEdits".to_string()),
            additional_directories: vec!["/tmp/shared".to_string()],
            ..Default::default()
        },
        hooks: vec![HookConfig {
            id: "stop-hook".to_string(),
            trigger: HookTrigger::Stop,
            command: "echo done".to_string(),
            matcher: None,
            cursor_event: None,
            gemini_event: None,
        }],
        model: Some("test-model".to_string()),
        env: {
            let mut m = std::collections::HashMap::new();
            m.insert("API_KEY".to_string(), "test-key".to_string());
            m
        },
        available_models: vec!["test-model".to_string(), "other-model".to_string()],
        ..Default::default()
    }
}

// ── Claude round-trip ────────────────────────────────────────────────────────

#[test]
fn claude_roundtrip_preserves_key_fields() {
    let library = test_library();
    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "claude").unwrap();

    // Write compiled output to temp dir
    let tmp = TempDir::new().unwrap();

    // CLAUDE.md (context file)
    if let Some(content) = &output.context_content {
        write_file(&tmp, "CLAUDE.md", content);
    }

    // .claude/settings.json
    if let Some(patch) = &output.claude_settings_patch {
        write_file(
            &tmp,
            ".claude/settings.json",
            &serde_json::to_string_pretty(patch).unwrap(),
        );
    }

    // .mcp.json
    let mcp_json = json!({ "mcpServers": output.mcp_servers });
    write_file(
        &tmp,
        ".mcp.json",
        &serde_json::to_string_pretty(&mcp_json).unwrap(),
    );

    // Decompile back
    let decompiled = decompile_claude(tmp.path());

    // Assert key fields preserved
    assert_eq!(decompiled.model.as_deref(), Some("test-model"));
    assert!(decompiled
        .permissions
        .tools
        .allow
        .contains(&"Read".to_string()));
    assert!(decompiled
        .permissions
        .tools
        .allow
        .contains(&"Bash(git *)".to_string()));
    assert!(decompiled
        .permissions
        .tools
        .deny
        .contains(&"Bash(rm -rf *)".to_string()));
    assert_eq!(
        decompiled.permissions.default_mode.as_deref(),
        Some("acceptEdits")
    );
    assert!(decompiled
        .permissions
        .additional_directories
        .contains(&"/tmp/shared".to_string()));
    assert_eq!(decompiled.hooks.len(), 1);
    assert_eq!(decompiled.hooks[0].command, "echo done");
    assert_eq!(
        decompiled.env.get("API_KEY").map(String::as_str),
        Some("test-key")
    );
    assert_eq!(decompiled.available_models.len(), 2);

    // MCP servers — the compiler adds a "ship" server, so we may have 2
    assert!(decompiled.mcp_servers.iter().any(|s| s.id == "ship"));
}

// ── Codex round-trip ─────────────────────────────────────────────────────────

#[test]
fn codex_roundtrip_preserves_model_and_mcp() {
    let mut library = test_library();
    library.codex_sandbox = Some("full".to_string());
    library.codex_approval_policy = Some("default".to_string());

    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "codex").unwrap();

    let tmp = TempDir::new().unwrap();

    // AGENTS.md
    if let Some(content) = &output.context_content {
        write_file(&tmp, "AGENTS.md", content);
    }

    // .codex/config.toml
    if let Some(toml_content) = &output.codex_config_patch {
        write_file(&tmp, ".codex/config.toml", toml_content);
    }

    let decompiled = decompile_codex(tmp.path());

    assert_eq!(decompiled.model.as_deref(), Some("test-model"));
    // Approval policy round-trips through translation
    assert!(decompiled.codex_approval_policy.is_some());
    assert!(decompiled.codex_sandbox.is_some());
    assert!(decompiled.mcp_servers.iter().any(|s| s.id == "ship"));
    assert!(!decompiled.rules.is_empty());
}

// ── Gemini round-trip ────────────────────────────────────────────────────────

#[test]
fn gemini_roundtrip_preserves_model_and_mcp() {
    let mut library = test_library();
    library.gemini_default_approval_mode = Some("default".to_string());
    library.gemini_max_session_turns = Some(50);

    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "gemini").unwrap();

    let tmp = TempDir::new().unwrap();

    // GEMINI.md
    if let Some(content) = &output.context_content {
        write_file(&tmp, "GEMINI.md", content);
    }

    // .gemini/settings.json — merge MCP servers + settings patch
    let mut settings = json!({});
    settings["mcpServers"] = output.mcp_servers.clone();
    if let Some(patch) = &output.gemini_settings_patch {
        if let Some(obj) = patch.as_object() {
            for (k, v) in obj {
                settings[k] = v.clone();
            }
        }
    }
    write_file(
        &tmp,
        ".gemini/settings.json",
        &serde_json::to_string_pretty(&settings).unwrap(),
    );

    let decompiled = decompile_gemini(tmp.path());

    assert_eq!(decompiled.model.as_deref(), Some("test-model"));
    assert!(decompiled.gemini_default_approval_mode.is_some());
    assert_eq!(decompiled.gemini_max_session_turns, Some(50));
    assert!(decompiled.mcp_servers.iter().any(|s| s.id == "ship"));
    assert!(!decompiled.rules.is_empty());
}

// ── Cursor round-trip ────────────────────────────────────────────────────────

#[test]
fn cursor_roundtrip_preserves_mcp_and_rules() {
    let library = test_library();
    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "cursor").unwrap();

    let tmp = TempDir::new().unwrap();

    // .cursor/mcp.json
    let mcp_json = json!({ "mcpServers": output.mcp_servers });
    write_file(
        &tmp,
        ".cursor/mcp.json",
        &serde_json::to_string_pretty(&mcp_json).unwrap(),
    );

    // .cursor/rules/*.mdc
    for (path, content) in &output.rule_files {
        write_file(&tmp, path, content);
    }

    // .cursor/hooks.json
    if let Some(hooks) = &output.cursor_hooks_patch {
        write_file(
            &tmp,
            ".cursor/hooks.json",
            &serde_json::to_string_pretty(hooks).unwrap(),
        );
    }

    // .cursor/cli.json
    if let Some(cli) = &output.cursor_cli_permissions {
        write_file(
            &tmp,
            ".cursor/cli.json",
            &serde_json::to_string_pretty(cli).unwrap(),
        );
    }

    let decompiled = decompile_cursor(tmp.path());

    assert!(decompiled.mcp_servers.iter().any(|s| s.id == "ship"));
    assert!(!decompiled.rules.is_empty());
    // Permissions round-trip through Cursor's Shell/Read/Write format
    if !decompiled.permissions.tools.allow.is_empty() {
        assert!(decompiled
            .permissions
            .tools
            .allow
            .iter()
            .any(|p| p.contains("Bash") || p.contains("Read")));
    }
}

// ── OpenCode round-trip ──────────────────────────────────────────────────────

#[test]
fn opencode_roundtrip_preserves_model_and_mcp() {
    let library = test_library();
    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "opencode").unwrap();

    let tmp = TempDir::new().unwrap();

    // opencode.json (config patch)
    if let Some(patch) = &output.opencode_config_patch {
        write_file(
            &tmp,
            "opencode.json",
            &serde_json::to_string_pretty(patch).unwrap(),
        );
    }

    // AGENTS.md (context file)
    if let Some(content) = &output.context_content {
        write_file(&tmp, "AGENTS.md", content);
    }

    let decompiled = decompile_opencode(tmp.path());

    // Model survives
    assert_eq!(decompiled.model.as_deref(), Some("test-model"));
    // MCP servers survive
    assert!(decompiled.mcp_servers.iter().any(|s| s.id == "ship"));
    // Rules survive (from AGENTS.md)
    assert!(!decompiled.rules.is_empty());
}
