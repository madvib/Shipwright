//! Higher-level Workspace and WorkspaceSession CRUD.
//!
//! These operate on the unified workspace table (branch PK).
//! Used by MCP tools and the studio CLI for clean struct-based access.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use std::path::Path;

use crate::db::{block_on, open_db};
use crate::gen_nanoid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id: String,
    pub branch: String,
    pub worktree_path: Option<String>,
    pub workspace_type: String,
    pub status: String,
    pub active_preset: Option<String>,
    pub providers: Vec<String>,
    pub skills: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub plugins: Vec<String>,
    pub compiled_at: Option<String>,
    pub compile_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSession {
    pub id: String,
    pub workspace_id: String,
    pub branch: String,
    pub status: String,
    pub preset_id: Option<String>,
    pub primary_provider: Option<String>,
    pub goal: Option<String>,
    pub summary: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

const W_COLS: &str =
    "COALESCE(id, branch), branch, worktree_path, workspace_type, status, active_preset,
     providers_json, skills_json, mcp_servers_json, plugins_json,
     compiled_at, compile_error, COALESCE(created_at, resolved_at, ''), COALESCE(updated_at, resolved_at, '')";

const S_COLS: &str = "id, workspace_id, workspace_branch, status, preset_id, primary_provider,
     goal, summary, started_at, ended_at, created_at, updated_at";

pub fn upsert_workspace(ship_dir: &Path, w: &Workspace) -> Result<()> {
    let mut conn = open_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    let providers = serde_json::to_string(&w.providers)?;
    let skills = serde_json::to_string(&w.skills)?;
    let mcp = serde_json::to_string(&w.mcp_servers)?;
    let plugins = serde_json::to_string(&w.plugins)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO workspace
               (branch, id, worktree_path, workspace_type, status, active_preset,
                providers_json, skills_json, mcp_servers_json, plugins_json,
                compiled_at, compile_error, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(branch) DO UPDATE SET
               id               = excluded.id,
               worktree_path    = excluded.worktree_path,
               workspace_type   = excluded.workspace_type,
               status           = excluded.status,
               active_preset    = excluded.active_preset,
               providers_json   = excluded.providers_json,
               skills_json      = excluded.skills_json,
               mcp_servers_json = excluded.mcp_servers_json,
               plugins_json     = excluded.plugins_json,
               compiled_at      = excluded.compiled_at,
               compile_error    = excluded.compile_error,
               updated_at       = excluded.updated_at",
        )
        .bind(&w.branch)
        .bind(&w.id)
        .bind(&w.worktree_path)
        .bind(&w.workspace_type)
        .bind(&w.status)
        .bind(&w.active_preset)
        .bind(&providers)
        .bind(&skills)
        .bind(&mcp)
        .bind(&plugins)
        .bind(&w.compiled_at)
        .bind(&w.compile_error)
        .bind(&w.created_at)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn get_workspace(ship_dir: &Path, id: &str) -> Result<Option<Workspace>> {
    let mut conn = open_db(ship_dir)?;
    let row = block_on(async {
        sqlx::query(&format!(
            "SELECT {W_COLS} FROM workspace WHERE id = ? OR branch = ?"
        ))
        .bind(id)
        .bind(id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.map(|r| row_to_workspace(&r)))
}

pub fn get_workspace_by_branch(ship_dir: &Path, branch: &str) -> Result<Option<Workspace>> {
    let mut conn = open_db(ship_dir)?;
    let row = block_on(async {
        sqlx::query(&format!("SELECT {W_COLS} FROM workspace WHERE branch = ?"))
            .bind(branch)
            .fetch_optional(&mut conn)
            .await
    })?;
    Ok(row.map(|r| row_to_workspace(&r)))
}

pub fn list_workspaces(ship_dir: &Path) -> Result<Vec<Workspace>> {
    let mut conn = open_db(ship_dir)?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {W_COLS} FROM workspace ORDER BY COALESCE(created_at, resolved_at) DESC"
        ))
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows.iter().map(row_to_workspace).collect())
}

pub fn start_session(
    ship_dir: &Path,
    workspace_id: &str,
    branch: &str,
    preset_id: Option<&str>,
    goal: Option<&str>,
) -> Result<WorkspaceSession> {
    let mut conn = open_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    let id = gen_nanoid();
    block_on(async {
        sqlx::query(
            "INSERT INTO workspace_session
               (id, workspace_id, workspace_branch, status, preset_id, goal,
                started_at, created_at, updated_at)
             VALUES (?, ?, ?, 'active', ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(workspace_id)
        .bind(branch)
        .bind(preset_id)
        .bind(goal)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(WorkspaceSession {
        id,
        workspace_id: workspace_id.to_string(),
        branch: branch.to_string(),
        status: "active".to_string(),
        preset_id: preset_id.map(str::to_string),
        primary_provider: None,
        goal: goal.map(str::to_string),
        summary: None,
        started_at: now.clone(),
        ended_at: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn end_session(ship_dir: &Path, session_id: &str, summary: Option<&str>) -> Result<()> {
    let mut conn = open_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace_session
             SET status = 'ended', ended_at = ?, summary = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&now)
        .bind(summary)
        .bind(&now)
        .bind(session_id)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn get_active_session(ship_dir: &Path, workspace_id: &str) -> Result<Option<WorkspaceSession>> {
    let mut conn = open_db(ship_dir)?;
    let row = block_on(async {
        sqlx::query(&format!(
            "SELECT {S_COLS} FROM workspace_session
             WHERE workspace_id = ? AND status = 'active'
             ORDER BY started_at DESC LIMIT 1"
        ))
        .bind(workspace_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.map(|r| row_to_session(&r)))
}

fn pj(s: String) -> Vec<String> {
    serde_json::from_str(&s).unwrap_or_default()
}

fn row_to_workspace(row: &sqlx::sqlite::SqliteRow) -> Workspace {
    Workspace {
        id: row.get(0),
        branch: row.get(1),
        worktree_path: row.get(2),
        workspace_type: row.get(3),
        status: row.get(4),
        active_preset: row.get(5),
        providers: pj(row.get(6)),
        skills: pj(row.get(7)),
        mcp_servers: pj(row.get(8)),
        plugins: pj(row.get(9)),
        compiled_at: row.get(10),
        compile_error: row.get(11),
        created_at: row.get(12),
        updated_at: row.get(13),
    }
}

fn row_to_session(row: &sqlx::sqlite::SqliteRow) -> WorkspaceSession {
    WorkspaceSession {
        id: row.get(0),
        workspace_id: row.get(1),
        branch: row.get(2),
        status: row.get(3),
        preset_id: row.get(4),
        primary_provider: row.get(5),
        goal: row.get(6),
        summary: row.get(7),
        started_at: row.get(8),
        ended_at: row.get(9),
        created_at: row.get(10),
        updated_at: row.get(11),
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

    fn sample(id: &str, branch: &str) -> Workspace {
        let now = Utc::now().to_rfc3339();
        Workspace {
            id: id.to_string(),
            branch: branch.to_string(),
            worktree_path: None,
            workspace_type: "declarative".to_string(),
            status: "active".to_string(),
            active_preset: Some("default".to_string()),
            providers: vec!["claude".to_string()],
            skills: vec![],
            mcp_servers: vec![],
            plugins: vec![],
            compiled_at: None,
            compile_error: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    #[test]
    fn test_upsert_and_get_workspace() {
        let (_tmp, ship_dir) = setup();
        let w = sample("ws-001", "feat/test");
        upsert_workspace(&ship_dir, &w).unwrap();
        let got = get_workspace(&ship_dir, "ws-001").unwrap().unwrap();
        assert_eq!(got.branch, "feat/test");
        assert_eq!(got.workspace_type, "declarative");
        assert_eq!(got.active_preset, Some("default".to_string()));
    }

    #[test]
    fn test_upsert_workspace_updates_on_conflict() {
        let (_tmp, ship_dir) = setup();
        let mut w = sample("ws-002", "main");
        upsert_workspace(&ship_dir, &w).unwrap();
        w.active_preset = Some("orchestrator".to_string());
        upsert_workspace(&ship_dir, &w).unwrap();
        let got = get_workspace(&ship_dir, "ws-002").unwrap().unwrap();
        assert_eq!(got.active_preset, Some("orchestrator".to_string()));
    }

    #[test]
    fn test_get_workspace_by_branch() {
        let (_tmp, ship_dir) = setup();
        upsert_workspace(&ship_dir, &sample("ws-003", "feat/branch")).unwrap();
        let got = get_workspace_by_branch(&ship_dir, "feat/branch")
            .unwrap()
            .unwrap();
        assert_eq!(got.id, "ws-003");
        assert!(
            get_workspace_by_branch(&ship_dir, "nonexistent")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn test_list_workspaces() {
        let (_tmp, ship_dir) = setup();
        upsert_workspace(&ship_dir, &sample("ws-a", "branch-a")).unwrap();
        upsert_workspace(&ship_dir, &sample("ws-b", "branch-b")).unwrap();
        assert_eq!(list_workspaces(&ship_dir).unwrap().len(), 2);
    }

    #[test]
    fn test_session_lifecycle() {
        let (_tmp, ship_dir) = setup();
        upsert_workspace(&ship_dir, &sample("ws-s1", "main")).unwrap();
        let sess =
            start_session(&ship_dir, "ws-s1", "main", Some("cli-lane"), Some("build")).unwrap();
        assert_eq!(sess.status, "active");
        let active = get_active_session(&ship_dir, "ws-s1").unwrap().unwrap();
        assert_eq!(active.id, sess.id);
        end_session(&ship_dir, &sess.id, Some("done")).unwrap();
        assert!(get_active_session(&ship_dir, "ws-s1").unwrap().is_none());
    }

    #[test]
    fn test_workspace_with_worktree_path() {
        let (_tmp, ship_dir) = setup();
        let mut w = sample("ws-wt1", "feat/worktree");
        w.worktree_path = Some("/tmp/worktrees/feat-worktree".to_string());
        upsert_workspace(&ship_dir, &w).unwrap();
        let got = get_workspace(&ship_dir, "ws-wt1").unwrap().unwrap();
        assert_eq!(
            got.worktree_path,
            Some("/tmp/worktrees/feat-worktree".to_string())
        );
    }

    #[test]
    fn test_list_workspaces_status_and_kind_visible() {
        let (_tmp, ship_dir) = setup();
        let mut w_active = sample("ws-filter-a", "branch-active");
        w_active.status = "active".to_string();
        w_active.workspace_type = "declarative".to_string();
        upsert_workspace(&ship_dir, &w_active).unwrap();

        let mut w_completed = sample("ws-filter-b", "branch-completed");
        w_completed.status = "completed".to_string();
        w_completed.workspace_type = "imperative".to_string();
        upsert_workspace(&ship_dir, &w_completed).unwrap();

        let all = list_workspaces(&ship_dir).unwrap();
        assert_eq!(all.len(), 2);

        let active_ones: Vec<_> = all.iter().filter(|w| w.status == "active").collect();
        assert_eq!(active_ones.len(), 1);
        assert_eq!(active_ones[0].id, "ws-filter-a");
    }

    #[test]
    fn test_workspace_full_lifecycle() {
        let (_tmp, ship_dir) = setup();
        let w = sample("ws-lc1", "feat/lifecycle");
        upsert_workspace(&ship_dir, &w).unwrap();

        let all = list_workspaces(&ship_dir).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, "ws-lc1");

        let sess = start_session(
            &ship_dir,
            "ws-lc1",
            "feat/lifecycle",
            Some("cli-lane"),
            Some("lifecycle test"),
        )
        .unwrap();
        assert_eq!(sess.workspace_id, "ws-lc1");
        assert_eq!(sess.goal, Some("lifecycle test".to_string()));
        assert!(sess.ended_at.is_none());

        let active = get_active_session(&ship_dir, "ws-lc1").unwrap().unwrap();
        assert_eq!(active.id, sess.id);
        assert_eq!(active.preset_id, Some("cli-lane".to_string()));

        end_session(&ship_dir, &sess.id, Some("all done")).unwrap();
        assert!(get_active_session(&ship_dir, "ws-lc1").unwrap().is_none());

        let mut w_done = w.clone();
        w_done.status = "completed".to_string();
        upsert_workspace(&ship_dir, &w_done).unwrap();
        let got = get_workspace(&ship_dir, "ws-lc1").unwrap().unwrap();
        assert_eq!(got.status, "completed");
    }

    #[test]
    fn test_get_workspace_missing_returns_none() {
        let (_tmp, ship_dir) = setup();
        let got = get_workspace(&ship_dir, "no-such-id").unwrap();
        assert!(got.is_none());
    }
}
