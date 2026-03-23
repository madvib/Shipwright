//! Job queue for agent coordination.
//! Written by `ship agent job` commands; referenced from skills.

pub mod file_ownership;
pub use file_ownership::{claim_file, get_file_owner};

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;

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
    pub touched_files: Vec<String>,
    pub assigned_to: Option<String>,
    pub priority: i32,
    pub blocked_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub file_scope: Vec<String>,
    pub capability_id: Option<String>,
}

/// Fields that can be patched in an [`update_job`] call.
/// `None` means "keep current value".
#[derive(Debug, Default)]
pub struct JobPatch {
    pub status: Option<String>,
    pub assigned_to: Option<String>,
    pub priority: Option<i32>,
    pub blocked_by: Option<String>,
    pub touched_files: Option<Vec<String>>,
    pub file_scope: Option<Vec<String>>,
    pub capability_id: Option<String>,
}

const J_COLS: &str = concat!(
    "id, kind, status, branch, payload_json, created_by, claimed_by,",
    " touched_files, assigned_to, priority, blocked_by, created_at, updated_at, file_scope, capability_id"
);

#[allow(clippy::too_many_arguments)]
pub fn create_job(
    kind: &str,
    branch: Option<&str>,
    payload: Option<serde_json::Value>,
    created_by: Option<&str>,
    assigned_to: Option<&str>,
    priority: i32,
    blocked_by: Option<&str>,
    touched_files: Vec<String>,
    file_scope: Vec<String>,
) -> Result<Job> {
    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    let id = gen_nanoid();
    let payload = payload.unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    let payload_str = serde_json::to_string(&payload)?;
    let files_str = serde_json::to_string(&touched_files)?;
    let scope_str = serde_json::to_string(&file_scope)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO job \
             (id, kind, status, branch, payload_json, created_by, \
              assigned_to, priority, blocked_by, touched_files, created_at, updated_at, file_scope) \
             VALUES (?, ?, 'pending', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id).bind(kind).bind(branch).bind(&payload_str).bind(created_by)
        .bind(assigned_to).bind(priority).bind(blocked_by).bind(&files_str)
        .bind(&now).bind(&now).bind(&scope_str)
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
        touched_files,
        assigned_to: assigned_to.map(str::to_string),
        priority,
        blocked_by: blocked_by.map(str::to_string),
        created_at: now.clone(),
        updated_at: now,
        file_scope,
        capability_id: None,
    })
}

/// Patch one or more mutable job fields atomically.
/// Reads current state, merges the patch, writes back, and releases file
/// claims when the job reaches a terminal status ("complete", "failed", "done").
pub fn update_job(job_id: &str, patch: JobPatch) -> Result<()> {
    let current =
        get_job(job_id)?.ok_or_else(|| anyhow::anyhow!("job {job_id} not found"))?;
    let now = Utc::now().to_rfc3339();
    let new_status = patch
        .status
        .as_deref()
        .unwrap_or(&current.status)
        .to_string();
    let new_assigned = patch.assigned_to.or(current.assigned_to);
    let new_priority = patch.priority.unwrap_or(current.priority);
    let new_blocked = patch.blocked_by.or(current.blocked_by);
    let new_files = patch.touched_files.unwrap_or(current.touched_files);
    let new_scope = patch.file_scope.unwrap_or(current.file_scope);
    let new_cap = patch.capability_id.or(current.capability_id);
    let files_str = serde_json::to_string(&new_files)?;
    let scope_str = serde_json::to_string(&new_scope)?;
    let mut conn = open_db()?;
    block_on(async {
        sqlx::query(
            "UPDATE job SET status=?, assigned_to=?, priority=?, blocked_by=?, \
             touched_files=?, file_scope=?, capability_id=?, updated_at=? WHERE id=?",
        )
        .bind(&new_status)
        .bind(&new_assigned)
        .bind(new_priority)
        .bind(&new_blocked)
        .bind(&files_str)
        .bind(&scope_str)
        .bind(&new_cap)
        .bind(&now)
        .bind(job_id)
        .execute(&mut conn)
        .await
    })?;
    if matches!(new_status.as_str(), "complete" | "failed" | "done") {
        file_ownership::release_job_files(job_id)?;
    }
    Ok(())
}

/// Append a single file path to the job's `touched_files` list (deduplicates).
pub fn append_touched_file(job_id: &str, path: &str) -> Result<()> {
    let current =
        get_job(job_id)?.ok_or_else(|| anyhow::anyhow!("job {job_id} not found"))?;
    let mut files = current.touched_files;
    let path_owned = path.to_string();
    if !files.contains(&path_owned) {
        files.push(path_owned);
    }
    update_job(
        job_id,
        JobPatch {
            touched_files: Some(files),
            ..Default::default()
        },
    )
}

/// Atomically claim a pending job. Returns false if already claimed — prevents
/// double-claiming when multiple commanders share the same queue (e.g. Claude + Codex,
/// or two machines syncing the same platform.db).
pub fn claim_job(job_id: &str, claimed_by: &str) -> Result<bool> {
    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    let rows = block_on(async {
        sqlx::query(
            "UPDATE job SET status = 'running', claimed_by = ?, updated_at = ?
             WHERE id = ? AND status = 'pending'",
        )
        .bind(claimed_by)
        .bind(&now)
        .bind(job_id)
        .execute(&mut conn)
        .await
    })?;
    Ok(rows.rows_affected() == 1)
}

pub fn update_job_status(job_id: &str, status: &str) -> Result<()> {
    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query("UPDATE job SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(job_id)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

pub fn get_job(job_id: &str) -> Result<Option<Job>> {
    let mut conn = open_db()?;
    let row = block_on(async {
        sqlx::query(&format!("SELECT {J_COLS} FROM job WHERE id = ?"))
            .bind(job_id)
            .fetch_optional(&mut conn)
            .await
    })?;
    Ok(row.map(|r| row_to_job(&r)))
}

pub fn list_jobs(branch: Option<&str>, status: Option<&str>) -> Result<Vec<Job>> {
    let mut conn = open_db()?;
    let rows = match (branch, status) {
        (Some(b), Some(s)) => block_on(async {
            sqlx::query(&format!(
                "SELECT {J_COLS} FROM job WHERE branch = ? AND status = ? ORDER BY created_at DESC"
            ))
            .bind(b)
            .bind(s)
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

fn row_to_job(row: &sqlx::sqlite::SqliteRow) -> Job {
    // Column order matches J_COLS:
    // 0:id 1:kind 2:status 3:branch 4:payload_json 5:created_by 6:claimed_by
    // 7:touched_files 8:assigned_to 9:priority 10:blocked_by 11:created_at 12:updated_at
    // 13:file_scope 14:capability_id
    let payload_str: String = row.get(4);
    let files_str: String = row.get(7);
    let scope_str: String = row.get(13);
    Job {
        id: row.get(0),
        kind: row.get(1),
        status: row.get(2),
        branch: row.get(3),
        payload: serde_json::from_str(&payload_str).unwrap_or_default(),
        created_by: row.get(5),
        claimed_by: row.get(6),
        touched_files: serde_json::from_str(&files_str).unwrap_or_default(),
        assigned_to: row.get(8),
        priority: row.get(9),
        blocked_by: row.get(10),
        created_at: row.get(11),
        updated_at: row.get(12),
        file_scope: serde_json::from_str(&scope_str).unwrap_or_default(),
        capability_id: row.get(14),
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
        ensure_db().unwrap();
        (tmp, ship_dir)
    }

    fn mkjob(kind: &str, branch: Option<&str>) -> Job {
        create_job(
            kind,
            branch,
            None,
            None,
            None,
            0,
            None,
            vec![],
            vec![],
        )
        .unwrap()
    }

    #[test]
    fn test_create_and_get_job() {
        let (_tmp, _ship_dir) = setup();
        let job = create_job(
            "compile",
            Some("feat/x"),
            None,
            Some("agent-1"),
            None,
            0,
            None,
            vec![],
            vec![],
        )
        .unwrap();
        assert_eq!(job.status, "pending");
        let got = get_job(&job.id).unwrap().unwrap();
        assert_eq!(got.kind, "compile");
        assert_eq!(got.branch, Some("feat/x".to_string()));
    }

    #[test]
    fn test_create_job_new_fields() {
        let (_tmp, _ship_dir) = setup();
        let job = create_job(
            "build",
            Some("feat/fields"),
            None,
            None,
            Some("agent-1"),
            5,
            Some("blocker-id"),
            vec!["src/lib.rs".to_string()],
            vec![],
        )
        .unwrap();
        assert_eq!(job.assigned_to, Some("agent-1".to_string()));
        assert_eq!(job.priority, 5);
        assert_eq!(job.blocked_by, Some("blocker-id".to_string()));
        assert_eq!(job.touched_files, vec!["src/lib.rs".to_string()]);
        let got = get_job(&job.id).unwrap().unwrap();
        assert_eq!(got.priority, 5);
        assert_eq!(got.touched_files, vec!["src/lib.rs".to_string()]);
    }

    #[test]
    fn test_update_job_status() {
        let (_tmp, _ship_dir) = setup();
        let job = mkjob("sync", None);
        update_job_status(&job.id, "running").unwrap();
        let got = get_job(&job.id).unwrap().unwrap();
        assert_eq!(got.status, "running");
    }

    #[test]
    fn test_list_jobs_filtered() {
        let (_tmp, _ship_dir) = setup();
        mkjob("compile", Some("main"));
        mkjob("sync", Some("feat/a"));
        let all = list_jobs(None, None).unwrap();
        assert_eq!(all.len(), 2);
        let main_jobs = list_jobs(Some("main"), None).unwrap();
        assert_eq!(main_jobs.len(), 1);
        let pending = list_jobs(None, Some("pending")).unwrap();
        assert_eq!(pending.len(), 2);
    }

    // ── Priority 2 gap tests ──────────────────────────────────────────────────

    /// Two-step status transition: pending → running → done.
    #[test]
    fn test_job_status_transitions_pending_running_done() {
        let (_tmp, _ship_dir) = setup();
        let job = mkjob("build", Some("feat/transitions"));
        assert_eq!(job.status, "pending");

        update_job_status(&job.id, "running").unwrap();
        let running = get_job(&job.id).unwrap().unwrap();
        assert_eq!(running.status, "running");

        update_job_status(&job.id, "done").unwrap();
        let done = get_job(&job.id).unwrap().unwrap();
        assert_eq!(done.status, "done");
    }

    /// list_jobs with both branch AND status filter combined returns only exact matches.
    #[test]
    fn test_list_jobs_branch_and_status_combined() {
        let (_tmp, _ship_dir) = setup();
        let j1 = mkjob("compile", Some("feat/combo"));
        let j2 = mkjob("sync", Some("feat/combo"));
        let _j3 = mkjob("compile", Some("main"));

        // Advance j2 to running so we have a mix of statuses on the same branch.
        update_job_status(&j2.id, "running").unwrap();

        // Branch=feat/combo + status=pending → only j1
        let pending_combo = list_jobs(Some("feat/combo"), Some("pending")).unwrap();
        assert_eq!(pending_combo.len(), 1);
        assert_eq!(pending_combo[0].id, j1.id);

        // Branch=feat/combo + status=running → only j2
        let running_combo = list_jobs(Some("feat/combo"), Some("running")).unwrap();
        assert_eq!(running_combo.len(), 1);
        assert_eq!(running_combo[0].id, j2.id);

        // Branch=main + status=pending → only j3
        let main_pending = list_jobs(Some("main"), Some("pending")).unwrap();
        assert_eq!(main_pending.len(), 1);

        // Branch=feat/combo + status=done → nothing
        let done_combo = list_jobs(Some("feat/combo"), Some("done")).unwrap();
        assert!(done_combo.is_empty());
    }

    /// update_job patches multiple fields in a single call.
    #[test]
    fn test_update_job_patches_fields() {
        let (_tmp, _ship_dir) = setup();
        let job = mkjob("patch-test", None);
        update_job(
            &job.id,
            JobPatch {
                status: Some("running".to_string()),
                assigned_to: Some("agent-42".to_string()),
                priority: Some(10),
                blocked_by: None,
                touched_files: Some(vec!["a.rs".to_string()]),
                file_scope: None,
                capability_id: None,
            },
        )
        .unwrap();
        let got = get_job(&job.id).unwrap().unwrap();
        assert_eq!(got.status, "running");
        assert_eq!(got.assigned_to, Some("agent-42".to_string()));
        assert_eq!(got.priority, 10);
        assert_eq!(got.touched_files, vec!["a.rs".to_string()]);
    }

    #[test]
    fn test_create_job_with_file_scope() {
        let (_tmp, _ship_dir) = setup();
        let scope = vec!["crates/core/".to_string(), "apps/mcp/".to_string()];
        let job = create_job(
            "build",
            None,
            None,
            None,
            None,
            0,
            None,
            vec![],
            scope.clone(),
        )
        .unwrap();
        assert_eq!(job.file_scope, scope);
        let got = get_job(&job.id).unwrap().unwrap();
        assert_eq!(got.file_scope, scope);
    }

    #[test]
    fn test_capability_id_round_trip() {
        let (_tmp, _ship_dir) = setup();
        // FK is enforced — create a real target + capability first.
        let t = crate::db::targets::create_target("surface", "test", None, None, None)
            .unwrap();
        let c = crate::db::targets::create_capability(&t.id, "test cap", None).unwrap();

        let job = mkjob("compile", None);
        assert!(job.capability_id.is_none());

        update_job(
            &job.id,
            JobPatch {
                capability_id: Some(c.id.clone()),
                ..Default::default()
            },
        )
        .unwrap();

        let got = get_job(&job.id).unwrap().unwrap();
        assert_eq!(got.capability_id, Some(c.id));
    }

    #[test]
    fn test_append_touched_file_deduplicates() {
        let (_tmp, _ship_dir) = setup();
        let job = mkjob("touch-test", None);
        assert!(job.touched_files.is_empty());

        append_touched_file(&job.id, "src/lib.rs").unwrap();
        append_touched_file(&job.id, "src/main.rs").unwrap();
        // Duplicate — should not add again.
        append_touched_file(&job.id, "src/lib.rs").unwrap();

        let got = get_job(&job.id).unwrap().unwrap();
        assert_eq!(
            got.touched_files,
            vec!["src/lib.rs".to_string(), "src/main.rs".to_string()]
        );
    }
}
