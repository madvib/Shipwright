//! Typed event queries against the `events` table.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::Row;
use ulid::Ulid;

use crate::db::{block_on, db_path, ensure_db, open_db_at};
use crate::events::envelope::EventEnvelope;
use crate::events::types::event_types;
use crate::events::types::{GateFailed, GatePassed};

const COLS: &str = "id, event_type, entity_id, actor, payload_json, version, \
    correlation_id, causation_id, workspace_id, session_id, \
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

/// Record a gate pass/fail outcome as a typed event.
///
/// Writes to the `events` table with the `job_id` column set.
/// If `passed`, also marks the job as "complete".
pub fn record_gate_outcome(
    job_id: &str,
    passed: bool,
    evidence: &str,
) -> Result<EventEnvelope> {
    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;

    let event_type = if passed {
        event_types::GATE_PASSED
    } else {
        event_types::GATE_FAILED
    };
    let payload_json = if passed {
        serde_json::to_string(&GatePassed {
            evidence: evidence.to_string(),
        })?
    } else {
        serde_json::to_string(&GateFailed {
            evidence: evidence.to_string(),
        })?
    };

    let id = Ulid::new().to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    block_on(async {
        sqlx::query(
            "INSERT INTO events \
             (id, event_type, entity_id, actor, payload_json, version, \
              correlation_id, causation_id, workspace_id, session_id, \
              actor_id, parent_actor_id, elevated, created_at, job_id) \
             VALUES (?, ?, ?, 'ship', ?, 1, NULL, NULL, NULL, NULL, NULL, NULL, 0, ?, ?)",
        )
        .bind(&id)
        .bind(event_type)
        .bind(job_id)
        .bind(&payload_json)
        .bind(&now_str)
        .bind(job_id)
        .execute(&mut conn)
        .await
    })?;

    if passed {
        crate::db::jobs::update_job_status(job_id, "complete")?;
    }

    Ok(EventEnvelope {
        id,
        event_type: event_type.to_string(),
        entity_id: job_id.to_string(),
        actor: "ship".to_string(),
        payload_json,
        version: 1,
        correlation_id: None,
        causation_id: None,
        workspace_id: None,
        session_id: None,
        actor_id: None,
        parent_actor_id: None,
        elevated: false,
        created_at: now,
    })
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

/// List gate outcomes (pass/fail events) for a specific job.
pub fn list_gate_outcomes(job_id: &str) -> Result<Vec<EventEnvelope>> {
    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM events \
             WHERE job_id = ? AND event_type IN ('gate.passed', 'gate.failed') \
             ORDER BY id ASC"
        ))
        .bind(job_id)
        .fetch_all(&mut conn)
        .await
    })?;
    rows.iter().map(row_to_envelope).collect()
}

fn row_to_envelope(row: &sqlx::sqlite::SqliteRow) -> Result<EventEnvelope> {
    // Columns: id(0) event_type(1) entity_id(2) actor(3) payload_json(4) version(5)
    //          correlation_id(6) causation_id(7) workspace_id(8) session_id(9)
    //          actor_id(10) parent_actor_id(11) elevated(12) created_at(13)
    let created_at_str: String = row.get(13);
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
        correlation_id: row.get(6),
        causation_id: row.get(7),
        workspace_id: row.get(8),
        session_id: row.get(9),
        actor_id: row.get(10),
        parent_actor_id: row.get(11),
        elevated: row.get::<i64, _>(12) != 0,
        created_at,
    })
}
