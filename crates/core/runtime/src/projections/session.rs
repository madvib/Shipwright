//! Session state projection — derived from session.* events.
//!
//! This projection maintains the `workspace_session` table in platform.db as a
//! read model. It handles session.started (INSERT) and session.ended (UPDATE).
//! session.progress is deliberately not handled — progress events don't affect
//! the session row.

use anyhow::Result;
use sqlx::SqliteConnection;

use super::async_projection::AsyncProjection;
use super::registry::Projection;
use crate::db::block_on;
use crate::events::types::event_types;
use crate::events::EventEnvelope;

/// Projection that maintains the workspace_session table from session.* events.
pub struct SessionProjection;

impl SessionProjection {
    pub fn new() -> Self {
        Self
    }
}

const HANDLED: &[&str] = &[
    event_types::SESSION_STARTED,
    event_types::SESSION_ENDED,
    event_types::SESSION_RECORDED,
    event_types::SESSION_DRAIN_STARTED,
    event_types::SESSION_DRAIN_COMPLETED,
    event_types::SESSION_DRAIN_ABORTED,
    event_types::SESSION_TOOL_COUNT_INCREMENTED,
];

impl Projection for SessionProjection {
    fn name(&self) -> &str {
        "session_state"
    }

    fn event_types(&self) -> &[&str] {
        HANDLED
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        match event.event_type.as_str() {
            event_types::SESSION_STARTED => apply_started(event, conn),
            event_types::SESSION_ENDED => apply_ended(event, conn),
            event_types::SESSION_RECORDED => apply_recorded(event, conn),
            event_types::SESSION_DRAIN_STARTED => apply_drain_started(event, conn),
            event_types::SESSION_DRAIN_COMPLETED => apply_drain_completed(event, conn),
            event_types::SESSION_DRAIN_ABORTED => apply_drain_aborted(event, conn),
            event_types::SESSION_TOOL_COUNT_INCREMENTED => apply_tool_count_incremented(event, conn),
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async {
            sqlx::query("DELETE FROM workspace_session")
                .execute(conn)
                .await?;
            Ok(())
        })
    }
}

impl AsyncProjection for SessionProjection {
    fn name(&self) -> &str {
        Projection::name(self)
    }
    fn event_types(&self) -> &[&str] {
        Projection::event_types(self)
    }
    fn apply(&self, event: &EventEnvelope, conn: &mut sqlx::SqliteConnection) -> anyhow::Result<()> {
        Projection::apply(self, event, conn)
    }
    fn truncate(&self, conn: &mut sqlx::SqliteConnection) -> anyhow::Result<()> {
        Projection::truncate(self, conn)
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct StartedPayload {
    #[serde(default)]
    goal: Option<String>,
    workspace_id: String,
    workspace_branch: String,
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    primary_provider: Option<String>,
    #[serde(default)]
    config_generation_at_start: Option<i64>,
    #[serde(default)]
    compiled_at: Option<String>,
}

fn apply_started(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let p: StartedPayload = serde_json::from_str(&event.payload_json)?;
    let now = event.created_at.to_rfc3339();
    let session_id = &event.entity_id;

    block_on(async {
        sqlx::query(
            "INSERT INTO workspace_session \
             (id, workspace_id, workspace_branch, status, started_at, ended_at, \
              agent_id, primary_provider, goal, summary, \
              updated_workspace_ids_json, compiled_at, compile_error, \
              config_generation_at_start, created_at, updated_at) \
             VALUES (?, ?, ?, 'active', ?, NULL, ?, ?, ?, NULL, '[]', ?, NULL, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
               workspace_id = excluded.workspace_id, \
               workspace_branch = excluded.workspace_branch, \
               status = excluded.status, \
               started_at = excluded.started_at, \
               agent_id = excluded.agent_id, \
               primary_provider = excluded.primary_provider, \
               goal = excluded.goal, \
               compiled_at = excluded.compiled_at, \
               config_generation_at_start = excluded.config_generation_at_start, \
               updated_at = excluded.updated_at",
        )
        .bind(session_id)
        .bind(&p.workspace_id)
        .bind(&p.workspace_branch)
        .bind(&now)
        .bind(&p.agent_id)
        .bind(&p.primary_provider)
        .bind(&p.goal)
        .bind(&p.compiled_at)
        .bind(p.config_generation_at_start)
        .bind(&now)
        .bind(&now)
        .execute(conn)
        .await?;
        Ok(())
    })
}

#[derive(serde::Deserialize)]
struct EndedPayload {
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    duration_secs: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)]
    gate_result: Option<String>,
    #[serde(default)]
    updated_workspace_ids: Vec<String>,
    #[serde(default)]
    compile_error: Option<String>,
}

fn apply_ended(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let p: EndedPayload = serde_json::from_str(&event.payload_json)?;
    let now = event.created_at.to_rfc3339();
    let session_id = &event.entity_id;
    let updated_ids_json = serde_json::to_string(&p.updated_workspace_ids)?;

    block_on(async {
        sqlx::query(
            "UPDATE workspace_session SET \
             status = 'ended', \
             ended_at = ?, \
             summary = ?, \
             updated_workspace_ids_json = ?, \
             compile_error = ?, \
             updated_at = ? \
             WHERE id = ?",
        )
        .bind(&now)
        .bind(&p.summary)
        .bind(&updated_ids_json)
        .bind(&p.compile_error)
        .bind(&now)
        .bind(session_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

#[derive(serde::Deserialize)]
struct RecordedPayload {
    record_id: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    updated_workspace_ids: Vec<String>,
    #[serde(default)]
    duration_secs: Option<i64>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    files_changed: Option<i64>,
    #[serde(default)]
    gate_result: Option<String>,
    workspace_branch: String,
}

fn apply_recorded(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let p: RecordedPayload = serde_json::from_str(&event.payload_json)?;
    let now = event.created_at.to_rfc3339();
    let session_id = &event.entity_id;
    let workspace_id = event.workspace_id.as_deref().unwrap_or("");
    let updated_ids_json = serde_json::to_string(&p.updated_workspace_ids)?;

    block_on(async {
        sqlx::query(
            "INSERT INTO workspace_session_record \
             (id, session_id, workspace_id, workspace_branch, summary, \
              updated_workspace_ids_json, duration_secs, provider, model, \
              agent_id, files_changed, gate_result, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(session_id) DO UPDATE SET \
               summary = excluded.summary, \
               updated_workspace_ids_json = excluded.updated_workspace_ids_json, \
               duration_secs = excluded.duration_secs, \
               provider = excluded.provider, \
               model = excluded.model, \
               agent_id = excluded.agent_id, \
               files_changed = excluded.files_changed, \
               gate_result = excluded.gate_result",
        )
        .bind(&p.record_id)
        .bind(session_id)
        .bind(workspace_id)
        .bind(&p.workspace_branch)
        .bind(&p.summary)
        .bind(&updated_ids_json)
        .bind(p.duration_secs)
        .bind(&p.provider)
        .bind(&p.model)
        .bind(&p.agent_id)
        .bind(p.files_changed)
        .bind(&p.gate_result)
        .bind(&now)
        .execute(conn)
        .await?;
        Ok(())
    })
}

// ── Drain handlers ──────────────────────────────────────────────────────────

fn apply_drain_started(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct P { drained_at: String }
    let p: P = serde_json::from_str(&event.payload_json)?;
    let now = event.created_at.to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace_session SET status = 'draining', drained_at = ?, updated_at = ? \
             WHERE id = ? AND status IN ('active', 'draining')",
        )
        .bind(&p.drained_at)
        .bind(&now)
        .bind(&event.entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

fn apply_drain_completed(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct P { ended_at: String }
    let p: P = serde_json::from_str(&event.payload_json)?;
    let now = event.created_at.to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace_session SET status = 'ended', ended_at = ?, updated_at = ? \
             WHERE id = ? AND status = 'draining'",
        )
        .bind(&p.ended_at)
        .bind(&now)
        .bind(&event.entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

fn apply_drain_aborted(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let now = event.created_at.to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace_session SET status = 'active', drained_at = NULL, updated_at = ? \
             WHERE id = ? AND status IN ('draining', 'active')",
        )
        .bind(&now)
        .bind(&event.entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

fn apply_tool_count_incremented(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    block_on(async {
        sqlx::query(
            "UPDATE workspace_session SET tool_call_count = tool_call_count + 1 WHERE id = ?",
        )
        .bind(&event.entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}
