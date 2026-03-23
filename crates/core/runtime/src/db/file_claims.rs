//! Atomic file-claim tracking for concurrent agent coordination.
//!
//! Unlike `job_file` (first-wins, single file at a time), this module
//! provides batch atomic claims: either ALL paths are claimed or NONE are.
//! Claims include an optional `workspace_id` for cross-workspace tracking.

use anyhow::{Result, anyhow};
use chrono::Utc;
use sqlx::Row;
use std::path::Path;

use crate::db::{block_on, open_db};

/// A single file claim record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileClaim {
    pub path: String,
    pub job_id: String,
    pub workspace_id: Option<String>,
    pub claimed_at: String,
}

/// Atomically claim a set of file paths for a job.
///
/// If ANY path is already claimed by a *different* job, the entire operation
/// fails with an error listing all conflicts. Re-claiming paths already owned
/// by the same `job_id` is a no-op (idempotent).
pub fn claim_files(
    _ship_dir: &Path,
    job_id: &str,
    workspace_id: Option<&str>,
    paths: &[&str],
) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }

    let conflicts = check_conflicts_for_job(_ship_dir, job_id, paths)?;
    if !conflicts.is_empty() {
        let detail: Vec<String> = conflicts
            .iter()
            .map(|(p, owner)| format!("  {p} (owned by {owner})"))
            .collect();
        return Err(anyhow!(
            "file claim conflict -- {} path(s) already claimed:\n{}",
            conflicts.len(),
            detail.join("\n")
        ));
    }

    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    for path in paths {
        block_on(async {
            sqlx::query(
                "INSERT OR IGNORE INTO file_claim (path, job_id, workspace_id, claimed_at) \
                 VALUES (?, ?, ?, ?)",
            )
            .bind(*path)
            .bind(job_id)
            .bind(workspace_id)
            .bind(&now)
            .execute(&mut conn)
            .await
        })?;
    }
    Ok(())
}

/// Release all file claims held by `job_id`. Returns the number of claims released.
pub fn release_claims(_ship_dir: &Path, job_id: &str) -> Result<usize> {
    let mut conn = open_db()?;
    let result = block_on(async {
        sqlx::query("DELETE FROM file_claim WHERE job_id = ?")
            .bind(job_id)
            .execute(&mut conn)
            .await
    })?;
    Ok(result.rows_affected() as usize)
}

/// Check which paths from `paths` are already claimed by any job.
/// Returns `(path, claiming_job_id)` pairs for every conflict.
pub fn check_conflicts(_ship_dir: &Path, paths: &[&str]) -> Result<Vec<(String, String)>> {
    if paths.is_empty() {
        return Ok(vec![]);
    }
    let mut conn = open_db()?;
    let placeholders: Vec<&str> = paths.iter().map(|_| "?").collect();
    let sql = format!(
        "SELECT path, job_id FROM file_claim WHERE path IN ({})",
        placeholders.join(", ")
    );
    let mut query = sqlx::query(&sql);
    for p in paths {
        query = query.bind(*p);
    }
    let rows = block_on(async { query.fetch_all(&mut conn).await })?;
    Ok(rows
        .iter()
        .map(|r| {
            let path: String = r.get(0);
            let job_id: String = r.get(1);
            (path, job_id)
        })
        .collect())
}

/// List file claims, optionally filtered by `job_id`.
pub fn list_claims(_ship_dir: &Path, job_id: Option<&str>) -> Result<Vec<FileClaim>> {
    let mut conn = open_db()?;
    let rows = match job_id {
        Some(jid) => block_on(async {
            sqlx::query(
                "SELECT path, job_id, workspace_id, claimed_at \
                 FROM file_claim WHERE job_id = ? ORDER BY path",
            )
            .bind(jid)
            .fetch_all(&mut conn)
            .await
        })?,
        None => block_on(async {
            sqlx::query(
                "SELECT path, job_id, workspace_id, claimed_at \
                 FROM file_claim ORDER BY path",
            )
            .fetch_all(&mut conn)
            .await
        })?,
    };
    Ok(rows.iter().map(row_to_claim).collect())
}

// ── internal ────────────────────────────────────────────────────────────────

/// Like `check_conflicts`, but excludes paths owned by `job_id` itself
/// (those are idempotent re-claims, not conflicts).
fn check_conflicts_for_job(
    _ship_dir: &Path,
    job_id: &str,
    paths: &[&str],
) -> Result<Vec<(String, String)>> {
    if paths.is_empty() {
        return Ok(vec![]);
    }
    let mut conn = open_db()?;
    let placeholders: Vec<&str> = paths.iter().map(|_| "?").collect();
    let sql = format!(
        "SELECT path, job_id FROM file_claim WHERE path IN ({}) AND job_id != ?",
        placeholders.join(", ")
    );
    let mut query = sqlx::query(&sql);
    for p in paths {
        query = query.bind(*p);
    }
    query = query.bind(job_id);
    let rows = block_on(async { query.fetch_all(&mut conn).await })?;
    Ok(rows
        .iter()
        .map(|r| {
            let path: String = r.get(0);
            let owner: String = r.get(1);
            (path, owner)
        })
        .collect())
}

fn row_to_claim(row: &sqlx::sqlite::SqliteRow) -> FileClaim {
    FileClaim {
        path: row.get(0),
        job_id: row.get(1),
        workspace_id: row.get(2),
        claimed_at: row.get(3),
    }
}

#[cfg(test)]
#[path = "file_claims_tests.rs"]
mod tests;
