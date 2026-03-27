//! Transactional session state + typed event emission.
//!
//! Each public function wraps the session DB write and the `events` INSERT
//! in a single SQLite BEGIN/COMMIT block so they succeed or roll back together.
//!
//! ADR GHihs2tn: all session lifecycle transitions must emit typed events.
//! Session is a child actor of its workspace — `parent_actor_id` = workspace ID.

use anyhow::{Context, Result};
use chrono::Utc;
use ulid::Ulid;

use crate::db::types::WorkspaceSessionDb;
use crate::db::{block_on, open_db};
use crate::events::types::event_types;
use crate::events::types::{SessionEnded, SessionProgress, SessionStarted};

// ── SQL constants ─────────────────────────────────────────────────────────────

const SESSION_INSERT: &str =
    "INSERT INTO workspace_session \
     (id, workspace_id, workspace_branch, status, started_at, ended_at, agent_id, \
      primary_provider, goal, summary, updated_workspace_ids_json, compiled_at, \
      compile_error, config_generation_at_start, created_at, updated_at) \
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

const SESSION_UPDATE: &str =
    "UPDATE workspace_session \
     SET workspace_id = ?, workspace_branch = ?, status = ?, started_at = ?, \
         ended_at = ?, agent_id = ?, primary_provider = ?, goal = ?, summary = ?, \
         updated_workspace_ids_json = ?, compiled_at = ?, compile_error = ?, \
         config_generation_at_start = ?, created_at = ?, updated_at = ? \
     WHERE id = ?";

// entity_id, workspace_id, session_id, actor_id, and parent_actor_id are all
// set explicitly per-event. `elevated` is passed as a bind parameter.
const EVENT_INSERT: &str =
    "INSERT INTO events \
     (id, event_type, entity_id, actor, payload_json, version, \
      correlation_id, causation_id, workspace_id, session_id, \
      actor_id, parent_actor_id, elevated, created_at) \
     VALUES (?, ?, ?, 'ship', ?, 1, NULL, NULL, ?, ?, ?, ?, ?, ?)";

// ── owned bind parameter bundle ───────────────────────────────────────────────

struct SessionBind {
    session_id: String,
    workspace_id: String,
    workspace_branch: String,
    status: String,
    started_at: String,
    ended_at: Option<String>,
    agent_id: Option<String>,
    primary_provider: Option<String>,
    goal: Option<String>,
    summary: Option<String>,
    updated_workspace_ids_json: String,
    compiled_at: Option<String>,
    compile_error: Option<String>,
    config_generation_at_start: Option<i64>,
    created_at: String,
    updated_at: String,
}

impl SessionBind {
    fn from_db(session: &WorkspaceSessionDb) -> Result<Self> {
        let updated_workspace_ids_json =
            serde_json::to_string(&session.updated_workspace_ids)
                .context("failed to serialise updated_workspace_ids")?;
        Ok(Self {
            session_id: session.id.clone(),
            workspace_id: session.workspace_id.clone(),
            workspace_branch: session.workspace_branch.clone(),
            status: session.status.clone(),
            started_at: session.started_at.clone(),
            ended_at: session.ended_at.clone(),
            agent_id: session.agent_id.clone(),
            primary_provider: session.primary_provider.clone(),
            goal: session.goal.clone(),
            summary: session.summary.clone(),
            updated_workspace_ids_json,
            compiled_at: session.compiled_at.clone(),
            compile_error: session.compile_error.clone(),
            config_generation_at_start: session.config_generation_at_start,
            created_at: session.created_at.clone(),
            updated_at: session.updated_at.clone(),
        })
    }
}

// ── public API ────────────────────────────────────────────────────────────────

/// Insert session row and emit `session.started` atomically.
///
/// On failure, both the session INSERT and the event INSERT are rolled back.
pub fn insert_session_with_started_event(
    session: &WorkspaceSessionDb,
    payload: &SessionStarted,
) -> Result<()> {
    let sb = SessionBind::from_db(session)?;
    let payload_json = serde_json::to_string(payload)
        .context("failed to serialise SessionStarted payload")?;
    let event_id = Ulid::new().to_string();
    let event_ts = Utc::now().to_rfc3339();

    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;

        let insert_result = sqlx::query(SESSION_INSERT)
            .bind(&sb.session_id)
            .bind(&sb.workspace_id)
            .bind(&sb.workspace_branch)
            .bind(&sb.status)
            .bind(&sb.started_at)
            .bind(&sb.ended_at)
            .bind(&sb.agent_id)
            .bind(&sb.primary_provider)
            .bind(&sb.goal)
            .bind(&sb.summary)
            .bind(&sb.updated_workspace_ids_json)
            .bind(&sb.compiled_at)
            .bind(&sb.compile_error)
            .bind(sb.config_generation_at_start)
            .bind(&sb.created_at)
            .bind(&sb.updated_at)
            .execute(&mut conn)
            .await;

        if let Err(e) = insert_result {
            let _ = sqlx::query("ROLLBACK").execute(&mut conn).await;
            return Err(e);
        }

        let ev_result = sqlx::query(EVENT_INSERT)
            .bind(&event_id)
            .bind(event_types::SESSION_STARTED)
            .bind(&sb.session_id)   // entity_id = session ID
            .bind(&payload_json)
            .bind(&sb.workspace_id) // workspace_id
            .bind(&sb.session_id)   // session_id
            .bind(&sb.session_id)   // actor_id = session ID
            .bind(&sb.workspace_id) // parent_actor_id = workspace ID
            .bind(1_i64)            // elevated = 1
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

/// Emit `session.progress` event atomically (no session row write).
///
/// Progress events are not elevated — too noisy to bubble to workspace level.
pub fn insert_session_progress_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionProgress,
) -> Result<()> {
    let payload_json = serde_json::to_string(payload)
        .context("failed to serialise SessionProgress payload")?;
    let event_id = Ulid::new().to_string();
    let event_ts = Utc::now().to_rfc3339();
    let session_id = session_id.to_string();
    let workspace_id = workspace_id.to_string();

    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;

        let ev_result = sqlx::query(EVENT_INSERT)
            .bind(&event_id)
            .bind(event_types::SESSION_PROGRESS)
            .bind(&session_id)   // entity_id = session ID
            .bind(&payload_json)
            .bind(&workspace_id) // workspace_id
            .bind(&session_id)   // session_id
            .bind(&session_id)   // actor_id = session ID
            .bind(&workspace_id) // parent_actor_id = workspace ID
            .bind(0_i64)         // elevated = 0
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

/// Update session row and emit `session.ended` atomically.
///
/// On failure, both the session UPDATE and the event INSERT are rolled back.
pub fn update_session_with_ended_event(
    session: &WorkspaceSessionDb,
    payload: &SessionEnded,
) -> Result<()> {
    let sb = SessionBind::from_db(session)?;
    let payload_json = serde_json::to_string(payload)
        .context("failed to serialise SessionEnded payload")?;
    let event_id = Ulid::new().to_string();
    let event_ts = Utc::now().to_rfc3339();

    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;

        let update_result = sqlx::query(SESSION_UPDATE)
            .bind(&sb.workspace_id)
            .bind(&sb.workspace_branch)
            .bind(&sb.status)
            .bind(&sb.started_at)
            .bind(&sb.ended_at)
            .bind(&sb.agent_id)
            .bind(&sb.primary_provider)
            .bind(&sb.goal)
            .bind(&sb.summary)
            .bind(&sb.updated_workspace_ids_json)
            .bind(&sb.compiled_at)
            .bind(&sb.compile_error)
            .bind(sb.config_generation_at_start)
            .bind(&sb.created_at)
            .bind(&sb.updated_at)
            .bind(&sb.session_id)
            .execute(&mut conn)
            .await;

        if let Err(e) = update_result {
            let _ = sqlx::query("ROLLBACK").execute(&mut conn).await;
            return Err(e);
        }

        let ev_result = sqlx::query(EVENT_INSERT)
            .bind(&event_id)
            .bind(event_types::SESSION_ENDED)
            .bind(&sb.session_id)   // entity_id = session ID
            .bind(&payload_json)
            .bind(&sb.workspace_id) // workspace_id
            .bind(&sb.session_id)   // session_id
            .bind(&sb.session_id)   // actor_id = session ID
            .bind(&sb.workspace_id) // parent_actor_id = workspace ID
            .bind(1_i64)            // elevated = 1
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

