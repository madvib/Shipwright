//! Job queue and job_log for agent coordination.
//! Written by `ship agent job` commands; referenced from skills.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use std::path::Path;

use crate::db::{block_on, open_db};
use crate::gen_nanoid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub branch: Option<String>,
    pub payload: serde_json::Value,
    pub created_by: Option<String>,
    pub claimed_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobLogEntry {
    pub id: i64,
    pub job_id: Option<String>,
    pub branch: Option<String>,
    pub message: String,
    pub actor: Option<String>,
    pub created_at: String,
}

const J_COLS: &str =
    "id, kind, status, branch, payload_json, created_by, claimed_by, created_at, updated_at";

const L_COLS: &str = "id, job_id, branch, message, actor, created_at";

pub fn create_job(
    ship_dir: &Path,
    kind: &str,
    branch: Option<&str>,
    payload: Option<serde_json::Value>,
    created_by: Option<&str>,
) -> Result<Job> {
    let mut conn = open_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    let id = gen_nanoid();
    let payload = payload.unwrap_or(serde_json::Value::Object(Default::default()));
    let payload_str = serde_json::to_string(&payload)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO job (id, kind, status, branch, payload_json, created_by, created_at, updated_at)
             VALUES (?, ?, 'pending', ?, ?, ?, ?, ?)",
        )
        .bind(&id).bind(kind).bind(branch).bind(&payload_str)
        .bind(created_by).bind(&now).bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(Job {
        id,
        kind: kind.to_string(),
        status: "pending".to_string(),
        branch: branch.map(str::to_string),
        payload,
        created_by: created_by.map(str::to_string),
        claimed_by: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// Atomically claim a pending job. Returns false if already claimed — prevents
/// double-claiming when multiple commanders share the same queue (e.g. Claude + Codex,
/// or two machines syncing the same platform.db).
pub fn claim_job(ship_dir: &Path, job_id: &str, claimed_by: &str) -> Result<bool> {
    let mut conn = open_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    let rows = block_on(async {
        sqlx::query(
            "UPDATE job SET status = 'running', claimed_by = ?, updated_at = ?
             WHERE id = ? AND status = 'pending'"
        )
        .bind(claimed_by).bind(&now).bind(job_id)
        .execute(&mut conn)
        .await
    })?;
    Ok(rows.rows_affected() == 1)
}

pub fn update_job_status(ship_dir: &Path, job_id: &str, status: &str) -> Result<()> {
    let mut conn = open_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query("UPDATE job SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status).bind(&now).bind(job_id)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

pub fn get_job(ship_dir: &Path, job_id: &str) -> Result<Option<Job>> {
    let mut conn = open_db(ship_dir)?;
    let row = block_on(async {
        sqlx::query(&format!("SELECT {J_COLS} FROM job WHERE id = ?"))
            .bind(job_id)
            .fetch_optional(&mut conn)
            .await
    })?;
    Ok(row.map(|r| row_to_job(&r)))
}

pub fn list_jobs(
    ship_dir: &Path,
    branch: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<Job>> {
    let mut conn = open_db(ship_dir)?;
    let rows = match (branch, status) {
        (Some(b), Some(s)) => block_on(async {
            sqlx::query(&format!(
                "SELECT {J_COLS} FROM job WHERE branch = ? AND status = ? ORDER BY created_at DESC"
            ))
            .bind(b).bind(s)
            .fetch_all(&mut conn)
            .await
        })?,
        (Some(b), None) => block_on(async {
            sqlx::query(&format!(
                "SELECT {J_COLS} FROM job WHERE branch = ? ORDER BY created_at DESC"
            ))
            .bind(b)
            .fetch_all(&mut conn)
            .await
        })?,
        (None, Some(s)) => block_on(async {
            sqlx::query(&format!(
                "SELECT {J_COLS} FROM job WHERE status = ? ORDER BY created_at DESC"
            ))
            .bind(s)
            .fetch_all(&mut conn)
            .await
        })?,
        (None, None) => block_on(async {
            sqlx::query(&format!(
                "SELECT {J_COLS} FROM job ORDER BY created_at DESC"
            ))
            .fetch_all(&mut conn)
            .await
        })?,
    };
    Ok(rows.iter().map(row_to_job).collect())
}

pub fn append_log(
    ship_dir: &Path,
    message: &str,
    job_id: Option<&str>,
    branch: Option<&str>,
    actor: Option<&str>,
) -> Result<()> {
    let mut conn = open_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO job_log (job_id, branch, message, actor, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(job_id).bind(branch).bind(message).bind(actor).bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn list_logs(
    ship_dir: &Path,
    branch: Option<&str>,
    job_id: Option<&str>,
    limit: Option<u32>,
) -> Result<Vec<JobLogEntry>> {
    let mut conn = open_db(ship_dir)?;
    let lim = limit.unwrap_or(100);
    let rows = match (branch, job_id) {
        (Some(b), Some(j)) => block_on(async {
            sqlx::query(&format!(
                "SELECT {L_COLS} FROM job_log WHERE branch = ? AND job_id = ? ORDER BY created_at DESC LIMIT ?"
            ))
            .bind(b).bind(j).bind(lim)
            .fetch_all(&mut conn)
            .await
        })?,
        (Some(b), None) => block_on(async {
            sqlx::query(&format!(
                "SELECT {L_COLS} FROM job_log WHERE branch = ? ORDER BY created_at DESC LIMIT ?"
            ))
            .bind(b).bind(lim)
            .fetch_all(&mut conn)
            .await
        })?,
        (None, Some(j)) => block_on(async {
            sqlx::query(&format!(
                "SELECT {L_COLS} FROM job_log WHERE job_id = ? ORDER BY created_at DESC LIMIT ?"
            ))
            .bind(j).bind(lim)
            .fetch_all(&mut conn)
            .await
        })?,
        (None, None) => block_on(async {
            sqlx::query(&format!(
                "SELECT {L_COLS} FROM job_log ORDER BY created_at DESC LIMIT ?"
            ))
            .bind(lim)
            .fetch_all(&mut conn)
            .await
        })?,
    };
    Ok(rows.iter().map(row_to_log).collect())
}

fn row_to_job(row: &sqlx::sqlite::SqliteRow) -> Job {
    let payload_str: String = row.get(4);
    Job {
        id: row.get(0),
        kind: row.get(1),
        status: row.get(2),
        branch: row.get(3),
        payload: serde_json::from_str(&payload_str).unwrap_or_default(),
        created_by: row.get(5),
        claimed_by: row.get(6),
        created_at: row.get(7),
        updated_at: row.get(8),
    }
}

fn row_to_log(row: &sqlx::sqlite::SqliteRow) -> JobLogEntry {
    JobLogEntry {
        id: row.get(0),
        job_id: row.get(1),
        branch: row.get(2),
        message: row.get(3),
        actor: row.get(4),
        created_at: row.get(5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ensure_db;
    use crate::project::init_project;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db(&ship_dir).unwrap();
        (tmp, ship_dir)
    }

    #[test]
    fn test_create_and_get_job() {
        let (_tmp, ship_dir) = setup();
        let job = create_job(&ship_dir, "compile", Some("feat/x"), None, Some("agent-1")).unwrap();
        assert_eq!(job.status, "pending");
        let got = get_job(&ship_dir, &job.id).unwrap().unwrap();
        assert_eq!(got.kind, "compile");
        assert_eq!(got.branch, Some("feat/x".to_string()));
    }

    #[test]
    fn test_update_job_status() {
        let (_tmp, ship_dir) = setup();
        let job = create_job(&ship_dir, "sync", None, None, None).unwrap();
        update_job_status(&ship_dir, &job.id, "running").unwrap();
        let got = get_job(&ship_dir, &job.id).unwrap().unwrap();
        assert_eq!(got.status, "running");
    }

    #[test]
    fn test_list_jobs_filtered() {
        let (_tmp, ship_dir) = setup();
        create_job(&ship_dir, "compile", Some("main"), None, None).unwrap();
        create_job(&ship_dir, "sync", Some("feat/a"), None, None).unwrap();
        let all = list_jobs(&ship_dir, None, None).unwrap();
        assert_eq!(all.len(), 2);
        let main_jobs = list_jobs(&ship_dir, Some("main"), None).unwrap();
        assert_eq!(main_jobs.len(), 1);
        let pending = list_jobs(&ship_dir, None, Some("pending")).unwrap();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_append_and_list_logs() {
        let (_tmp, ship_dir) = setup();
        append_log(&ship_dir, "starting compile", None, Some("feat/x"), Some("agent-1")).unwrap();
        append_log(&ship_dir, "done", None, Some("feat/x"), Some("agent-1")).unwrap();
        append_log(&ship_dir, "unrelated", None, Some("main"), None).unwrap();
        let logs = list_logs(&ship_dir, Some("feat/x"), None, None).unwrap();
        assert_eq!(logs.len(), 2);
        let all = list_logs(&ship_dir, None, None, None).unwrap();
        assert_eq!(all.len(), 3);
    }

    // ── Priority 2 gap tests ──────────────────────────────────────────────────

    /// Two-step status transition: pending → running → done.
    #[test]
    fn test_job_status_transitions_pending_running_done() {
        let (_tmp, ship_dir) = setup();
        let job = create_job(&ship_dir, "build", Some("feat/transitions"), None, None).unwrap();
        assert_eq!(job.status, "pending");

        update_job_status(&ship_dir, &job.id, "running").unwrap();
        let running = get_job(&ship_dir, &job.id).unwrap().unwrap();
        assert_eq!(running.status, "running");

        update_job_status(&ship_dir, &job.id, "done").unwrap();
        let done = get_job(&ship_dir, &job.id).unwrap().unwrap();
        assert_eq!(done.status, "done");
    }

    /// list_jobs with both branch AND status filter combined returns only exact matches.
    #[test]
    fn test_list_jobs_branch_and_status_combined() {
        let (_tmp, ship_dir) = setup();
        let j1 = create_job(&ship_dir, "compile", Some("feat/combo"), None, None).unwrap();
        let j2 = create_job(&ship_dir, "sync",    Some("feat/combo"), None, None).unwrap();
        let _j3 = create_job(&ship_dir, "compile", Some("main"), None, None).unwrap();

        // Advance j2 to running so we have a mix of statuses on the same branch.
        update_job_status(&ship_dir, &j2.id, "running").unwrap();

        // Branch=feat/combo + status=pending → only j1
        let pending_combo = list_jobs(&ship_dir, Some("feat/combo"), Some("pending")).unwrap();
        assert_eq!(pending_combo.len(), 1);
        assert_eq!(pending_combo[0].id, j1.id);

        // Branch=feat/combo + status=running → only j2
        let running_combo = list_jobs(&ship_dir, Some("feat/combo"), Some("running")).unwrap();
        assert_eq!(running_combo.len(), 1);
        assert_eq!(running_combo[0].id, j2.id);

        // Branch=main + status=pending → only j3
        let main_pending = list_jobs(&ship_dir, Some("main"), Some("pending")).unwrap();
        assert_eq!(main_pending.len(), 1);

        // Branch=feat/combo + status=done → nothing
        let done_combo = list_jobs(&ship_dir, Some("feat/combo"), Some("done")).unwrap();
        assert!(done_combo.is_empty());
    }

    /// append_log with job_id=None (branch-only log entry) is stored and retrievable.
    #[test]
    fn test_append_log_null_job_id() {
        let (_tmp, ship_dir) = setup();
        // No job created — log is purely branch-scoped.
        append_log(&ship_dir, "branch-only event", None, Some("feat/no-job"), Some("watcher")).unwrap();
        let logs = list_logs(&ship_dir, Some("feat/no-job"), None, None).unwrap();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].job_id.is_none());
        assert_eq!(logs[0].message, "branch-only event");
        assert_eq!(logs[0].actor, Some("watcher".to_string()));
    }

    /// list_logs respects the limit parameter — returns at most `limit` entries.
    #[test]
    fn test_list_logs_limit_respected() {
        let (_tmp, ship_dir) = setup();
        for i in 0..10 {
            append_log(&ship_dir, &format!("msg-{i}"), None, Some("feat/limit"), None).unwrap();
        }
        let all = list_logs(&ship_dir, Some("feat/limit"), None, None).unwrap();
        assert_eq!(all.len(), 10);

        let limited = list_logs(&ship_dir, Some("feat/limit"), None, Some(3)).unwrap();
        assert_eq!(limited.len(), 3);

        // limit=1 returns exactly one entry
        let one = list_logs(&ship_dir, Some("feat/limit"), None, Some(1)).unwrap();
        assert_eq!(one.len(), 1);
    }
}
