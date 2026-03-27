//! Transactional actor state + typed event emission.
//!
//! Each public function wraps the actors DB write and the `events` INSERT
//! in a single SQLite BEGIN/COMMIT block so they succeed or roll back together.
//!
//! ADR GHihs2tn: write path is BEGIN IMMEDIATE → UPDATE/INSERT actors →
//! INSERT INTO events → COMMIT.  All actor lifecycle events are elevated=1.

use anyhow::{Context, Result};
use chrono::Utc;
use ulid::Ulid;

use crate::db::{block_on, open_db};
use crate::events::types::event_types;
use crate::events::types::{ActorCrashed, ActorCreated, ActorSlept, ActorStopped, ActorWoke};

// ── SQL constants ─────────────────────────────────────────────────────────────

const ACTOR_INSERT: &str =
    "INSERT OR REPLACE INTO actors \
     (id, kind, environment_type, status, workspace_id, parent_actor_id, \
      restart_count, created_at, updated_at) \
     VALUES (?, ?, ?, 'created', ?, ?, ?, datetime('now'), datetime('now'))";

const ACTOR_UPDATE_STATUS: &str =
    "UPDATE actors SET status = ?, updated_at = ? WHERE id = ?";

const ACTOR_UPDATE_CRASHED: &str =
    "UPDATE actors SET status = 'crashed', restart_count = ?, updated_at = ? WHERE id = ?";

// elevated is always 1 for actor lifecycle events (they bubble to workspace).
const EVENT_INSERT: &str =
    "INSERT INTO events \
     (id, event_type, entity_id, actor, payload_json, version, \
      correlation_id, causation_id, workspace_id, session_id, \
      actor_id, parent_actor_id, elevated, created_at) \
     VALUES (?, ?, ?, 'ship', ?, 1, NULL, NULL, ?, NULL, ?, ?, 1, ?)";

// ── public data type ──────────────────────────────────────────────────────────

/// Caller-supplied fields for creating a new actor row.
pub struct ActorUpsert<'a> {
    pub id: &'a str,
    pub kind: &'a str,
    pub environment_type: &'a str,
    pub workspace_id: Option<&'a str>,
    pub parent_actor_id: Option<&'a str>,
    pub restart_count: u32,
}

// ── private transactional helpers ─────────────────────────────────────────────

/// Emit a lifecycle event for an actor with no preceding row write.
///
/// Used for woke / slept / stopped where only the status UPDATE is needed
/// before the event.
fn run_status_tx<P: serde::Serialize>(
    id: &str,
    new_status: &str,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
    event_type: &'static str,
    payload: &P,
) -> Result<()> {
    let payload_json =
        serde_json::to_string(payload).context("failed to serialise actor event payload")?;
    let event_id = Ulid::new().to_string();
    let now = Utc::now().to_rfc3339();
    let id = id.to_string();
    let new_status = new_status.to_string();
    let workspace_id = workspace_id.map(str::to_string);
    let parent_actor_id = parent_actor_id.map(str::to_string);

    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;

        let update_result = sqlx::query(ACTOR_UPDATE_STATUS)
            .bind(&new_status)
            .bind(&now)
            .bind(&id)
            .execute(&mut conn)
            .await;

        if let Err(e) = update_result {
            let _ = sqlx::query("ROLLBACK").execute(&mut conn).await;
            return Err(e);
        }

        let ev_result = sqlx::query(EVENT_INSERT)
            .bind(&event_id)
            .bind(event_type)
            .bind(&id)          // entity_id
            .bind(&payload_json)
            .bind(&workspace_id)
            .bind(&id)          // actor_id
            .bind(&parent_actor_id)
            .bind(&now)
            .execute(&mut conn)
            .await;

        if let Err(e) = ev_result {
            let _ = sqlx::query("ROLLBACK").execute(&mut conn).await;
            return Err(e);
        }

        sqlx::query("COMMIT").execute(&mut conn).await?;
        Ok(())
    })
}

// ── public API ────────────────────────────────────────────────────────────────

/// INSERT OR REPLACE actors row (status='created') + emit `actor.created` atomically.
pub fn insert_actor_created(upsert: &ActorUpsert<'_>, payload: &ActorCreated) -> Result<()> {
    let payload_json =
        serde_json::to_string(payload).context("failed to serialise ActorCreated payload")?;
    let event_id = Ulid::new().to_string();
    let now = Utc::now().to_rfc3339();
    let id = upsert.id.to_string();
    let kind = upsert.kind.to_string();
    let environment_type = upsert.environment_type.to_string();
    let workspace_id = upsert.workspace_id.map(str::to_string);
    let parent_actor_id = upsert.parent_actor_id.map(str::to_string);
    let restart_count = upsert.restart_count as i64;

    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;

        let insert_result = sqlx::query(ACTOR_INSERT)
            .bind(&id)
            .bind(&kind)
            .bind(&environment_type)
            .bind(&workspace_id)
            .bind(&parent_actor_id)
            .bind(restart_count)
            .execute(&mut conn)
            .await;

        if let Err(e) = insert_result {
            let _ = sqlx::query("ROLLBACK").execute(&mut conn).await;
            return Err(e);
        }

        let ev_result = sqlx::query(EVENT_INSERT)
            .bind(&event_id)
            .bind(event_types::ACTOR_CREATED)
            .bind(&id)           // entity_id
            .bind(&payload_json)
            .bind(&workspace_id)
            .bind(&id)           // actor_id
            .bind(&parent_actor_id)
            .bind(&now)
            .execute(&mut conn)
            .await;

        if let Err(e) = ev_result {
            let _ = sqlx::query("ROLLBACK").execute(&mut conn).await;
            return Err(e);
        }

        sqlx::query("COMMIT").execute(&mut conn).await?;
        Ok(())
    })
}

/// UPDATE actors SET status='active' + emit `actor.woke` atomically.
pub fn update_actor_woke(
    id: &str,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    run_status_tx(id, "active", workspace_id, parent_actor_id, event_types::ACTOR_WOKE, &ActorWoke {})
}

/// UPDATE actors SET status='sleeping' + emit `actor.slept` atomically.
pub fn update_actor_slept(
    id: &str,
    idle_secs: u64,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    run_status_tx(
        id,
        "sleeping",
        workspace_id,
        parent_actor_id,
        event_types::ACTOR_SLEPT,
        &ActorSlept { idle_secs },
    )
}

/// UPDATE actors SET status='stopped' + emit `actor.stopped` atomically.
pub fn update_actor_stopped(
    id: &str,
    reason: &str,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    run_status_tx(
        id,
        "stopped",
        workspace_id,
        parent_actor_id,
        event_types::ACTOR_STOPPED,
        &ActorStopped { reason: reason.to_string() },
    )
}

/// UPDATE actors SET status='crashed', restart_count+=1 + emit `actor.crashed` atomically.
pub fn update_actor_crashed(
    id: &str,
    error: &str,
    restart_count: u32,
    workspace_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    let payload_json = serde_json::to_string(&ActorCrashed {
        error: error.to_string(),
        restart_count,
    })
    .context("failed to serialise ActorCrashed payload")?;
    let event_id = Ulid::new().to_string();
    let now = Utc::now().to_rfc3339();
    let id = id.to_string();
    let workspace_id = workspace_id.map(str::to_string);
    let parent_actor_id = parent_actor_id.map(str::to_string);
    let new_count = restart_count as i64;

    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;

        let update_result = sqlx::query(ACTOR_UPDATE_CRASHED)
            .bind(new_count)
            .bind(&now)
            .bind(&id)
            .execute(&mut conn)
            .await;

        if let Err(e) = update_result {
            let _ = sqlx::query("ROLLBACK").execute(&mut conn).await;
            return Err(e);
        }

        let ev_result = sqlx::query(EVENT_INSERT)
            .bind(&event_id)
            .bind(event_types::ACTOR_CRASHED)
            .bind(&id)           // entity_id
            .bind(&payload_json)
            .bind(&workspace_id)
            .bind(&id)           // actor_id
            .bind(&parent_actor_id)
            .bind(&now)
            .execute(&mut conn)
            .await;

        if let Err(e) = ev_result {
            let _ = sqlx::query("ROLLBACK").execute(&mut conn).await;
            return Err(e);
        }

        sqlx::query("COMMIT").execute(&mut conn).await?;
        Ok(())
    })
}
