//! Actor state projection — derived from actor.* events.
//!
//! This projection maintains the `actors` table in per-workspace DBs as a read
//! model. It handles every actor event type and applies the state change.
//! On rebuild, the table is truncated and replayed from the event log.

use anyhow::Result;
use sqlx::SqliteConnection;

use super::registry::Projection;
use crate::db::block_on;
use crate::events::types::event_types;
use crate::events::EventEnvelope;

/// Projection that maintains the actors table from actor.* events.
pub struct ActorProjection;

impl ActorProjection {
    pub fn new() -> Self {
        Self
    }
}

const HANDLED: &[&str] = &[
    event_types::ACTOR_CREATED,
    event_types::ACTOR_WOKE,
    event_types::ACTOR_SLEPT,
    event_types::ACTOR_STOPPED,
    event_types::ACTOR_CRASHED,
];

impl Projection for ActorProjection {
    fn name(&self) -> &str {
        "actor_state"
    }

    fn event_types(&self) -> &[&str] {
        HANDLED
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        let entity = &event.entity_id;
        match event.event_type.as_str() {
            event_types::ACTOR_CREATED => apply_created(entity, &event.payload_json, conn),
            event_types::ACTOR_WOKE => apply_status(entity, "active", conn),
            event_types::ACTOR_SLEPT => apply_status(entity, "sleeping", conn),
            event_types::ACTOR_STOPPED => apply_status(entity, "stopped", conn),
            event_types::ACTOR_CRASHED => apply_crashed(entity, &event.payload_json, conn),
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async {
            sqlx::query("DELETE FROM actors").execute(conn).await?;
            Ok(())
        })
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct CreatedPayload {
    kind: String,
    environment_type: String,
    #[serde(default)]
    workspace_id: Option<String>,
    #[serde(default)]
    parent_actor_id: Option<String>,
    #[serde(default)]
    restart_count: u32,
}

fn apply_created(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: CreatedPayload = serde_json::from_str(payload_json)?;
    block_on(async {
        sqlx::query(
            "INSERT OR REPLACE INTO actors \
             (id, kind, environment_type, status, workspace_id, parent_actor_id, \
              restart_count, created_at, updated_at) \
             VALUES (?, ?, ?, 'created', ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(entity_id)
        .bind(&p.kind)
        .bind(&p.environment_type)
        .bind(&p.workspace_id)
        .bind(&p.parent_actor_id)
        .bind(p.restart_count as i64)
        .execute(conn)
        .await?;
        Ok(())
    })
}

fn apply_status(entity_id: &str, status: &str, conn: &mut SqliteConnection) -> Result<()> {
    block_on(async {
        sqlx::query("UPDATE actors SET status = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(status)
            .bind(entity_id)
            .execute(conn)
            .await?;
        Ok(())
    })
}

#[derive(serde::Deserialize)]
struct CrashedPayload {
    #[allow(dead_code)]
    error: String,
    restart_count: u32,
}

fn apply_crashed(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: CrashedPayload = serde_json::from_str(payload_json)?;
    block_on(async {
        sqlx::query(
            "UPDATE actors SET status = 'crashed', restart_count = ?, \
             updated_at = datetime('now') WHERE id = ?",
        )
        .bind(p.restart_count as i64)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}
