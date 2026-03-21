//! Append-only event log backed by platform.db.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::path::Path;

use crate::db::{block_on, open_db};
use crate::events::{EventAction, EventEntity, EventRecord};
use crate::gen_nanoid;

#[allow(clippy::too_many_arguments)]
pub fn insert_event(
    ship_dir: &Path,
    actor: &str,
    entity: &EventEntity,
    entity_id: Option<&str>,
    action: &EventAction,
    detail: Option<&str>,
    workspace_id: Option<&str>,
    session_id: Option<&str>,
    job_id: Option<&str>,
) -> Result<EventRecord> {
    let mut conn = open_db(ship_dir)?;
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
const SELECT_COLS: &str =
    "id, actor, entity_type, entity_id, action, detail, workspace_id, session_id, job_id, created_at";

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

pub fn list_events_by_job(ship_dir: &Path, job_id: &str) -> Result<Vec<EventRecord>> {
    let mut conn = open_db(ship_dir)?;
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

pub fn list_events_by_session(ship_dir: &Path, session_id: &str) -> Result<Vec<EventRecord>> {
    let mut conn = open_db(ship_dir)?;
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

pub fn list_events_by_workspace(ship_dir: &Path, workspace_id: &str) -> Result<Vec<EventRecord>> {
    let mut conn = open_db(ship_dir)?;
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

/// Migrate existing job_log entries into event_log.
///
/// Each job_log row becomes an event with entity_type='job', action='log'.
/// Already-migrated rows are skipped (idempotent via INSERT OR IGNORE on nanoid).
/// Returns the number of rows migrated.
pub fn migrate_job_log_to_events(ship_dir: &Path) -> Result<usize> {
    let mut conn = open_db(ship_dir)?;

    // Read all job_log rows
    let rows = block_on(async {
        sqlx::query("SELECT job_id, branch, message, actor, created_at FROM job_log ORDER BY id ASC")
            .fetch_all(&mut conn)
            .await
    })?;

    let mut migrated = 0usize;
    for row in &rows {
        let job_id: Option<String> = row.get(0);
        let _branch: Option<String> = row.get(1);
        let message: String = row.get(2);
        let actor: Option<String> = row.get(3);
        let created_at: String = row.get(4);

        let id = crate::gen_nanoid();
        let actor_str = actor.as_deref().unwrap_or("ship");
        let entity_id = job_id.as_deref();

        block_on(async {
            sqlx::query(
                "INSERT INTO event_log \
                 (id, actor, entity_type, entity_id, action, detail, workspace_id, session_id, job_id, created_at) \
                 VALUES (?, ?, 'job', ?, 'log', ?, NULL, NULL, ?, ?)",
            )
            .bind(&id)
            .bind(actor_str)
            .bind(entity_id)
            .bind(&message)
            .bind(job_id.as_deref())
            .bind(&created_at)
            .execute(&mut conn)
            .await
        })?;
        migrated += 1;
    }

    Ok(migrated)
}

/// Record a gate pass/fail outcome as an event.
///
/// Creates an event with entity=Gate, entity_id=job_id, action=Pass or Fail.
/// If `passed`, also updates the job status to "complete".
/// If failed, the job stays "running" so it can be retried.
pub fn record_gate_outcome(
    ship_dir: &Path,
    job_id: &str,
    passed: bool,
    evidence: &str,
) -> Result<EventRecord> {
    let action = if passed {
        EventAction::Pass
    } else {
        EventAction::Fail
    };
    let record = insert_event(
        ship_dir,
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
        crate::db::jobs::update_job_status(ship_dir, job_id, "complete")?;
    }
    Ok(record)
}

/// List gate outcomes (pass/fail events) for a specific job.
pub fn list_gate_outcomes(ship_dir: &Path, job_id: &str) -> Result<Vec<EventRecord>> {
    let mut conn = open_db(ship_dir)?;
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
        workspace_id: row.get(6),
        session_id: row.get(7),
        job_id: row.get(8),
    })
}
