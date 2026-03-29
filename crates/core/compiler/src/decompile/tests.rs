//! Decompile module tests.

use std::io::Write;

use serde_json::json;
use tempfile::TempDir;

use super::claude::decompile_claude;
use super::codex::decompile_codex;
use super::cursor::decompile_cursor;
use super::gemini::decompile_gemini;
use super::opencode::decompile_opencode;
use super::{decompile_all, detect_providers};
use crate::types::HookTrigger;

fn write_file(dir: &TempDir, path: &str, content: &str) {
    let full = dir.path().join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = std::fs::File::create(&full).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

// ── detect_providers ─────────────────────────────────────────────────────────

#[test]
fn detect_empty_dir() {
    let tmp = TempDir::new().unwrap();
    let detected = detect_providers(tmp.path());
    assert!(!detected.any());
    assert!(detected.as_list().is_empty());
}

#[test]
fn detect_claude_from_mcp_json() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, ".mcp.json", "{}");
    let detected = detect_providers(tmp.path());
    assert!(detected.claude);
    assert_eq!(detected.as_list(), vec!["claude"]);
}

#[test]
fn detect_claude_from_claude_dir() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join(".claude")).unwrap();
    let detected = detect_providers(tmp.path());
    assert!(detected.claude);
}

#[test]
fn detect_claude_from_claude_md() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, "CLAUDE.md", "# Rules");
    let detected = detect_providers(tmp.path());
    assert!(detected.claude);
}

#[test]
fn detect_multiple_providers() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, "CLAUDE.md", "# Claude rules");
    std::fs::create_dir_all(tmp.path().join(".codex")).unwrap();
    std::fs::create_dir_all(tmp.path().join(".gemini")).unwrap();
    std::fs::create_dir_all(tmp.path().join(".cursor")).unwrap();
    let detected = detect_providers(tmp.path());
    assert_eq!(
        detected.as_list(),
        vec!["claude", "codex", "gemini", "cursor"]
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Claude Code
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn claude_settings_permissions() {
    let tmp = TempDir::new().unwrap();
    let settings = json!({
        "permissions": {
            "allow": ["Read", "Write", "Bash(git *)"],
            "deny": ["Bash(rm -rf *)"],
            "ask": ["Bash(cargo publish)"],
            "defaultMode": "acceptEdits",
            "additionalDirectories": ["/home/user/shared"]
        }
    });
    write_file(
        &tmp,
        ".claude/settings.json",
        &serde_json::to_string(&settings).unwrap(),
    );

    let lib = decompile_claude(tmp.path());

    assert_eq!(
        lib.permissions.tools.allow,
        vec!["Read", "Write", "Bash(git *)"]
    );
    assert_eq!(lib.permissions.tools.deny, vec!["Bash(rm -rf *)"]);
    assert_eq!(lib.permissions.tools.ask, vec!["Bash(cargo publish)"]);
    assert_eq!(lib.permissions.default_mode.as_deref(), Some("acceptEdits"));
    assert_eq!(
        lib.permissions.additional_directories,
        vec!["/home/user/shared"]
    );
}

#[test]
fn claude_settings_hooks() {
    let tmp = TempDir::new().unwrap();
    let settings = json!({
        "hooks": {
            "Stop": [
                {"hooks": [{"type": "command", "command": "echo done"}]}
            ],
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [{"type": "command", "command": "echo pre-bash"}]
                }
            ]
        }
    });
    write_file(
        &tmp,
        ".claude/settings.json",
        &serde_json::to_string(&settings).unwrap(),
    );

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.hooks.len(), 2);
    let stop = lib
        .hooks
        .iter()
        .find(|h| h.trigger == HookTrigger::Stop)
        .unwrap();
    assert_eq!(stop.command, "echo done");
    let pre = lib
        .hooks
        .iter()
        .find(|h| h.trigger == HookTrigger::PreToolUse)
        .unwrap();
    assert_eq!(pre.command, "echo pre-bash");
    assert_eq!(pre.matcher.as_deref(), Some("Bash"));
}

#[test]
fn claude_settings_model_and_env() {
    let tmp = TempDir::new().unwrap();
    let settings = json!({
        "model": "claude-sonnet-4-5-20250514",
        "env": {"API_KEY": "sk-test", "DEBUG": "1"},
        "availableModels": ["claude-sonnet-4-5-20250514", "claude-haiku-3-5-20241022"],
        "maxCostPerSession": 5.0,
        "maxTurns": 100
    });
    write_file(
        &tmp,
        ".claude/settings.json",
        &serde_json::to_string(&settings).unwrap(),
    );

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.model.as_deref(), Some("claude-sonnet-4-5-20250514"));
    assert_eq!(lib.env.get("API_KEY").map(String::as_str), Some("sk-test"));
    assert_eq!(lib.env.get("DEBUG").map(String::as_str), Some("1"));
    assert_eq!(lib.available_models.len(), 2);
    assert_eq!(lib.permissions.agent.max_cost_per_session, Some(5.0));
    assert_eq!(lib.permissions.agent.max_turns, Some(100));
}

#[test]
fn claude_settings_theme_and_scalar_fields() {
    let tmp = TempDir::new().unwrap();
    let settings = json!({
        "theme": "dark",
        "autoUpdates": false,
        "includeCoAuthoredBy": true
    });
    write_file(
        &tmp,
        ".claude/settings.json",
        &serde_json::to_string(&settings).unwrap(),
    );

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.claude_theme.as_deref(), Some("dark"));
    assert_eq!(lib.claude_auto_updates, Some(false));
    assert_eq!(lib.claude_include_co_authored_by, Some(true));
}

#[test]
fn claude_settings_unknown_keys_into_provider_defaults() {
    let tmp = TempDir::new().unwrap();
    let settings = json!({
        "model": "claude-sonnet-4-5-20250514",
        "customSetting": true,
        "experimentalFeatures": {"betaFlag": "on"}
    });
    write_file(
        &tmp,
        ".claude/settings.json",
        &serde_json::to_string(&settings).unwrap(),
    );

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.model.as_deref(), Some("claude-sonnet-4-5-20250514"));
    let defaults = lib.provider_defaults.get("claude").unwrap();
    assert_eq!(defaults["customSetting"], true);
    assert_eq!(defaults["experimentalFeatures"]["betaFlag"], "on");
    assert!(defaults.get("model").is_none());
}

#[test]
fn claude_md_imported_as_rule() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, "CLAUDE.md", "# Project Rules\n\nAlways use TDD.\n");

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.rules.len(), 1);
    assert_eq!(lib.rules[0].file_name, "CLAUDE.md");
    assert!(lib.rules[0].content.contains("Always use TDD."));
    assert!(lib.rules[0].always_apply);
}

#[test]
fn claude_md_empty_skipped() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, "CLAUDE.md", "   \n  ");

    let lib = decompile_claude(tmp.path());
    assert!(lib.rules.is_empty());
}

#[test]
fn mcp_json_servers_parsed() {
    let tmp = TempDir::new().unwrap();
    let mcp = json!({
        "mcpServers": {
            "ship": {
                "command": "ship",
                "args": ["mcp", "serve"],
                "env": {"SHIP_PROJECT": "/home/user/project"}
            },
            "postgres": {
                "url": "http://localhost:5433/mcp",
                "disabled": true
            }
        }
    });
    write_file(&tmp, ".mcp.json", &serde_json::to_string(&mcp).unwrap());

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.mcp_servers.len(), 2);
    let ship = lib.mcp_servers.iter().find(|s| s.id == "ship").unwrap();
    assert_eq!(ship.command, "ship");
    assert_eq!(ship.args, vec!["mcp", "serve"]);
    let pg = lib.mcp_servers.iter().find(|s| s.id == "postgres").unwrap();
    assert_eq!(pg.url.as_deref(), Some("http://localhost:5433/mcp"));
    assert!(pg.disabled);
}

#[test]
fn full_claude_project_decompile() {
    let tmp = TempDir::new().unwrap();

    write_file(&tmp, "CLAUDE.md", "# Engineering\n\nWrite tests first.");
    write_file(
        &tmp,
        ".claude/settings.json",
        &serde_json::to_string(&json!({
            "permissions": {
                "allow": ["Read", "Glob", "Grep"],
                "deny": ["Bash(rm -rf *)"],
                "defaultMode": "default"
            },
            "hooks": {
                "Stop": [{"hooks": [{"type": "command", "command": "ship log"}]}]
            },
            "model": "claude-opus-4-20250514",
            "theme": "dark",
            "autoMemoryEnabled": false,
            "customPlugin": {"enabled": true}
        }))
        .unwrap(),
    );
    write_file(
        &tmp,
        ".mcp.json",
        &serde_json::to_string(&json!({
            "mcpServers": {
                "ship": {"command": "ship", "args": ["mcp", "serve"]}
            }
        }))
        .unwrap(),
    );

    let lib = decompile_claude(tmp.path());

    assert_eq!(lib.rules.len(), 1);
    assert_eq!(lib.permissions.tools.allow, vec!["Read", "Glob", "Grep"]);
    assert_eq!(lib.hooks.len(), 1);
    assert_eq!(lib.model.as_deref(), Some("claude-opus-4-20250514"));
    assert_eq!(lib.mcp_servers.len(), 1);
    let defaults = lib.provider_defaults.get("claude").unwrap();
    assert_eq!(defaults["customPlugin"]["enabled"], true);
}

// ══════════════════════════════════════════════════════════════════════════════
// Codex CLI
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn codex_config_model_and_settings() {
    let tmp = TempDir::new().unwrap();
    let config = r#"
model = "o3"
approval_policy = "auto-edit"
sandbox_mode = "network-disabled"
model_reasoning_effort = "high"
shell_environment_policy = "inherit"

[agents]
max_threads = 4
max_depth = 3
job_max_runtime_seconds = 300
"#;
    write_file(&tmp, ".codex/config.toml", config);

    let lib = decompile_codex(tmp.path());

    assert_eq!(lib.model.as_deref(), Some("o3"));
    assert_eq!(lib.codex_approval_policy.as_deref(), Some("auto_edit"));
    assert_eq!(lib.codex_sandbox.as_deref(), Some("network-only"));
    assert_eq!(lib.codex_reasoning_effort.as_deref(), Some("high"));
    assert_eq!(lib.codex_shell_env_policy.as_deref(), Some("inherit"));
    assert_eq!(lib.codex_max_threads, Some(4));
    assert_eq!(lib.codex_max_depth, Some(3));
    assert_eq!(lib.codex_job_max_runtime_seconds, Some(300));
}

#[test]
fn codex_config_mcp_servers() {
    let tmp = TempDir::new().unwrap();
    let config = r#"
[mcp_servers.ship]
command = "ship"
args = ["mcp", "serve"]

[mcp_servers.pg]
url = "http://localhost:5433/mcp"

[mcp_servers.filtered]
command = "mcp-filtered"
enabled_tools = ["read", "write"]
disabled_tools = ["delete"]
"#;
    write_file(&tmp, ".codex/config.toml", config);

    let lib = decompile_codex(tmp.path());

    assert_eq!(lib.mcp_servers.len(), 3);
    let ship = lib.mcp_servers.iter().find(|s| s.id == "ship").unwrap();
    assert_eq!(ship.command, "ship");

    let filtered = lib.mcp_servers.iter().find(|s| s.id == "filtered").unwrap();
    assert_eq!(filtered.codex_enabled_tools, vec!["read", "write"]);
    assert_eq!(filtered.codex_disabled_tools, vec!["delete"]);
}

#[test]
fn codex_agents_md_imported() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, "AGENTS.md", "# Agent Instructions\n\nFollow TDD.");
    std::fs::create_dir_all(tmp.path().join(".codex")).unwrap();

    let lib = decompile_codex(tmp.path());

    assert_eq!(lib.rules.len(), 1);
    assert_eq!(lib.rules[0].file_name, "AGENTS.md");
    assert!(lib.rules[0].content.contains("Follow TDD."));
}

#[test]
fn codex_unknown_keys_into_provider_defaults() {
    let tmp = TempDir::new().unwrap();
    let config = r#"
model = "o3"
custom_flag = true
"#;
    write_file(&tmp, ".codex/config.toml", config);

    let lib = decompile_codex(tmp.path());

    let defaults = lib.provider_defaults.get("codex").unwrap();
    assert_eq!(defaults["custom_flag"], true);
    assert!(defaults.get("model").is_none());
}

// ══════════════════════════════════════════════════════════════════════════════
// Gemini CLI
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn gemini_settings_model_and_general() {
    let tmp = TempDir::new().unwrap();
    let settings = json!({
        "model": {"name": "gemini-2.5-pro"},
        "general": {
            "defaultApprovalMode": "yolo",
            "maxSessionTurns": 50
        },
        "security": {
            "disableYoloMode": false,
            "disableAlwaysAllow": true
        },
        "tools": {"sandbox": "docker"}
    });
    write_file(
        &tmp,
        ".gemini/settings.json",
        &serde_json::to_string(&settings).unwrap(),
    );

    let lib = decompile_gemini(tmp.path());

    assert_eq!(lib.model.as_deref(), Some("gemini-2.5-pro"));
    assert_eq!(lib.gemini_default_approval_mode.as_deref(), Some("plan"));
    assert_eq!(lib.gemini_max_session_turns, Some(50));
    assert_eq!(lib.gemini_disable_yolo_mode, Some(false));
    assert_eq!(lib.gemini_disable_always_allow, Some(true));
    assert_eq!(lib.gemini_tools_sandbox.as_deref(), Some("docker"));
}

#[test]
fn gemini_mcp_servers_with_provider_fields() {
    let tmp = TempDir::new().unwrap();
    let settings = json!({
        "mcpServers": {
            "ship": {
                "command": "ship",
                "args": ["mcp", "serve"]
            },
            "trusted": {
                "command": "mcp-trusted",
                "trust": true,
                "includeTools": ["read"],
                "excludeTools": ["delete"],
                "timeout": 5000
            },
            "remote": {
                "httpUrl": "https://api.example.com/mcp"
            }
        }
    });
    write_file(
        &tmp,
        ".gemini/settings.json",
        &serde_json::to_string(&settings).unwrap(),
    );

    let lib = decompile_gemini(tmp.path());

    assert_eq!(lib.mcp_servers.len(), 3);

    let trusted = lib.mcp_servers.iter().find(|s| s.id == "trusted").unwrap();
    assert_eq!(trusted.gemini_trust, Some(true));
    assert_eq!(trusted.gemini_include_tools, vec!["read"]);
    assert_eq!(trusted.gemini_exclude_tools, vec!["delete"]);
    assert_eq!(trusted.gemini_timeout_ms, Some(5000));

    let remote = lib.mcp_servers.iter().find(|s| s.id == "remote").unwrap();
    assert_eq!(remote.url.as_deref(), Some("https://api.example.com/mcp"));
    assert_eq!(remote.server_type, crate::types::McpServerType::Http);
}

#[test]
fn gemini_md_imported() {
    let tmp = TempDir::new().unwrap();
    write_file(
        &tmp,
        "GEMINI.md",
        "# Gemini Rules\n\nUse structured output.",
    );

    let lib = decompile_gemini(tmp.path());

    assert_eq!(lib.rules.len(), 1);
    assert_eq!(lib.rules[0].file_name, "GEMINI.md");
}

#[test]
fn gemini_policy_file_parsed() {
    let tmp = TempDir::new().unwrap();
    let policy = r#"
[[tool_policies]]
tool = "shell"
pattern = "rm.*"
decision = "deny"

[[tool_policies]]
tool = "file_read"
decision = "allow"
"#;
    write_file(&tmp, ".gemini/policies/ship.toml", policy);

    let lib = decompile_gemini(tmp.path());

    assert!(
        lib.permissions
            .tools
            .deny
            .iter()
            .any(|p| p.contains("Bash"))
    );
    assert!(lib.permissions.tools.allow.contains(&"Read".to_string()));
}

// ══════════════════════════════════════════════════════════════════════════════
// Cursor
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn cursor_mcp_servers() {
    let tmp = TempDir::new().unwrap();
    let mcp = json!({
        "mcpServers": {
            "ship": {
                "command": "ship",
                "args": ["mcp", "serve"],
                "envFile": ".env.local"
            }
        }
    });
    write_file(
        &tmp,
        ".cursor/mcp.json",
        &serde_json::to_string(&mcp).unwrap(),
    );

    let lib = decompile_cursor(tmp.path());

    assert_eq!(lib.mcp_servers.len(), 1);
    assert_eq!(lib.mcp_servers[0].id, "ship");
    assert_eq!(
        lib.mcp_servers[0].cursor_env_file.as_deref(),
        Some(".env.local")
    );
}

#[test]
fn cursor_rules_parsed() {
    let tmp = TempDir::new().unwrap();
    let rule =
        "---\ndescription: \"Style guide\"\nalwaysApply: true\n---\n\nUse functional components.";
    write_file(&tmp, ".cursor/rules/style.mdc", rule);

    let lib = decompile_cursor(tmp.path());

    assert_eq!(lib.rules.len(), 1);
    assert_eq!(lib.rules[0].file_name, "style.mdc");
    assert!(lib.rules[0].content.contains("Use functional components."));
    assert!(lib.rules[0].always_apply);
    assert_eq!(lib.rules[0].description.as_deref(), Some("Style guide"));
}

#[test]
fn cursor_hooks_parsed() {
    let tmp = TempDir::new().unwrap();
    let hooks = json!({
        "beforeShellExecution": [
            {"command": "echo pre-shell", "matcher": "npm *"}
        ],
        "sessionEnd": [
            {"command": "echo done"}
        ]
    });
    write_file(
        &tmp,
        ".cursor/hooks.json",
        &serde_json::to_string(&hooks).unwrap(),
    );

    let lib = decompile_cursor(tmp.path());

    assert_eq!(lib.hooks.len(), 2);
}

#[test]
fn cursor_cli_json_permissions() {
    let tmp = TempDir::new().unwrap();
    let cli = json!({
        "version": 1,
        "permissions": {
            "allow": ["Shell(git *)", "Read(*)", "Write(src/*)"],
            "deny": ["Shell(rm -rf *)"]
        },
        "experimentalFeature": true
    });
    write_file(
        &tmp,
        ".cursor/cli.json",
        &serde_json::to_string(&cli).unwrap(),
    );

    let lib = decompile_cursor(tmp.path());

    assert!(
        lib.permissions
            .tools
            .allow
            .contains(&"Bash(git *)".to_string())
    );
    assert!(lib.permissions.tools.allow.contains(&"Read(*)".to_string()));
    assert!(
        lib.permissions
            .tools
            .allow
            .contains(&"Write(src/*)".to_string())
    );
    assert!(
        lib.permissions
            .tools
            .deny
            .contains(&"Bash(rm -rf *)".to_string())
    );

    // Unknown keys → provider_defaults
    let defaults = lib.provider_defaults.get("cursor").unwrap();
    assert_eq!(defaults["experimentalFeature"], true);
}

#[test]
fn cursor_environment_json() {
    let tmp = TempDir::new().unwrap();
    let env = json!({"variables": {"NODE_ENV": "development"}});
    write_file(
        &tmp,
        ".cursor/environment.json",
        &serde_json::to_string(&env).unwrap(),
    );

    let lib = decompile_cursor(tmp.path());

    assert!(lib.cursor_environment.is_some());
    assert_eq!(
        lib.cursor_environment.unwrap()["variables"]["NODE_ENV"],
        "development"
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// decompile_all — multi-provider merge
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn decompile_all_merges_providers() {
    let tmp = TempDir::new().unwrap();

    // Claude
    write_file(&tmp, "CLAUDE.md", "# Claude rules");
    write_file(
        &tmp,
        ".mcp.json",
        &serde_json::to_string(&json!({
            "mcpServers": {
                "ship": {"command": "ship", "args": ["mcp", "serve"]}
            }
        }))
        .unwrap(),
    );

    // Cursor
    write_file(
        &tmp,
        ".cursor/mcp.json",
        &serde_json::to_string(&json!({
            "mcpServers": {
                "ship": {"command": "ship", "args": ["mcp", "serve"]},
                "cursor-only": {"command": "cursor-mcp"}
            }
        }))
        .unwrap(),
    );
    let rule = "---\nalwaysApply: true\n---\n\nCursor style rule.";
    write_file(&tmp, ".cursor/rules/style.mdc", rule);

    let lib = decompile_all(tmp.path());

    // Rules from both providers
    assert_eq!(lib.rules.len(), 2);
    assert!(lib.rules.iter().any(|r| r.file_name == "CLAUDE.md"));
    assert!(lib.rules.iter().any(|r| r.file_name == "style.mdc"));

    // MCP servers — "ship" deduped, "cursor-only" kept
    assert_eq!(lib.mcp_servers.len(), 2);
    assert!(lib.mcp_servers.iter().any(|s| s.id == "ship"));
    assert!(lib.mcp_servers.iter().any(|s| s.id == "cursor-only"));
}

// ══════════════════════════════════════════════════════════════════════════════
// OpenCode
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn opencode_config_model_and_mcp() {
    let tmp = TempDir::new().unwrap();
    let config = json!({
        "model": "gpt-4o",
        "mcpServers": {
            "ship": {
                "command": "ship",
                "args": ["mcp", "serve"]
            },
            "remote": {
                "url": "http://localhost:8080/mcp"
            }
        }
    });
    write_file(
        &tmp,
        "opencode.json",
        &serde_json::to_string(&config).unwrap(),
    );

    let lib = decompile_opencode(tmp.path());

    assert_eq!(lib.model.as_deref(), Some("gpt-4o"));
    assert_eq!(lib.mcp_servers.len(), 2);
    let ship = lib.mcp_servers.iter().find(|s| s.id == "ship").unwrap();
    assert_eq!(ship.command, "ship");
}

#[test]
fn opencode_unknown_keys_into_provider_defaults() {
    let tmp = TempDir::new().unwrap();
    let config = json!({
        "model": "gpt-4o",
        "plugins": [{"name": "my-plugin"}],
        "theme": "dark"
    });
    write_file(
        &tmp,
        "opencode.json",
        &serde_json::to_string(&config).unwrap(),
    );

    let lib = decompile_opencode(tmp.path());

    let defaults = lib.provider_defaults.get("opencode").unwrap();
    assert!(defaults.get("plugins").is_some());
    assert_eq!(defaults["theme"], "dark");
    assert!(defaults.get("model").is_none());
}

#[test]
fn opencode_agents_md_imported() {
    let tmp = TempDir::new().unwrap();
    write_file(&tmp, "AGENTS.md", "# Instructions\n\nUse OpenCode.");
    write_file(&tmp, "opencode.json", "{}");

    let lib = decompile_opencode(tmp.path());

    assert_eq!(lib.rules.len(), 1);
    assert_eq!(lib.rules[0].file_name, "AGENTS.md");
}
