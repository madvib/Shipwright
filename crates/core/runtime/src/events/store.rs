use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, Row};
use std::path::{Path, PathBuf};

use crate::db::{block_on, db_path, ensure_db, open_db_at};
use crate::events::envelope::EventEnvelope;
use crate::events::filter::EventFilter;

pub trait EventStore: Send + Sync {
    fn append(&self, event: &EventEnvelope) -> Result<()>;
    fn get(&self, id: &str) -> Result<Option<EventEnvelope>>;
    fn query(&self, filter: &EventFilter) -> Result<Vec<EventEnvelope>>;
    fn query_aggregate(&self, entity_id: &str) -> Result<Vec<EventEnvelope>>;
    fn query_correlation(&self, correlation_id: &str) -> Result<Vec<EventEnvelope>>;
}

pub struct SqliteEventStore {
    db_path: PathBuf,
}

impl SqliteEventStore {
    pub fn new() -> Result<Self> {
        ensure_db()?;
        Ok(Self { db_path: db_path()? })
    }
}

const COLS: &str = "id, event_type, entity_id, actor, payload_json, version, \
    correlation_id, causation_id, workspace_id, session_id, \
    actor_id, parent_actor_id, elevated, created_at";

/// Query a single DB file with the given filter. Returns empty vec if the DB
/// does not have an events table (e.g. an uninitialized file).
fn query_one_db(path: &Path, filter: &EventFilter) -> Result<Vec<EventEnvelope>> {
    let mut conn = open_db_at(path)?;
    let mut qb: QueryBuilder<sqlx::Sqlite> =
        QueryBuilder::new(format!("SELECT {COLS} FROM events WHERE 1=1"));

    if let Some(ref v) = filter.entity_id {
        qb.push(" AND entity_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.event_type {
        qb.push(" AND event_type = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.workspace_id {
        qb.push(" AND workspace_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.session_id {
        qb.push(" AND session_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.correlation_id {
        qb.push(" AND correlation_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.actor_id {
        qb.push(" AND actor_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.parent_actor_id {
        qb.push(" AND parent_actor_id = ").push_bind(v.clone());
    }
    if filter.elevated_only {
        qb.push(" AND elevated = 1");
    }
    if let Some(ref since) = filter.since {
        qb.push(" AND created_at >= ").push_bind(since.to_rfc3339());
    }
    qb.push(" ORDER BY id ASC");
    if let Some(limit) = filter.limit {
        qb.push(" LIMIT ").push_bind(limit as i64);
    }

    let rows = block_on(async { qb.build().fetch_all(&mut conn).await })?;
    rows.iter().map(row_to_envelope).collect()
}

/// Enumerate all workspace DB files under `{global_dir}/workspaces/*/events.db`.
fn workspace_db_paths() -> Vec<PathBuf> {
    let Ok(global_dir) = crate::project::get_global_dir() else {
        return Vec::new();
    };
    let ws_root = global_dir.join("workspaces");
    if !ws_root.is_dir() {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(&ws_root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|e| e.path().join("events.db"))
        .filter(|p| p.is_file())
        .collect()
}

impl EventStore for SqliteEventStore {
    fn append(&self, event: &EventEnvelope) -> Result<()> {
        let mut conn = open_db_at(&self.db_path)?;
        let created_at = event.created_at.to_rfc3339();
        block_on(async {
            sqlx::query(
                "INSERT INTO events \
                 (id, event_type, entity_id, actor, payload_json, version, \
                  correlation_id, causation_id, workspace_id, session_id, \
                  actor_id, parent_actor_id, elevated, created_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&event.id)
            .bind(&event.event_type)
            .bind(&event.entity_id)
            .bind(&event.actor)
            .bind(&event.payload_json)
            .bind(event.version as i64)
            .bind(&event.correlation_id)
            .bind(&event.causation_id)
            .bind(&event.workspace_id)
            .bind(&event.session_id)
            .bind(&event.actor_id)
            .bind(&event.parent_actor_id)
            .bind(event.elevated as i64)
            .bind(&created_at)
            .execute(&mut conn)
            .await
        })?;
        Ok(())
    }

    fn get(&self, id: &str) -> Result<Option<EventEnvelope>> {
        // Try platform DB first.
        let mut conn = open_db_at(&self.db_path)?;
        let row = block_on(async {
            sqlx::query(&format!("SELECT {COLS} FROM events WHERE id = ?"))
                .bind(id)
                .fetch_optional(&mut conn)
                .await
        })?;
        if let Some(r) = row {
            return row_to_envelope(&r).map(Some);
        }

        // Fall back to workspace DBs.
        let id = id.to_string();
        for ws_path in workspace_db_paths() {
            if let Ok(mut ws_conn) = open_db_at(&ws_path) {
                let row = block_on(async {
                    sqlx::query(&format!("SELECT {COLS} FROM events WHERE id = ?"))
                        .bind(&id)
                        .fetch_optional(&mut ws_conn)
                        .await
                });
                if let Ok(Some(r)) = row {
                    return row_to_envelope(&r).map(Some);
                }
            }
        }
        Ok(None)
    }

    fn query(&self, filter: &EventFilter) -> Result<Vec<EventEnvelope>> {
        let mut results = query_one_db(&self.db_path, filter)?;

        // Also query all workspace DBs so workspace-local events are visible.
        for ws_path in workspace_db_paths() {
            if let Ok(ws_events) = query_one_db(&ws_path, filter) {
                results.extend(ws_events);
            }
        }

        // Re-sort by ULID id (encodes insertion time).
        results.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(results)
    }

    fn query_aggregate(&self, entity_id: &str) -> Result<Vec<EventEnvelope>> {
        self.query(&EventFilter {
            entity_id: Some(entity_id.to_string()),
            ..Default::default()
        })
    }

    fn query_correlation(&self, correlation_id: &str) -> Result<Vec<EventEnvelope>> {
        self.query(&EventFilter {
            correlation_id: Some(correlation_id.to_string()),
            ..Default::default()
        })
    }
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
