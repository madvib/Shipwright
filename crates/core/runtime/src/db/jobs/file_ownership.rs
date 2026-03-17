//! Atomic file-ownership tracking for jobs.
//!
//! `claim_file` is first-wins: INSERT OR IGNORE on the path PRIMARY KEY means
//! the first caller wins and subsequent callers get `false`. File claims are
//! released automatically when a job reaches a terminal status via `update_job`.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use std::path::Path;

use crate::db::{block_on, open_db};

/// Atomically claim `path` for `job_id`. Returns `true` if the claim was
/// granted (this job is now the owner), `false` if another job already owns it.
pub fn claim_file(ship_dir: &Path, job_id: &str, path: &str) -> Result<bool> {
    let mut conn = open_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    let result = block_on(async {
        sqlx::query(
            "INSERT OR IGNORE INTO job_file (path, job_id, claimed_at) VALUES (?, ?, ?)",
        )
        .bind(path)
        .bind(job_id)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(result.rows_affected() == 1)
}

/// Return the job_id that currently owns `path`, or `None` if unclaimed.
pub fn get_file_owner(ship_dir: &Path, path: &str) -> Result<Option<String>> {
    let mut conn = open_db(ship_dir)?;
    let row = block_on(async {
        sqlx::query("SELECT job_id FROM job_file WHERE path = ?")
            .bind(path)
            .fetch_optional(&mut conn)
            .await
    })?;
    Ok(row.map(|r| r.get(0)))
}

/// Release all file claims held by `job_id`. Called automatically by
/// `update_job` when the job reaches a terminal status.
pub fn release_job_files(ship_dir: &Path, job_id: &str) -> Result<()> {
    let mut conn = open_db(ship_dir)?;
    block_on(async {
        sqlx::query("DELETE FROM job_file WHERE job_id = ?")
            .bind(job_id)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ensure_db;
    use crate::db::jobs::{create_job, update_job, JobPatch};
    use crate::project::init_project;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db(&ship_dir).unwrap();
        (tmp, ship_dir)
    }

    fn mkjob(ship_dir: &Path, kind: &str) -> String {
        create_job(ship_dir, kind, None, None, None, None, 0, None, vec![], vec![])
            .unwrap()
            .id
    }

    /// First caller wins; second caller for the same path gets false.
    #[test]
    fn test_concurrent_claim_conflict() {
        let (_tmp, ship_dir) = setup();
        let job_a = mkjob(&ship_dir, "build");
        let job_b = mkjob(&ship_dir, "lint");

        let first = claim_file(&ship_dir, &job_a, "src/lib.rs").unwrap();
        let second = claim_file(&ship_dir, &job_b, "src/lib.rs").unwrap();

        assert!(first, "first claim should succeed");
        assert!(!second, "second claim for same path must fail");
    }

    /// A path can only ever have one owner at a time.
    #[test]
    fn test_single_owner_invariant() {
        let (_tmp, ship_dir) = setup();
        let job_a = mkjob(&ship_dir, "build");
        let job_b = mkjob(&ship_dir, "test");

        claim_file(&ship_dir, &job_a, "Cargo.toml").unwrap();
        claim_file(&ship_dir, &job_b, "Cargo.toml").unwrap(); // no-op

        let owner = get_file_owner(&ship_dir, "Cargo.toml").unwrap();
        assert_eq!(owner, Some(job_a), "only first claimer is owner");
    }

    /// File claims are released when the job reaches a terminal status.
    #[test]
    fn test_release_on_completion() {
        let (_tmp, ship_dir) = setup();
        let job_a = mkjob(&ship_dir, "build");
        let job_b = mkjob(&ship_dir, "build");

        claim_file(&ship_dir, &job_a, "src/main.rs").unwrap();
        assert_eq!(
            get_file_owner(&ship_dir, "src/main.rs").unwrap(),
            Some(job_a.clone())
        );

        // Complete job_a — its claims must be released.
        update_job(&ship_dir, &job_a, JobPatch {
            status: Some("complete".to_string()),
            ..Default::default()
        }).unwrap();

        assert_eq!(get_file_owner(&ship_dir, "src/main.rs").unwrap(), None);

        // job_b can now claim the file.
        let claimed = claim_file(&ship_dir, &job_b, "src/main.rs").unwrap();
        assert!(claimed, "file should be claimable after original owner completes");
    }
}
