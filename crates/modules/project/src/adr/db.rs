/// SQLite CRUD helpers for the `adr` and `adr_option` tables — now using
/// runtime's public state_db primitives instead of crate-internal ones.
use super::types::{ADR, AdrEntry, AdrMetadata, AdrStatus};
use anyhow::Result;
use chrono::Utc;
use serde_json;
use std::path::Path;
use std::str::FromStr;

pub struct AdrDbRow {
    pub id: String,
    pub title: String,
    pub status: String,
    pub date: String,
    pub context: String,
    pub decision: String,
    pub tags_json: String,
    pub supersedes_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl AdrDbRow {
    pub fn into_entry(self) -> AdrEntry {
        let status = AdrStatus::from_str(&self.status).unwrap_or_default();
        let tags: Vec<String> = serde_json::from_str(&self.tags_json).unwrap_or_default();
        let file_name = runtime::project::sanitize_file_name(&self.title) + ".md";
        AdrEntry {
            id: self.id.clone(),
            file_name,
            path: String::new(),
            status,
            adr: ADR {
                metadata: AdrMetadata {
                    id: self.id,
                    title: self.title,
                    date: self.date,
                    tags,
                    supersedes_id: self.supersedes_id,
                },
                context: self.context,
                decision: self.decision,
            },
        }
    }
}

pub fn upsert_adr_db(ship_dir: &Path, row: &AdrDbRow) -> Result<()> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    runtime::state_db::block_on(async {
        sqlx::query(
            "INSERT INTO adr
               (id, title, status, date, context, decision, tags_json,
                supersedes_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               title         = excluded.title,
               status        = excluded.status,
               date          = excluded.date,
               context       = excluded.context,
               decision      = excluded.decision,
               tags_json     = excluded.tags_json,
               supersedes_id = excluded.supersedes_id,
               updated_at    = excluded.updated_at",
        )
        .bind(&row.id)
        .bind(&row.title)
        .bind(&row.status)
        .bind(&row.date)
        .bind(&row.context)
        .bind(&row.decision)
        .bind(&row.tags_json)
        .bind(&row.supersedes_id)
        .bind(&row.created_at)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn get_adr_db(ship_dir: &Path, id: &str) -> Result<Option<AdrDbRow>> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    use sqlx::Row;
    let row_opt = runtime::state_db::block_on(async {
        sqlx::query(
            "SELECT id, title, status, date, context, decision, tags_json,
                    supersedes_id, created_at, updated_at
             FROM adr WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row_opt.map(|r| AdrDbRow {
        id: r.get(0),
        title: r.get(1),
        status: r.get(2),
        date: r.get(3),
        context: r.get(4),
        decision: r.get(5),
        tags_json: r.get(6),
        supersedes_id: r.get(7),
        created_at: r.get(8),
        updated_at: r.get(9),
    }))
}

pub fn list_adrs_db(ship_dir: &Path) -> Result<Vec<AdrDbRow>> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    use sqlx::Row;
    let rows = runtime::state_db::block_on(async {
        sqlx::query(
            "SELECT id, title, status, date, context, decision, tags_json,
                    supersedes_id, created_at, updated_at
             FROM adr ORDER BY date DESC, created_at DESC",
        )
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows
        .into_iter()
        .map(|r| AdrDbRow {
            id: r.get(0),
            title: r.get(1),
            status: r.get(2),
            date: r.get(3),
            context: r.get(4),
            decision: r.get(5),
            tags_json: r.get(6),
            supersedes_id: r.get(7),
            created_at: r.get(8),
            updated_at: r.get(9),
        })
        .collect())
}

pub fn delete_adr_db(ship_dir: &Path, id: &str) -> Result<()> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        sqlx::query("DELETE FROM adr WHERE id = ?")
            .bind(id)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

pub fn update_adr_status_db(ship_dir: &Path, id: &str, new_status: &str) -> Result<()> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    runtime::state_db::block_on(async {
        sqlx::query("UPDATE adr SET status = ?, updated_at = ? WHERE id = ?")
            .bind(new_status)
            .bind(&now)
            .bind(id)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

pub fn adr_to_db_row(adr: &ADR, status: &str, created_at: Option<&str>) -> AdrDbRow {
    let tags_json = serde_json::to_string(&adr.metadata.tags).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now().to_rfc3339();
    AdrDbRow {
        id: adr.metadata.id.clone(),
        title: adr.metadata.title.clone(),
        status: status.to_string(),
        date: adr.metadata.date.clone(),
        context: adr.context.clone(),
        decision: adr.decision.clone(),
        tags_json,
        supersedes_id: adr.metadata.supersedes_id.clone(),
        created_at: created_at.unwrap_or(&now).to_string(),
        updated_at: now,
    }
}
