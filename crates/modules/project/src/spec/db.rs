use super::types::{Spec, SpecEntry, SpecMetadata, SpecStatus};
use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use std::path::Path;
use std::str::FromStr;

fn virtual_spec_file_name(id: &str) -> String {
    format!("{}.md", id)
}

fn virtual_spec_path(ship_dir: &Path, status: &SpecStatus, id: &str) -> String {
    runtime::project::specs_dir(ship_dir)
        .join(status.to_string())
        .join(virtual_spec_file_name(id))
        .to_string_lossy()
        .to_string()
}

pub fn upsert_spec_db(ship_dir: &Path, spec: &Spec, status: &SpecStatus) -> Result<()> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    let now = Utc::now().to_rfc3339();

    runtime::state_db::block_on(async {
        sqlx::query(
            "INSERT INTO spec
               (id, title, body, status, author, branch, feature_id, release_id, tags_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               title      = excluded.title,
               body       = excluded.body,
               status     = excluded.status,
               author     = excluded.author,
               branch     = excluded.branch,
               feature_id = excluded.feature_id,
               release_id = excluded.release_id,
               tags_json  = excluded.tags_json,
               updated_at = excluded.updated_at",
        )
        .bind(&spec.metadata.id)
        .bind(&spec.metadata.title)
        .bind(&spec.body)
        .bind(status.to_string())
        .bind(&spec.metadata.author)
        .bind(&spec.metadata.branch)
        .bind(&spec.metadata.feature_id)
        .bind(&spec.metadata.release_id)
        .bind(serde_json::to_string(&spec.metadata.tags).unwrap_or_else(|_| "[]".to_string()))
        .bind(&spec.metadata.created)
        .bind(&now)
        .execute(&mut conn)
        .await?;
        Ok(())
    })?;
    Ok(())
}

pub fn get_spec_db(ship_dir: &Path, id: &str) -> Result<Option<SpecEntry>> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        let row_opt = sqlx::query(
            "SELECT id, title, body, status, author, branch, feature_id, release_id, tags_json, created_at, updated_at
             FROM spec WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut conn)
        .await?;

        if let Some(r) = row_opt {
            let id: String = r.get(0);
            let title: String = r.get(1);
            let body: String = r.get(2);
            let status_str: String = r.get(3);
            let author: Option<String> = r.get(4);
            let branch: Option<String> = r.get(5);
            let feature_id: Option<String> = r.get(6);
            let release_id: Option<String> = r.get(7);
            let tags_json: String = r.get(8);
            let created: String = r.get(9);
            let updated: String = r.get(10);

            let status = SpecStatus::from_str(&status_str).ok().unwrap_or_default();
            let tags = serde_json::from_str(&tags_json).unwrap_or_default();
            let file_name = virtual_spec_file_name(&id);
            let path = virtual_spec_path(ship_dir, &status, &id);

            Ok(Some(SpecEntry {
                id: id.clone(),
                file_name,
                path,
                status,
                spec: Spec {
                    metadata: SpecMetadata {
                        id,
                        title,
                        created,
                        updated,
                        author,
                        branch,
                        feature_id,
                        release_id,
                        tags,
                    },
                    body,
                },
            }))
        } else {
            Ok(None)
        }
    })
}

pub fn list_specs_db(ship_dir: &Path) -> Result<Vec<SpecEntry>> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        let rows = sqlx::query(
            "SELECT id, title, body, status, author, branch, feature_id, release_id, tags_json, created_at, updated_at
             FROM spec ORDER BY updated_at DESC",
        )
        .fetch_all(&mut conn)
        .await?;

        let mut entries = Vec::new();
        for r in rows {
            let id: String = r.get(0);
            let title: String = r.get(1);
            let body: String = r.get(2);
            let status_str: String = r.get(3);
            let author: Option<String> = r.get(4);
            let branch: Option<String> = r.get(5);
            let feature_id: Option<String> = r.get(6);
            let release_id: Option<String> = r.get(7);
            let tags_json: String = r.get(8);
            let created: String = r.get(9);
            let updated: String = r.get(10);

            let status = SpecStatus::from_str(&status_str).ok().unwrap_or_default();
            let tags = serde_json::from_str(&tags_json).unwrap_or_default();
            let file_name = virtual_spec_file_name(&id);
            let path = virtual_spec_path(ship_dir, &status, &id);

            entries.push(SpecEntry {
                id: id.clone(),
                file_name,
                path,
                status,
                spec: Spec {
                    metadata: SpecMetadata {
                        id,
                        title,
                        created,
                        updated,
                        author,
                        branch,
                        feature_id,
                        release_id,
                        tags,
                    },
                    body,
                },
            });
        }
        Ok(entries)
    })
}

pub fn delete_spec_db(ship_dir: &Path, id: &str) -> Result<()> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        sqlx::query("DELETE FROM spec WHERE id = ?")
            .bind(id)
            .execute(&mut conn)
            .await?;
        Ok(())
    })?;
    Ok(())
}
