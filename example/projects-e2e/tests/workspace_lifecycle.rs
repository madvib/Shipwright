mod helpers;

use helpers::TestProject;
use runtime::{WorkspaceStatus, get_workspace};
use std::process::Output;

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
    assert_eq!(planned.status, WorkspaceStatus::Planned);
    assert_eq!(planned.feature_id.as_deref(), Some("feat-planned"));

    let out = run_cli(&project, &["workspace", "list"]);
    assert_success(&out, "workspace list failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("[planned] feature/planned (feature) feature=feat-planned"),
        "workspace list missing planned entry:\n{stdout}"
    );

    let out = run_cli(&project, &["workspace", "archive", "feature/planned"]);
    assert_success(&out, "workspace archive failed");

    let archived = get_workspace(&project.ship_dir, "feature/planned")
        .unwrap()
        .expect("workspace should still exist after archive");
    assert_eq!(archived.status, WorkspaceStatus::Archived);
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
    assert_eq!(alpha_after.status, WorkspaceStatus::Idle);
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
            "--feature",
            "feat-auth-runtime",
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
    assert_eq!(workspace.feature_id.as_deref(), Some("feat-auth-runtime"));
}

#[test]
fn workspace_archive_rejects_active_workspace_transition() {
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
    assert!(
        !out.status.success(),
        "archive should fail for active workspace\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Invalid workspace transition: active -> archived"),
        "unexpected archive failure error:\n{stderr}"
    );

    let workspace = get_workspace(&project.ship_dir, "feature/active-cannot-archive")
        .unwrap()
        .expect("workspace should still exist");
    assert_eq!(workspace.status, WorkspaceStatus::Active);
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
        occupied_path.join("already.txt").exists(),
        "occupied path contents should remain untouched after failed worktree create"
    );
    assert_eq!(
        project.current_branch(),
        "main",
        "failed worktree create should not switch the main checkout branch"
    );
}
