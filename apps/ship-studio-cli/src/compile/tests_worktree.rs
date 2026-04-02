//! Tests for compile pipeline worktree output-root separation.

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
fn compile_worktree_output_root_separates_from_project_root() {
    // Simulates the actual worktree case: .ship/ config lives in project_root
    // but compiled output (CLAUDE.md, .mcp.json) goes to a separate output_root.
    let project = TempDir::new().unwrap();
    let output = TempDir::new().unwrap();
    setup_minimal_project(&project);

    run_compile(CompileOptions {
        project_root: project.path(),
        output_root: Some(output.path()),
        provider: Some("claude"),
        dry_run: false,
        active_agent: None,
        extra_skills: vec![],
    })
    .unwrap();

    // Output files must be in output_root, not project_root.
    assert!(output.path().join(".mcp.json").exists(), ".mcp.json must be written to output_root");
    assert!(!project.path().join(".mcp.json").exists(), ".mcp.json must NOT be written to project_root");
    assert!(output.path().join("CLAUDE.md").exists(), "CLAUDE.md must be written to output_root");
    assert!(!project.path().join("CLAUDE.md").exists(), "CLAUDE.md must NOT be written to project_root");

    // Verify .mcp.json content: ship server with correct args and env.
    let content = std::fs::read_to_string(output.path().join(".mcp.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    let ship = &parsed["mcpServers"]["ship"];
    assert!(ship.is_object(), "ship server must be present in .mcp.json");
    assert_eq!(ship["command"].as_str(), Some("ship"), "ship server command must be 'ship'");
    let args: Vec<&str> = ship["args"]
        .as_array()
        .expect("args must be present")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(args, vec!["mcp", "serve"], "ship args must be [mcp, serve]");
    assert!(ship["env"].is_object(), "env section must be present in ship server entry");
    let global_dir = ship["env"]["SHIP_GLOBAL_DIR"].as_str()
        .expect("SHIP_GLOBAL_DIR must be set in ship server env");
    assert!(global_dir.ends_with("/.ship"), "SHIP_GLOBAL_DIR must end with /.ship, got: {global_dir}");
}
