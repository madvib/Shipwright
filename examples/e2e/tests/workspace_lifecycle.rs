mod helpers;

use helpers::TestProject;
use runtime::{WorkspaceStatus, get_workspace};
use ship_module_project::ops::feature::{create_feature, get_feature_by_id};
use std::process::{Command, Output};

fn run_cli(project: &TestProject, args: &[&str]) -> Output {
    project.cli(args).output().unwrap()
}

fn assert_success(out: &Output, context: &str) {
    assert!(
        out.status.success(),
        "{}\nstdout: {}\nstderr: {}",
        context,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn branch_exists(project: &TestProject, branch: &str) -> bool {
    Command::new("git")
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}"),
        ])
        .current_dir(project.root())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[test]
fn workspace_create_list_and_archive_happy_path() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            "feature/planned",
            "--feature",
            "feat-planned",
        ],
    );
    assert_success(&out, "workspace create failed");

    let planned = get_workspace(&project.ship_dir, "feature/planned")
        .unwrap()
        .expect("workspace should exist after create");
    assert_eq!(planned.status, WorkspaceStatus::Active);
    assert_eq!(planned.feature_id.as_deref(), Some("feat-planned"));

    let out = run_cli(&project, &["workspace", "list"]);
    assert_success(&out, "workspace list failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("[active] feature/planned (feature) feature=feat-planned"),
        "workspace list missing active entry:\n{stdout}"
    );

    let out = run_cli(&project, &["workspace", "archive", "feature/planned"]);
    assert_success(&out, "workspace archive failed");

    let archived = get_workspace(&project.ship_dir, "feature/planned")
        .unwrap()
        .expect("workspace should still exist after archive");
    assert_eq!(archived.status, WorkspaceStatus::Archived);
}

#[test]
fn workspace_create_feature_type_auto_creates_and_links_feature() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            "feature/workspace-first",
            "--type",
            "feature",
            "--feature-title",
            "Workspace First Runtime",
        ],
    );
    assert_success(&out, "workspace create should auto-create a feature");

    let workspace = get_workspace(&project.ship_dir, "feature/workspace-first")
        .unwrap()
        .expect("workspace should exist");
    let feature_id = workspace
        .feature_id
        .clone()
        .expect("workspace should link a feature");
    let feature = get_feature_by_id(&project.ship_dir, &feature_id).unwrap();
    assert_eq!(feature.feature.metadata.title, "Workspace First Runtime");
    assert_eq!(
        feature.feature.metadata.branch.as_deref(),
        Some("feature/workspace-first")
    );
}

#[test]
fn workspace_create_feature_type_reuses_existing_branch_linked_feature() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let existing = create_feature(
        &project.ship_dir,
        "Existing Workspace Feature",
        "",
        None,
        Some("feature/reuse-existing"),
    )
    .unwrap();

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            "feature/reuse-existing",
            "--type",
            "feature",
        ],
    );
    assert_success(
        &out,
        "workspace create should reuse an existing branch-linked feature",
    );

    let workspace = get_workspace(&project.ship_dir, "feature/reuse-existing")
        .unwrap()
        .expect("workspace should exist");
    assert_eq!(workspace.feature_id.as_deref(), Some(existing.id.as_str()));
}

#[test]
fn workspace_session_start_status_end_and_list_happy_path() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let out = run_cli(
        &project,
        &["workspace", "create", "feature/session-flow", "--checkout"],
    );
    assert_success(&out, "workspace create --checkout failed");

    let out = run_cli(
        &project,
        &[
            "workspace",
            "session",
            "start",
            "--goal",
            "Implement workspace sessions",
        ],
    );
    assert_success(&out, "workspace session start failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Session started:"),
        "session start output mismatch:\n{stdout}"
    );

    let out = run_cli(&project, &["workspace", "session", "status"]);
    assert_success(&out, "workspace session status failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("[active]"),
        "session status should show an active session:\n{stdout}"
    );

    let out = run_cli(
        &project,
        &[
            "workspace",
            "session",
            "end",
            "--summary",
            "Implemented lifecycle wiring",
        ],
    );
    assert_success(&out, "workspace session end failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Session ended:"),
        "session end output mismatch:\n{stdout}"
    );

    let out = run_cli(
        &project,
        &[
            "workspace",
            "session",
            "list",
            "--branch",
            "feature/session-flow",
            "--limit",
            "5",
        ],
    );
    assert_success(&out, "workspace session list failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("[ended]"),
        "session list should include ended session:\n{stdout}"
    );
    assert!(
        stdout.contains("summary=\"Implemented lifecycle wiring\""),
        "session list should include summary:\n{stdout}"
    );
    assert!(
        stdout.contains("record="),
        "session list should include session record id:\n{stdout}"
    );
}

#[test]
fn workspace_checkout_activation_demotes_previous_active_workspace() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            "feature/alpha",
            "--checkout",
            "--feature",
            "feat-alpha",
        ],
    );
    assert_success(&out, "workspace create --checkout for alpha failed");

    let alpha = get_workspace(&project.ship_dir, "feature/alpha")
        .unwrap()
        .expect("alpha workspace should exist");
    assert_eq!(alpha.status, WorkspaceStatus::Active);
    assert_eq!(alpha.feature_id.as_deref(), Some("feat-alpha"));

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            "feature/beta",
            "--checkout",
            "--feature",
            "feat-beta",
        ],
    );
    assert_success(&out, "workspace create --checkout for beta failed");

    let alpha_after = get_workspace(&project.ship_dir, "feature/alpha")
        .unwrap()
        .expect("alpha workspace should remain tracked");
    let beta_after = get_workspace(&project.ship_dir, "feature/beta")
        .unwrap()
        .expect("beta workspace should exist");
    assert_eq!(alpha_after.status, WorkspaceStatus::Active);
    assert_eq!(beta_after.status, WorkspaceStatus::Active);
    assert_eq!(project.current_branch(), "feature/beta");

    let out = run_cli(&project, &["workspace", "sync"]);
    assert_success(&out, "workspace sync failed");
    let sync_stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        sync_stdout.contains("Workspace synced: feature/beta [active]"),
        "sync should target current branch:\n{}",
        sync_stdout
    );
}

#[test]
fn workspace_sync_regenerates_agent_context_for_current_workspace() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            "feature/sync-context",
            "--type",
            "feature",
            "--feature-title",
            "Sync Context",
            "--checkout",
        ],
    );
    assert_success(&out, "workspace create --checkout failed");
    assert_eq!(project.current_branch(), "feature/sync-context");

    project.assert_root_file_contains("CLAUDE.md", "# [ship] Sync Context");
    std::fs::remove_file(project.root().join("CLAUDE.md")).unwrap();
    project.assert_no_root_file("CLAUDE.md");

    let out = run_cli(
        &project,
        &["workspace", "sync", "--branch", "feature/sync-context"],
    );
    assert_success(&out, "workspace sync failed");
    project.assert_root_file_contains("CLAUDE.md", "# [ship] Sync Context");
}

#[test]
fn workspace_create_worktree_sets_metadata_and_creates_worktree_dir() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let worktree_path = project
        .root()
        .join(".worktrees")
        .join("feature-auth-runtime");
    let worktree_arg = worktree_path.to_string_lossy().to_string();

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            "feature/auth-runtime",
            "--worktree",
            "--worktree-path",
            &worktree_arg,
            "--type",
            "feature",
            "--feature-title",
            "Auth Runtime",
        ],
    );
    assert_success(&out, "workspace create --worktree failed");

    assert!(
        worktree_path.exists(),
        "expected git worktree path to exist: {}",
        worktree_path.display()
    );

    let workspace = get_workspace(&project.ship_dir, "feature/auth-runtime")
        .unwrap()
        .expect("worktree workspace should exist");
    assert_eq!(workspace.status, WorkspaceStatus::Active);
    assert!(workspace.is_worktree);
    assert_eq!(
        workspace.worktree_path.as_deref(),
        Some(worktree_arg.as_str())
    );
    assert!(workspace.feature_id.is_some());
    assert!(
        !project.root().join("CLAUDE.md").exists(),
        "main checkout should not receive worktree branch context file"
    );
    assert!(
        !project.root().join(".mcp.json").exists(),
        "main checkout should not receive worktree provider config"
    );
    let worktree_context = std::fs::read_to_string(worktree_path.join("CLAUDE.md"))
        .expect("worktree context file should be generated during workspace create");
    assert!(
        worktree_context.contains("# [ship] Auth Runtime"),
        "worktree context should contain linked feature title:\n{}",
        worktree_context
    );
    assert!(
        worktree_path.join(".mcp.json").exists(),
        "worktree provider config should be generated during workspace create"
    );

    std::fs::remove_file(worktree_path.join("CLAUDE.md")).unwrap();
    let out = run_cli(
        &project,
        &["workspace", "sync", "--branch", "feature/auth-runtime"],
    );
    assert_success(&out, "workspace sync should target worktree context root");
    assert!(
        !project.root().join("CLAUDE.md").exists(),
        "workspace sync should not write context to main checkout for worktree branches"
    );
    assert!(
        !project.root().join(".mcp.json").exists(),
        "workspace sync should not write provider config to main checkout for worktree branches"
    );
    let regenerated = std::fs::read_to_string(worktree_path.join("CLAUDE.md"))
        .expect("worktree context should be regenerated by workspace sync");
    assert!(
        regenerated.contains("# [ship] Auth Runtime"),
        "workspace sync should regenerate worktree context:\n{}",
        regenerated
    );
    assert!(
        worktree_path.join(".mcp.json").exists(),
        "workspace sync should regenerate provider config at worktree root"
    );
}

#[test]
fn workspace_switch_to_worktree_branch_activates_without_main_checkout() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let worktree_path = project.root().join(".worktrees").join("feature-ai-runtime");
    let worktree_arg = worktree_path.to_string_lossy().to_string();

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            "feature/ai-runtime",
            "--worktree",
            "--worktree-path",
            &worktree_arg,
            "--type",
            "feature",
            "--feature-title",
            "AI Runtime",
        ],
    );
    assert_success(&out, "workspace create --worktree failed");

    let out = run_cli(
        &project,
        &["workspace", "create", "feature/main-runtime", "--checkout"],
    );
    assert_success(&out, "workspace create --checkout failed");
    assert_eq!(project.current_branch(), "feature/main-runtime");
    let root_context_before = std::fs::read_to_string(project.root().join("CLAUDE.md")).ok();

    let out = run_cli(&project, &["workspace", "switch", "feature/ai-runtime"]);
    assert_success(
        &out,
        "workspace switch should activate worktree branch without git checkout in main checkout",
    );

    let worktree_workspace = get_workspace(&project.ship_dir, "feature/ai-runtime")
        .unwrap()
        .expect("worktree workspace should exist");
    let main_workspace = get_workspace(&project.ship_dir, "feature/main-runtime")
        .unwrap()
        .expect("main workspace should exist");
    assert_eq!(worktree_workspace.status, WorkspaceStatus::Active);
    assert_eq!(main_workspace.status, WorkspaceStatus::Active);
    assert_eq!(
        project.current_branch(),
        "feature/main-runtime",
        "main checkout branch should remain unchanged when switching to a worktree workspace"
    );
    let root_context_after = std::fs::read_to_string(project.root().join("CLAUDE.md")).ok();
    assert_eq!(
        root_context_before, root_context_after,
        "switching to a worktree workspace should not mutate context in main checkout root"
    );
    let worktree_context = std::fs::read_to_string(worktree_path.join("CLAUDE.md"))
        .expect("switching worktree workspace should ensure worktree context is present");
    assert!(
        worktree_context.contains("# [ship] AI Runtime"),
        "worktree context should match linked feature title:\n{}",
        worktree_context
    );
}

#[test]
fn workspace_archive_allows_active_workspace_transition() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            "feature/active-cannot-archive",
            "--checkout",
            "--feature",
            "feat-active-cannot-archive",
        ],
    );
    assert_success(&out, "workspace create --checkout failed");

    let out = run_cli(
        &project,
        &["workspace", "archive", "feature/active-cannot-archive"],
    );
    assert_success(&out, "archive should succeed for active workspace");

    let workspace = get_workspace(&project.ship_dir, "feature/active-cannot-archive")
        .unwrap()
        .expect("workspace should still exist");
    assert_eq!(workspace.status, WorkspaceStatus::Archived);
}

#[test]
fn workspace_checkout_failure_does_not_persist_workspace_record() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let invalid_branch = "feature invalid branch";
    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            invalid_branch,
            "--checkout",
            "--feature",
            "feat-invalid-checkout",
        ],
    );
    assert!(
        !out.status.success(),
        "workspace create --checkout should fail for invalid branch\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let workspace = get_workspace(&project.ship_dir, invalid_branch).unwrap();
    assert!(
        workspace.is_none(),
        "failed checkout should not leave a persisted workspace row"
    );
    assert_eq!(
        project.current_branch(),
        "main",
        "failed checkout should not move current branch"
    );
}

#[test]
fn workspace_worktree_failure_does_not_persist_workspace_record() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let occupied_path = project.root().join(".worktrees").join("occupied");
    std::fs::create_dir_all(&occupied_path).unwrap();
    std::fs::write(occupied_path.join("already.txt"), "occupied").unwrap();
    let occupied_arg = occupied_path.to_string_lossy().to_string();
    let branch = "feature/worktree-create-failure";

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            branch,
            "--worktree",
            "--worktree-path",
            &occupied_arg,
            "--feature",
            "feat-worktree-create-failure",
        ],
    );
    assert!(
        !out.status.success(),
        "workspace create --worktree should fail for occupied path\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let workspace = get_workspace(&project.ship_dir, branch).unwrap();
    assert!(
        workspace.is_none(),
        "failed worktree add should not leave a persisted workspace row"
    );
    assert!(
        !branch_exists(&project, branch),
        "failed worktree add should not leave a dangling branch"
    );
    assert!(
        occupied_path.join("already.txt").exists(),
        "occupied path contents should remain untouched after failed worktree create"
    );
    assert_eq!(
        project.current_branch(),
        "main",
        "failed worktree create should not switch the main checkout branch"
    );
}

#[test]
fn workspace_create_rejects_worktree_path_without_worktree_flag() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let branch = "feature/worktree-path-without-flag";
    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            branch,
            "--worktree-path",
            "../worktrees/ignored",
            "--feature",
            "feat-worktree-path-without-flag",
        ],
    );
    assert!(
        !out.status.success(),
        "workspace create should fail when --worktree-path is used without --worktree\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--worktree-path requires --worktree"),
        "unexpected validation error:\n{stderr}"
    );

    let workspace = get_workspace(&project.ship_dir, branch).unwrap();
    assert!(
        workspace.is_none(),
        "validation failure should not persist a workspace row"
    );
    assert_eq!(
        project.current_branch(),
        "main",
        "validation failure should not switch branches"
    );
}

#[test]
fn workspace_create_rejects_checkout_and_worktree_flag_combo() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let branch = "feature/invalid-flag-combo";
    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            branch,
            "--checkout",
            "--worktree",
            "--feature",
            "feat-invalid-flag-combo",
        ],
    );
    assert!(
        !out.status.success(),
        "workspace create should fail when --checkout and --worktree are combined\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--worktree and --checkout cannot be used together"),
        "unexpected validation error:\n{stderr}"
    );

    let workspace = get_workspace(&project.ship_dir, branch).unwrap();
    assert!(
        workspace.is_none(),
        "validation failure should not persist a workspace row"
    );
    assert_eq!(
        project.current_branch(),
        "main",
        "validation failure should not switch branches"
    );
}

#[test]
fn workspace_recreate_without_worktree_clears_worktree_metadata() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let worktree_path = project
        .root()
        .join(".worktrees")
        .join("feature-worktree-metadata");
    let worktree_arg = worktree_path.to_string_lossy().to_string();
    let branch = "feature/worktree-metadata";

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            branch,
            "--worktree",
            "--worktree-path",
            &worktree_arg,
            "--feature",
            "feat-worktree-metadata",
        ],
    );
    assert_success(&out, "workspace create --worktree failed");

    let worktree_workspace = get_workspace(&project.ship_dir, branch)
        .unwrap()
        .expect("worktree workspace should exist");
    assert!(worktree_workspace.is_worktree);
    assert_eq!(
        worktree_workspace.worktree_path.as_deref(),
        Some(worktree_arg.as_str())
    );

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            branch,
            "--feature",
            "feat-worktree-metadata",
        ],
    );
    assert_success(&out, "workspace recreate without --worktree failed");

    let updated = get_workspace(&project.ship_dir, branch)
        .unwrap()
        .expect("updated workspace should exist");
    assert!(
        !updated.is_worktree,
        "workspace should no longer be marked as worktree"
    );
    assert!(
        updated.worktree_path.is_none(),
        "worktree path should be cleared when workspace is no longer a worktree"
    );
}

#[test]
fn workspace_worktree_failure_preserves_preexisting_branch() {
    let project = TestProject::with_git().unwrap();
    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let branch = "feature/worktree-existing-branch";
    project.checkout_new(branch).unwrap();
    project.checkout("main").unwrap();
    assert!(
        branch_exists(&project, branch),
        "precondition failed: branch should exist before test action"
    );

    let occupied_path = project.root().join(".worktrees").join("occupied-existing");
    std::fs::create_dir_all(&occupied_path).unwrap();
    std::fs::write(occupied_path.join("already.txt"), "occupied").unwrap();
    let occupied_arg = occupied_path.to_string_lossy().to_string();

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            branch,
            "--worktree",
            "--worktree-path",
            &occupied_arg,
            "--feature",
            "feat-worktree-existing-branch",
        ],
    );
    assert!(
        !out.status.success(),
        "workspace create --worktree should fail for occupied path\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        branch_exists(&project, branch),
        "failed worktree create should preserve an existing branch"
    );

    let workspace = get_workspace(&project.ship_dir, branch).unwrap();
    assert!(
        workspace.is_none(),
        "failed worktree add should not leave a persisted workspace row"
    );
}
