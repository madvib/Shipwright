mod helpers;

use helpers::TestProject;
use runtime::{
    AgentLimits, CommandPermissions, McpServerConfig, McpServerType, ModeConfig,
    NetworkPermissions, NetworkPolicy, Permissions, ProjectConfig, ToolPermissions, create_skill,
    delete_skill, save_config, save_permissions, sync_active_mode,
};
use std::collections::HashMap;
use std::path::Path;

fn stdio_server(id: &str) -> McpServerConfig {
    McpServerConfig {
        id: id.to_string(),
        name: id.to_string(),
        command: "npx".to_string(),
        args: vec!["-y".to_string(), format!("@mcp/{id}")],
        env: HashMap::new(),
        scope: "project".to_string(),
        server_type: McpServerType::Stdio,
        url: None,
        disabled: false,
        timeout_secs: None,
    }
}

fn http_server(id: &str, url: &str) -> McpServerConfig {
    McpServerConfig {
        id: id.to_string(),
        name: id.to_string(),
        command: String::new(),
        args: vec![],
        env: HashMap::new(),
        scope: "project".to_string(),
        server_type: McpServerType::Http,
        url: Some(url.to_string()),
        disabled: false,
        timeout_secs: None,
    }
}

fn assert_skill_has_yaml_frontmatter(path: &Path, expected_name: &str) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("missing skill file at {}", path.display()));
    assert!(
        content.starts_with("---\n"),
        "expected YAML frontmatter in {}",
        path.display()
    );
    assert!(
        content.contains(&format!("\nname: {expected_name}\n")),
        "expected name '{}' in frontmatter at {}",
        expected_name,
        path.display()
    );
    assert!(
        content.contains("\ndescription: "),
        "expected description in frontmatter at {}",
        path.display()
    );
}

#[test]
fn compiler_matrix_webapp_multi_provider_exports_all_surfaces() {
    let p = TestProject::with_fixture_project("webapp-nextjs").unwrap();

    create_skill(
        &p.ship_dir,
        "release-guard",
        "Release Guard",
        "Run regression checks before shipping.",
    )
    .unwrap();

    std::fs::write(
        p.ship_dir.join("agents/rules/no-secrets.md"),
        "Never commit secrets in source files.",
    )
    .unwrap();

    let mut config = ProjectConfig::default();
    config.providers = vec![
        "claude".to_string(),
        "gemini".to_string(),
        "codex".to_string(),
    ];
    config.active_mode = Some("release".to_string());
    config.modes = vec![ModeConfig {
        id: "release".to_string(),
        name: "Release".to_string(),
        target_agents: vec![
            "claude".to_string(),
            "gemini".to_string(),
            "codex".to_string(),
        ],
        ..Default::default()
    }];
    config.mcp_servers = vec![
        stdio_server("github"),
        http_server("figma", "https://mcp.figma.com/mcp"),
    ];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    save_permissions(
        p.ship_dir.clone(),
        &Permissions {
            tools: ToolPermissions {
                allow: vec!["*".to_string()],
                deny: vec![],
            },
            commands: CommandPermissions {
                allow: vec!["cargo *".to_string()],
                deny: vec!["rm -rf *".to_string()],
            },
            agent: AgentLimits {
                require_confirmation: vec!["git push *".to_string()],
                ..Default::default()
            },
            network: NetworkPermissions {
                policy: NetworkPolicy::AllowList,
                allow_hosts: vec!["github.com".to_string()],
            },
            ..Default::default()
        },
    )
    .unwrap();

    let synced = sync_active_mode(&p.ship_dir).unwrap();
    assert!(synced.iter().any(|p| p == "claude"));
    assert!(synced.iter().any(|p| p == "gemini"));
    assert!(synced.iter().any(|p| p == "codex"));

    assert!(
        p.root()
            .join(".claude/skills/release-guard/SKILL.md")
            .exists(),
        "claude skill should be exported"
    );
    assert!(
        p.root()
            .join(".gemini/skills/release-guard/SKILL.md")
            .exists(),
        "gemini skill should be exported"
    );
    assert!(
        p.root()
            .join(".agents/skills/release-guard/SKILL.md")
            .exists(),
        "codex skill should be exported"
    );
    assert_skill_has_yaml_frontmatter(
        &p.root().join(".claude/skills/release-guard/SKILL.md"),
        "release-guard",
    );
    assert_skill_has_yaml_frontmatter(
        &p.root().join(".gemini/skills/release-guard/SKILL.md"),
        "release-guard",
    );
    assert_skill_has_yaml_frontmatter(
        &p.root().join(".agents/skills/release-guard/SKILL.md"),
        "release-guard",
    );

    let claude_mcp: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(p.root().join(".mcp.json")).unwrap())
            .unwrap();
    assert!(claude_mcp["mcpServers"]["ship"].is_object());
    assert_eq!(
        claude_mcp["mcpServers"]["figma"]["url"].as_str(),
        Some("https://mcp.figma.com/mcp")
    );

    let gemini_settings: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(p.root().join(".gemini/settings.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        gemini_settings["mcpServers"]["figma"]["httpUrl"].as_str(),
        Some("https://mcp.figma.com/mcp")
    );

    let codex_cfg: toml::Value =
        toml::from_str(&std::fs::read_to_string(p.root().join(".codex/config.toml")).unwrap())
            .unwrap();
    assert!(codex_cfg["mcp_servers"]["ship"].is_table());
    assert_eq!(
        codex_cfg["sandbox_workspace_write"]["network_access"].as_bool(),
        Some(true)
    );
    assert!(
        codex_cfg.get("allow").is_none(),
        "legacy codex allow field should not be emitted"
    );

    let codex_rules = std::fs::read_to_string(p.root().join(".codex/rules/ship.rules")).unwrap();
    assert!(codex_rules.contains("decision = \"forbidden\""));
    assert!(codex_rules.contains("decision = \"prompt\""));
    assert!(codex_rules.contains("decision = \"allow\""));
    assert!(codex_rules.contains("pattern = [\"rm\", \"-rf\"]"));
    assert!(codex_rules.contains("pattern = [\"git\", \"push\"]"));

    let gemini_policy =
        std::fs::read_to_string(p.root().join(".gemini/policies/ship-permissions.toml")).unwrap();
    assert!(gemini_policy.contains("commandPrefix = \"rm -rf\""));
    assert!(gemini_policy.contains("decision = \"ask_user\""));
}

#[test]
fn compiler_matrix_rust_fixture_codex_only_respects_provider_scope() {
    let p = TestProject::with_fixture_project("rust-cli").unwrap();

    create_skill(
        &p.ship_dir,
        "rust-release",
        "Rust Release",
        "Build and test before creating tags.",
    )
    .unwrap();

    let mut config = ProjectConfig::default();
    config.providers = vec!["codex".to_string()];
    config.active_mode = Some("codex-only".to_string());
    config.modes = vec![ModeConfig {
        id: "codex-only".to_string(),
        name: "Codex Only".to_string(),
        target_agents: vec!["codex".to_string()],
        ..Default::default()
    }];
    config.mcp_servers = vec![stdio_server("github")];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    save_permissions(
        p.ship_dir.clone(),
        &Permissions {
            commands: CommandPermissions {
                allow: vec![],
                deny: vec!["rm -rf *".to_string()],
            },
            agent: AgentLimits {
                require_confirmation: vec!["git push *".to_string()],
                ..Default::default()
            },
            network: NetworkPermissions {
                policy: NetworkPolicy::None,
                allow_hosts: vec![],
            },
            ..Default::default()
        },
    )
    .unwrap();

    let synced = sync_active_mode(&p.ship_dir).unwrap();
    assert_eq!(synced, vec!["codex".to_string()]);

    assert!(p.root().join(".codex/config.toml").exists());
    assert!(p.root().join(".codex/rules/ship.rules").exists());
    assert!(
        p.root()
            .join(".agents/skills/rust-release/SKILL.md")
            .exists()
    );
    assert!(
        !p.root().join(".mcp.json").exists(),
        "claude mcp file should not be emitted"
    );
    assert!(
        !p.root().join(".gemini/settings.json").exists(),
        "gemini settings should not be emitted"
    );
}

#[test]
fn compiler_matrix_prunes_stale_managed_skills_for_all_providers() {
    let p = TestProject::with_fixture_project("webapp-nextjs").unwrap();
    create_skill(&p.ship_dir, "rt-live", "Live", "live body").unwrap();
    create_skill(&p.ship_dir, "rt-stale", "Stale", "stale body").unwrap();

    let mut config = ProjectConfig::default();
    config.providers = vec![
        "claude".to_string(),
        "gemini".to_string(),
        "codex".to_string(),
    ];
    config.active_mode = Some("tri-provider".to_string());
    config.modes = vec![ModeConfig {
        id: "tri-provider".to_string(),
        name: "Tri Provider".to_string(),
        target_agents: vec![
            "claude".to_string(),
            "gemini".to_string(),
            "codex".to_string(),
        ],
        ..Default::default()
    }];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    sync_active_mode(&p.ship_dir).unwrap();
    assert!(p.root().join(".claude/skills/rt-stale/SKILL.md").exists());
    assert!(p.root().join(".gemini/skills/rt-stale/SKILL.md").exists());
    assert!(p.root().join(".agents/skills/rt-stale/SKILL.md").exists());

    delete_skill(&p.ship_dir, "rt-stale").unwrap();
    sync_active_mode(&p.ship_dir).unwrap();

    assert!(p.root().join(".claude/skills/rt-live/SKILL.md").exists());
    assert!(p.root().join(".gemini/skills/rt-live/SKILL.md").exists());
    assert!(p.root().join(".agents/skills/rt-live/SKILL.md").exists());

    assert!(
        !p.root().join(".claude/skills/rt-stale").exists(),
        "stale claude skill dir should be pruned"
    );
    assert!(
        !p.root().join(".gemini/skills/rt-stale").exists(),
        "stale gemini skill dir should be pruned"
    );
    assert!(
        !p.root().join(".agents/skills/rt-stale").exists(),
        "stale codex skill dir should be pruned"
    );
}
