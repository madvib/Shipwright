mod helpers;
use crate::helpers::create_feature;
use helpers::TestProject;
use runtime::agent_config::FeatureAgentConfig;
use runtime::config::{McpServerConfig, McpServerType, ProjectConfig, save_config};
use runtime::create_skill;
use runtime::update_skill;
use ship_module_git::on_post_checkout;
use std::collections::HashMap;
use std::path::Path;

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

fn set_feature_agent(ship_dir: &Path, feature_id: &str, agent: FeatureAgentConfig) {
    let mut entry = ship_module_project::get_feature_by_id(ship_dir, feature_id).unwrap();
    entry.feature.metadata.agent = Some(agent);
    ship_module_project::update_feature(ship_dir, feature_id, entry.feature).unwrap();
}

#[test]
fn checkout_feature_branch_writes_claude_md() {
    let p = TestProject::with_git().unwrap();
    create_feature(
        p.ship_dir.clone(),
        "Auth flow",
        "Ship auth end-to-end.",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    assert!(p.checkout_new("feature/auth").unwrap().status.success());

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

    let claude_md = p.root().join("CLAUDE.md");
    assert!(claude_md.exists(), "CLAUDE.md should be generated");
    let content = std::fs::read_to_string(claude_md).unwrap();
    assert!(content.contains("# [ship] Auth flow"));
}

#[test]
fn checkout_non_feature_branch_is_noop() {
    let p = TestProject::with_git().unwrap();
    create_feature(
        p.ship_dir.clone(),
        "Auth flow",
        "",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();

    on_post_checkout(&p.ship_dir, "main", &p.root()).unwrap();
    assert!(
        p.root().join("CLAUDE.md").exists(),
        "workspace CLAUDE.md should be generated on non-feature branches"
    );
}

#[test]
fn claude_md_omits_issue_section() {
    let p = TestProject::with_git().unwrap();
    create_feature(
        p.ship_dir.clone(),
        "Auth flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();
    let content = std::fs::read_to_string(p.root().join("CLAUDE.md")).unwrap();
    assert!(!content.contains("## Open Issues"));
}

#[test]
fn claude_md_excludes_skill_body_and_exports_provider_skill_file() {
    let p = TestProject::with_git().unwrap();
    let feature_path = create_feature(
        p.ship_dir.clone(),
        "Auth flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    create_skill(
        &p.ship_dir,
        "nextjs-conventions",
        "Next.js Conventions",
        "Prefer route groups for auth pages.",
    )
    .unwrap();
    set_feature_agent(
        &p.ship_dir,
        &feature_path.0,
        FeatureAgentConfig {
            model: None,
            mcp_servers: vec![],
            skills: vec!["nextjs-conventions".to_string()],
            providers: vec![],
        },
    );

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

    let content = std::fs::read_to_string(p.root().join("CLAUDE.md")).unwrap();
    let skill_file = std::fs::read_to_string(
        p.root()
            .join(".claude")
            .join("skills")
            .join("nextjs-conventions")
            .join("SKILL.md"),
    )
    .unwrap();
    assert!(!content.contains("Prefer route groups for auth pages."));
    assert!(skill_file.contains("Prefer route groups for auth pages."));
}

#[test]
fn mcp_json_written_on_checkout() {
    let p = TestProject::with_git().unwrap();
    let mut config = ProjectConfig::default();
    config.mcp_servers = vec![make_stdio_server("github")];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    let feature_path = create_feature(
        p.ship_dir.clone(),
        "Auth flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    set_feature_agent(
        &p.ship_dir,
        &feature_path.0,
        FeatureAgentConfig {
            model: None,
            mcp_servers: vec!["github".to_string()],
            skills: vec![],
            providers: vec![],
        },
    );

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

    let mcp_json = p.root().join(".mcp.json");
    assert!(mcp_json.exists(), ".mcp.json should be generated");
    let val: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(mcp_json).unwrap()).unwrap();
    assert!(val["mcpServers"]["github"].is_object());
}

#[test]
fn repeated_post_checkout_is_deterministic_for_claude_md() {
    let p = TestProject::with_git().unwrap();
    create_feature(
        p.ship_dir.clone(),
        "Determinism",
        "Stable context generation.",
        None,
        None,
        Some("feature/determinism"),
    )
    .unwrap();

    on_post_checkout(&p.ship_dir, "feature/determinism", &p.root()).unwrap();
    let first = std::fs::read_to_string(p.root().join("CLAUDE.md")).unwrap();

    on_post_checkout(&p.ship_dir, "feature/determinism", &p.root()).unwrap();
    let second = std::fs::read_to_string(p.root().join("CLAUDE.md")).unwrap();

    assert_eq!(first, second, "CLAUDE.md should be stable across reruns");
}

#[test]
fn default_task_policy_is_not_inlined_into_generated_context() {
    let p = TestProject::with_git().unwrap();
    let feature_path = create_feature(
        p.ship_dir.clone(),
        "Policy Inclusion",
        "Ensure policy text is present.",
        None,
        None,
        Some("feature/policy"),
    )
    .unwrap();
    create_skill(
        &p.ship_dir,
        "ship-policy",
        "Ship Policy",
        "Use Ship as the system of record.",
    )
    .unwrap();
    set_feature_agent(
        &p.ship_dir,
        &feature_path.0,
        FeatureAgentConfig {
            model: None,
            mcp_servers: vec![],
            skills: vec!["ship-policy".to_string()],
            providers: vec![],
        },
    );

    on_post_checkout(&p.ship_dir, "feature/policy", &p.root()).unwrap();
    let content = std::fs::read_to_string(p.root().join("CLAUDE.md")).unwrap();
    let skill_file = std::fs::read_to_string(
        p.root()
            .join(".claude")
            .join("skills")
            .join("ship-policy")
            .join("SKILL.md"),
    )
    .unwrap();
    assert!(
        !content.contains("Use Ship as the system of record"),
        "skill guidance should not be included in CLAUDE.md"
    );
    assert!(
        skill_file.contains("Use Ship as the system of record"),
        "skill guidance should be exported to provider skill files"
    );
}

#[test]
fn claude_md_reflects_rule_updates_after_regeneration() {
    let p = TestProject::with_git().unwrap();
    create_feature(
        p.ship_dir.clone(),
        "Rule Sync",
        "Ensure rule changes flow into generated context.",
        None,
        None,
        Some("feature/rule-sync"),
    )
    .unwrap();

    let custom_rule = p.ship_dir.join("agents/rules/999-test-rule-sync.md");
    std::fs::write(
        &custom_rule,
        "Always include migration notes in release docs.",
    )
    .unwrap();

    on_post_checkout(&p.ship_dir, "feature/rule-sync", &p.root()).unwrap();
    let first = std::fs::read_to_string(p.root().join("CLAUDE.md")).unwrap();
    assert!(
        first.contains("Always include migration notes in release docs."),
        "initial custom rule should be present in CLAUDE.md"
    );

    std::fs::write(&custom_rule, "Never ship without explicit rollback notes.").unwrap();
    on_post_checkout(&p.ship_dir, "feature/rule-sync", &p.root()).unwrap();
    let second = std::fs::read_to_string(p.root().join("CLAUDE.md")).unwrap();

    assert!(
        second.contains("Never ship without explicit rollback notes."),
        "updated custom rule should be present after regeneration"
    );
    assert!(
        !second.contains("Always include migration notes in release docs."),
        "stale rule content should not remain after regeneration"
    );
}

#[test]
fn claude_md_reflects_skill_updates_after_regeneration() {
    let p = TestProject::with_git().unwrap();
    let feature_path = create_feature(
        p.ship_dir.clone(),
        "Skill Sync",
        "Ensure skill changes flow into generated context.",
        None,
        None,
        Some("feature/skill-sync"),
    )
    .unwrap();

    create_skill(
        &p.ship_dir,
        "skill-sync-test",
        "Skill Sync Test",
        "Always validate API contracts with baseline snapshots.",
    )
    .unwrap();
    set_feature_agent(
        &p.ship_dir,
        &feature_path.0,
        FeatureAgentConfig {
            model: None,
            mcp_servers: vec![],
            skills: vec!["skill-sync-test".to_string()],
            providers: vec![],
        },
    );

    on_post_checkout(&p.ship_dir, "feature/skill-sync", &p.root()).unwrap();
    let first = std::fs::read_to_string(p.root().join("CLAUDE.md")).unwrap();
    let first_skill = std::fs::read_to_string(
        p.root()
            .join(".claude")
            .join("skills")
            .join("skill-sync-test")
            .join("SKILL.md"),
    )
    .unwrap();
    assert!(
        !first.contains("Always validate API contracts with baseline snapshots."),
        "skill body should not be inlined into CLAUDE.md"
    );
    assert!(
        first_skill.contains("Always validate API contracts with baseline snapshots."),
        "initial skill content should be present in claude SKILL.md output"
    );

    update_skill(
        &p.ship_dir,
        "skill-sync-test",
        None,
        Some("Always validate API contracts with schema-driven checks."),
    )
    .unwrap();
    on_post_checkout(&p.ship_dir, "feature/skill-sync", &p.root()).unwrap();
    let second = std::fs::read_to_string(p.root().join("CLAUDE.md")).unwrap();
    let second_skill = std::fs::read_to_string(
        p.root()
            .join(".claude")
            .join("skills")
            .join("skill-sync-test")
            .join("SKILL.md"),
    )
    .unwrap();

    assert!(
        !second.contains("Always validate API contracts with baseline snapshots."),
        "stale skill content should not remain in CLAUDE.md after regeneration"
    );
    assert!(
        !second.contains("Always validate API contracts with schema-driven checks."),
        "updated skill content should not be inlined into CLAUDE.md"
    );
    assert!(
        second_skill.contains("Always validate API contracts with schema-driven checks."),
        "updated skill content should be present in claude SKILL.md output after regeneration"
    );
    assert!(
        !second_skill.contains("Always validate API contracts with baseline snapshots."),
        "stale skill content should not remain in claude SKILL.md output after regeneration"
    );
}

#[test]
fn agents_md_reflects_skill_updates_for_codex_after_regeneration() {
    let p = TestProject::with_git().unwrap();
    let feature_path = create_feature(
        p.ship_dir.clone(),
        "Codex Skill Sync",
        "Ensure Codex context reflects skill updates.",
        None,
        None,
        Some("feature/codex-skill-sync"),
    )
    .unwrap();

    create_skill(
        &p.ship_dir,
        "codex-skill-sync-test",
        "Codex Skill Sync Test",
        "Use strict release gating for codex provider.",
    )
    .unwrap();
    set_feature_agent(
        &p.ship_dir,
        &feature_path.0,
        FeatureAgentConfig {
            model: None,
            mcp_servers: vec![],
            skills: vec!["codex-skill-sync-test".to_string()],
            providers: vec!["codex".to_string()],
        },
    );

    on_post_checkout(&p.ship_dir, "feature/codex-skill-sync", &p.root()).unwrap();
    let first = std::fs::read_to_string(p.root().join("AGENTS.md")).unwrap();
    let first_skill = std::fs::read_to_string(
        p.root()
            .join(".agents")
            .join("skills")
            .join("codex-skill-sync-test")
            .join("SKILL.md"),
    )
    .unwrap();
    assert!(
        !first.contains("Use strict release gating for codex provider."),
        "skill content should not be inlined into AGENTS.md"
    );
    assert!(
        first_skill.contains("Use strict release gating for codex provider."),
        "initial skill content should be present in codex SKILL.md output"
    );
    assert!(
        !p.root().join("CLAUDE.md").exists(),
        "codex-only provider output should not emit CLAUDE.md"
    );

    update_skill(
        &p.ship_dir,
        "codex-skill-sync-test",
        None,
        Some("Use explicit rollback gates for codex provider."),
    )
    .unwrap();
    on_post_checkout(&p.ship_dir, "feature/codex-skill-sync", &p.root()).unwrap();
    let second = std::fs::read_to_string(p.root().join("AGENTS.md")).unwrap();
    let second_skill = std::fs::read_to_string(
        p.root()
            .join(".agents")
            .join("skills")
            .join("codex-skill-sync-test")
            .join("SKILL.md"),
    )
    .unwrap();

    assert!(
        !second.contains("Use strict release gating for codex provider."),
        "stale skill content should not remain in AGENTS.md after regeneration"
    );
    assert!(
        !second.contains("Use explicit rollback gates for codex provider."),
        "updated skill content should not be inlined into AGENTS.md after regeneration"
    );
    assert!(
        second_skill.contains("Use explicit rollback gates for codex provider."),
        "updated skill content should be present in codex SKILL.md after regeneration"
    );
    assert!(
        !second_skill.contains("Use strict release gating for codex provider."),
        "stale skill content should not remain in codex SKILL.md after regeneration"
    );
}
