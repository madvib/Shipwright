use super::types::{Spec, SpecEntry, SpecMetadata, SpecStatus};
use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use std::path::{Path, PathBuf};
use std::str::FromStr;

fn find_spec_file(ship_dir: &Path, id: &str, status: &SpecStatus, title: &str) -> Option<PathBuf> {
    let base = runtime::project::sanitize_file_name(title);
    let dir = runtime::project::specs_dir(ship_dir).join(status.to_string());
    if !dir.exists() {
        return None;
    }

    for suffix in &["", "-2", "-3", "-4", "-5", "-6", "-7", "-8", "-9", "-10"] {
        let file_name = if suffix.is_empty() {
            format!("{}.md", base)
        } else {
            format!("{}{}.md", base, suffix)
        };
        let p = dir.join(file_name);
        if p.exists() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                if content.contains(id) {
                    return Some(p);
                }
            }
        }
    }

    // Fallback: scan everywhere in specs/
    let specs_dir = runtime::project::specs_dir(ship_dir);
    for status_name in &["draft", "active", "archived"] {
        let dir = specs_dir.join(status_name);
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().map_or(false, |e| e == "md") {
                    if let Ok(content) = std::fs::read_to_string(&p) {
                        if content.contains(id) {
                            return Some(p);
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn upsert_spec_db(ship_dir: &Path, spec: &Spec, status: &SpecStatus) -> Result<()> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    let now = Utc::now().to_rfc3339();

    runtime::state_db::block_on(async {
        sqlx::query(
            "INSERT INTO spec
               (id, title, status, author, branch, feature_id, release_id, tags_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               title      = excluded.title,
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
            "SELECT id, title, status, author, branch, feature_id, release_id, tags_json, created_at, updated_at
             FROM spec WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut conn)
        .await?;

        if let Some(r) = row_opt {
            let id: String = r.get(0);
            let title: String = r.get(1);
            let status_str: String = r.get(2);
            let author: Option<String> = r.get(3);
            let branch: Option<String> = r.get(4);
            let feature_id: Option<String> = r.get(5);
            let release_id: Option<String> = r.get(6);
            let tags_json: String = r.get(7);
            let created: String = r.get(8);
            let updated: String = r.get(9);

            let status = SpecStatus::from_str(&status_str).ok().unwrap_or_default();
            let tags = serde_json::from_str(&tags_json).unwrap_or_default();
            let file_path = find_spec_file(ship_dir, &id, &status, &title);
            let mut body = String::new();
            let mut path = String::new();
            let mut file_name = runtime::project::sanitize_file_name(&title) + ".md";
            let mut metadata = SpecMetadata {
                id: id.clone(),
                title: title.clone(),
                created: created.clone(),
                updated: updated.clone(),
                author,
                branch,
                feature_id,
                release_id,
                tags,
            };

            if let Some(p) = file_path {
                path = p.to_string_lossy().to_string();
                file_name = p.file_name().unwrap().to_string_lossy().to_string();
                if let Ok(content) = std::fs::read_to_string(&p) {
                    if let Ok(spec) = Spec::from_markdown(&content) {
                        body = spec.body;
                        metadata = spec.metadata; // Overwrite metadata from file
                    }
                }
            }

            Ok(Some(SpecEntry {
                id: id.clone(),
                file_name,
                path,
                status,
                spec: Spec { metadata, body },
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
            "SELECT id, title, status, author, branch, feature_id, release_id, tags_json, created_at, updated_at
             FROM spec ORDER BY updated_at DESC",
        )
        .fetch_all(&mut conn)
        .await?;

        let mut entries = Vec::new();
        for r in rows {
            let id: String = r.get(0);
            let title: String = r.get(1);
            let status_str: String = r.get(2);
            let author: Option<String> = r.get(3);
            let branch: Option<String> = r.get(4);
            let feature_id: Option<String> = r.get(5);
            let release_id: Option<String> = r.get(6);
            let tags_json: String = r.get(7);
            let created: String = r.get(8);
            let updated: String = r.get(9);

            let status = SpecStatus::from_str(&status_str).ok().unwrap_or_default();
            let tags = serde_json::from_str(&tags_json).unwrap_or_default();
            let file_path = find_spec_file(ship_dir, &id, &status, &title);
            let mut body = String::new();
            let mut path = String::new();
            let mut file_name = runtime::project::sanitize_file_name(&title) + ".md";
            let mut metadata = SpecMetadata {
                id: id.clone(),
                title: title.clone(),
                created: created.clone(),
                updated: updated.clone(),
                author,
                branch,
                feature_id,
                release_id,
                tags,
            };

            if let Some(p) = file_path {
                path = p.to_string_lossy().to_string();
                file_name = p.file_name().unwrap().to_string_lossy().to_string();
                if let Ok(content) = std::fs::read_to_string(&p) {
                    if let Ok(spec) = Spec::from_markdown(&content) {
                        body = spec.body;
                        metadata = spec.metadata; // Overwrite metadata from file
                    }
                }
            }

            entries.push(SpecEntry {
                id: id.clone(),
                file_name,
                path,
                status,
                spec: Spec { metadata, body },
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
