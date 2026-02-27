/// Branch → Agent Configuration E2E Tests
///
/// Tests the full lifecycle: feature creation → branch checkout → CLAUDE.md/.mcp.json
/// generation → teardown on branch switch. Also covers worktrees and multi-provider.
///
/// Tests marked #[ignore] document planned behaviour that is not yet implemented.
/// Run passing tests: cargo test --test branch_config
/// Run all including planned: cargo test --test branch_config -- --include-ignored
mod helpers;

use helpers::TestProject;
use runtime::config::{McpServerConfig, McpServerType, ProjectConfig, save_config};
use runtime::{
    FeatureAgentConfig, FeatureMcpRef, FeatureSkillRef, create_feature, create_issue, create_skill,
    get_feature,
};
use ship_module_git::on_post_checkout;
use std::collections::HashMap;
use std::path::Path;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn stdio_server(id: &str) -> McpServerConfig {
    McpServerConfig {
        id: id.to_string(),
        name: id.to_string(),
        command: "echo".to_string(),
        args: vec![id.to_string()],
        env: HashMap::new(),
        scope: "project".to_string(),
        server_type: McpServerType::Stdio,
        url: None,
        disabled: false,
        timeout_secs: None,
    }
}

fn set_feature_agent(path: &Path, agent: FeatureAgentConfig) {
    let mut feature = get_feature(path.to_path_buf()).unwrap();
    feature.metadata.agent = Some(agent);
    std::fs::write(path, feature.to_markdown().unwrap()).unwrap();
}

// ─── Happy path ──────────────────────────────────────────────────────────────

/// Full happy path via library call: feature linked to branch → CLAUDE.md written.
#[test]
fn happy_path_feature_branch_generates_claude_md() {
    let p = TestProject::with_git().unwrap();

    create_feature(p.ship_dir.clone(), "Auth Flow", "Implement auth.", None, None, Some("feature/auth")).unwrap();
    on_post_checkout(&p.ship_dir, "feature/auth").unwrap();

    p.assert_root_file("CLAUDE.md");
    p.assert_root_file_contains("CLAUDE.md", "# [ship] Auth Flow");
    p.assert_root_file_contains("CLAUDE.md", "Implement auth.");
}

/// Open issues (not done) appear in CLAUDE.md; closed ones do not.
#[test]
fn claude_md_lists_open_issues_only() {
    let p = TestProject::with_git().unwrap();
    create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, Some("feature/auth")).unwrap();
    create_issue(p.ship_dir.clone(), "Add login page", "", "backlog").unwrap();
    create_issue(p.ship_dir.clone(), "Already shipped", "", "done").unwrap();

    on_post_checkout(&p.ship_dir, "feature/auth").unwrap();

    p.assert_root_file_contains("CLAUDE.md", "Add login page");
    p.assert_root_file_not_contains("CLAUDE.md", "Already shipped");
}

/// Skill content is inlined into CLAUDE.md under ## Skills.
#[test]
fn claude_md_inlines_skill_content() {
    let p = TestProject::with_git().unwrap();
    let feat = create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, Some("feature/auth")).unwrap();
    create_skill(&p.ship_dir, "conventions", "Project Conventions", "Always use anyhow for errors.").unwrap();
    set_feature_agent(&feat, FeatureAgentConfig { providers: vec![], model: None,
        max_cost_per_session: None,
        mcp_servers: vec![],
        skills: vec![FeatureSkillRef { id: "conventions".to_string() }],
    });

    on_post_checkout(&p.ship_dir, "feature/auth").unwrap();

    p.assert_root_file_contains("CLAUDE.md", "Always use anyhow for errors.");
}

/// Feature-level MCP filtering should restrict .mcp.json to only the declared servers.
/// Currently broken: agent_export::export_to rebuilds payload from project config,
/// ignoring the resolved feature agent. All project servers are written regardless.
///
/// Fix: pass resolved mcp_server_ids through to export_claude so it can filter.
#[test]
#[ignore = "feature-level MCP server filtering not propagated to .mcp.json — all project servers written"]
fn mcp_json_contains_only_feature_declared_servers() {
    let p = TestProject::with_git().unwrap();
    let mut config = ProjectConfig::default();
    config.mcp_servers = vec![stdio_server("github"), stdio_server("linear")];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    let feat = create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, Some("feature/auth")).unwrap();
    set_feature_agent(&feat, FeatureAgentConfig { providers: vec![], model: None,
        max_cost_per_session: None,
        mcp_servers: vec![FeatureMcpRef { id: "github".to_string() }],
        skills: vec![],
    });

    on_post_checkout(&p.ship_dir, "feature/auth").unwrap();

    let mcp_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(p.root().join(".mcp.json")).unwrap()
    ).unwrap();
    assert!(mcp_json["mcpServers"]["github"].is_object(), "github should be in .mcp.json");
    // linear was not declared on the feature — should be absent
    assert!(mcp_json["mcpServers"]["linear"].is_null(), "linear should be absent from .mcp.json");
}

/// Documents the actual (broken) current behavior: all project servers are written
/// to .mcp.json even when the feature declares only a subset.
#[test]
fn mcp_json_currently_includes_all_project_servers_ignoring_feature_filter() {
    let p = TestProject::with_git().unwrap();
    let mut config = ProjectConfig::default();
    config.mcp_servers = vec![stdio_server("github"), stdio_server("linear")];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    let feat = create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, Some("feature/auth")).unwrap();
    set_feature_agent(&feat, FeatureAgentConfig { providers: vec![], model: None, max_cost_per_session: None,
        mcp_servers: vec![FeatureMcpRef { id: "github".to_string() }], // only github
        skills: vec![],
    });

    on_post_checkout(&p.ship_dir, "feature/auth").unwrap();

    let mcp_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(p.root().join(".mcp.json")).unwrap()
    ).unwrap();
    // BUG: linear appears even though it wasn't declared on the feature
    assert!(mcp_json["mcpServers"]["linear"].is_object(),
        "KNOWN BUG: linear is currently written despite not being declared on the feature");
}

/// When no agent config is set on the feature, all project servers are included.
#[test]
fn mcp_json_falls_back_to_all_project_servers() {
    let p = TestProject::with_git().unwrap();
    let mut config = ProjectConfig::default();
    config.mcp_servers = vec![stdio_server("github"), stdio_server("linear")];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, Some("feature/auth")).unwrap();
    on_post_checkout(&p.ship_dir, "feature/auth").unwrap();

    let mcp_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(p.root().join(".mcp.json")).unwrap()
    ).unwrap();
    assert!(mcp_json["mcpServers"]["github"].is_object());
    assert!(mcp_json["mcpServers"]["linear"].is_object());
}

/// Non-feature branch (no match) does not produce CLAUDE.md.
#[test]
fn non_feature_branch_does_not_generate_claude_md() {
    let p = TestProject::with_git().unwrap();
    create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, Some("feature/auth")).unwrap();

    on_post_checkout(&p.ship_dir, "main").unwrap();

    p.assert_no_root_file("CLAUDE.md");
}

// ─── Teardown (currently broken — no cleanup pass) ───────────────────────────

/// When switching away from a feature branch to main, CLAUDE.md should be removed.
#[test]
fn switching_to_main_removes_stale_claude_md() {
    let p = TestProject::with_git().unwrap();
    create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, Some("feature/auth")).unwrap();

    // Generate files on feature branch
    on_post_checkout(&p.ship_dir, "feature/auth").unwrap();
    p.assert_root_file("CLAUDE.md");

    // Switch to main — should clean up
    on_post_checkout(&p.ship_dir, "main").unwrap();
    p.assert_no_root_file("CLAUDE.md");
}

/// Same teardown requirement for .mcp.json: Ship-managed servers should be removed
/// when checking out a non-feature branch. User servers must be preserved.
#[test]
fn switching_to_main_removes_ship_managed_mcp_servers() {
    let p = TestProject::with_git().unwrap();
    let mut config = ProjectConfig::default();
    config.mcp_servers = vec![stdio_server("github")];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    let feat = create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, Some("feature/auth")).unwrap();
    set_feature_agent(&feat, FeatureAgentConfig { providers: vec![], model: None, max_cost_per_session: None,
        mcp_servers: vec![FeatureMcpRef { id: "github".to_string() }],
        skills: vec![],
    });
    on_post_checkout(&p.ship_dir, "feature/auth").unwrap();

    // Switch away — ship-managed github entry should be removed
    on_post_checkout(&p.ship_dir, "main").unwrap();
    let mcp_path = p.root().join(".mcp.json");
    if mcp_path.exists() {
        let val: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&mcp_path).unwrap()).unwrap();
        assert!(val["mcpServers"]["github"].is_null(), "ship-managed server should be removed on teardown");
    }
}

// ─── Real hook firing via git checkout ──────────────────────────────────────

/// The actual post-checkout hook fires when git checkout is run, not just the library.
/// Requires hooks to be installed and the ship binary to be in PATH.
#[test]
fn real_git_checkout_fires_hook_and_generates_claude_md() {
    let p = TestProject::with_git().unwrap();
    p.install_hooks().unwrap();
    p.initial_commit().unwrap();

    create_feature(p.ship_dir.clone(), "Auth Flow", "Hook test.", None, None, Some("feature/auth")).unwrap();

    // This fires the real post-checkout hook
    let out = p.checkout_new("feature/auth").unwrap();
    assert!(out.status.success(), "git checkout failed: {:?}", String::from_utf8_lossy(&out.stderr));

    p.assert_root_file("CLAUDE.md");
    p.assert_root_file_contains("CLAUDE.md", "Auth Flow");
}

// ─── Worktrees ───────────────────────────────────────────────────────────────

/// Documents the current worktree limitation: on_post_checkout derives project_root
/// from ship_dir.parent(), which is always the main repo root — not the worktree root.
/// CLAUDE.md and .mcp.json are written to the main repo, not the worktree.
///
/// Fix: on_post_checkout needs a worktree_root parameter (or detect from CWD) so
/// generated files land in the correct working directory.
#[test]
#[ignore = "on_post_checkout writes CLAUDE.md to main repo root, not worktree root — API needs worktree_root param"]
fn worktree_claude_md_written_to_worktree_root() {
    let p = TestProject::with_git().unwrap();
    p.install_hooks().unwrap();
    p.initial_commit().unwrap();

    create_feature(p.ship_dir.clone(), "Auth Flow", "Worktree test.", None, None, Some("feature/auth")).unwrap();
    p.checkout_new("feature/auth").unwrap();
    p.checkout("main").unwrap();

    let wt = p.add_worktree("feature/auth").unwrap();
    on_post_checkout(&wt.ship_dir, "feature/auth").unwrap();

    // Should land in the worktree directory, not the main repo root
    let claude_md = wt.path.join("CLAUDE.md");
    assert!(claude_md.exists(), "CLAUDE.md should be written in the worktree root, not main repo");
}

/// Documents actual (broken) worktree behavior: CLAUDE.md goes to main repo root.
#[test]
fn worktree_currently_writes_claude_md_to_main_repo_root() {
    let p = TestProject::with_git().unwrap();
    p.install_hooks().unwrap();
    p.initial_commit().unwrap();

    create_feature(p.ship_dir.clone(), "Auth Flow", "Worktree test.", None, None, Some("feature/auth")).unwrap();
    p.checkout_new("feature/auth").unwrap();
    p.checkout("main").unwrap();

    let wt = p.add_worktree("feature/auth").unwrap();
    on_post_checkout(&wt.ship_dir, "feature/auth").unwrap();

    // BUG: lands in main repo root, not worktree
    assert!(p.root().join("CLAUDE.md").exists(),
        "KNOWN BUG: CLAUDE.md written to main repo root, not worktree root");
    assert!(!wt.path.join("CLAUDE.md").exists(),
        "CLAUDE.md is absent from the worktree — this is the bug");
}

/// Without SHIP_DIR, ship fails to find .ship/ from a worktree directory.
/// This is the bug — ship should auto-resolve via .git file → gitdir → common dir.
#[test]
#[ignore = "auto-resolution of .ship/ from worktree not yet implemented"]
fn worktree_resolves_ship_dir_automatically_without_env_var() {
    let p = TestProject::with_git().unwrap();
    p.install_hooks().unwrap();
    p.initial_commit().unwrap();

    create_feature(p.ship_dir.clone(), "Auth Flow", "Auto-resolve test.", None, None, Some("feature/auth")).unwrap();
    p.checkout_new("feature/auth").unwrap();
    p.checkout("main").unwrap();

    let wt = p.add_worktree("feature/auth").unwrap();

    // ship sync run from worktree dir, without SHIP_DIR set — should find .ship/ automatically
    let out = std::process::Command::new(
        std::env::var("SHIP_BIN").unwrap_or_else(|_| "ship".to_string())
    )
    .args(["git", "sync"])
    .current_dir(&wt.path)
    // No SHIP_DIR env var — must resolve from .git file
    .output()
    .unwrap();

    assert!(out.status.success(), "ship git sync failed in worktree without SHIP_DIR:\n{}", String::from_utf8_lossy(&out.stderr));
    assert!(wt.path.join("CLAUDE.md").exists());
}

// ─── Provider filtering ──────────────────────────────────────────────────────

/// export_to("claude") only writes .mcp.json — not Gemini or Codex files.
/// This test documents that multi-provider dispatch is missing: the hook hardcodes
/// "claude" regardless of which providers the user has configured.
#[test]
fn checkout_does_not_write_gemini_config_by_default() {
    let p = TestProject::with_git().unwrap();
    create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, Some("feature/auth")).unwrap();

    on_post_checkout(&p.ship_dir, "feature/auth").unwrap();

    // Gemini config should NOT be written — no provider declared
    assert!(!p.root().join(".gemini").join("settings.json").exists(),
        ".gemini/settings.json should not be written unless gemini is a declared provider");
}

/// When a user declares gemini as their provider, checkout should write .gemini/settings.json.
#[test]
fn checkout_writes_gemini_config_when_declared_as_provider() {
    use runtime::config::{ProjectConfig, save_config};

    let p = TestProject::with_git().unwrap();
    let mut config = ProjectConfig::default();
    config.providers = vec!["gemini".to_string()];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, Some("feature/auth")).unwrap();
    on_post_checkout(&p.ship_dir, "feature/auth").unwrap();

    assert!(p.root().join(".gemini").join("settings.json").exists(),
        ".gemini/settings.json should be written when gemini is declared provider");
}

// ─── Encapsulated branch creation (ship feature start) ───────────────────────

/// `ship feature start <file>` should create the git branch, write it into the feature
/// frontmatter, and generate CLAUDE.md + .mcp.json atomically.
/// This removes the need for string-matching branch names.
#[test]
#[ignore = "ship feature start not yet implemented"]
fn feature_start_creates_branch_and_generates_config() {
    let p = TestProject::with_git().unwrap();
    p.initial_commit().unwrap();

    let feat = create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, None).unwrap();
    let feat_file = feat.file_name().unwrap().to_str().unwrap().to_string();

    // feature has no branch yet
    let f = get_feature(feat.clone()).unwrap();
    assert!(f.metadata.branch.is_none(), "branch should be unset before start");

    let out = p.cli_output(&["feature", "start", &feat_file]).unwrap();
    assert!(out.status.success(), "ship feature start failed:\n{}", String::from_utf8_lossy(&out.stderr));

    // branch written into frontmatter
    let f = get_feature(feat).unwrap();
    assert!(f.metadata.branch.is_some(), "branch should be set after start");

    // config generated
    p.assert_root_file("CLAUDE.md");
    p.assert_root_file_contains("CLAUDE.md", "Auth Flow");
}

/// `ship feature switch <file>` checks out the linked branch and regenerates config.
#[test]
#[ignore = "ship feature switch not yet implemented"]
fn feature_switch_checks_out_branch_and_syncs_config() {
    let p = TestProject::with_git().unwrap();
    p.install_hooks().unwrap();
    p.initial_commit().unwrap();

    let feat = create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, Some("feature/auth")).unwrap();
    let feat_file = feat.file_name().unwrap().to_str().unwrap().to_string();
    p.checkout_new("feature/auth").unwrap();
    p.checkout("main").unwrap();

    let out = p.cli_output(&["feature", "switch", &feat_file]).unwrap();
    assert!(out.status.success());
    p.assert_root_file("CLAUDE.md");
    p.assert_root_file_contains("CLAUDE.md", "Auth Flow");
}

// ─── Generated file gitignore ─────────────────────────────────────────────────

/// CLAUDE.md, .mcp.json, and SHIPWRIGHT.md must be gitignored so they are
/// never committed. Currently missing from root .gitignore — they will be committed.
#[test]
#[ignore = "CLAUDE.md and .mcp.json not yet gitignored at project root"]
fn generated_agent_files_are_gitignored() {
    let p = TestProject::with_git().unwrap();
    p.initial_commit().unwrap();
    create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, Some("feature/auth")).unwrap();
    on_post_checkout(&p.ship_dir, "feature/auth").unwrap();

    // git status should show these as ignored, not untracked
    let out = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(p.root())
        .output()
        .unwrap();
    let status = String::from_utf8_lossy(&out.stdout);
    assert!(!status.contains("CLAUDE.md"), "CLAUDE.md should be gitignored, got: {}", status);
    assert!(!status.contains(".mcp.json"), ".mcp.json should be gitignored, got: {}", status);
    assert!(!status.contains("SHIPWRIGHT.md"), "SHIPWRIGHT.md should be gitignored, got: {}", status);
}
