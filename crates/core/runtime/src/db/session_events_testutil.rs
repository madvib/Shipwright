//! Test-only helpers for session event atomicity verification.
//!
//! These functions bypass the operational session row write so tests can
//! exercise event-only transaction semantics without needing a real session.

use anyhow::{Context, Result};
use chrono::Utc;
use ulid::Ulid;

use crate::db::{block_on, open_db};
use crate::events::types::event_types;
use crate::events::types::SessionStarted;

const EVENT_INSERT: &str =
    "INSERT INTO events \
     (id, event_type, entity_id, actor, payload_json, version, \
      causation_id, workspace_id, session_id, \
      actor_id, parent_actor_id, elevated, created_at) \
     VALUES (?, ?, ?, 'ship', ?, 1, NULL, ?, ?, ?, ?, ?, ?)";

/// Emit only a `session.started` event (no session row write).
///
/// Used in the atomicity test to verify event-only transaction semantics.
pub fn insert_session_started_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionStarted,
) -> Result<()> {
    let payload_json = serde_json::to_string(payload)
        .context("failed to serialise SessionStarted payload")?;
    let event_id = Ulid::new().to_string();
    let event_ts = Utc::now().to_rfc3339();
    let session_id = session_id.to_string();
    let workspace_id = workspace_id.to_string();

    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;

        let ev_result = sqlx::query(EVENT_INSERT)
            .bind(&event_id)
            .bind(event_types::SESSION_STARTED)
            .bind(&session_id)   // entity_id = session ID
            .bind(&payload_json)
            .bind(&workspace_id) // workspace_id
            .bind(&session_id)   // session_id
            .bind(&session_id)   // actor_id = session ID
            .bind(&workspace_id) // parent_actor_id = workspace ID
            .bind(1_i64)         // elevated = 1
            .bind(&event_ts)
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
