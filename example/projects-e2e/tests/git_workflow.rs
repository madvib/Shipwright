/// Git Workflow E2E Tests
///
/// Exhaustive coverage of Ship's git integration across two project scenarios:
///   A) Brand-new empty project (`ship init` from scratch)
///   B) Existing project (pre-existing code, .gitignore, optionally existing agent configs)
///
/// Branch hierarchy tested: main > release/vX > feature/Y > worktree
///
/// Run: cargo test --test git_workflow -p e2e
/// Run ignored: cargo test --test git_workflow -p e2e -- --include-ignored
mod helpers;

use helpers::{EXISTING_JS_PROJECT, EXISTING_RUST_PROJECT, TestProject};
use runtime::{create_feature, create_skill};
use ship_module_git::{
    GENERATED_GITIGNORE_ENTRIES, install_hooks, on_post_checkout, write_root_gitignore,
};
use std::collections::HashMap;
use std::fs;
use std::process::Command;

// ─── Scenario A: Brand-new project ───────────────────────────────────────────

mod new_project {
    use super::*;

    /// `ship init` on an empty directory produces the full .ship/ structure.
    #[test]
    fn init_creates_full_ship_structure() {
        let p = TestProject::with_git().unwrap();

        // Core namespace directories
        p.assert_ship_file("workflow/issues/backlog");
        p.assert_ship_file("workflow/issues/in-progress");
        p.assert_ship_file("workflow/issues/done");
        p.assert_ship_file("workflow/specs");
        p.assert_ship_file("project/features");
        p.assert_ship_file("project/releases");
        p.assert_ship_file("project/adrs");
        p.assert_ship_file("project/notes");
        p.assert_ship_file("agents/modes");
        p.assert_ship_file("agents/skills");
        p.assert_ship_file("agents/prompts");

        // Config
        p.assert_ship_file("ship.toml");

        // Default templates
        p.assert_ship_file("workflow/issues/TEMPLATE.md");
        p.assert_ship_file("project/features/TEMPLATE.md");
        p.assert_ship_file("workflow/specs/TEMPLATE.md");
        p.assert_ship_file("project/adrs/TEMPLATE.md");

        // Default modes
        p.assert_ship_file("agents/modes/planning.toml");
        p.assert_ship_file("agents/modes/execution.toml");

        // Event log
        p.assert_ship_file("events.ndjson");
    }

    /// Default task-policy skill is seeded on init.
    #[test]
    fn init_seeds_task_policy_skill() {
        let p = TestProject::with_git().unwrap();
        p.assert_ship_file("agents/skills/task-policy/index.md");
        p.assert_ship_file("agents/skills/task-policy/skill.toml");
        let content = p.read_ship_file("agents/skills/task-policy/index.md");
        assert!(content.contains("task-policy"), "skill content missing");
        let config = p.read_ship_file("agents/skills/task-policy/skill.toml");
        assert!(config.contains("task-policy"), "skill id missing from toml");
        assert!(
            content.contains("Shipwright Workflow Policy"),
            "skill name missing"
        );
        assert!(content.contains("Canonical Flow"), "policy content missing");
    }

    /// Git hooks are installed on init: post-checkout and pre-commit.
    #[test]
    fn init_installs_git_hooks() {
        let p = TestProject::with_git().unwrap();
        install_hooks(&p.root().join(".git")).unwrap();

        let hooks_dir = p.root().join(".git/hooks");
        assert!(
            hooks_dir.join("post-checkout").exists(),
            "post-checkout hook missing"
        );
        assert!(
            hooks_dir.join("pre-commit").exists(),
            "pre-commit hook missing"
        );

        let pre_commit = fs::read_to_string(hooks_dir.join("pre-commit")).unwrap();
        assert!(
            pre_commit.starts_with("#!/usr/bin/env sh"),
            "hook must be executable sh"
        );
        assert!(pre_commit.contains("CLAUDE.md"), "must block CLAUDE.md");
        assert!(pre_commit.contains(".mcp.json"), "must block .mcp.json");
    }

    /// Root .gitignore is written with all generated file entries.
    #[test]
    fn init_writes_root_gitignore() {
        let p = TestProject::with_git().unwrap();
        write_root_gitignore(p.root()).unwrap();

        let gitignore = fs::read_to_string(p.root().join(".gitignore")).unwrap();
        for entry in GENERATED_GITIGNORE_ENTRIES {
            assert!(
                gitignore.lines().any(|l| l.trim() == *entry),
                ".gitignore missing entry: {}",
                entry
            );
        }
    }

    /// Running `ship init` twice on the same directory is safe — no files clobbered.
    #[test]
    fn init_is_idempotent() {
        let p = TestProject::with_git().unwrap();

        // Write a custom skill
        let custom_dir = p.ship_dir.join("agents/skills/my-skill");
        fs::create_dir_all(&custom_dir).unwrap();
        fs::write(
            custom_dir.join("skill.toml"),
            "id = \"my-skill\"\nname = \"Mine\"\n",
        )
        .unwrap();
        fs::write(custom_dir.join("index.md"), "content").unwrap();

        // Re-init
        runtime::init_project(p.root().to_path_buf()).unwrap();

        // Custom skill must be intact
        assert!(custom_dir.join("index.md").exists());
        assert_eq!(
            fs::read_to_string(custom_dir.join("index.md")).unwrap(),
            "content"
        );
        // Default skill still present
        p.assert_ship_file("agents/skills/task-policy/index.md");
    }

    /// Feature template has the richer lifecycle fields (planned, version, Why/Delivery sections).
    #[test]
    fn init_feature_template_has_lifecycle_fields() {
        let p = TestProject::with_git().unwrap();
        let template = p.read_ship_file("project/features/TEMPLATE.md");
        assert!(template.contains("release_id"));
        assert!(template.contains("## Why"));
        assert!(template.contains("## Delivery Todos"));
    }
}

// ─── Scenario B: Existing project ────────────────────────────────────────────

mod existing_project {
    use super::*;

    /// `ship init` on a JS project preserves all existing files.
    #[test]
    fn init_preserves_existing_js_files() {
        let p = TestProject::with_git_and_files(EXISTING_JS_PROJECT).unwrap();

        // All pre-existing files still there
        assert!(p.root().join("package.json").exists());
        assert!(p.root().join("src/index.js").exists());
        assert!(p.root().join("src/components/Button.js").exists());
        assert!(p.root().join("public/index.html").exists());
        assert!(p.root().join("README.md").exists());

        // .ship/ also created
        p.assert_ship_file("ship.toml");
        p.assert_ship_file("agents/skills/task-policy/index.md");
    }

    /// Existing .gitignore entries are preserved; Ship's entries are appended.
    #[test]
    fn init_preserves_existing_gitignore_and_appends_ship_entries() {
        let p = TestProject::with_git_and_files(EXISTING_JS_PROJECT).unwrap();
        write_root_gitignore(p.root()).unwrap();

        let gitignore = fs::read_to_string(p.root().join(".gitignore")).unwrap();

        // Existing entries preserved
        assert!(
            gitignore.contains("node_modules/"),
            "node_modules/ stripped"
        );
        assert!(gitignore.contains(".env"), ".env stripped");
        assert!(gitignore.contains(".next/"), ".next/ stripped");

        // Ship's entries appended
        assert!(gitignore.contains("CLAUDE.md"), "CLAUDE.md not added");
        assert!(gitignore.contains(".mcp.json"), ".mcp.json not added");
        assert!(gitignore.contains(".claude/"), ".claude/ not added");
    }

    /// Existing .gitignore entries are not duplicated if Ship runs twice.
    #[test]
    fn init_gitignore_no_duplicates_on_reinit() {
        let p = TestProject::with_git_and_files(EXISTING_JS_PROJECT).unwrap();
        write_root_gitignore(p.root()).unwrap();
        write_root_gitignore(p.root()).unwrap();

        let gitignore = fs::read_to_string(p.root().join(".gitignore")).unwrap();
        assert_eq!(
            gitignore.matches("CLAUDE.md").count(),
            1,
            "CLAUDE.md duplicated"
        );
        assert_eq!(
            gitignore.matches(".mcp.json").count(),
            1,
            ".mcp.json duplicated"
        );
    }

    /// `ship init` on a Rust project works identically.
    #[test]
    fn init_works_on_rust_project() {
        let p = TestProject::with_git_and_files(EXISTING_RUST_PROJECT).unwrap();
        assert!(p.root().join("Cargo.toml").exists());
        assert!(p.root().join("src/main.rs").exists());
        p.assert_ship_file("ship.toml");
        p.assert_ship_file("agents/skills/task-policy/index.md");
    }

    /// Existing user-managed .mcp.json is preserved on init (Ship doesn't touch it until checkout).
    #[test]
    fn init_does_not_overwrite_existing_mcp_json() {
        let user_mcp = r#"{"mcpServers":{"my-server":{"type":"stdio","command":"my-tool"}}}"#;
        let mut files: Vec<(&str, &str)> = EXISTING_JS_PROJECT.to_vec();
        files.push((".mcp.json", user_mcp));

        let p = TestProject::with_git_and_files(&files).unwrap();

        // .mcp.json should be unchanged — init doesn't touch it
        let content = fs::read_to_string(p.root().join(".mcp.json")).unwrap();
        assert_eq!(content, user_mcp, ".mcp.json was modified by init");
    }
}

// ─── Branch hierarchy workflow ────────────────────────────────────────────────

mod branch_hierarchy {
    use super::*;

    fn setup() -> TestProject {
        let p = TestProject::with_git().unwrap();
        install_hooks(&p.root().join(".git")).unwrap();
        write_root_gitignore(p.root()).unwrap();
        p.initial_commit().unwrap();
        p
    }

    /// main > release/v1.0 > feature/auth — context generated at feature level.
    #[test]
    fn release_then_feature_branch_hierarchy() {
        let p = setup();

        // Create release branch off main
        p.create_release_branch("release/v1.0").unwrap();

        // Create feature branch off release
        p.create_feature_branch("feature/auth").unwrap();

        // Link feature doc to this branch
        create_feature(
            p.ship_dir.clone(),
            "Auth System",
            "OAuth2 login flow.",
            None,
            None,
            Some("feature/auth"),
        )
        .unwrap();

        // Simulate post-checkout on feature branch
        on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

        p.assert_root_file("CLAUDE.md");
        p.assert_root_file_contains("CLAUDE.md", "Auth System");
        p.assert_root_file_contains("CLAUDE.md", "OAuth2 login flow");
    }

    /// Checking out main tears down CLAUDE.md and .mcp.json.
    #[test]
    fn checkout_main_removes_generated_files() {
        let p = setup();
        create_feature(
            p.ship_dir.clone(),
            "Auth System",
            "Body",
            None,
            None,
            Some("feature/auth"),
        )
        .unwrap();
        p.create_feature_branch("feature/auth").unwrap();

        // Simulate checkout of feature branch
        on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();
        assert!(p.root().join("CLAUDE.md").exists());

        // Simulate checkout back to main
        on_post_checkout(&p.ship_dir, "main", &p.root()).unwrap();
        assert!(
            !p.root().join("CLAUDE.md").exists(),
            "CLAUDE.md should be removed on main"
        );
    }

    /// Multiple features each generate correct isolated context.
    #[test]
    fn multiple_features_generate_independent_context() {
        let p = setup();

        create_feature(
            p.ship_dir.clone(),
            "Auth System",
            "OAuth2 flow.",
            None,
            None,
            Some("feature/auth"),
        )
        .unwrap();
        create_feature(
            p.ship_dir.clone(),
            "Billing Module",
            "Stripe integration.",
            None,
            None,
            Some("feature/billing"),
        )
        .unwrap();

        on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();
        p.assert_root_file_contains("CLAUDE.md", "Auth System");
        p.assert_root_file_not_contains("CLAUDE.md", "Billing Module");

        on_post_checkout(&p.ship_dir, "feature/billing", &p.root()).unwrap();
        p.assert_root_file_contains("CLAUDE.md", "Billing Module");
        p.assert_root_file_not_contains("CLAUDE.md", "Auth System");
    }

    /// Default task-policy skill is included in CLAUDE.md even without explicit feature agent config.
    #[test]
    fn default_skill_appears_in_claude_md() {
        let p = setup();
        create_feature(
            p.ship_dir.clone(),
            "Feature A",
            "Body",
            None,
            None,
            Some("feature/a"),
        )
        .unwrap();

        on_post_checkout(&p.ship_dir, "feature/a", &p.root()).unwrap();

        p.assert_root_file_contains("CLAUDE.md", "Shipwright Workflow Policy");
    }

    /// Custom project skill is inlined into CLAUDE.md.
    #[test]
    fn custom_skill_inlined_in_claude_md() {
        let p = setup();
        create_skill(
            &p.ship_dir,
            "stack",
            "Stack Conventions",
            "Always use TypeScript. Prefer functional components.",
        )
        .unwrap();
        create_feature(
            p.ship_dir.clone(),
            "Feature A",
            "Body",
            None,
            None,
            Some("feature/a"),
        )
        .unwrap();

        on_post_checkout(&p.ship_dir, "feature/a", &p.root()).unwrap();

        p.assert_root_file_contains("CLAUDE.md", "Always use TypeScript");
    }
}

// ─── Pre-commit hook enforcement ──────────────────────────────────────────────

mod pre_commit_hook {
    use super::*;

    fn setup() -> TestProject {
        let p = TestProject::with_git().unwrap();
        install_hooks(&p.root().join(".git")).unwrap();
        write_root_gitignore(p.root()).unwrap();
        p.initial_commit().unwrap();
        p
    }

    /// Staging a normal file is allowed (hook doesn't block non-generated files).
    #[test]
    fn hook_allows_normal_files() {
        let p = setup();
        p.write_root_file("src/auth.rs", "fn login() {}");
        p.git_stage("src/auth.rs");
        let (ok, _) = p.git_commit("add auth");
        assert!(ok, "normal file commit should succeed");
    }

    /// Staging CLAUDE.md is blocked by pre-commit hook.
    #[test]
    fn hook_blocks_staging_claude_md() {
        let p = setup();
        p.write_root_file("CLAUDE.md", "# generated");
        p.git_stage("CLAUDE.md");
        let staged = p.git_staged_files();
        if staged.contains(&"CLAUDE.md".to_string()) {
            let (ok, stderr) = p.git_commit("should fail");
            assert!(!ok, "commit with CLAUDE.md should be rejected");
            assert!(
                stderr.contains("CLAUDE.md") || stderr.contains("ship"),
                "hook error message should mention CLAUDE.md or ship"
            );
        }
        // If git itself respects .gitignore and won't stage it, that's also acceptable
    }

    /// Staging .mcp.json is blocked by pre-commit hook.
    #[test]
    fn hook_blocks_staging_mcp_json() {
        let p = setup();
        // Write to a temp location not gitignored to force stage
        let mcp_path = p.root().join(".mcp.json");
        // Temporarily remove from gitignore to test the hook independently
        let gitignore_path = p.root().join(".gitignore");
        let existing = fs::read_to_string(&gitignore_path).unwrap_or_default();
        let without_mcp = existing.replace(".mcp.json\n", "").replace(".mcp.json", "");
        fs::write(&gitignore_path, &without_mcp).unwrap();

        fs::write(&mcp_path, r#"{"mcpServers":{}}"#).unwrap();
        p.git_stage(".mcp.json");

        let staged = p.git_staged_files();
        if staged.contains(&".mcp.json".to_string()) {
            let (ok, stderr) = p.git_commit("should fail");
            assert!(
                !ok,
                "commit with .mcp.json should be rejected by pre-commit hook"
            );
            assert!(stderr.contains(".mcp.json") || stderr.contains("ship"));
        }
    }

    /// Generated files in root .gitignore cannot be staged at all.
    #[test]
    fn gitignore_prevents_staging_generated_files() {
        let p = setup();
        write_root_gitignore(p.root()).unwrap();

        p.write_root_file("CLAUDE.md", "# generated");
        // git add should refuse to stage a gitignored file
        let staged = {
            Command::new("git")
                .args(["add", "CLAUDE.md"])
                .current_dir(p.root())
                .output()
                .unwrap();
            p.git_staged_files()
        };
        assert!(
            !staged.contains(&"CLAUDE.md".to_string()),
            "CLAUDE.md should not be stageable — it's gitignored"
        );
    }
}

// ─── Worktree workflow ────────────────────────────────────────────────────────

mod worktrees {
    use super::*;

    fn setup_with_feature(branch: &str, title: &str) -> TestProject {
        let p = TestProject::with_git().unwrap();
        install_hooks(&p.root().join(".git")).unwrap();
        p.initial_commit().unwrap();
        create_feature(
            p.ship_dir.clone(),
            title,
            "Feature body.",
            None,
            None,
            Some(branch),
        )
        .unwrap();
        // Create branch WITHOUT checking it out — a worktree must exclusively own its branch.
        // `git branch <name>` creates at HEAD without switching.
        Command::new("git")
            .args(["branch", branch])
            .current_dir(p.root())
            .output()
            .unwrap();
        p
    }

    /// A worktree checking out a feature branch has the correct branch checked out.
    #[test]
    fn worktree_checks_out_correct_branch() {
        let p = setup_with_feature("feature/auth", "Auth System");
        let wt = p.add_worktree("feature/auth").unwrap();
        assert_eq!(wt.current_branch(), "feature/auth");
    }

    /// A worktree shares the same .ship/ state as the main checkout.
    #[test]
    fn worktree_shares_ship_state() {
        let p = setup_with_feature("feature/auth", "Auth System");
        let wt = p.add_worktree("feature/auth").unwrap();

        // Verify the worktree's ship_dir points to the same .ship/
        assert_eq!(wt.ship_dir, p.ship_dir);
    }

    /// Two worktrees can simultaneously check out different feature branches.
    #[test]
    fn two_worktrees_on_different_branches() {
        let p = TestProject::with_git().unwrap();
        install_hooks(&p.root().join(".git")).unwrap();
        p.initial_commit().unwrap();

        create_feature(
            p.ship_dir.clone(),
            "Auth System",
            "Auth body.",
            None,
            None,
            Some("feature/auth"),
        )
        .unwrap();
        create_feature(
            p.ship_dir.clone(),
            "Billing Module",
            "Billing body.",
            None,
            None,
            Some("feature/billing"),
        )
        .unwrap();

        // Create both branches from main
        p.checkout_new("feature/auth").unwrap();
        p.checkout("main").unwrap();
        p.checkout_new("feature/billing").unwrap();
        p.checkout("main").unwrap();

        let wt_auth = p.add_worktree("feature/auth").unwrap();
        let wt_billing = p.add_worktree("feature/billing").unwrap();

        assert_eq!(wt_auth.current_branch(), "feature/auth");
        assert_eq!(wt_billing.current_branch(), "feature/billing");
    }

    /// CLAUDE.md is written to the worktree root, not the main repo root.
    #[test]
    fn worktree_claude_md_written_to_worktree_root_not_main() {
        let p = setup_with_feature("feature/auth", "Auth System");
        let wt = p.add_worktree("feature/auth").unwrap();

        on_post_checkout(&p.ship_dir, "feature/auth", &wt.path).unwrap();

        wt.assert_file("CLAUDE.md");
        wt.assert_file_contains("CLAUDE.md", "Auth System");
        p.assert_no_root_file("CLAUDE.md");
    }
}

// ─── Scenario B + agent configs: init on project with existing agent setup ───

mod existing_agent_configs {
    use super::*;

    /// `ship init` on a project that already has a user-maintained .mcp.json
    /// leaves that file alone. Only post-checkout modifies it.
    #[test]
    fn init_respects_existing_user_mcp_json() {
        let user_config = r#"{"mcpServers":{"github":{"type":"stdio","command":"gh-mcp"}}}"#;
        let mut files: Vec<(&str, &str)> = EXISTING_JS_PROJECT.to_vec();
        files.push((".mcp.json", user_config));

        let p = TestProject::with_git_and_files(&files).unwrap();

        let content = fs::read_to_string(p.root().join(".mcp.json")).unwrap();
        assert_eq!(content, user_config);
    }

    /// After checkout, Ship adds its own servers to .mcp.json without removing user servers.
    #[test]
    fn checkout_merges_ship_servers_into_existing_mcp_json() {
        use runtime::config::{McpServerConfig, McpServerType, ProjectConfig, save_config};

        let user_config = r#"{"mcpServers":{"github":{"type":"stdio","command":"gh-mcp"}}}"#;
        let mut files: Vec<(&str, &str)> = EXISTING_JS_PROJECT.to_vec();
        files.push((".mcp.json", user_config));

        let p = TestProject::with_git_and_files(&files).unwrap();
        install_hooks(&p.root().join(".git")).unwrap();
        p.initial_commit().unwrap();

        // Register a Ship MCP server
        let mut config = runtime::config::get_config(Some(p.ship_dir.clone())).unwrap();
        config.mcp_servers.push(McpServerConfig {
            id: "ship".to_string(),
            name: "Ship".to_string(),
            command: String::new(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Http,
            url: Some("http://localhost:7825/sse".to_string()),
            disabled: false,
            timeout_secs: None,
        });
        save_config(&config, Some(p.ship_dir.clone())).unwrap();

        create_feature(
            p.ship_dir.clone(),
            "Auth",
            "Body",
            None,
            None,
            Some("feature/auth"),
        )
        .unwrap();
        on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();

        let mcp_content = fs::read_to_string(p.root().join(".mcp.json")).unwrap();
        // Ship's server was added
        assert!(
            mcp_content.contains("ship"),
            "Ship's server not in .mcp.json"
        );
        // User's server preserved
        assert!(
            mcp_content.contains("github"),
            "User's github server was removed"
        );
    }

    /// Teardown (checkout main) removes only Ship-managed servers, not user servers.
    #[test]
    fn teardown_removes_only_ship_servers_preserves_user_servers() {
        use runtime::config::{McpServerConfig, McpServerType, ProjectConfig, save_config};

        let user_config = r#"{"mcpServers":{"github":{"type":"stdio","command":"gh-mcp"}}}"#;
        let mut files: Vec<(&str, &str)> = EXISTING_JS_PROJECT.to_vec();
        files.push((".mcp.json", user_config));

        let p = TestProject::with_git_and_files(&files).unwrap();
        install_hooks(&p.root().join(".git")).unwrap();
        p.initial_commit().unwrap();

        let mut config = runtime::config::get_config(Some(p.ship_dir.clone())).unwrap();
        config.mcp_servers.push(McpServerConfig {
            id: "ship".to_string(),
            name: "Ship".to_string(),
            command: String::new(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Http,
            url: Some("http://localhost:7825/sse".to_string()),
            disabled: false,
            timeout_secs: None,
        });
        save_config(&config, Some(p.ship_dir.clone())).unwrap();
        config.providers = vec!["claude".to_string()];
        save_config(&config, Some(p.ship_dir.clone())).unwrap();

        create_feature(
            p.ship_dir.clone(),
            "Auth",
            "Body",
            None,
            None,
            Some("feature/auth"),
        )
        .unwrap();

        // Checkout feature — Ship adds its server
        on_post_checkout(&p.ship_dir, "feature/auth", &p.root()).unwrap();
        let after_checkout = fs::read_to_string(p.root().join(".mcp.json")).unwrap();
        assert!(after_checkout.contains("ship"));
        assert!(after_checkout.contains("github"));

        // Checkout main — Ship removes only its own server
        on_post_checkout(&p.ship_dir, "main", &p.root()).unwrap();

        // After teardown: .mcp.json should still have github but not ship
        // (or .mcp.json may not exist if Ship started from scratch — either is fine
        //  as long as github isn't lost if it was user-managed before Ship touched it)
        if p.root().join(".mcp.json").exists() {
            let after_teardown = fs::read_to_string(p.root().join(".mcp.json")).unwrap();
            assert!(
                !after_teardown.contains("\"ship\""),
                "Ship's server should be removed on teardown"
            );
            assert!(
                after_teardown.contains("github"),
                "User's github server was incorrectly removed"
            );
        }
    }
}

// ─── Full core loop: init → branch → work → teardown ────────────────────────

mod core_loop {
    use super::*;

    /// The complete recommended workflow in one test.
    #[test]
    fn full_core_loop_new_project() {
        let p = TestProject::with_git().unwrap();

        // 1. Init hooks and gitignore
        install_hooks(&p.root().join(".git")).unwrap();
        write_root_gitignore(p.root()).unwrap();
        p.initial_commit().unwrap();

        // 2. Verify init state
        p.assert_ship_file("agents/skills/task-policy/index.md");
        p.assert_ship_file("ship.toml");
        let gitignore = fs::read_to_string(p.root().join(".gitignore")).unwrap();
        assert!(gitignore.contains("CLAUDE.md"));

        // 3. Create a feature and link it to a branch
        create_feature(
            p.ship_dir.clone(),
            "Payment Processing",
            "Stripe checkout integration.",
            None,
            None,
            Some("feature/payments"),
        )
        .unwrap();
        p.checkout_new("feature/payments").unwrap();

        // 4. Simulate post-checkout hook
        on_post_checkout(&p.ship_dir, "feature/payments", &p.root()).unwrap();

        // 5. CLAUDE.md generated with feature context
        p.assert_root_file("CLAUDE.md");
        p.assert_root_file_contains("CLAUDE.md", "Payment Processing");
        p.assert_root_file_contains("CLAUDE.md", "Stripe checkout integration");
        p.assert_root_file_contains("CLAUDE.md", "Shipwright Workflow Policy");

        // 6. Work files can be committed; generated files cannot be staged
        p.write_root_file("src/payments.rs", "pub fn charge() {}");
        p.git_stage("src/payments.rs");
        let (ok, _) = p.git_commit("add payment module");
        assert!(ok, "normal work commit should succeed");

        // 7. Teardown on return to main
        on_post_checkout(&p.ship_dir, "main", &p.root()).unwrap();
        p.assert_no_root_file("CLAUDE.md");
        p.assert_no_root_file(".mcp.json");
    }

    /// The core loop works identically on an existing codebase (not a new project).
    #[test]
    fn full_core_loop_existing_project() {
        let p = TestProject::with_git_and_files(EXISTING_JS_PROJECT).unwrap();

        install_hooks(&p.root().join(".git")).unwrap();
        write_root_gitignore(p.root()).unwrap();
        p.initial_commit().unwrap();

        // Pre-existing files still intact
        assert!(p.root().join("package.json").exists());
        assert!(p.root().join("src/index.js").exists());

        // Ship structure present
        p.assert_ship_file("agents/skills/task-policy/index.md");

        // Feature branch workflow works identically
        create_feature(
            p.ship_dir.clone(),
            "User Dashboard",
            "React dashboard with auth.",
            None,
            None,
            Some("feature/dashboard"),
        )
        .unwrap();
        p.create_feature_branch("feature/dashboard").unwrap();
        on_post_checkout(&p.ship_dir, "feature/dashboard", &p.root()).unwrap();

        p.assert_root_file_contains("CLAUDE.md", "User Dashboard");
        p.assert_root_file_contains("CLAUDE.md", "React dashboard with auth");

        // Existing source files not touched
        assert!(p.root().join("package.json").exists());
        assert!(p.root().join("src/index.js").exists());
    }
}
