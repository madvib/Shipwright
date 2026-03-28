use super::*;
use std::path::PathBuf;

fn project_dir() -> PathBuf {
    // Use the repo root — this test runs inside a real git repo
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn status_returns_valid_json_with_branch() {
    let result = get_git_status(&project_dir());
    assert!(!result.starts_with("Error"), "unexpected error: {result}");
    let v: serde_json::Value = serde_json::from_str(&result).expect("invalid JSON");
    assert!(v["branch"].is_string());
    assert!(v["clean"].is_boolean());
    assert!(v["staged"].is_array());
    assert!(v["modified"].is_array());
    assert!(v["untracked"].is_array());
}

#[test]
fn status_errors_on_non_git_dir() {
    let result = get_git_status(Path::new("/tmp"));
    assert!(
        result.starts_with("Error"),
        "expected error for non-git dir: {result}"
    );
}

#[test]
fn diff_returns_string() {
    let result = get_git_diff(
        &project_dir(),
        GetGitDiffRequest {
            base: None,
            path: None,
        },
    );
    // Either "No differences found." or actual diff text — not an error
    assert!(!result.starts_with("Error"), "unexpected error: {result}");
}

#[test]
fn diff_with_base_ref() {
    let result = get_git_diff(
        &project_dir(),
        GetGitDiffRequest {
            base: Some("HEAD~1".into()),
            path: None,
        },
    );
    assert!(!result.starts_with("Error"), "unexpected error: {result}");
}

#[test]
fn log_returns_valid_json_array() {
    let result = get_git_log(
        &project_dir(),
        GetGitLogRequest {
            limit: Some(3),
            path: None,
        },
    );
    assert!(!result.starts_with("Error"), "unexpected error: {result}");
    let v: Vec<serde_json::Value> = serde_json::from_str(&result).expect("invalid JSON");
    assert!(!v.is_empty(), "log should have entries");
    assert!(v.len() <= 3, "should respect limit");
    let entry = &v[0];
    assert!(entry["hash"].is_string());
    assert!(entry["short_hash"].is_string());
    assert!(entry["message"].is_string());
    assert!(entry["author"].is_string());
    assert!(entry["date"].is_string());
    assert!(entry["files_changed"].is_number());
}

#[test]
fn log_with_path_filter() {
    let result = get_git_log(
        &project_dir(),
        GetGitLogRequest {
            limit: Some(2),
            path: Some("Cargo.toml".into()),
        },
    );
    assert!(!result.starts_with("Error"), "unexpected error: {result}");
    let v: Vec<serde_json::Value> = serde_json::from_str(&result).expect("invalid JSON");
    // May be empty if Cargo.toml hasn't been touched in recent non-merge commits
    assert!(v.len() <= 2);
}

#[test]
fn worktrees_returns_valid_json() {
    let result = list_worktrees(&project_dir());
    assert!(!result.starts_with("Error"), "unexpected error: {result}");
    let v: Vec<serde_json::Value> = serde_json::from_str(&result).expect("invalid JSON");
    assert!(!v.is_empty(), "should have at least one worktree");
    let entry = &v[0];
    assert!(entry["path"].is_string());
    assert!(entry["branch"].is_string());
    assert!(entry["head"].is_string());
}

#[test]
fn worktrees_errors_on_non_git_dir() {
    let result = list_worktrees(Path::new("/tmp"));
    assert!(
        result.starts_with("Error"),
        "expected error for non-git dir: {result}"
    );
}
