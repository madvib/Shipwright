mod helpers;
use crate::helpers::create_feature;
use helpers::TestProject;
use runtime::agent_config::FeatureAgentConfig;
use runtime::config::{McpServerConfig, McpServerType, ProjectConfig, save_config};
use runtime::create_skill;
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

fn get_feature_id(path: &Path) -> String {
    let content = std::fs::read_to_string(path).unwrap();
    for line in content.lines() {
        if line.starts_with("id = ") {
            return line.split('"').nth(1).unwrap().to_string();
        }
    }
    panic!("No ID found in {:?}", path);
}

fn set_feature_agent(path: &Path, agent: FeatureAgentConfig) {
    let ship_dir = path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let id = get_feature_id(path);
    let mut entry = ship_module_project::get_feature_by_id(ship_dir, &id).unwrap();
    entry.feature.metadata.agent = Some(agent);
    ship_module_project::update_feature(ship_dir, &id, entry.feature).unwrap();
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
        !p.root().join("CLAUDE.md").exists(),
        "CLAUDE.md should not be generated on non-feature branches"
    );
}

#[test]
fn claude_md_includes_open_issues() {
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
    ship_module_project::create_issue(
        &p.ship_dir,
        "Handle login timeout",
        "Need a retry strategy",
        ship_module_project::IssueStatus::Backlog,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    ship_module_project::create_issue(
        &p.ship_dir,
        "Already shipped",
        "Closed issue",
        ship_module_project::IssueStatus::Done,
        None,
        None,
        None,
        None,
    )
    .unwrap();

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();
    let content = std::fs::read_to_string(p.root().join("CLAUDE.md")).unwrap();
    assert!(content.contains("Handle login timeout"));
    assert!(!content.contains("Already shipped"));
}

#[test]
fn claude_md_includes_skill_body() {
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
        &feature_path.1,
        FeatureAgentConfig {
            model: None,
            max_cost_per_session: None,
            mcp_servers: vec![],
            skills: vec!["nextjs-conventions".to_string()],
            providers: vec![],
        },
    );

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

    let content = std::fs::read_to_string(p.root().join("CLAUDE.md")).unwrap();
    assert!(content.contains("Prefer route groups for auth pages."));
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
        &feature_path.1,
        FeatureAgentConfig {
            model: None,
            max_cost_per_session: None,
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
