//! Append-only event log backed by platform.db.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::db::{block_on, open_db};
use crate::events::{EventAction, EventEntity, EventRecord};
use crate::gen_nanoid;

#[allow(clippy::too_many_arguments)]
pub fn insert_event(
    actor: &str,
    entity: &EventEntity,
    entity_id: Option<&str>,
    action: &EventAction,
    detail: Option<&str>,
    workspace_id: Option<&str>,
    session_id: Option<&str>,
    job_id: Option<&str>,
) -> Result<EventRecord> {
    let mut conn = open_db()?;
    let id = gen_nanoid();
    let now = Utc::now().to_rfc3339();
    let entity_type = entity.as_str();
    let action_str = action.as_str();

    block_on(async {
        sqlx::query(
            "INSERT INTO event_log \
             (id, actor, entity_type, entity_id, action, detail, workspace_id, session_id, job_id, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(actor)
        .bind(entity_type)
        .bind(entity_id)
        .bind(action_str)
        .bind(detail)
        .bind(workspace_id)
        .bind(session_id)
        .bind(job_id)
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
        workspace_id: workspace_id.map(str::to_string),
        session_id: session_id.map(str::to_string),
        job_id: job_id.map(str::to_string),
    })
}

// Column order: 0:id 1:actor 2:entity_type 3:entity_id 4:action 5:detail
//               6:workspace_id 7:session_id 8:job_id 9:created_at
const SELECT_COLS: &str = "id, actor, entity_type, entity_id, action, detail, workspace_id, session_id, job_id, created_at";

pub fn list_all_events() -> Result<Vec<EventRecord>> {
    let mut conn = open_db()?;
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
    since: &DateTime<Utc>,
    limit: Option<usize>,
) -> Result<Vec<EventRecord>> {
    let mut conn = open_db()?;
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

pub fn list_recent_events(limit: usize) -> Result<Vec<EventRecord>> {
    let mut conn = open_db()?;
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

pub fn list_events_by_job(job_id: &str) -> Result<Vec<EventRecord>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM event_log WHERE job_id = ? ORDER BY created_at ASC, rowid ASC"
        ))
        .bind(job_id)
        .fetch_all(&mut conn)
        .await
    })?;
    rows.iter().map(row_to_record).collect()
}

pub fn list_events_by_session(session_id: &str) -> Result<Vec<EventRecord>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM event_log WHERE session_id = ? ORDER BY created_at ASC, rowid ASC"
        ))
        .bind(session_id)
        .fetch_all(&mut conn)
        .await
    })?;
    rows.iter().map(row_to_record).collect()
}

pub fn list_events_by_workspace(workspace_id: &str) -> Result<Vec<EventRecord>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM event_log WHERE workspace_id = ? ORDER BY created_at ASC, rowid ASC"
        ))
        .bind(workspace_id)
        .fetch_all(&mut conn)
        .await
    })?;
    rows.iter().map(row_to_record).collect()
}

/// Record a gate pass/fail outcome as an event.
///
/// Creates an event with entity=Gate, entity_id=job_id, action=Pass or Fail.
/// If `passed`, also updates the job status to "complete".
/// If failed, the job stays "running" so it can be retried.
pub fn record_gate_outcome(job_id: &str, passed: bool, evidence: &str) -> Result<EventRecord> {
    let action = if passed {
        EventAction::Pass
    } else {
        EventAction::Fail
    };
    let record = insert_event(
        "ship",
        &EventEntity::Gate,
        Some(job_id),
        &action,
        Some(evidence),
        None,
        None,
        Some(job_id),
    )?;
    if passed {
        crate::db::jobs::update_job_status(job_id, "complete")?;
    }
    Ok(record)
}

/// List gate outcomes (pass/fail events) for a specific job.
pub fn list_gate_outcomes(job_id: &str) -> Result<Vec<EventRecord>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {SELECT_COLS} FROM event_log \
             WHERE entity_type = 'gate' AND entity_id = ? \
             ORDER BY created_at ASC, rowid ASC"
        ))
        .bind(job_id)
        .fetch_all(&mut conn)
        .await
    })?;
    rows.iter().map(row_to_record).collect()
}

fn row_to_record(row: &sqlx::sqlite::SqliteRow) -> Result<EventRecord> {
    let entity_type: String = row.get(2);
    let action_str: String = row.get(4);
    let created_at: String = row.get(9);
    let timestamp = created_at
        .parse::<DateTime<Utc>>()
        .or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(&created_at).map(|dt| dt.with_timezone(&Utc))
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
        workspace_id: row.get(6),
        session_id: row.get(7),
        job_id: row.get(8),
    })
}
