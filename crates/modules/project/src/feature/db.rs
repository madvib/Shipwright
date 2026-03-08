use super::types::{
    Feature, FeatureCriterion, FeatureDocStatus, FeatureDocumentation, FeatureEntry,
    FeatureMetadata, FeatureStatus, FeatureTodo,
};
use anyhow::Result;
use chrono::Utc;
use sqlx::{Connection, Row};
use std::path::Path;
use std::str::FromStr;

pub fn upsert_feature_db(ship_dir: &Path, feature: &Feature, status: &FeatureStatus) -> Result<()> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    let now = Utc::now().to_rfc3339();

    runtime::state_db::block_on(async {
        let mut tx = conn.begin().await?;

        // Upsert feature
        sqlx::query(
            "INSERT INTO feature
               (id, title, description, status, release_id, active_target_id, spec_id, branch, agent_json, tags_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               title       = excluded.title,
               description = excluded.description,
               status      = excluded.status,
               release_id  = excluded.release_id,
               active_target_id = excluded.active_target_id,
               spec_id     = excluded.spec_id,
               branch      = excluded.branch,
               agent_json  = excluded.agent_json,
               tags_json   = excluded.tags_json,
               updated_at  = excluded.updated_at",
        )
        .bind(&feature.metadata.id)
        .bind(&feature.metadata.title)
        .bind(&feature.metadata.description)
        .bind(status.to_string())
        .bind(&feature.metadata.release_id)
        .bind(&feature.metadata.active_target_id)
        .bind(&feature.metadata.spec_id)
        .bind(&feature.metadata.branch)
        .bind(serde_json::to_string(&feature.metadata.agent).unwrap_or_default())
        .bind(serde_json::to_string(&feature.metadata.tags).unwrap_or_else(|_| "[]".to_string()))
        .bind(&feature.metadata.created)
        .bind(&now)
        .execute(&mut *tx)
        .await?;

        // Delete existing todos/criteria to replace
        sqlx::query("DELETE FROM feature_todo WHERE feature_id = ?")
            .bind(&feature.metadata.id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM feature_criterion WHERE feature_id = ?")
            .bind(&feature.metadata.id)
            .execute(&mut *tx)
            .await?;

        // Insert todos
        for (i, todo) in feature.todos.iter().enumerate() {
            sqlx::query(
                "INSERT INTO feature_todo (id, feature_id, text, completed, ord) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&todo.id)
            .bind(&feature.metadata.id)
            .bind(&todo.text)
            .bind(if todo.completed { 1 } else { 0 })
            .bind(i as i64)
            .execute(&mut *tx)
            .await?;
        }

        // Insert criteria
        for (i, criterion) in feature.criteria.iter().enumerate() {
            sqlx::query(
                "INSERT INTO feature_criterion (id, feature_id, text, met, ord) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&criterion.id)
            .bind(&feature.metadata.id)
            .bind(&criterion.text)
            .bind(if criterion.met { 1 } else { 0 })
            .bind(i as i64)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    })?;
    Ok(())
}

pub fn get_feature_db(ship_dir: &Path, id: &str) -> Result<Option<FeatureEntry>> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        let row_opt = sqlx::query(
            "SELECT id, title, description, status, release_id, active_target_id, spec_id, branch, agent_json, tags_json, created_at, updated_at
             FROM feature WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut conn)
        .await?;

        if let Some(r) = row_opt {
            let id: String = r.get(0);
            let title: String = r.get(1);
            let description: Option<String> = r.get(2);
            let status_str: String = r.get(3);
            let release_id: Option<String> = r.get(4);
            let active_target_id: Option<String> = r.get(5);
            let spec_id: Option<String> = r.get(6);
            let branch: Option<String> = r.get(7);
            let agent_json: Option<String> = r.get(8);
            let tags_json: String = r.get(9);
            let created: String = r.get(10);
            let updated: String = r.get(11);

            let status = FeatureStatus::from_str(&status_str).unwrap_or_default();
            let agent = agent_json.and_then(|j| serde_json::from_str(&j).ok());
            let tags = serde_json::from_str(&tags_json).unwrap_or_default();

            // Fetch todos
            let todos_rows = sqlx::query(
                "SELECT id, text, completed FROM feature_todo WHERE feature_id = ? ORDER BY ord ASC",
            )
            .bind(&id)
            .fetch_all(&mut conn)
            .await?;

            let todos = todos_rows
                .into_iter()
                .map(|tr| FeatureTodo {
                    id: tr.get(0),
                    text: tr.get(1),
                    completed: tr.get::<i64, _>(2) != 0,
                })
                .collect();

            // Fetch criteria
            let criteria_rows = sqlx::query(
                "SELECT id, text, met FROM feature_criterion WHERE feature_id = ? ORDER BY ord ASC",
            )
            .bind(&id)
            .fetch_all(&mut conn)
            .await?;

            let criteria = criteria_rows
                .into_iter()
                .map(|cr| FeatureCriterion {
                    id: cr.get(0),
                    text: cr.get(1),
                    met: cr.get::<i64, _>(2) != 0,
                })
                .collect();

            let file_name = runtime::project::sanitize_file_name(&title) + ".md";

            Ok(Some(FeatureEntry {
                id: id.clone(),
                file_name,
                path: String::new(),
                status,
                feature: Feature {
                    metadata: FeatureMetadata {
                        id,
                        title,
                        description,
                        created,
                        updated,
                        release_id,
                        active_target_id,
                        spec_id,
                        branch,
                        agent,
                        tags,
                    },
                    body: String::new(), // Body handled by file system or separate field
                    todos,
                    criteria,
                },
            }))
        } else {
            Ok(None)
        }
    })
}

pub fn list_features_db(ship_dir: &Path) -> Result<Vec<FeatureEntry>> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        let rows = sqlx::query(
            "SELECT id, title, description, status, release_id, active_target_id, spec_id, branch, agent_json, tags_json, created_at, updated_at
             FROM feature ORDER BY updated_at DESC",
        )
        .fetch_all(&mut conn)
        .await?;

        let mut entries = Vec::new();
        for r in rows {
            let id: String = r.get(0);
            let title: String = r.get(1);
            let description: Option<String> = r.get(2);
            let status_str: String = r.get(3);
            let release_id: Option<String> = r.get(4);
            let active_target_id: Option<String> = r.get(5);
            let spec_id: Option<String> = r.get(6);
            let branch: Option<String> = r.get(7);
            let agent_json: Option<String> = r.get(8);
            let tags_json: String = r.get(9);
            let created: String = r.get(10);
            let updated: String = r.get(11);

            let status = FeatureStatus::from_str(&status_str).unwrap_or_default();
            let agent = agent_json.and_then(|j| serde_json::from_str(&j).ok());
            let tags = serde_json::from_str(&tags_json).unwrap_or_default();
            let file_name = runtime::project::sanitize_file_name(&title) + ".md";

            entries.push(FeatureEntry {
                id: id.clone(),
                file_name,
                path: String::new(),
                status,
                feature: Feature {
                    metadata: FeatureMetadata {
                        id,
                        title,
                        description,
                        created,
                        updated,
                        release_id,
                        active_target_id,
                        spec_id,
                        branch,
                        agent,
                        tags,
                    },
                    body: String::new(),
                    todos: Vec::new(), // Optional: lazy load or join? Joining is better.
                    criteria: Vec::new(), // For list, maybe we don't need the full checklists.
                },
            });
        }
        Ok(entries)
    })
}

pub fn delete_feature_db(ship_dir: &Path, id: &str) -> Result<()> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        sqlx::query("DELETE FROM feature WHERE id = ?")
            .bind(id)
            .execute(&mut conn)
            .await?;
        Ok(())
    })?;
    Ok(())
}

pub fn get_feature_doc_db(
    ship_dir: &Path,
    feature_id: &str,
) -> Result<Option<FeatureDocumentation>> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        let row = sqlx::query(
            "SELECT feature_id, status, content, revision, last_verified_at, created_at, updated_at
             FROM feature_doc
             WHERE feature_id = ?",
        )
        .bind(feature_id)
        .fetch_optional(&mut conn)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let status_raw: String = row.get(1);
        let status = FeatureDocStatus::from_str(&status_raw).unwrap_or_default();
        Ok(Some(FeatureDocumentation {
            feature_id: row.get(0),
            status,
            content: row.get(2),
            revision: row.get(3),
            last_verified_at: row.get(4),
            created_at: row.get(5),
            updated_at: row.get(6),
        }))
    })
}

pub fn upsert_feature_doc_db(
    ship_dir: &Path,
    doc: &FeatureDocumentation,
    actor: Option<&str>,
) -> Result<FeatureDocumentation> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        let mut tx = conn.begin().await?;
        let now = Utc::now().to_rfc3339();
        let actor = actor.unwrap_or("ship");

        let current = sqlx::query(
            "SELECT status, content, revision, created_at FROM feature_doc WHERE feature_id = ?",
        )
        .bind(&doc.feature_id)
        .fetch_optional(&mut *tx)
        .await?;

        let (next_revision, created_at, changed) = if let Some(row) = current {
            let current_status: String = row.get(0);
            let current_content: String = row.get(1);
            let current_revision: i64 = row.get(2);
            let created_at: String = row.get(3);
            let changed =
                current_status != doc.status.to_string() || current_content != doc.content;
            let next_revision = if changed {
                current_revision + 1
            } else {
                current_revision
            };
            (next_revision, created_at, changed)
        } else {
            (1_i64, now.clone(), true)
        };

        sqlx::query(
            "INSERT INTO feature_doc
               (feature_id, status, content, revision, last_verified_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(feature_id) DO UPDATE SET
               status = excluded.status,
               content = excluded.content,
               revision = excluded.revision,
               last_verified_at = excluded.last_verified_at,
               updated_at = excluded.updated_at",
        )
        .bind(&doc.feature_id)
        .bind(doc.status.to_string())
        .bind(&doc.content)
        .bind(next_revision)
        .bind(&doc.last_verified_at)
        .bind(&created_at)
        .bind(&now)
        .execute(&mut *tx)
        .await?;

        if changed {
            sqlx::query(
                "INSERT INTO feature_doc_revision
                   (id, feature_id, revision, status, content, actor, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(runtime::gen_nanoid())
            .bind(&doc.feature_id)
            .bind(next_revision)
            .bind(doc.status.to_string())
            .bind(&doc.content)
            .bind(actor)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(FeatureDocumentation {
            feature_id: doc.feature_id.clone(),
            status: doc.status.clone(),
            content: doc.content.clone(),
            revision: next_revision,
            last_verified_at: doc.last_verified_at.clone(),
            created_at,
            updated_at: now,
        })
    })
}
