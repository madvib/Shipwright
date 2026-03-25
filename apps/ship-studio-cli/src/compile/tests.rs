//! Tests for the compile pipeline — core provider output.

use super::*;
use std::path::Path;
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn setup_minimal_project(tmp: &TempDir) {
    write(tmp.path(), ".ship/rules/style.md", "Use explicit types.");
    write(
        tmp.path(),
        ".ship/mcp.jsonc",
        r#"{ "mcp": { "servers": { "github": { "id": "github", "command": "npx", "args": ["-y", "@mcp/github"] } } } }"#,
    );
}

#[test]
fn compile_writes_claude_md() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("claude"),
        dry_run: false,
        active_agent: None,
    })
    .unwrap();
    let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("Use explicit types."));
}

#[test]
fn compile_writes_mcp_json() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("claude"),
        dry_run: false,
        active_agent: None,
    })
    .unwrap();
    let content = std::fs::read_to_string(tmp.path().join(".mcp.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(
        parsed["mcpServers"]["ship"].is_object(),
        "ship server must be in .mcp.json"
    );
    assert!(parsed["mcpServers"]["github"].is_object());
}

#[test]
fn compile_dry_run_writes_nothing() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("claude"),
        dry_run: true,
        active_agent: None,
    })
    .unwrap();
    assert!(
        !tmp.path().join("CLAUDE.md").exists(),
        "dry-run must not write files"
    );
    assert!(!tmp.path().join(".mcp.json").exists());
}

#[test]
fn compile_gemini_writes_settings_json_with_mcp_and_context() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("gemini"),
        dry_run: false,
        active_agent: None,
    })
    .unwrap();
    let path = tmp.path().join(".gemini/settings.json");
    assert!(
        path.exists(),
        ".gemini/settings.json must be written for gemini"
    );
    let parsed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(parsed["mcpServers"]["ship"].is_object());
}

#[test]
fn compile_gemini_writes_gemini_md_with_rules() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("gemini"),
        dry_run: false,
        active_agent: None,
    })
    .unwrap();
    let content = std::fs::read_to_string(tmp.path().join("GEMINI.md")).unwrap();
    assert!(
        content.contains("Use explicit types."),
        "GEMINI.md must contain rules"
    );
}

#[test]
fn compile_codex_writes_agents_md_with_rules() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("codex"),
        dry_run: false,
        active_agent: None,
    })
    .unwrap();
    let content = std::fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(
        content.contains("Use explicit types."),
        "AGENTS.md must contain rules"
    );
}

#[test]
fn compile_codex_writes_toml_config_with_mcp_servers() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("codex"),
        dry_run: false,
        active_agent: None,
    })
    .unwrap();
    let path = tmp.path().join(".codex/config.toml");
    assert!(
        path.exists(),
        ".codex/config.toml must be written for codex"
    );
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("mcp_servers"),
        "config.toml must contain mcp_servers section"
    );
    assert!(
        content.contains("ship"),
        "ship server must appear in codex config"
    );
}

#[test]
fn compile_cursor_writes_mdc_rule_files() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("cursor"),
        dry_run: false,
        active_agent: None,
    })
    .unwrap();
    let mdc = tmp.path().join(".cursor/rules/style.mdc");
    assert!(mdc.exists(), ".cursor/rules/style.mdc must be written");
    let content = std::fs::read_to_string(&mdc).unwrap();
    assert!(content.contains("Use explicit types."));
    assert!(content.starts_with("---\n"), "must have frontmatter");
}

#[test]
fn compile_opencode_writes_agents_md_with_rules() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("opencode"),
        dry_run: false,
        active_agent: None,
    })
    .unwrap();
    let content = std::fs::read_to_string(tmp.path().join("AGENTS.md")).unwrap();
    assert!(
        content.contains("Use explicit types."),
        "AGENTS.md must contain rules for opencode"
    );
}

#[test]
fn compile_opencode_writes_opencode_json_with_mcp() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("opencode"),
        dry_run: false,
        active_agent: None,
    })
    .unwrap();
    let path = tmp.path().join("opencode.json");
    assert!(path.exists(), "opencode.json must be written for opencode");
    let parsed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(
        parsed["mcp"]["ship"].is_object(),
        "ship server must be in opencode.json mcp"
    );
    assert!(
        parsed["mcp"]["github"].is_object(),
        "github server must be in opencode.json mcp"
    );
}

#[test]
fn compile_opencode_dry_run_writes_nothing() {
    let tmp = TempDir::new().unwrap();
    setup_minimal_project(&tmp);
    run_compile(CompileOptions {
        project_root: tmp.path(),
        output_root: None,
        provider: Some("opencode"),
        dry_run: true,
        active_agent: None,
    })
    .unwrap();
    assert!(
        !tmp.path().join("opencode.json").exists(),
        "dry-run must not write opencode.json"
    );
    assert!(
        !tmp.path().join("AGENTS.md").exists(),
        "dry-run must not write AGENTS.md"
    );
}
