//! Append-only event log backed by platform.db.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::path::Path;

use crate::db::{block_on, open_db};
use crate::events::{EventAction, EventEntity, EventRecord};
use crate::gen_nanoid;

pub fn insert_event(
    ship_dir: &Path,
    actor: &str,
    entity: &EventEntity,
    entity_id: Option<&str>,
    action: &EventAction,
    detail: Option<&str>,
) -> Result<EventRecord> {
    let mut conn = open_db(ship_dir)?;
    let id = gen_nanoid();
    let now = Utc::now().to_rfc3339();
    let entity_type = entity.as_db();
    let action_str = action.as_db();

    block_on(async {
        sqlx::query(
            "INSERT INTO event_log (id, actor, entity_type, entity_id, action, detail, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(actor)
        .bind(entity_type)
        .bind(entity_id)
        .bind(action_str)
        .bind(detail)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;

    Ok(EventRecord {
        id,
        timestamp: Utc::now(),
        actor: actor.to_string(),
        entity: entity.clone(),
        action: action.clone(),
        subject: entity_id.unwrap_or_default().to_string(),
        details: detail.map(str::to_string),
    })
}

const SELECT_COLS: &str =
    "id, actor, entity_type, entity_id, action, detail, created_at";

pub fn list_all_events(ship_dir: &Path) -> Result<Vec<EventRecord>> {
    let mut conn = open_db(ship_dir)?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM event_log ORDER BY created_at ASC, rowid ASC"
        ))
        .fetch_all(&mut conn)
        .await
    })?;
    rows.iter().map(row_to_record).collect()
}

pub fn list_events_since_time(
    ship_dir: &Path,
    since: &DateTime<Utc>,
    limit: Option<usize>,
) -> Result<Vec<EventRecord>> {
    let mut conn = open_db(ship_dir)?;
    let since_str = since.to_rfc3339();
    let rows = match limit {
        Some(n) => block_on(async {
            sqlx::query(&format!(
                "SELECT {SELECT_COLS} FROM event_log
                 WHERE created_at >= ? ORDER BY created_at ASC, rowid ASC LIMIT ?"
            ))
            .bind(&since_str)
            .bind(n as i64)
            .fetch_all(&mut conn)
            .await
        })?,
        None => block_on(async {
            sqlx::query(&format!(
                "SELECT {SELECT_COLS} FROM event_log
                 WHERE created_at >= ? ORDER BY created_at ASC, rowid ASC"
            ))
            .bind(&since_str)
            .fetch_all(&mut conn)
            .await
        })?,
    };
    rows.iter().map(row_to_record).collect()
}

pub fn list_recent_events(ship_dir: &Path, limit: usize) -> Result<Vec<EventRecord>> {
    let mut conn = open_db(ship_dir)?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM event_log
             ORDER BY created_at DESC, rowid DESC LIMIT ?"
        ))
        .bind(limit as i64)
        .fetch_all(&mut conn)
        .await
    })?;
    let mut records: Vec<EventRecord> = rows.iter().map(row_to_record).collect::<Result<_>>()?;
    records.reverse(); // Return in ASC order
    Ok(records)
}

fn row_to_record(row: &sqlx::sqlite::SqliteRow) -> Result<EventRecord> {
    let entity_type: String = row.get(2);
    let action_str: String = row.get(4);
    let created_at: String = row.get(6);
    let timestamp = created_at
        .parse::<DateTime<Utc>>()
        .or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&Utc))
        })
        .unwrap_or_else(|_| Utc::now());

    Ok(EventRecord {
        id: row.get(0),
        timestamp,
        actor: row.get(1),
        entity: EventEntity::from_db(&entity_type)?,
        action: EventAction::from_db(&action_str)?,
        subject: row.get::<Option<String>, _>(3).unwrap_or_default(),
        details: row.get(5),
    })
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
    fn insert_and_read_event() {
        let (_tmp, ship_dir) = setup();
        let record = insert_event(
            &ship_dir,
            "ship",
            &EventEntity::Project,
            Some("my-project"),
            &EventAction::Init,
            Some("initialized"),
        )
        .unwrap();
        assert_eq!(record.entity, EventEntity::Project);
        assert_eq!(record.subject, "my-project");
        assert_eq!(record.actor, "ship");

        let all = list_all_events(&ship_dir).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, record.id);
        assert_eq!(all[0].entity, EventEntity::Project);
        assert_eq!(all[0].actor, "ship");
    }

    #[test]
    fn append_only_ordering() {
        let (_tmp, ship_dir) = setup();
        insert_event(&ship_dir, "ship", &EventEntity::Workspace, Some("feat/a"), &EventAction::Create, None).unwrap();
        insert_event(&ship_dir, "agent", &EventEntity::Session, Some("sess-1"), &EventAction::Start, None).unwrap();
        insert_event(&ship_dir, "logic", &EventEntity::Config, Some("ship.toml"), &EventAction::Update, None).unwrap();

        let all = list_all_events(&ship_dir).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].entity, EventEntity::Workspace);
        assert_eq!(all[1].entity, EventEntity::Session);
        assert_eq!(all[2].entity, EventEntity::Config);
    }

    #[test]
    fn list_since_filters_by_time() {
        let (_tmp, ship_dir) = setup();
        insert_event(&ship_dir, "ship", &EventEntity::Project, Some("p1"), &EventAction::Log, None).unwrap();

        let future = Utc::now() + chrono::Duration::hours(1);
        let filtered = list_events_since_time(&ship_dir, &future, None).unwrap();
        assert!(filtered.is_empty());

        let past = Utc::now() - chrono::Duration::hours(1);
        let all = list_events_since_time(&ship_dir, &past, None).unwrap();
        assert_eq!(all.len(), 1);
    }
}
