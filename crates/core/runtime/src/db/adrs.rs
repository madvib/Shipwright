//! ADR CRUD for platform.db — architecture decision records.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;

use crate::db::{block_on, open_db};
use crate::gen_nanoid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdrRecord {
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

const COLS: &str =
    "id, title, status, date, context, decision, tags_json, supersedes_id, created_at, updated_at";

pub fn create_adr(
    title: &str,
    context: &str,
    decision: &str,
    status: &str,
) -> Result<AdrRecord> {
    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    let id = gen_nanoid();
    block_on(async {
        sqlx::query(
            "INSERT INTO adr (id, title, status, date, context, decision, tags_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, '[]', ?, ?)",
        )
        .bind(&id)
        .bind(title)
        .bind(status)
        .bind(&now)
        .bind(context)
        .bind(decision)
        .bind(&now)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(AdrRecord {
        id,
        title: title.to_string(),
        status: status.to_string(),
        date: now.clone(),
        context: context.to_string(),
        decision: decision.to_string(),
        tags_json: "[]".to_string(),
        supersedes_id: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_adr(
    conn: &mut sqlx::SqliteConnection,
    id: &str,
    title: Option<&str>,
    context: Option<&str>,
    decision: Option<&str>,
    status: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let current = get_adr_impl(conn, id)?.ok_or_else(|| anyhow::anyhow!("ADR {id} not found"))?;
    let new_title = title.unwrap_or(&current.title);
    let new_context = context.unwrap_or(&current.context);
    let new_decision = decision.unwrap_or(&current.decision);
    let new_status = status.unwrap_or(&current.status);
    block_on(async {
        sqlx::query(
            "UPDATE adr SET title = ?, context = ?, decision = ?, status = ?, updated_at = ? WHERE id = ?",
        )
        .bind(new_title)
        .bind(new_context)
        .bind(new_decision)
        .bind(new_status)
        .bind(&now)
        .bind(id)
        .execute(conn)
        .await
    })?;
    Ok(())
}

pub fn get_adr(id: &str) -> Result<Option<AdrRecord>> {
    let mut conn = open_db()?;
    get_adr_impl(&mut conn, id)
}

fn get_adr_impl(conn: &mut sqlx::SqliteConnection, id: &str) -> Result<Option<AdrRecord>> {
    let row = block_on(async {
        sqlx::query(&format!("SELECT {COLS} FROM adr WHERE id = ?"))
            .bind(id)
            .fetch_optional(conn)
            .await
    })?;
    Ok(row.map(|r| row_to_adr(&r)))
}

pub fn list_adrs() -> Result<Vec<AdrRecord>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM adr ORDER BY date DESC, created_at DESC"
        ))
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows.iter().map(row_to_adr).collect())
}

pub fn delete_adr(id: &str) -> Result<()> {
    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("DELETE FROM adr WHERE id = ?")
            .bind(id)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

fn row_to_adr(row: &sqlx::sqlite::SqliteRow) -> AdrRecord {
    AdrRecord {
        id: row.get(0),
        title: row.get(1),
        status: row.get(2),
        date: row.get(3),
        context: row.get(4),
        decision: row.get(5),
        tags_json: row.get(6),
        supersedes_id: row.get(7),
        created_at: row.get(8),
        updated_at: row.get(9),
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

    #[test]
    fn test_create_adr() {
        let (_tmp, _ship_dir) = setup();
        let adr = create_adr(
            "Use SQLite",
            "Need local storage",
            "Use SQLite for all local state",
            "proposed",
        )
        .unwrap();
        assert_eq!(adr.title, "Use SQLite");
        assert_eq!(adr.status, "proposed");
        assert_eq!(adr.context, "Need local storage");
        assert_eq!(adr.decision, "Use SQLite for all local state");
        assert!(adr.supersedes_id.is_none());
    }

    #[test]
    fn test_get_adr() {
        let (_tmp, _ship_dir) = setup();
        let adr = create_adr(
            "Use Rust",
            "Performance matters",
            "Use Rust",
            "accepted",
        )
        .unwrap();
        let got = get_adr(&adr.id).unwrap().unwrap();
        assert_eq!(got.id, adr.id);
        assert_eq!(got.title, "Use Rust");
        assert_eq!(got.status, "accepted");
    }

    #[test]
    fn test_list_adrs() {
        let (_tmp, _ship_dir) = setup();
        create_adr("ADR One", "ctx1", "dec1", "proposed").unwrap();
        create_adr("ADR Two", "ctx2", "dec2", "accepted").unwrap();
        create_adr("ADR Three", "ctx3", "dec3", "rejected").unwrap();
        let adrs = list_adrs().unwrap();
        assert_eq!(adrs.len(), 3);
    }

    #[test]
    fn test_delete_adr() {
        let (_tmp, _ship_dir) = setup();
        let adr = create_adr("Temporary ADR", "", "", "proposed").unwrap();
        delete_adr(&adr.id).unwrap();
        assert!(get_adr(&adr.id).unwrap().is_none());
    }

    #[test]
    fn test_update_adr() {
        let (_tmp, _ship_dir) = setup();
        let adr = create_adr(
            "Draft ADR",
            "initial context",
            "initial decision",
            "proposed",
        )
        .unwrap();
        let mut conn = open_db().unwrap();
        update_adr(
            &mut conn,
            &adr.id,
            Some("Final ADR"),
            None,
            Some("final decision"),
            Some("accepted"),
        )
        .unwrap();
        let got = get_adr(&adr.id).unwrap().unwrap();
        assert_eq!(got.title, "Final ADR");
        assert_eq!(got.context, "initial context");
        assert_eq!(got.decision, "final decision");
        assert_eq!(got.status, "accepted");
    }
}
