mod helpers;

use helpers::TestProject;
use std::path::Path;
use std::process::Output;

fn run_cli_with_global(project: &TestProject, global_dir: &Path, args: &[&str]) -> Output {
    project
        .cli(args)
        .env("SHIP_GLOBAL_DIR", global_dir)
        .output()
        .unwrap()
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
fn projects_track_persists_custom_registered_name() {
    let project = TestProject::with_git().unwrap();
    let global_dir = project.root().join(".test-global");
    std::fs::create_dir_all(&global_dir).unwrap();

    let project_root = project.root().to_string_lossy().to_string();
    let custom_name = "Ship Runtime";

    let out = run_cli_with_global(
        &project,
        &global_dir,
        &["projects", "track", custom_name, &project_root],
    );
    assert_success(&out, "projects track failed");

    let out = run_cli_with_global(&project, &global_dir, &["projects", "list"]);
    assert_success(&out, "projects list failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<_> = stdout
        .lines()
        .filter(|line| line.starts_with("- "))
        .collect();
    assert_eq!(
        lines.len(),
        1,
        "expected exactly one tracked project:\n{stdout}"
    );
    assert!(
        lines[0].starts_with("- Ship Runtime ("),
        "unexpected list entry: {}",
        lines[0]
    );
    assert!(
        lines[0].contains(project.ship_dir.to_string_lossy().as_ref()),
        "expected canonical .ship path in list entry: {}",
        lines[0]
    );
}

#[test]
fn worktree_track_dedupes_to_main_project_and_preserves_custom_name() {
    let project = TestProject::with_git().unwrap();
    let global_dir = project.root().join(".test-global");
    std::fs::create_dir_all(&global_dir).unwrap();

    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let checkout_feature = project.checkout_new("feature/auth").unwrap();
    assert_success(&checkout_feature, "creating feature branch failed");

    let checkout_main = project.checkout("main").unwrap();
    assert_success(&checkout_main, "checkout main failed");

    let worktree = project.add_worktree("feature/auth").unwrap();

    let project_root = project.root().to_string_lossy().to_string();
    let worktree_root = worktree.root().to_string_lossy().to_string();
    let default_name = project
        .root()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let custom_name = "Ship Core";
    let out = run_cli_with_global(
        &project,
        &global_dir,
        &["projects", "track", custom_name, &project_root],
    );
    assert_success(&out, "tracking main project failed");

    let out = run_cli_with_global(
        &project,
        &global_dir,
        &["projects", "track", &default_name, &worktree_root],
    );
    assert_success(
        &out,
        "tracking worktree path after custom naming should not fail or duplicate",
    );

    let out = run_cli_with_global(&project, &global_dir, &["projects", "list"]);
    assert_success(&out, "projects list failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<_> = stdout
        .lines()
        .filter(|line| line.starts_with("- "))
        .collect();
    assert_eq!(
        lines.len(),
        1,
        "expected deduped single project entry:\n{stdout}"
    );
    assert!(
        lines[0].starts_with("- Ship Core ("),
        "custom renamed name should be preserved: {}",
        lines[0]
    );
    assert!(
        lines[0].contains(project.ship_dir.to_string_lossy().as_ref()),
        "expected canonical .ship path in list entry: {}",
        lines[0]
    );
}

#[test]
fn worktree_rename_updates_main_project_entry_without_duplicate() {
    let project = TestProject::with_git().unwrap();
    let global_dir = project.root().join(".test-global");
    std::fs::create_dir_all(&global_dir).unwrap();

    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let checkout_feature = project.checkout_new("feature/auth").unwrap();
    assert_success(&checkout_feature, "creating feature branch failed");

    let checkout_main = project.checkout("main").unwrap();
    assert_success(&checkout_main, "checkout main failed");

    let worktree = project.add_worktree("feature/auth").unwrap();

    let project_root = project.root().to_string_lossy().to_string();
    let worktree_root = worktree.root().to_string_lossy().to_string();

    let out = run_cli_with_global(
        &project,
        &global_dir,
        &["projects", "track", "Ship Core", &project_root],
    );
    assert_success(&out, "tracking main project failed");

    let out = run_cli_with_global(
        &project,
        &global_dir,
        &["projects", "rename", &worktree_root, "Ship Core Renamed"],
    );
    assert_success(&out, "renaming via worktree path failed");

    let out = run_cli_with_global(&project, &global_dir, &["projects", "list"]);
    assert_success(&out, "projects list failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<_> = stdout
        .lines()
        .filter(|line| line.starts_with("- "))
        .collect();
    assert_eq!(
        lines.len(),
        1,
        "expected one deduped project after rename:\n{stdout}"
    );
    assert!(
        lines[0].starts_with("- Ship Core Renamed ("),
        "rename should update canonical project entry: {}",
        lines[0]
    );
    assert!(
        lines[0].contains(project.ship_dir.to_string_lossy().as_ref()),
        "expected canonical .ship path in list entry: {}",
        lines[0]
    );
}

#[test]
fn worktree_untrack_removes_main_project_entry() {
    let project = TestProject::with_git().unwrap();
    let global_dir = project.root().join(".test-global");
    std::fs::create_dir_all(&global_dir).unwrap();

    let init = project.initial_commit().unwrap();
    assert_success(&init, "initial git commit failed");

    let checkout_feature = project.checkout_new("feature/auth").unwrap();
    assert_success(&checkout_feature, "creating feature branch failed");

    let checkout_main = project.checkout("main").unwrap();
    assert_success(&checkout_main, "checkout main failed");

    let worktree = project.add_worktree("feature/auth").unwrap();

    let project_root = project.root().to_string_lossy().to_string();
    let worktree_root = worktree.root().to_string_lossy().to_string();

    let out = run_cli_with_global(
        &project,
        &global_dir,
        &["projects", "track", "Ship Core", &project_root],
    );
    assert_success(&out, "tracking main project failed");

    let out = run_cli_with_global(
        &project,
        &global_dir,
        &["projects", "untrack", &worktree_root],
    );
    assert_success(&out, "untracking via worktree path failed");

    let out = run_cli_with_global(&project, &global_dir, &["projects", "list"]);
    assert_success(&out, "projects list failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<_> = stdout
        .lines()
        .filter(|line| line.starts_with("- "))
        .collect();
    assert!(
        lines.is_empty(),
        "expected project list to be empty after untracking via worktree path:\n{stdout}"
    );
}
