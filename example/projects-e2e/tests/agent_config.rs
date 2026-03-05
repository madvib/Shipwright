mod helpers;
use helpers::TestProject;
use runtime::config::{McpServerConfig, McpServerType, ModeConfig, ProjectConfig, save_config};
use runtime::{
    Permissions, ToolPermissions, create_skill, delete_skill, save_permissions, update_skill,
};
use std::collections::HashMap;
use std::process::Output;

fn assert_success(out: &Output, context: &str) {
    assert!(
        out.status.success(),
        "{}\nstdout: {}\nstderr: {}",
        context,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn make_stdio_server(id: &str) -> McpServerConfig {
    McpServerConfig {
        id: id.to_string(),
        name: id.to_string(),
        command: "npx".to_string(),
        args: vec!["-y".to_string(), format!("@mcp/{}", id)],
        env: HashMap::new(),
        scope: "project".to_string(),
        server_type: McpServerType::Stdio,
        url: None,
        disabled: false,
        timeout_secs: None,
    }
}

/// export_to("claude") writes .mcp.json at project root.
#[test]
fn claude_export_writes_mcp_json_at_project_root() {
    let p = TestProject::new().unwrap();
    let mut config = ProjectConfig::default();
    config.mcp_servers = vec![make_stdio_server("github")];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    runtime::agents::export::export_to(p.ship_dir.clone(), "claude").unwrap();

    let mcp_json = p.root().join(".mcp.json");
    assert!(mcp_json.exists(), ".mcp.json should exist at project root");

    let content = std::fs::read_to_string(&mcp_json).unwrap();
    let val: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(val["mcpServers"]["github"].is_object());
    assert!(
        val["mcpServers"]["ship"].is_object(),
        "ship server always injected"
    );
}

/// Disabled servers are not exported.
#[test]
fn disabled_server_not_exported() {
    let p = TestProject::new().unwrap();
    let mut server = make_stdio_server("disabled-one");
    server.disabled = true;
    let mut config = ProjectConfig::default();
    config.mcp_servers = vec![server];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    runtime::agents::export::export_to(p.ship_dir.clone(), "claude").unwrap();

    let content = std::fs::read_to_string(p.root().join(".mcp.json")).unwrap();
    let val: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(
        val["mcpServers"]["disabled-one"].is_null(),
        "disabled server should not appear in .mcp.json"
    );
}

/// Second export preserves user-added servers (no _ship marker).
#[test]
fn export_preserves_user_servers() {
    let p = TestProject::new().unwrap();
    let mut config = ProjectConfig::default();
    config.mcp_servers = vec![make_stdio_server("mine")];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    runtime::agents::export::export_to(p.ship_dir.clone(), "claude").unwrap();

    // Manually inject a user server
    let mcp_json = p.root().join(".mcp.json");
    let mut val: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&mcp_json).unwrap()).unwrap();
    val["mcpServers"]["user-server"] = serde_json::json!({ "command": "user-tool", "args": [] });
    std::fs::write(&mcp_json, serde_json::to_string_pretty(&val).unwrap()).unwrap();

    // Re-export — user server must survive
    runtime::agents::export::export_to(p.ship_dir.clone(), "claude").unwrap();

    let content = std::fs::read_to_string(&mcp_json).unwrap();
    let val2: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(
        val2["mcpServers"]["user-server"].is_object(),
        "user server was clobbered by re-export"
    );
}

/// Gemini export uses httpUrl field (not url) for HTTP servers.
#[test]
fn gemini_http_server_uses_httpurl() {
    let p = TestProject::new().unwrap();
    let mut config = ProjectConfig::default();
    config.mcp_servers = vec![McpServerConfig {
        id: "figma".to_string(),
        name: "figma".to_string(),
        command: String::new(),
        args: vec![],
        env: HashMap::new(),
        scope: "project".to_string(),
        server_type: McpServerType::Http,
        url: Some("https://mcp.figma.com/mcp".to_string()),
        disabled: false,
        timeout_secs: None,
    }];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    runtime::agents::export::export_to(p.ship_dir.clone(), "gemini").unwrap();

    let settings = p.root().join(".gemini/settings.json");
    assert!(settings.exists());
    let val: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
    assert!(val["mcpServers"]["figma"]["httpUrl"].is_string());
    assert!(val["mcpServers"]["figma"]["url"].is_null());
}

/// Canonical permissions.toml propagates through CLI export to ~/.claude/settings.json.
#[test]
fn claude_export_writes_permissions_to_home_settings() {
    let p = TestProject::new().unwrap();
    let home = tempfile::TempDir::new().unwrap();

    save_permissions(
        p.ship_dir.clone(),
        &Permissions {
            tools: ToolPermissions {
                allow: vec!["Bash(*)".to_string()],
                deny: vec!["WebFetch(*)".to_string()],
            },
            ..Default::default()
        },
    )
    .unwrap();

    let out = p
        .cli(&["config", "export", "--target", "claude"])
        .env("HOME", home.path())
        .env("SHIP_GLOBAL_DIR", &p.global_dir)
        .output()
        .unwrap();
    assert_success(&out, "ship config export claude failed");

    let settings_path = home.path().join(".claude/settings.json");
    assert!(
        settings_path.exists(),
        "expected Claude settings at {}",
        settings_path.display()
    );

    let settings_raw = std::fs::read_to_string(settings_path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&settings_raw).unwrap();

    let allow = value["permissions"]["allow"].as_array().unwrap();
    let deny = value["permissions"]["deny"].as_array().unwrap();
    assert!(
        allow.iter().any(|v| v.as_str() == Some("Bash(*)")),
        "missing allow permission in Claude settings"
    );
    assert!(
        deny.iter().any(|v| v.as_str() == Some("WebFetch(*)")),
        "missing deny permission in Claude settings"
    );
}

/// Non-Claude exports should not touch Claude home settings.
#[test]
fn gemini_export_does_not_write_claude_settings_file() {
    let p = TestProject::new().unwrap();
    let home = tempfile::TempDir::new().unwrap();

    save_permissions(
        p.ship_dir.clone(),
        &Permissions {
            tools: ToolPermissions {
                allow: vec!["Bash(*)".to_string()],
                deny: vec!["WebFetch(*)".to_string()],
            },
            ..Default::default()
        },
    )
    .unwrap();

    let out = p
        .cli(&["config", "export", "--target", "gemini"])
        .env("HOME", home.path())
        .env("SHIP_GLOBAL_DIR", &p.global_dir)
        .output()
        .unwrap();
    assert_success(&out, "ship config export gemini failed");

    let settings_path = home.path().join(".claude/settings.json");
    assert!(
        !settings_path.exists(),
        "gemini export should not create Claude settings at {}",
        settings_path.display()
    );
}

/// Instruction skill content updates should propagate to regenerated provider docs.
#[test]
fn gemini_export_reflects_instruction_skill_updates_after_regeneration() {
    let p = TestProject::new().unwrap();
    create_skill(
        &p.ship_dir,
        "release-gate-prompt",
        "Release Gate Prompt",
        "Always run the initial release gate checklist.",
    )
    .unwrap();

    let mut config = ProjectConfig::default();
    config.modes = vec![ModeConfig {
        id: "release".to_string(),
        name: "Release".to_string(),
        prompt_id: Some("release-gate-prompt".to_string()),
        ..Default::default()
    }];
    config.active_mode = Some("release".to_string());
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    runtime::agents::export::export_to(p.ship_dir.clone(), "gemini").unwrap();
    let first = std::fs::read_to_string(p.root().join("GEMINI.md")).unwrap();
    assert!(
        first.contains("Always run the initial release gate checklist."),
        "initial instruction content should be written to GEMINI.md"
    );

    update_skill(
        &p.ship_dir,
        "release-gate-prompt",
        None,
        Some("Always run the updated release gate checklist."),
    )
    .unwrap();

    runtime::agents::export::export_to(p.ship_dir.clone(), "gemini").unwrap();
    let second = std::fs::read_to_string(p.root().join("GEMINI.md")).unwrap();
    assert!(
        second.contains("Always run the updated release gate checklist."),
        "updated instruction content should be written after regeneration"
    );
    assert!(
        !second.contains("Always run the initial release gate checklist."),
        "stale instruction content should not remain after regeneration"
    );
}

/// Codex export should keep AGENTS.md instruction output in sync across repeated regeneration.
#[test]
fn codex_export_reflects_instruction_skill_updates_after_regeneration() {
    let p = TestProject::new().unwrap();
    create_skill(
        &p.ship_dir,
        "codex-release-prompt",
        "Codex Release Prompt",
        "Codex release checklist v1.",
    )
    .unwrap();

    let mut config = ProjectConfig::default();
    config.modes = vec![ModeConfig {
        id: "release".to_string(),
        name: "Release".to_string(),
        prompt_id: Some("codex-release-prompt".to_string()),
        ..Default::default()
    }];
    config.active_mode = Some("release".to_string());
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    runtime::agents::export::export_to(p.ship_dir.clone(), "codex").unwrap();
    let first = std::fs::read_to_string(p.root().join("AGENTS.md")).unwrap();
    assert!(
        first.contains("Codex release checklist v1."),
        "initial instruction content should be written to AGENTS.md"
    );

    update_skill(
        &p.ship_dir,
        "codex-release-prompt",
        None,
        Some("Codex release checklist v2."),
    )
    .unwrap();

    runtime::agents::export::export_to(p.ship_dir.clone(), "codex").unwrap();
    let second = std::fs::read_to_string(p.root().join("AGENTS.md")).unwrap();
    assert!(
        second.contains("Codex release checklist v2."),
        "updated instruction content should be written to AGENTS.md after regeneration"
    );
    assert!(
        !second.contains("Codex release checklist v1."),
        "stale instruction content should not remain in AGENTS.md after regeneration"
    );
}

fn assert_deleted_skill_is_pruned_for_target(target: &str, skill_dir_prefix: &str, skill_id: &str) {
    let p = TestProject::new().unwrap();
    create_skill(
        &p.ship_dir,
        skill_id,
        &format!("{} skill", skill_id),
        "Managed skill content that should be pruned when deleted.",
    )
    .unwrap();

    runtime::agents::export::export_to(p.ship_dir.clone(), target).unwrap();
    let skill_dir = p.root().join(skill_dir_prefix).join(skill_id);
    assert!(
        skill_dir.join("SKILL.md").exists(),
        "expected managed skill output for target {target}: {}",
        skill_dir.display()
    );

    delete_skill(&p.ship_dir, skill_id).unwrap();
    runtime::agents::export::export_to(p.ship_dir.clone(), target).unwrap();
    assert!(
        !skill_dir.exists(),
        "deleted skill should be pruned for target {target}: {}",
        skill_dir.display()
    );
}

#[test]
fn claude_export_prunes_deleted_managed_skill_dirs() {
    assert_deleted_skill_is_pruned_for_target("claude", ".claude/skills", "prune-skill-claude");
}

#[test]
fn gemini_export_prunes_deleted_managed_skill_dirs() {
    assert_deleted_skill_is_pruned_for_target("gemini", ".gemini/skills", "prune-skill-gemini");
}

#[test]
fn codex_export_prunes_deleted_managed_skill_dirs() {
    assert_deleted_skill_is_pruned_for_target("codex", ".agents/skills", "prune-skill-codex");
}
