use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use std::path::Path;

use crate::agents::config::WorkspaceAgentSettings;

use super::init::open_project_db;
use super::types::FeatureBranchLinks;
use super::util::{block_on, column_exists};

/// Look up which linked entity is associated with `branch`.
/// Returns `(link_type, link_id)` or `None`.
pub fn get_branch_link(ship_dir: &Path, branch: &str) -> Result<Option<(String, String)>> {
    let mut conn = open_project_db(ship_dir)?;
    let has_legacy_doc_columns = column_exists(&mut conn, "branch_context", "doc_type")?
        && column_exists(&mut conn, "branch_context", "doc_id")?;
    let sql = if has_legacy_doc_columns {
        "SELECT
           COALESCE(NULLIF(link_type, ''), doc_type),
           COALESCE(NULLIF(link_id, ''), doc_id)
         FROM branch_context
         WHERE branch = ?"
    } else {
        "SELECT link_type, link_id FROM branch_context WHERE branch = ?"
    };
    let row_opt = block_on(async {
        sqlx::query(sql)
            .bind(branch)
            .fetch_optional(&mut conn)
            .await
    })?;
    if let Some(row) = row_opt {
        Ok(Some((row.get(0), row.get(1))))
    } else {
        Ok(None)
    }
}

/// Record that `branch` is associated with `link_type` and entity id.
pub fn set_branch_link(
    ship_dir: &Path,
    branch: &str,
    link_type: &str,
    link_id: &str,
) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    let has_legacy_doc_columns = column_exists(&mut conn, "branch_context", "doc_type")?
        && column_exists(&mut conn, "branch_context", "doc_id")?;
    let now = Utc::now().to_rfc3339();
    if has_legacy_doc_columns {
        block_on(async {
            sqlx::query(
                "INSERT INTO branch_context
                   (branch, link_type, link_id, doc_type, doc_id, last_synced)
                 VALUES (?, ?, ?, ?, ?, ?)
                 ON CONFLICT(branch) DO UPDATE SET
                   link_type = excluded.link_type,
                   link_id = excluded.link_id,
                   doc_type = excluded.doc_type,
                   doc_id = excluded.doc_id,
                   last_synced = excluded.last_synced",
            )
            .bind(branch)
            .bind(link_type)
            .bind(link_id)
            .bind(link_type)
            .bind(link_id)
            .bind(&now)
            .execute(&mut conn)
            .await
        })?;
    } else {
        block_on(async {
            sqlx::query(
                "INSERT INTO branch_context (branch, link_type, link_id, last_synced)
                 VALUES (?, ?, ?, ?)
                 ON CONFLICT(branch) DO UPDATE SET
                   link_type = excluded.link_type,
                   link_id = excluded.link_id,
                   last_synced = excluded.last_synced",
            )
            .bind(branch)
            .bind(link_type)
            .bind(link_id)
            .bind(&now)
            .execute(&mut conn)
            .await
        })?;
    }
    Ok(())
}

/// Remove branch link mapping for `branch` when no entity is associated anymore.
pub fn clear_branch_link(ship_dir: &Path, branch: &str) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    block_on(async {
        sqlx::query("DELETE FROM branch_context WHERE branch = ?")
            .bind(branch)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

/// Legacy alias kept for compatibility with older call sites.
pub fn get_branch_doc(ship_dir: &Path, branch: &str) -> Result<Option<(String, String)>> {
    get_branch_link(ship_dir, branch)
}

/// Legacy alias kept for compatibility with older call sites.
pub fn set_branch_doc(
    ship_dir: &Path,
    branch: &str,
    doc_type: &str,
    doc_uuid: &str,
) -> Result<()> {
    set_branch_link(ship_dir, branch, doc_type, doc_uuid)
}

/// Legacy alias kept for compatibility with older call sites.
pub fn clear_branch_doc(ship_dir: &Path, branch: &str) -> Result<()> {
    clear_branch_link(ship_dir, branch)
}

/// Look up feature-linked target id used by workspace hydration.
/// Returns `target_id` when the feature exists.
pub fn get_feature_links(ship_dir: &Path, feature_id: &str) -> Result<Option<Option<String>>> {
    let mut conn = open_project_db(ship_dir)?;
    let row_opt = block_on(async {
        sqlx::query("SELECT active_target_id, release_id FROM feature WHERE id = ?")
            .bind(feature_id)
            .fetch_optional(&mut conn)
            .await
    })?;
    if let Some(row) = row_opt {
        let active_target_id: Option<String> = row.get(0);
        let release_id: Option<String> = row.get(1);
        Ok(Some(active_target_id.or(release_id)))
    } else {
        Ok(None)
    }
}

/// Resolve a feature by git branch and return `(feature_id, target_id)`.
/// Uses most recently updated row when multiple features share the same branch.
pub fn get_feature_by_branch_links(
    ship_dir: &Path,
    branch: &str,
) -> Result<Option<FeatureBranchLinks>> {
    let mut conn = open_project_db(ship_dir)?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT id, active_target_id, release_id
             FROM feature
             WHERE branch = ?
             ORDER BY updated_at DESC
             LIMIT 1",
        )
        .bind(branch)
        .fetch_optional(&mut conn)
        .await
    })?;
    if let Some(row) = row_opt {
        let feature_id: String = row.get(0);
        let active_target_id: Option<String> = row.get(1);
        let release_id: Option<String> = row.get(2);
        Ok(Some((feature_id, active_target_id.or(release_id))))
    } else {
        Ok(None)
    }
}

/// Read provider candidates declared on a feature's `agent_json.providers`.
/// Returns:
/// - `None` when the feature row does not exist
/// - `Some(vec![])` when present but unset/invalid/empty
pub fn get_feature_agent_providers(
    ship_dir: &Path,
    feature_id: &str,
) -> Result<Option<Vec<String>>> {
    let feature_agent = get_feature_agent_config(ship_dir, feature_id)?;
    let Some(agent) = feature_agent else {
        return Ok(None);
    };
    Ok(Some(agent.providers))
}

/// Read and parse a feature's `agent_json` payload.
/// Returns:
/// - `None` when the feature row does not exist
/// - `Some(None)` semantics are represented as `Some(WorkspaceAgentSettings::default())`
///   when `agent_json` is unset/empty/invalid
pub fn get_feature_agent_config(
    ship_dir: &Path,
    feature_id: &str,
) -> Result<Option<WorkspaceAgentSettings>> {
    let mut conn = open_project_db(ship_dir)?;
    let row_opt = block_on(async {
        sqlx::query("SELECT agent_json FROM feature WHERE id = ?")
            .bind(feature_id)
            .fetch_optional(&mut conn)
            .await
    })?;

    let Some(row) = row_opt else {
        return Ok(None);
    };

    let agent_json: Option<String> = row.get(0);
    let Some(raw) = agent_json else {
        return Ok(Some(WorkspaceAgentSettings::default()));
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "{}" || trimmed.eq_ignore_ascii_case("null") {
        return Ok(Some(WorkspaceAgentSettings::default()));
    }

    let parsed: WorkspaceAgentSettings = match serde_json::from_str(trimmed) {
        Ok(value) => value,
        Err(_) => return Ok(Some(WorkspaceAgentSettings::default())),
    };
    Ok(Some(parsed))
}

/// Replace the ordered feature slice for a target/release.
pub fn replace_target_features_db(
    ship_dir: &Path,
    target_id: &str,
    feature_ids: &[String],
) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query("DELETE FROM target_feature WHERE target_id = ?")
            .bind(target_id)
            .execute(&mut conn)
            .await?;

        for (ord, feature_id) in feature_ids.iter().enumerate() {
            sqlx::query(
                "INSERT OR IGNORE INTO target_feature (target_id, feature_id, ord, created_at)
                 VALUES (?, ?, ?, ?)",
            )
            .bind(target_id)
            .bind(feature_id)
            .bind(ord as i64)
            .bind(&now)
            .execute(&mut conn)
            .await?;
        }

        Ok::<(), sqlx::Error>(())
    })?;
    Ok(())
}

/// List feature ids currently linked to a target/release ordered by `ord`.
pub fn list_target_features_db(ship_dir: &Path, target_id: &str) -> Result<Vec<String>> {
    let mut conn = open_project_db(ship_dir)?;
    block_on(async {
        sqlx::query_scalar::<_, String>(
            "SELECT feature_id
             FROM target_feature
             WHERE target_id = ?
             ORDER BY ord ASC, created_at ASC",
        )
        .bind(target_id)
        .fetch_all(&mut conn)
        .await
    })
}

