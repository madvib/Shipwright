//! Branch config — tracks compiled preset state per branch.
//! Written by `ship use`, read by the post-checkout hook.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;

use crate::db::{block_on, open_db};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchConfig {
    pub branch: String,
    pub preset_id: String,
    pub workspace_id: Option<String>,
    pub plugins: Vec<String>,
    pub compiled_at: String,
    pub updated_at: String,
}

const COLS: &str = "branch, preset_id, workspace_id, plugins_json, compiled_at, updated_at";

pub fn upsert_branch_config(cfg: &BranchConfig) -> Result<()> {
    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    let plugins = serde_json::to_string(&cfg.plugins)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO branch_config
               (branch, preset_id, workspace_id, plugins_json, compiled_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(branch) DO UPDATE SET
               preset_id    = excluded.preset_id,
               workspace_id = excluded.workspace_id,
               plugins_json = excluded.plugins_json,
               compiled_at  = excluded.compiled_at,
               updated_at   = excluded.updated_at",
        )
        .bind(&cfg.branch)
        .bind(&cfg.preset_id)
        .bind(&cfg.workspace_id)
        .bind(&plugins)
        .bind(&cfg.compiled_at)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn get_branch_config(branch: &str) -> Result<Option<BranchConfig>> {
    let mut conn = open_db()?;
    let row = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM branch_config WHERE branch = ?"
        ))
        .bind(branch)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.map(|r| row_to_cfg(&r)))
}

pub fn list_branch_configs() -> Result<Vec<BranchConfig>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM branch_config ORDER BY updated_at DESC"
        ))
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows.iter().map(row_to_cfg).collect())
}

fn row_to_cfg(row: &sqlx::sqlite::SqliteRow) -> BranchConfig {
    BranchConfig {
        branch: row.get(0),
        preset_id: row.get(1),
        workspace_id: row.get(2),
        plugins: serde_json::from_str::<Vec<String>>(&row.get::<String, _>(3)).unwrap_or_default(),
        compiled_at: row.get(4),
        updated_at: row.get(5),
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

    fn sample(branch: &str, preset_id: &str) -> BranchConfig {
        let now = Utc::now().to_rfc3339();
        BranchConfig {
            branch: branch.to_string(),
            preset_id: preset_id.to_string(),
            workspace_id: None,
            plugins: vec!["superpowers@claude-plugins-official".to_string()],
            compiled_at: now.clone(),
            updated_at: now,
        }
    }

    #[test]
    fn test_upsert_and_get_branch_config() {
        let (_tmp, _ship_dir) = setup();
        upsert_branch_config(&sample("feat/cli-init", "cli-lane")).unwrap();
        let got = get_branch_config("feat/cli-init")
            .unwrap()
            .unwrap();
        assert_eq!(got.preset_id, "cli-lane");
        assert_eq!(got.plugins.len(), 1);
    }

    #[test]
    fn test_upsert_branch_config_overwrites() {
        let (_tmp, _ship_dir) = setup();
        upsert_branch_config(&sample("main", "default")).unwrap();
        let mut updated = sample("main", "orchestrator");
        updated.plugins = vec![];
        upsert_branch_config(&updated).unwrap();
        let got = get_branch_config("main").unwrap().unwrap();
        assert_eq!(got.preset_id, "orchestrator");
        assert!(got.plugins.is_empty());
    }

    #[test]
    fn test_get_branch_config_missing_returns_none() {
        let (_tmp, _ship_dir) = setup();
        assert!(
            get_branch_config("nonexistent")
                .unwrap()
                .is_none()
        );
    }
}
