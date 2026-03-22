//! Tests for the compile pipeline — agent/mode overrides and utility helpers.

use super::*;
use super::output::{ensure_session_gitignored, merge_json, merge_json_file};
use std::path::Path;
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

#[test]
fn compile_with_deny_writes_claude_settings() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/permissions.toml",
        r#"
[tools]
deny = ["Bash(rm -rf *)"]
"#,
    );
    run_compile(CompileOptions {
        project_root: tmp.path(),
        provider: Some("claude"),
        dry_run: false,
        active_agent: None,
    })
    .unwrap();
    let settings_path = tmp.path().join(".claude/settings.json");
    assert!(settings_path.exists());
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&settings_path).unwrap()).unwrap();
    assert_eq!(v["permissions"]["deny"][0], "Bash(rm -rf *)");
}

#[test]
fn compile_with_mode_applies_permissions() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/readonly.toml",
        r#"
[agent]
name = "ReadOnly"
id = "readonly"
providers = ["claude"]
[permissions]
preset = "ship-readonly"
"#,
    );
    run_compile(CompileOptions {
        project_root: tmp.path(),
        provider: Some("claude"),
        dry_run: false,
        active_agent: Some("readonly"),
    })
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(tmp.path().join(".claude/settings.json")).unwrap(),
    )
    .unwrap();
    let deny = v["permissions"]["deny"].as_array().unwrap();
    assert!(deny.iter().any(|d| d == "Write(*)"));
}

#[test]
fn compile_with_mode_inline_rules_adds_to_context() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/strict.toml",
        r#"
[agent]
name = "Strict"
id = "strict"
providers = ["claude"]
[rules]
inline = "Never delete files without explicit confirmation."
"#,
    );
    run_compile(CompileOptions {
        project_root: tmp.path(),
        provider: Some("claude"),
        dry_run: false,
        active_agent: Some("strict"),
    })
    .unwrap();
    let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("Never delete files without explicit confirmation."));
}

#[test]
fn compile_with_profile_stop_hook_emits_to_settings() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/commander.toml",
        r#"
[agent]
name = "Commander"
id = "commander"
providers = ["claude"]

[hooks]
stop = "ship permissions sync"
"#,
    );
    run_compile(CompileOptions {
        project_root: tmp.path(),
        provider: Some("claude"),
        dry_run: false,
        active_agent: Some("commander"),
    })
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(tmp.path().join(".claude/settings.json")).unwrap(),
    )
    .unwrap();
    let stop_hooks = v["hooks"]["Stop"]
        .as_array()
        .expect("Stop hooks array must be present");
    assert!(
        stop_hooks.iter().any(|entry| {
            entry["hooks"].as_array().is_some_and(|hooks| {
                hooks
                    .iter()
                    .any(|h| h["command"] == "ship permissions sync")
            })
        }),
        "stop hook command must be emitted"
    );
}

#[test]
fn compile_with_mode_uses_permissions_toml_preset() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/permissions.toml",
        r#"
[ship-fast]
default_mode = "bypassPermissions"
tools_deny = ["Bash(git push --force*)"]
"#,
    );
    write(
        tmp.path(),
        ".ship/agents/fast.toml",
        r#"
[agent]
name = "Fast"
id = "fast"
providers = ["claude"]
[permissions]
preset = "ship-fast"
"#,
    );
    run_compile(CompileOptions {
        project_root: tmp.path(),
        provider: Some("claude"),
        dry_run: false,
        active_agent: Some("fast"),
    })
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(tmp.path().join(".claude/settings.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        v["permissions"]["defaultMode"], "bypassPermissions",
        "defaultMode from permissions.toml preset must be written"
    );
    let deny = v["permissions"]["deny"].as_array().unwrap();
    assert!(
        deny.iter().any(|d| d == "Bash(git push --force*)"),
        "tools_deny from permissions.toml preset must be written"
    );
}

#[test]
fn compile_with_bypass_permissions_mode_writes_default_mode() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        ".ship/agents/autonomous.toml",
        r#"
[agent]
name = "Autonomous"
id = "autonomous"
providers = ["claude"]
[permissions]
default_mode = "bypassPermissions"
"#,
    );
    run_compile(CompileOptions {
        project_root: tmp.path(),
        provider: Some("claude"),
        dry_run: false,
        active_agent: Some("autonomous"),
    })
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(tmp.path().join(".claude/settings.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(v["permissions"]["defaultMode"], "bypassPermissions");
}

#[test]
fn ensure_ship_mcp_globally_allowed_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let settings_path = tmp.path().join("settings.json");
    std::fs::create_dir_all(tmp.path()).unwrap();
    std::fs::write(
        &settings_path,
        r#"{"permissions":{"allow":["mcp__ship__*"]}}"#,
    )
    .unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&settings_path).unwrap()).unwrap();
    let allow = v["permissions"]["allow"].as_array().unwrap();
    assert_eq!(
        allow
            .iter()
            .filter(|x| x.as_str() == Some("mcp__ship__*"))
            .count(),
        1
    );
}

#[test]
fn merge_json_deep_merge() {
    let mut base = serde_json::json!({ "a": { "x": 1 }, "b": 2 });
    let patch = serde_json::json!({ "a": { "y": 2 }, "c": 3 });
    merge_json(&mut base, &patch);
    assert_eq!(base["a"]["x"], 1, "existing key must survive");
    assert_eq!(base["a"]["y"], 2, "patch key must be added");
    assert_eq!(base["c"], 3);
}

#[test]
fn ensure_session_gitignored_adds_entry() {
    let tmp = TempDir::new().unwrap();
    ensure_session_gitignored(tmp.path()).unwrap();
    let content = std::fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
    assert!(
        content.contains(".ship-session/"),
        "must add .ship-session/ entry"
    );
}

#[test]
fn ensure_session_gitignored_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    ensure_session_gitignored(tmp.path()).unwrap();
    ensure_session_gitignored(tmp.path()).unwrap();
    let content = std::fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
    assert_eq!(
        content
            .lines()
            .filter(|l| l.trim() == ".ship-session/")
            .count(),
        1,
        "must not duplicate the entry"
    );
}

#[test]
fn ensure_session_gitignored_appends_to_existing() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join(".gitignore"), "node_modules/\n").unwrap();
    ensure_session_gitignored(tmp.path()).unwrap();
    let content = std::fs::read_to_string(tmp.path().join(".gitignore")).unwrap();
    assert!(
        content.contains("node_modules/"),
        "must preserve existing entries"
    );
    assert!(content.contains(".ship-session/"), "must add new entry");
}

#[test]
fn merge_json_file_creates_if_missing() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("settings.json");
    merge_json_file(
        &path,
        &serde_json::json!({ "model": "claude-opus-4-6" }),
    )
    .unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(v["model"], "claude-opus-4-6");
}
