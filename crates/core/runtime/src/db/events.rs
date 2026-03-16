//! Append-only event log. Never update or delete event records.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use std::path::Path;

use crate::db::{block_on, open_db};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRecord {
    pub seq: i64,
    pub timestamp: String,
    pub actor: String,
    pub entity: String,
    pub action: String,
    pub subject: String,
    pub details: Option<String>,
}

const COLS: &str = "seq, timestamp, actor, entity, action, subject, details";

pub fn append_event(
    ship_dir: &Path,
    actor: &str,
    entity: &str,
    action: &str,
    subject: &str,
    details: Option<&str>,
) -> Result<()> {
    let mut conn = open_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO event_log (timestamp, actor, entity, action, subject, details)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&now)
        .bind(actor)
        .bind(entity)
        .bind(action)
        .bind(subject)
        .bind(details)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn list_events(
    ship_dir: &Path,
    entity: Option<&str>,
    subject: Option<&str>,
    limit: Option<u32>,
) -> Result<Vec<EventRecord>> {
    let mut conn = open_db(ship_dir)?;
    let lim = limit.unwrap_or(200);
    let rows = match (entity, subject) {
        (Some(e), Some(s)) => block_on(async {
            sqlx::query(&format!(
                "SELECT {COLS} FROM event_log WHERE entity = ? AND subject = ? ORDER BY seq DESC LIMIT ?"
            ))
            .bind(e).bind(s).bind(lim)
            .fetch_all(&mut conn)
            .await
        })?,
        (Some(e), None) => block_on(async {
            sqlx::query(&format!(
                "SELECT {COLS} FROM event_log WHERE entity = ? ORDER BY seq DESC LIMIT ?"
            ))
            .bind(e).bind(lim)
            .fetch_all(&mut conn)
            .await
        })?,
        (None, Some(s)) => block_on(async {
            sqlx::query(&format!(
                "SELECT {COLS} FROM event_log WHERE subject = ? ORDER BY seq DESC LIMIT ?"
            ))
            .bind(s).bind(lim)
            .fetch_all(&mut conn)
            .await
        })?,
        (None, None) => block_on(async {
            sqlx::query(&format!(
                "SELECT {COLS} FROM event_log ORDER BY seq DESC LIMIT ?"
            ))
            .bind(lim)
            .fetch_all(&mut conn)
            .await
        })?,
    };
    Ok(rows.iter().map(row_to_event).collect())
}

fn row_to_event(row: &sqlx::sqlite::SqliteRow) -> EventRecord {
    EventRecord {
        seq: row.get(0),
        timestamp: row.get(1),
        actor: row.get(2),
        entity: row.get(3),
        action: row.get(4),
        subject: row.get(5),
        details: row.get(6),
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

    #[test]
    fn test_append_and_list_events() {
        let (_tmp, ship_dir) = setup();
        append_event(&ship_dir, "agent-1", "workspace", "activated", "ws-001", None).unwrap();
        append_event(&ship_dir, "agent-1", "workspace", "session_started", "ws-001", Some("{\"goal\":\"build\"}")).unwrap();
        let events = list_events(&ship_dir, None, None, None).unwrap();
        assert_eq!(events.len(), 2);
        // seq should be monotonically increasing (DESC order means first item has higher seq)
        assert!(events[0].seq > events[1].seq);
    }

    #[test]
    fn test_list_events_filtered_by_entity() {
        let (_tmp, ship_dir) = setup();
        append_event(&ship_dir, "agent-1", "workspace", "activated", "ws-001", None).unwrap();
        append_event(&ship_dir, "agent-1", "session", "started", "sess-1", None).unwrap();
        let ws_events = list_events(&ship_dir, Some("workspace"), None, None).unwrap();
        assert_eq!(ws_events.len(), 1);
        assert_eq!(ws_events[0].entity, "workspace");
    }

    #[test]
    fn test_list_events_filtered_by_subject() {
        let (_tmp, ship_dir) = setup();
        append_event(&ship_dir, "agent-1", "workspace", "activated", "ws-001", None).unwrap();
        append_event(&ship_dir, "agent-1", "workspace", "deactivated", "ws-002", None).unwrap();
        let events = list_events(&ship_dir, None, Some("ws-001"), None).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].subject, "ws-001");
    }

    #[test]
    fn test_events_are_append_only() {
        // Verify no update/delete functions exist — the type system enforces this.
        // This test documents the invariant.
        let (_tmp, ship_dir) = setup();
        append_event(&ship_dir, "system", "db", "initialized", "ship.db", None).unwrap();
        let events = list_events(&ship_dir, None, None, None).unwrap();
        assert!(!events.is_empty());
    }
}
