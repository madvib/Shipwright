mod helpers;

use helpers::TestProject;
use ship_module_project::{IssueStatus, get_issue_by_id};
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

fn parse_issue_file_name(stdout: &str) -> String {
    let prefix = "Issue created: ";
    let line = stdout
        .lines()
        .find(|line| line.starts_with(prefix))
        .expect("issue create output missing expected prefix");
    let rest = &line[prefix.len()..];
    let (file_name, _) = rest
        .split_once(" (")
        .expect("issue create output missing file/id split");
    file_name.to_string()
}

#[test]
fn release_get_missing_returns_non_zero_exit() {
    let project = TestProject::new().unwrap();

    let out = run_cli(&project, &["release", "get", "v9.9.9"]);
    assert!(
        !out.status.success(),
        "release get should fail for missing release\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Release not found: v9.9.9"),
        "unexpected error for missing release:\n{stderr}"
    );
}

#[test]
fn release_cli_create_update_and_get_round_trip() {
    let project = TestProject::new().unwrap();

    let out = run_cli(
        &project,
        &[
            "release",
            "create",
            "v0.2.0",
            "--content",
            "Initial release content",
        ],
    );
    assert_success(&out, "release create failed");

    let out = run_cli(&project, &["release", "list"]);
    assert_success(&out, "release list failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("v0.2.0"),
        "release list missing created release:\n{stdout}"
    );

    let out = run_cli(
        &project,
        &[
            "release",
            "update",
            "v0.2.0",
            "--content",
            "Updated release content",
        ],
    );
    assert_success(&out, "release update failed");

    let out = run_cli(&project, &["release", "get", "v0.2.0"]);
    assert_success(&out, "release get failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Updated release content"),
        "release get missing updated content:\n{stdout}"
    );

    let suffixed = project.ship_dir.join("project/releases/v0.2.0-2.md");
    assert!(
        !suffixed.exists(),
        "release update should overwrite canonical file, found unexpected suffixed file at {}",
        suffixed.display()
    );
}

#[test]
fn issue_move_with_from_status_mismatch_fails_and_keeps_state() {
    let project = TestProject::new().unwrap();

    let out = run_cli(
        &project,
        &[
            "issue",
            "create",
            "Mismatch test issue",
            "Issue description",
        ],
    );
    assert_success(&out, "issue create failed");
    let created_stdout = String::from_utf8_lossy(&out.stdout);
    let file_name = parse_issue_file_name(&created_stdout);

    let out = run_cli(&project, &["issue", "move", &file_name, "review", "done"]);
    assert!(
        !out.status.success(),
        "issue move with mismatched from-status should fail\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let issue = get_issue_by_id(&project.ship_dir, &file_name).expect("issue should still exist");
    assert_eq!(
        issue.status,
        IssueStatus::Backlog,
        "mismatched move should keep issue in backlog"
    );

    let out = run_cli(
        &project,
        &["issue", "move", &file_name, "backlog", "in-progress"],
    );
    assert_success(&out, "issue move backlog -> in-progress failed");

    let moved = get_issue_by_id(&project.ship_dir, &file_name).expect("issue should still exist");
    assert_eq!(
        moved.status,
        IssueStatus::InProgress,
        "valid move should update issue status"
    );
}
