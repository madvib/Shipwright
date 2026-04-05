//! ActorStore — scoped event handle for a single actor.
//!
//! Wraps an isolated SQLite DB file in the actor's directory. Enforces
//! namespace boundaries on writes and typed-filter reads. The kernel creates
//! `ActorStore` instances via `KernelRouter::spawn_actor`; actors never
//! construct their own.

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use sqlx::{Connection, QueryBuilder, Row};
use sqlx::sqlite::SqliteConnectOptions;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::db::{block_on, open_db_at};
use crate::events::envelope::EventEnvelope;
use crate::events::filter::EventFilter;

const COLS: &str = "id, event_type, entity_id, actor, payload_json, version, \
    causation_id, workspace_id, session_id, \
    actor_id, parent_actor_id, elevated, created_at, target_actor_id";

/// A scoped event handle bound to a single actor's SQLite DB.
///
/// Writes are restricted to `write_namespaces`. Typed-filter queries are
/// restricted to `read_namespaces`. Unfiltered queries always return only
/// this actor's own events.
pub struct ActorStore {
    actor_id: String,
    db_path: PathBuf,
    write_namespaces: Vec<String>,
    read_namespaces: Vec<String>,
}

impl ActorStore {
    pub(crate) fn new(
        actor_id: impl Into<String>,
        db_path: PathBuf,
        write_namespaces: Vec<String>,
        read_namespaces: Vec<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            db_path,
            write_namespaces,
            read_namespaces,
        }
    }

    /// Open (or create) an ActorStore for the given actor.
    ///
    /// `base_dir` is the global Ship directory (e.g. `~/.ship`).
    /// The DB is created at `{base_dir}/actors/{actor_id}/events.db`.
    pub fn open(
        actor_id: &str,
        base_dir: &Path,
        write_namespaces: Vec<String>,
        read_namespaces: Vec<String>,
    ) -> Result<Self> {
        let db_path = base_dir.join("actors").join(actor_id).join("events.db");
        init_actor_db(&db_path)?;
        Ok(Self::new(actor_id, db_path, write_namespaces, read_namespaces))
    }

    pub fn actor_id(&self) -> &str {
        &self.actor_id
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Persist an event. Rejects `event_type` outside `write_namespaces`.
    pub fn append(&self, event: &EventEnvelope) -> Result<()> {
        self.check_write_namespace(&event.event_type)?;
        raw_append(&self.db_path, event)
    }

    /// Query this actor's store only.
    ///
    /// If `filter.event_type` is set it must be covered by `read_namespaces`.
    /// Unfiltered queries return all events in this actor's DB.
    pub fn query(&self, filter: &EventFilter) -> Result<Vec<EventEnvelope>> {
        if let Some(ref et) = filter.event_type {
            self.check_read_namespace(et)?;
        }
        query_db(&self.db_path, filter)
    }

    fn check_write_namespace(&self, event_type: &str) -> Result<()> {
        if self
            .write_namespaces
            .iter()
            .any(|ns| event_type.starts_with(ns.as_str()))
        {
            return Ok(());
        }
        Err(anyhow!(
            "write namespace violation: '{}' not permitted for actor '{}'",
            event_type,
            self.actor_id
        ))
    }

    fn check_read_namespace(&self, event_type: &str) -> Result<()> {
        if self
            .read_namespaces
            .iter()
            .any(|ns| event_type.starts_with(ns.as_str()))
        {
            return Ok(());
        }
        Err(anyhow!(
            "read namespace violation: '{}' not permitted for actor '{}'",
            event_type,
            self.actor_id
        ))
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Write an event to a DB path without namespace enforcement.
/// Used by `ActorStore::append` (after it has already checked namespaces)
/// and by `KernelRouter::route` for the kernel store.
pub(crate) fn raw_append(db_path: &Path, event: &EventEnvelope) -> Result<()> {
    let mut conn = open_db_at(db_path)?;
    let created_at = event.created_at.to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO events \
             (id, event_type, entity_id, actor, payload_json, version, \
              causation_id, workspace_id, session_id, \
              actor_id, parent_actor_id, elevated, created_at, target_actor_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&event.id)
        .bind(&event.event_type)
        .bind(&event.entity_id)
        .bind(&event.actor)
        .bind(&event.payload_json)
        .bind(event.version as i64)
        .bind(&event.causation_id)
        .bind(&event.workspace_id)
        .bind(&event.session_id)
        .bind(&event.actor_id)
        .bind(&event.parent_actor_id)
        .bind(event.elevated as i64)
        .bind(&created_at)
        .bind(&event.target_actor_id)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

fn query_db(db_path: &Path, filter: &EventFilter) -> Result<Vec<EventEnvelope>> {
    let mut conn = open_db_at(db_path)?;
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

fn row_to_envelope(row: &sqlx::sqlite::SqliteRow) -> Result<EventEnvelope> {
    let created_at_str: String = row.get(12);
    let created_at = created_at_str
        .parse::<DateTime<Utc>>()
        .or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
        })
        .map_err(|e| anyhow!("invalid created_at '{}': {}", created_at_str, e))?;
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
        target_actor_id: row.get(13),
    })
}

/// Initialise an actor's SQLite DB at `path`, running actor-scoped migrations.
/// Idempotent — safe to call on an already-initialised DB.
pub(crate) fn init_actor_db(path: &Path) -> Result<()> {
    use anyhow::Context;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create actor db dir: {parent:?}"))?;
    }

    let raw_cow = path.to_string_lossy();
    let raw_str = raw_cow.replace('\\', "/");
    let raw = if raw_str.starts_with('/') {
        raw_str
    } else {
        format!("/{raw_str}")
    };
    let url = format!("sqlite://{raw}");

    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .create_if_missing(true);
    let mut conn = block_on(async { sqlx::SqliteConnection::connect_with(&opts).await })?;

    block_on(async {
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&mut conn)
            .await
    })?;
    block_on(async {
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut conn)
            .await
    })?;

    let migrate_result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| {
            handle
                .block_on(sqlx::migrate!("./migrations/actor").run(&mut conn))
                .map_err(|e| anyhow!("actor migration failed: {e}"))
        })
    } else {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .map_err(|e| anyhow!("failed to create runtime: {e}"))?;
        rt.block_on(sqlx::migrate!("./migrations/actor").run(&mut conn))
            .map_err(|e| anyhow!("actor migration failed: {e}"))
    };
    migrate_result
}
