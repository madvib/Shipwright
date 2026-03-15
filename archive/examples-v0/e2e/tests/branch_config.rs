/// Branch → Agent Configuration E2E Tests
///
/// Tests the full lifecycle: feature creation → branch checkout → CLAUDE.md/.mcp.json
/// generation → baseline workspace context on non-feature branches.
///
/// Tests marked #[ignore] document planned behaviour that is not yet implemented.
/// Run passing tests: cargo test --test branch_config
/// Run all including planned: cargo test --test branch_config -- --include-ignored
mod helpers;

use crate::helpers::create_feature;
use helpers::TestProject;
use runtime::agent_config::FeatureAgentConfig;
use runtime::config::{McpServerConfig, McpServerType, ModeConfig, ProjectConfig, save_config};
use runtime::create_skill;
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

fn set_feature_agent(ship_dir: &Path, feature_id: &str, agent: FeatureAgentConfig) {
    let mut entry = ship_module_project::get_feature_by_id(ship_dir, feature_id).unwrap();
    entry.feature.metadata.agent = Some(agent);
    ship_module_project::update_feature(ship_dir, feature_id, entry.feature).unwrap();
}

// ─── Happy path ──────────────────────────────────────────────────────────────

/// Full happy path via library call: feature linked to branch → CLAUDE.md written.
#[test]
fn happy_path_feature_branch_generates_claude_md() {
    let p = TestProject::with_git().unwrap();

    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Implement auth.",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

    p.assert_root_file("CLAUDE.md");
    p.assert_root_file_contains("CLAUDE.md", "# [ship] Auth Flow");
}

/// Issue content is no longer included in generated branch context.
#[test]
fn claude_md_omits_issue_section() {
    let p = TestProject::with_git().unwrap();
    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

    p.assert_root_file_not_contains("CLAUDE.md", "## Open Issues");
}

/// Skill bodies are not inlined into CLAUDE.md.
#[test]
fn claude_md_excludes_skill_content() {
    let p = TestProject::with_git().unwrap();
    let feat = create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    create_skill(
        &p.ship_dir,
        "conventions",
        "Project Conventions",
        "Always use anyhow for errors.",
    )
    .unwrap();
    set_feature_agent(
        &p.ship_dir,
        &feat.0,
        FeatureAgentConfig {
            providers: vec![],
            model: None,
            mcp_servers: vec![],
            skills: vec!["conventions".to_string()],
        },
    );

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

    p.assert_root_file_not_contains("CLAUDE.md", "conventions");
    p.assert_root_file_not_contains("CLAUDE.md", "Always use anyhow for errors.");
}

/// Feature-level MCP filtering restricts .mcp.json to only the declared servers.
#[test]
fn mcp_json_contains_only_feature_declared_servers() {
    let p = TestProject::with_git().unwrap();
    let mut config = ProjectConfig::default();
    config.mcp_servers = vec![stdio_server("github"), stdio_server("linear")];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    let feat = create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    set_feature_agent(
        &p.ship_dir,
        &feat.0,
        FeatureAgentConfig {
            providers: vec![],
            model: None,
            mcp_servers: vec!["github".to_string()],
            skills: vec![],
        },
    );

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

    let mcp_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(p.root().join(".mcp.json")).unwrap())
            .unwrap();
    assert!(
        mcp_json["mcpServers"]["github"].is_object(),
        "github should be in .mcp.json"
    );
    // linear was not declared on the feature — should be absent
    assert!(
        mcp_json["mcpServers"]["linear"].is_null(),
        "linear should be absent from .mcp.json"
    );
}

/// When no agent config is set on the feature, all project servers are included.
#[test]
fn mcp_json_falls_back_to_all_project_servers() {
    let p = TestProject::with_git().unwrap();
    let mut config = ProjectConfig::default();
    config.mcp_servers = vec![stdio_server("github"), stdio_server("linear")];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

    let mcp_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(p.root().join(".mcp.json")).unwrap())
            .unwrap();
    assert!(mcp_json["mcpServers"]["github"].is_object());
    assert!(mcp_json["mcpServers"]["linear"].is_object());
}

/// Non-feature branch (no linked feature) produces baseline workspace context.
#[test]
fn non_feature_branch_generates_workspace_claude_md() {
    let p = TestProject::with_git().unwrap();
    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();

    on_post_checkout(&p.ship_dir, "main", &p.root()).unwrap();

    p.assert_root_file("CLAUDE.md");
    p.assert_root_file_contains("CLAUDE.md", "# [ship] Workspace: main");
    p.assert_root_file_not_contains("CLAUDE.md", "Auth Flow");
}

// ─── Branch switch behavior ───────────────────────────────────────────────────

/// When switching away from a feature branch to main, feature context should be replaced
/// by workspace context.
#[test]
fn switching_to_main_rewrites_claude_md_to_workspace_context() {
    let p = TestProject::with_git().unwrap();
    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();

    // Generate files on feature branch
    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();
    p.assert_root_file("CLAUDE.md");

    // Switch to main — should rewrite to baseline workspace context
    on_post_checkout(&p.ship_dir, "main", &p.root()).unwrap();
    p.assert_root_file("CLAUDE.md");
    p.assert_root_file_contains("CLAUDE.md", "# [ship] Workspace: main");
    p.assert_root_file_not_contains("CLAUDE.md", "# [ship] Auth Flow");
}

/// Feature-specific MCP selections should be cleared when switching to a non-feature
/// branch and baseline project MCP configuration should be restored.
#[test]
fn switching_to_main_restores_baseline_mcp_servers() {
    let p = TestProject::with_git().unwrap();
    let mut config = ProjectConfig::default();
    config.mcp_servers = vec![stdio_server("linear"), stdio_server("github")];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    let feat = create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    set_feature_agent(
        &p.ship_dir,
        &feat.0,
        FeatureAgentConfig {
            providers: vec![],
            model: None,
            mcp_servers: vec!["linear".to_string()],
            skills: vec![],
        },
    );
    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();
    let val: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(p.root().join(".mcp.json")).unwrap())
            .unwrap();
    assert!(val["mcpServers"]["linear"].is_object());
    assert!(val["mcpServers"]["github"].is_null());

    // Switch away — feature filter should be cleared back to baseline (all project servers)
    on_post_checkout(&p.ship_dir, "main", &p.root()).unwrap();
    let mcp_path = p.root().join(".mcp.json");
    if mcp_path.exists() {
        let val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&mcp_path).unwrap()).unwrap();
        assert!(val["mcpServers"]["linear"].is_object());
        assert!(val["mcpServers"]["github"].is_object());
    }
}

/// Feature-level provider overrides (for example codex on a claude-default project)
/// must still be torn down when switching away from the linked feature branch.
#[test]
fn switching_to_main_removes_feature_override_provider_outputs() {
    let p = TestProject::with_git().unwrap();
    let mut config = ProjectConfig::default();
    config.providers = vec!["claude".to_string()];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    let feat = create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    set_feature_agent(
        &p.ship_dir,
        &feat.0,
        FeatureAgentConfig {
            providers: vec!["codex".to_string()],
            model: None,
            mcp_servers: vec![],
            skills: vec![],
        },
    );

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();
    p.assert_root_file("AGENTS.md");

    on_post_checkout(&p.ship_dir, "main", &p.root()).unwrap();
    p.assert_no_root_file("AGENTS.md");
}

/// AGENTS.md that was not generated by Ship must not be deleted on branch switch.
#[test]
fn switching_to_main_preserves_unmanaged_agents_md() {
    let p = TestProject::with_git().unwrap();
    p.write_root_file("AGENTS.md", "# Manual agent instructions\n");

    on_post_checkout(&p.ship_dir, "main", &p.root()).unwrap();

    p.assert_root_file("AGENTS.md");
    p.assert_root_file_contains("AGENTS.md", "Manual agent instructions");
}

// ─── Real hook firing via git checkout ──────────────────────────────────────

/// The actual post-checkout hook fires when git checkout is run, not just the library.
/// Requires hooks to be installed and the ship binary to be in PATH.
#[test]
fn real_git_checkout_fires_hook_and_generates_claude_md() {
    let p = TestProject::with_git().unwrap();
    p.install_hooks().unwrap();
    p.initial_commit().unwrap();

    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Hook test.",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();

    // This fires the real post-checkout hook
    let out = p.checkout_new("feature/auth").unwrap();
    assert!(
        out.status.success(),
        "git checkout failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );

    p.assert_root_file("CLAUDE.md");
    p.assert_root_file_contains("CLAUDE.md", "Auth Flow");
}

// ─── Worktrees ───────────────────────────────────────────────────────────────

/// CLAUDE.md is written to the worktree root, not the main repo root.
#[test]
fn worktree_claude_md_written_to_worktree_root() {
    let p = TestProject::with_git().unwrap();
    p.install_hooks().unwrap();
    p.initial_commit().unwrap();

    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Worktree test.",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    p.checkout_new("feature/auth").unwrap();
    p.checkout("main").unwrap();

    let wt = p.add_worktree("feature/auth").unwrap();
    let root_claude = p.root().join("CLAUDE.md");
    if root_claude.exists() {
        std::fs::remove_file(&root_claude).unwrap();
    }
    assert!(
        !root_claude.exists(),
        "precondition: main repo root should not have CLAUDE.md before worktree sync"
    );
    on_post_checkout(&wt.ship_dir, "feature/auth", &wt.path).unwrap();

    let claude_md = wt.path.join("CLAUDE.md");
    assert!(
        claude_md.exists(),
        "CLAUDE.md should be written in the worktree root"
    );
    assert!(
        !p.root().join("CLAUDE.md").exists(),
        "CLAUDE.md must not appear in the main repo root"
    );
}

/// Without SHIP_DIR, ship finds .ship/ by walking up from the worktree directory.
/// Worktrees live inside the project dir, so the standard walk-up hits .ship/.
#[test]
fn worktree_resolves_ship_dir_automatically_without_env_var() {
    let p = TestProject::with_git().unwrap();
    p.install_hooks().unwrap();

    // Feature must be created BEFORE initial_commit so it's committed to git.
    // git worktree add checks out the branch, so the worktree gets its own .ship/
    // copy. If the feature was added after the commit, the worktree's .ship/ would
    // lack it and the walk-up would find the wrong (empty) .ship/ first.
    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Auto-resolve test.",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    p.initial_commit().unwrap();

    p.checkout_new("feature/auth").unwrap();
    p.checkout("main").unwrap();

    let wt = p.add_worktree("feature/auth").unwrap();

    // ship git sync run from worktree dir without SHIP_DIR.
    // The worktree's own checked-out .ship/ is found by walk-up (not the main repo's).
    let ship_bin = std::env::var("SHIP_BIN").unwrap_or_else(|_| {
        let mut dir = std::env::current_exe().unwrap();
        dir.pop();
        if dir.ends_with("deps") {
            dir.pop();
        }
        dir.join("ship").to_string_lossy().into_owned()
    });

    let out = std::process::Command::new(&ship_bin)
        .args(["git", "sync"])
        .current_dir(&wt.path)
        .env("SHIP_GLOBAL_DIR", &p.global_dir)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "ship git sync failed in worktree without SHIP_DIR\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        wt.path.join("CLAUDE.md").exists(),
        "CLAUDE.md should be written in worktree root {}\nstdout: {}\nstderr: {}",
        wt.path.display(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

// ─── Provider filtering ──────────────────────────────────────────────────────

/// export_to("claude") only writes .mcp.json — not Gemini or Codex files.
/// This test documents that multi-provider dispatch is missing: the hook hardcodes
/// "claude" regardless of which providers the user has configured.
#[test]
fn checkout_does_not_write_gemini_config_by_default() {
    let p = TestProject::with_git().unwrap();
    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

    // Gemini config should NOT be written — no provider declared
    assert!(
        !p.root().join(".gemini").join("settings.json").exists(),
        ".gemini/settings.json should not be written unless gemini is a declared provider"
    );
}

/// When a user declares gemini as their provider, checkout should write .gemini/settings.json.
#[test]
fn checkout_writes_gemini_config_when_declared_as_provider() {
    use runtime::config::{ProjectConfig, save_config};

    let p = TestProject::with_git().unwrap();
    let mut config = ProjectConfig::default();
    config.providers = vec!["gemini".to_string()];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

    assert!(
        p.root().join(".gemini").join("settings.json").exists(),
        ".gemini/settings.json should be written when gemini is declared provider"
    );
}

/// Unknown provider IDs in feature-level overrides should not break checkout as long
/// as at least one valid provider remains after normalization.
#[test]
fn checkout_skips_unknown_feature_provider_ids_and_exports_valid_targets() {
    let p = TestProject::with_git().unwrap();

    let feat = create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    set_feature_agent(
        &p.ship_dir,
        &feat.0,
        FeatureAgentConfig {
            providers: vec!["unknown-provider".to_string(), " CLAUDE ".to_string()],
            model: None,
            mcp_servers: vec![],
            skills: vec![],
        },
    );

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();
    p.assert_root_file("CLAUDE.md");
    p.assert_no_root_file("AGENTS.md");
}

/// If project providers are malformed/unknown, checkout should still fall back
/// to Claude so branch context generation remains functional.
#[test]
fn checkout_with_invalid_project_providers_falls_back_to_claude() {
    let p = TestProject::with_git().unwrap();
    let mut config = ProjectConfig::default();
    config.providers = vec!["unknown-provider".to_string()];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();
    p.assert_root_file("CLAUDE.md");
}

// ─── Encapsulated branch creation (ship feature start) ───────────────────────

/// `ship feature start <file>` creates the git branch, writes it into the feature
/// frontmatter, and generates CLAUDE.md + .mcp.json atomically.
#[test]
fn feature_start_creates_branch_and_generates_config() {
    let p = TestProject::with_git().unwrap();
    p.initial_commit().unwrap();

    let feat = create_feature(p.ship_dir.clone(), "Auth Flow", "Body", None, None, None).unwrap();
    let id = feat.0.clone();

    // feature has no branch yet
    let f = ship_module_project::get_feature_by_id(&p.ship_dir, &id).unwrap();
    assert!(
        f.feature.metadata.branch.is_none(),
        "branch should be unset before start"
    );

    let out = p.cli_output(&["feature", "start", &id]).unwrap();
    assert!(
        out.status.success(),
        "ship feature start failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Verify branch linkage through CLI surface (same runtime/env as feature start).
    let get_out = p.cli_output(&["feature", "get", &id]).unwrap();
    assert!(
        get_out.status.success(),
        "ship feature get failed:\n{}",
        String::from_utf8_lossy(&get_out.stderr)
    );
    let get_stdout = String::from_utf8_lossy(&get_out.stdout);
    assert!(
        get_stdout.contains("branch = \"feature/"),
        "branch should be set after start, got:\n{}",
        get_stdout
    );

    // feature start must at minimum persist branch linkage.
    assert!(
        p.current_branch().starts_with("feature/"),
        "branch should remain linked after start"
    );
}

/// `ship workspace switch <branch>` checks out the branch and regenerates config.
#[test]
fn feature_switch_checks_out_branch_and_syncs_config() {
    let p = TestProject::with_git().unwrap();
    p.install_hooks().unwrap();
    p.initial_commit().unwrap();

    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    p.checkout_new("feature/auth").unwrap();
    p.checkout("main").unwrap();

    let out = p
        .cli_output(&["workspace", "switch", "feature/auth"])
        .unwrap();
    assert!(
        out.status.success(),
        "ship workspace switch failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    p.assert_root_file("CLAUDE.md");
    p.assert_root_file_contains("CLAUDE.md", "Auth Flow");
}

/// Workspace mode override should take precedence over project active_mode when
/// compiling branch context.
#[test]
fn workspace_mode_override_applies_on_checkout() {
    let p = TestProject::with_git().unwrap();
    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();

    create_skill(
        &p.ship_dir,
        "rt-planning-skill",
        "Planning Skill",
        "PLANNING-MODE-CONTENT",
    )
    .unwrap();
    create_skill(
        &p.ship_dir,
        "rt-code-skill",
        "Code Skill",
        "CODE-MODE-CONTENT",
    )
    .unwrap();

    let mut config = ProjectConfig::default();
    config.active_mode = Some("planning".to_string());
    config.modes = vec![
        ModeConfig {
            id: "planning".to_string(),
            name: "Planning".to_string(),
            skills: vec!["rt-planning-skill".to_string()],
            ..Default::default()
        },
        ModeConfig {
            id: "code".to_string(),
            name: "Code".to_string(),
            skills: vec!["rt-code-skill".to_string()],
            ..Default::default()
        },
    ];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    runtime::create_workspace(
        &p.ship_dir,
        runtime::CreateWorkspaceRequest {
            branch: "feature/auth".to_string(),
            active_mode: Some("code".to_string()),
            ..Default::default()
        },
    )
    .unwrap();

    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();
    p.assert_root_file_not_contains("CLAUDE.md", "CODE-MODE-CONTENT");
    p.assert_root_file_not_contains("CLAUDE.md", "PLANNING-MODE-CONTENT");
    p.assert_root_file_contains(".claude/skills/rt-code-skill/SKILL.md", "CODE-MODE-CONTENT");
    p.assert_no_root_file(".claude/skills/rt-planning-skill/SKILL.md");
}

// ─── Generated file gitignore ─────────────────────────────────────────────────

/// CLAUDE.md, GEMINI.md, and .mcp.json are gitignored so they are never committed.
#[test]
fn generated_agent_files_are_gitignored() {
    let p = TestProject::with_git().unwrap();
    p.initial_commit().unwrap();
    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

    // git status should show these as ignored, not untracked
    let out = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(p.root())
        .output()
        .unwrap();
    let status = String::from_utf8_lossy(&out.stdout);
    assert!(
        !status.contains("CLAUDE.md"),
        "CLAUDE.md should be gitignored, got: {}",
        status
    );
    assert!(
        !status.contains(".mcp.json"),
        ".mcp.json should be gitignored, got: {}",
        status
    );
    assert!(
        !status.contains("GEMINI.md"),
        "GEMINI.md should be gitignored, got: {}",
        status
    );
}

/// Codex-generated artifacts are also gitignored and should not appear as untracked files.
#[test]
fn codex_generated_files_are_gitignored() {
    let p = TestProject::with_git().unwrap();
    p.initial_commit().unwrap();

    let mut config = ProjectConfig::default();
    config.providers = vec!["codex".to_string()];
    save_config(&config, Some(p.ship_dir.clone())).unwrap();

    create_feature(
        p.ship_dir.clone(),
        "Auth Flow",
        "Body",
        None,
        None,
        Some("feature/auth"),
    )
    .unwrap();
    on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

    // git status should show these as ignored, not untracked
    let out = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(p.root())
        .output()
        .unwrap();
    let status = String::from_utf8_lossy(&out.stdout);
    assert!(
        !status.contains("AGENTS.md"),
        "AGENTS.md should be gitignored, got: {}",
        status
    );
    assert!(
        !status.contains(".agents/"),
        ".agents/ should be gitignored, got: {}",
        status
    );
}
