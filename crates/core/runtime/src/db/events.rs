//! Typed event queries against the `events` table.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::db::{block_on, db_path, ensure_db, open_db_at};
use crate::events::envelope::EventEnvelope;

const COLS: &str = "id, event_type, entity_id, actor, payload_json, version, \
    causation_id, workspace_id, session_id, \
    actor_id, parent_actor_id, elevated, created_at";

pub fn list_all_events() -> Result<Vec<EventEnvelope>> {
    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
    let rows = block_on(async {
        sqlx::query(&format!("SELECT {COLS} FROM events ORDER BY id ASC"))
            .fetch_all(&mut conn)
            .await
    })?;
    rows.iter().map(row_to_envelope).collect()
}

pub fn list_events_since_time(
    since: &DateTime<Utc>,
    limit: Option<usize>,
) -> Result<Vec<EventEnvelope>> {
    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
    let since_str = since.to_rfc3339();
    let rows = match limit {
        Some(n) => block_on(async {
            sqlx::query(&format!(
                "SELECT {COLS} FROM events WHERE created_at >= ? ORDER BY id ASC LIMIT ?"
            ))
            .bind(&since_str)
            .bind(n as i64)
            .fetch_all(&mut conn)
            .await
        })?,
        None => block_on(async {
            sqlx::query(&format!(
                "SELECT {COLS} FROM events WHERE created_at >= ? ORDER BY id ASC"
            ))
            .bind(&since_str)
            .fetch_all(&mut conn)
            .await
        })?,
    };
    rows.iter().map(row_to_envelope).collect()
}

pub fn list_recent_events(limit: usize) -> Result<Vec<EventEnvelope>> {
    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM events ORDER BY id DESC LIMIT ?"
        ))
        .bind(limit as i64)
        .fetch_all(&mut conn)
        .await
    })?;
    let mut envs: Vec<EventEnvelope> = rows.iter().map(row_to_envelope).collect::<Result<_>>()?;
    envs.reverse();
    Ok(envs)
}

/// Query events with ID greater than the given cursor.
///
/// Used by the sync client to find events that haven't been pushed yet.
/// If `cursor` is None, returns all matching events.
/// If `elevated_only` is true, only returns elevated events (platform-scope).
pub fn query_events_since(
    cursor: Option<&str>,
    elevated_only: bool,
) -> Result<Vec<EventEnvelope>> {
    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
    let query = match (cursor, elevated_only) {
        (Some(c), true) => {
            block_on(async {
                sqlx::query(&format!(
                    "SELECT {COLS} FROM events WHERE id > ? AND elevated = 1 ORDER BY id ASC"
                ))
                .bind(c)
                .fetch_all(&mut conn)
                .await
            })?
        }
        (Some(c), false) => {
            block_on(async {
                sqlx::query(&format!(
                    "SELECT {COLS} FROM events WHERE id > ? ORDER BY id ASC"
                ))
                .bind(c)
                .fetch_all(&mut conn)
                .await
            })?
        }
        (None, true) => {
            block_on(async {
                sqlx::query(&format!(
                    "SELECT {COLS} FROM events WHERE elevated = 1 ORDER BY id ASC"
                ))
                .fetch_all(&mut conn)
                .await
            })?
        }
        (None, false) => {
            block_on(async {
                sqlx::query(&format!("SELECT {COLS} FROM events ORDER BY id ASC"))
                    .fetch_all(&mut conn)
                    .await
            })?
        }
    };
    query.iter().map(row_to_envelope).collect()
}

fn row_to_envelope(row: &sqlx::sqlite::SqliteRow) -> Result<EventEnvelope> {
    // Columns: id(0) event_type(1) entity_id(2) actor(3) payload_json(4) version(5)
    //          causation_id(6) workspace_id(7) session_id(8)
    //          actor_id(9) parent_actor_id(10) elevated(11) created_at(12)
    let created_at_str: String = row.get(12);
    let created_at = created_at_str
        .parse::<DateTime<Utc>>()
        .or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
        })
        .map_err(|e| anyhow::anyhow!("invalid created_at '{}': {}", created_at_str, e))?;

    Ok(EventEnvelope {
        id: row.get(0),
        event_type: row.get(1),
        entity_id: row.get(2),
        actor: row.get(3),
        payload_json: row.get(4),
        version: row.get::<i64, _>(5) as u32,
        causation_id: row.get(6),
        workspace_id: row.get(7),
        session_id: row.get(8),
        actor_id: row.get(9),
        parent_actor_id: row.get(10),
        elevated: row.get::<i64, _>(11) != 0,
        created_at,
        target_actor_id: None,
    })
}
