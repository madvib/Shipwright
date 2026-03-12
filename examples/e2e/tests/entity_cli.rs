mod helpers;

use helpers::TestProject;
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
fn workspace_session_cli_rejects_updated_spec_and_emits_session_record() {
    let project = TestProject::with_git().unwrap();

    let out = run_cli(
        &project,
        &[
            "workspace",
            "session",
            "end",
            "--branch",
            "main",
            "--updated-spec",
            "legacy-spec-id",
        ],
    );
    assert!(
        !out.status.success(),
        "workspace session end should reject --updated-spec\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--updated-spec"),
        "expected parse error mentioning --updated-spec:\n{stderr}"
    );

    let out = run_cli(
        &project,
        &[
            "workspace",
            "create",
            "main",
            "--type",
            "patch",
            "--activate",
            "--no-input",
        ],
    );
    assert_success(&out, "workspace create failed");

    let out = run_cli(
        &project,
        &[
            "workspace",
            "session",
            "start",
            "--goal",
            "validate session record",
        ],
    );
    assert_success(&out, "workspace session start failed");

    let out = run_cli(
        &project,
        &["workspace", "session", "end", "--summary", "validated"],
    );
    assert_success(&out, "workspace session end failed");

    let out = run_cli(
        &project,
        &[
            "workspace",
            "session",
            "list",
            "--branch",
            "main",
            "--limit",
            "1",
        ],
    );
    assert_success(&out, "workspace session list failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("record="),
        "session list should include session record id:\n{stdout}"
    );
}

#[test]
fn cli_root_help_omits_legacy_spec_and_issue_commands() {
    let project = TestProject::new().unwrap();

    let out = run_cli(&project, &["--help"]);
    assert_success(&out, "ship --help failed");
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(
        !stdout.contains("\n  spec"),
        "legacy `spec` command should not appear in root help:\n{stdout}"
    );
    assert!(
        !stdout.contains("\n  issue"),
        "legacy `issue` command should not appear in root help:\n{stdout}"
    );
    assert!(
        stdout.contains("\n  workspace"),
        "workspace command should appear in root help:\n{stdout}"
    );
}
