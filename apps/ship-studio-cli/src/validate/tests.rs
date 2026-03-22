use super::*;
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn valid_agent_toml() -> &'static str {
    r#"[agent]
name = "test"
id = "test"
providers = ["claude"]
[skills]
refs = []
[mcp]
servers = []
[permissions]
preset = "ship-standard"
"#
}

fn setup() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    let agents_dir = tmp.path().join(".ship").join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    write(tmp.path(), ".ship/agents/permissions.toml",
        "[ship-standard]\ndefault_mode = \"acceptEdits\"\n");
    (tmp, agents_dir)
}

#[test]
fn valid_agent_passes() {
    let (tmp, agents_dir) = setup();
    write(tmp.path(), ".ship/agents/test.toml", valid_agent_toml());
    let agent_path = agents_dir.join("test.toml");
    let report = validate_agent("test", &agent_path, &agents_dir);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
}

#[test]
fn malformed_toml_reports_error() {
    let (tmp, agents_dir) = setup();
    write(tmp.path(), ".ship/agents/bad.toml", "this is not [[valid toml");
    let agent_path = agents_dir.join("bad.toml");
    let report = validate_agent("bad", &agent_path, &agents_dir);
    assert_eq!(report.errors.len(), 1);
    assert!(report.errors[0].error.contains("TOML parse error"), "{:?}", report.errors[0].error);
}

#[test]
fn missing_skill_ref_reports_error() {
    let (tmp, agents_dir) = setup();
    let toml = r#"[agent]
name = "test"
id = "test"
providers = ["claude"]
[skills]
refs = ["nonexistent-skill"]
[mcp]
servers = []
"#;
    write(tmp.path(), ".ship/agents/test.toml", toml);
    let report = validate_agent("test", &agents_dir.join("test.toml"), &agents_dir);
    assert!(report.errors.iter().any(|e| e.error.contains("nonexistent-skill")), "{:?}", report.errors);
}

#[test]
fn existing_skill_ref_passes() {
    let (tmp, agents_dir) = setup();
    write(tmp.path(), ".ship/agents/skills/my-skill.md", "---\nname: My Skill\n---\nContent.");
    let toml = r#"[agent]
name = "test"
id = "test"
providers = ["claude"]
[skills]
refs = ["my-skill"]
[mcp]
servers = []
"#;
    write(tmp.path(), ".ship/agents/test.toml", toml);
    let report = validate_agent("test", &agents_dir.join("test.toml"), &agents_dir);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
}

#[test]
fn stdio_mcp_missing_command_reports_error() {
    let (tmp, agents_dir) = setup();
    write(tmp.path(), ".ship/agents/mcp.toml", r#"
[[servers]]
id = "bad-stdio"
server_type = "stdio"
"#);
    write(tmp.path(), ".ship/agents/test.toml", valid_agent_toml());
    let report = validate_agent("test", &agents_dir.join("test.toml"), &agents_dir);
    assert!(report.errors.iter().any(|e| e.error.contains("missing 'command'")), "{:?}", report.errors);
}

#[test]
fn http_mcp_missing_url_reports_error() {
    let (tmp, agents_dir) = setup();
    write(tmp.path(), ".ship/agents/mcp.toml", r#"
[[servers]]
id = "bad-http"
server_type = "http"
"#);
    write(tmp.path(), ".ship/agents/test.toml", valid_agent_toml());
    let report = validate_agent("test", &agents_dir.join("test.toml"), &agents_dir);
    assert!(report.errors.iter().any(|e| e.error.contains("missing 'url'")), "{:?}", report.errors);
}

#[test]
fn unknown_permissions_preset_reports_error() {
    let (tmp, agents_dir) = setup();
    let toml = r#"[agent]
name = "test"
id = "test"
providers = ["claude"]
[skills]
refs = []
[mcp]
servers = []
[permissions]
preset = "nonexistent-preset"
"#;
    write(tmp.path(), ".ship/agents/test.toml", toml);
    let report = validate_agent("test", &agents_dir.join("test.toml"), &agents_dir);
    assert!(report.errors.iter().any(|e| e.error.contains("nonexistent-preset")), "{:?}", report.errors);
}

#[test]
fn run_validate_exits_nonzero_on_errors() {
    let (tmp, _agents_dir) = setup();
    write(tmp.path(), ".ship/agents/bad.toml", "not valid toml [[");
    let result = run_validate(None, false, tmp.path());
    assert!(result.is_err());
}

#[test]
fn run_validate_json_flag_emits_array() {
    let (tmp, _agents_dir) = setup();
    write(tmp.path(), ".ship/agents/bad.toml", "not valid toml [[");
    // json mode returns Err (errors found) but prints JSON — we just check no panic
    let _ = run_validate(None, true, tmp.path());
}

#[test]
fn run_validate_passes_on_valid_config() {
    let (tmp, _agents_dir) = setup();
    write(tmp.path(), ".ship/agents/test.toml", valid_agent_toml());
    let result = run_validate(None, false, tmp.path());
    assert!(result.is_ok(), "{:?}", result);
}
