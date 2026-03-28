use super::*;
use crate::db::ensure_db;
use crate::db::jobs::create_job;
use crate::project::init_project;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    ensure_db().unwrap();
    (tmp, ship_dir)
}

fn mkjob() -> String {
    create_job("build", None, None, None, None, 0, None, vec![], vec![])
        .unwrap()
        .id
}

#[test]
fn test_claim_files_basic() {
    let (_tmp, _sd) = setup();
    let job = mkjob();
    claim_files(&job, None, &["src/lib.rs", "src/main.rs"]).unwrap();

    let claims = list_claims(Some(&job)).unwrap();
    assert_eq!(claims.len(), 2);
    assert_eq!(claims[0].path, "src/lib.rs");
    assert_eq!(claims[1].path, "src/main.rs");
    assert_eq!(claims[0].job_id, job);
}

#[test]
fn test_claim_files_with_workspace_id() {
    let (_tmp, _sd) = setup();
    let job = mkjob();
    claim_files(&job, Some("ws-1"), &["README.md"]).unwrap();

    let claims = list_claims(Some(&job)).unwrap();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].workspace_id, Some("ws-1".to_string()));
}

#[test]
fn test_idempotent_reclaim() {
    let (_tmp, _sd) = setup();
    let job = mkjob();
    claim_files(&job, None, &["src/lib.rs"]).unwrap();
    // Re-claiming the same paths with the same job is a no-op.
    claim_files(&job, None, &["src/lib.rs"]).unwrap();

    let claims = list_claims(Some(&job)).unwrap();
    assert_eq!(claims.len(), 1);
}

#[test]
fn test_conflict_detection() {
    let (_tmp, _sd) = setup();
    let job_a = mkjob();
    let job_b = mkjob();

    claim_files(&job_a, None, &["src/lib.rs", "Cargo.toml"]).unwrap();

    // job_b tries to claim overlapping paths.
    let err = claim_files(&job_b, None, &["src/lib.rs", "src/new.rs"]).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("file claim conflict"), "got: {msg}");
    assert!(
        msg.contains("src/lib.rs"),
        "should mention conflicting path, got: {msg}"
    );
    assert!(
        msg.contains(&job_a),
        "should mention owning job, got: {msg}"
    );

    // job_b should not have partially claimed anything.
    let b_claims = list_claims(Some(&job_b)).unwrap();
    assert!(b_claims.is_empty(), "no partial claims after conflict");
}

#[test]
fn test_check_conflicts_returns_all() {
    let (_tmp, _sd) = setup();
    let job_a = mkjob();
    claim_files(&job_a, None, &["a.rs", "b.rs"]).unwrap();

    let conflicts = check_conflicts(&["a.rs", "b.rs", "c.rs"]).unwrap();
    assert_eq!(conflicts.len(), 2);
    let paths: Vec<&str> = conflicts.iter().map(|(p, _)| p.as_str()).collect();
    assert!(paths.contains(&"a.rs"));
    assert!(paths.contains(&"b.rs"));
}

#[test]
fn test_release_and_reclaim() {
    let (_tmp, _sd) = setup();
    let job_a = mkjob();
    let job_b = mkjob();

    claim_files(&job_a, None, &["src/lib.rs", "src/main.rs"]).unwrap();
    let released = release_claims(&job_a).unwrap();
    assert_eq!(released, 2);

    // After release, job_b can claim the same paths.
    claim_files(&job_b, None, &["src/lib.rs", "src/main.rs"]).unwrap();
    let claims = list_claims(Some(&job_b)).unwrap();
    assert_eq!(claims.len(), 2);
}

#[test]
fn test_list_claims_all() {
    let (_tmp, _sd) = setup();
    let job_a = mkjob();
    let job_b = mkjob();

    claim_files(&job_a, None, &["a.rs"]).unwrap();
    claim_files(&job_b, None, &["b.rs"]).unwrap();

    let all = list_claims(None).unwrap();
    assert_eq!(all.len(), 2);

    let a_only = list_claims(Some(&job_a)).unwrap();
    assert_eq!(a_only.len(), 1);
    assert_eq!(a_only[0].path, "a.rs");
}

#[test]
fn test_claim_empty_paths_is_noop() {
    let (_tmp, _sd) = setup();
    let job = mkjob();
    claim_files(&job, None, &[]).unwrap();
    let claims = list_claims(Some(&job)).unwrap();
    assert!(claims.is_empty());
}

#[test]
fn test_release_nonexistent_returns_zero() {
    let (_tmp, _sd) = setup();
    let released = release_claims("no-such-job").unwrap();
    assert_eq!(released, 0);
}
