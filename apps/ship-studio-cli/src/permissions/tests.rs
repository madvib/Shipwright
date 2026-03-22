use super::*;
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, content: &str) {
    let p = dir.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

#[test]
fn patch_adds_new_allows_to_empty_permissions() {
    let toml = r#"[agent]
name = "Test"
id = "test"
providers = ["claude"]
"#;
    let result = patch_profile_toml(toml, &["Bash(cargo test*)".to_string()], &[]).unwrap();
    let parsed: ProfileForSync = toml::from_str(&result).unwrap();
    assert!(parsed.permissions.tools_allow.contains(&"Bash(cargo test*)".to_string()));
}

#[test]
fn patch_adds_new_denies() {
    let toml = r#"[agent]
name = "Test"
id = "test"
providers = ["claude"]

[permissions]
tools_allow = ["Read"]
"#;
    let result = patch_profile_toml(toml, &[], &["Bash(rm -rf *)".to_string()]).unwrap();
    let parsed: ProfileForSync = toml::from_str(&result).unwrap();
    assert!(parsed.permissions.tools_deny.contains(&"Bash(rm -rf *)".to_string()));
    // Existing allow must survive.
    assert!(parsed.permissions.tools_allow.contains(&"Read".to_string()));
}

#[test]
fn patch_is_idempotent() {
    let toml = r#"[agent]
name = "Test"
id = "test"
providers = ["claude"]

[permissions]
tools_allow = ["Bash(cargo test*)"]
"#;
    // Delta is now empty — already in agent.
    let result = patch_profile_toml(toml, &[], &[]).unwrap();
    let parsed: ProfileForSync = toml::from_str(&result).unwrap();
    // Existing allow must survive unchanged.
    assert_eq!(parsed.permissions.tools_allow, vec!["Bash(cargo test*)"]);
}

#[test]
fn is_shadowing_allow_detects_prefix_match() {
    let allow = vec!["Bash(cargo*)".to_string()];
    // "Bash" shadows "Bash(cargo*)"
    assert!(is_shadowing_allow("Bash", &allow));
    // "Read" does not shadow "Bash(cargo*)"
    assert!(!is_shadowing_allow("Read", &allow));
}

#[test]
fn load_local_settings_returns_default_when_missing() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join(".claude").join("settings.local.json");
    let settings = load_local_settings(&path).unwrap();
    assert!(settings.permissions.allow.is_empty());
    assert!(settings.permissions.deny.is_empty());
}

#[test]
fn load_local_settings_parses_allow_deny() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("settings.local.json");
    write(tmp.path(), "settings.local.json", r#"
{
  "permissions": {
    "allow": ["Bash(cargo test*)"],
    "deny": ["Bash(rm -rf *)"]
  }
}
"#);
    let settings = load_local_settings(&path).unwrap();
    assert_eq!(settings.permissions.allow, vec!["Bash(cargo test*)"]);
    assert_eq!(settings.permissions.deny, vec!["Bash(rm -rf *)"]);
}

#[test]
fn sync_writes_new_allows_to_agent() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    write(tmp.path(), ".ship/ship.toml", "id = \"test-proj\"\nname = \"test\"\n");
    // Write workspace state with active agent via platform.db
    let mut state = WorkspaceState::default();
    state.active_agent = Some("test".to_string());
    state.save(&ship_dir).unwrap();
    // Write the agent
    write(tmp.path(), ".ship/agents/test.toml", r#"[agent]
name = "Test"
id = "test"
providers = ["claude"]
"#);
    // Write local session decisions
    write(tmp.path(), ".claude/settings.local.json", r#"
{
  "permissions": {
    "allow": ["Bash(cargo build*)"],
    "deny": []
  }
}
"#);
    run_permissions_sync(tmp.path()).unwrap();
    let updated = std::fs::read_to_string(
        tmp.path().join(".ship/agents/test.toml")
    ).unwrap();
    let parsed: ProfileForSync = toml::from_str(&updated).unwrap();
    assert!(parsed.permissions.tools_allow.contains(&"Bash(cargo build*)".to_string()));
}

#[test]
fn sync_warns_deny_that_shadows_allow_but_does_not_import() {
    let tmp = TempDir::new().unwrap();
    let ship_dir = tmp.path().join(".ship");
    write(tmp.path(), ".ship/ship.toml", "id = \"test-proj\"\nname = \"test\"\n");
    // Write workspace state with active agent via platform.db
    let mut state = WorkspaceState::default();
    state.active_agent = Some("test".to_string());
    state.save(&ship_dir).unwrap();
    write(tmp.path(), ".ship/agents/test.toml", r#"[agent]
name = "Test"
id = "test"
providers = ["claude"]

[permissions]
tools_allow = ["Bash(cargo*)"]
"#);
    // Session tried to deny "Bash" which would shadow "Bash(cargo*)"
    write(tmp.path(), ".claude/settings.local.json", r#"
{
  "permissions": {
    "allow": [],
    "deny": ["Bash"]
  }
}
"#);
    // Should succeed without panicking; the deny must NOT be imported.
    run_permissions_sync(tmp.path()).unwrap();
    let updated = std::fs::read_to_string(
        tmp.path().join(".ship/agents/test.toml")
    ).unwrap();
    let parsed: ProfileForSync = toml::from_str(&updated).unwrap();
    assert!(!parsed.permissions.tools_deny.contains(&"Bash".to_string()),
        "shadowing deny must not be imported");
}
