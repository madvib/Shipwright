//! Transactional workspace state + typed event emission.
//!
//! Each public function wraps the workspace DB write and the `events` INSERT
//! in a single SQLite BEGIN/COMMIT block so they succeed or roll back together.
//!
//! ADR GHihs2tn: all workspace lifecycle transitions must emit typed events.

use anyhow::{Context, Result};
use chrono::Utc;
use ulid::Ulid;

use crate::db::types::WorkspaceUpsert;
use crate::db::{block_on, open_db};
use crate::events::types::event_types;
use crate::events::types::{
    WorkspaceActivated, WorkspaceArchived, WorkspaceCompileFailed, WorkspaceCompiled,
};

// ── SQL constants ─────────────────────────────────────────────────────────────

const WORKSPACE_UPSERT: &str =
    "INSERT INTO workspace (branch, id, workspace_type, status, active_agent, \
     providers_json, mcp_servers_json, skills_json, is_worktree, worktree_path, \
     last_activated_at, context_hash, config_generation, compiled_at, compile_error, \
     created_at, updated_at) \
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
     ON CONFLICT(branch) DO UPDATE SET \
       id                = excluded.id, \
       workspace_type    = excluded.workspace_type, \
       status            = excluded.status, \
       active_agent      = excluded.active_agent, \
       providers_json    = excluded.providers_json, \
       mcp_servers_json  = excluded.mcp_servers_json, \
       skills_json       = excluded.skills_json, \
       is_worktree       = excluded.is_worktree, \
       worktree_path     = excluded.worktree_path, \
       last_activated_at = excluded.last_activated_at, \
       context_hash      = excluded.context_hash, \
       config_generation = excluded.config_generation, \
       compiled_at       = excluded.compiled_at, \
       compile_error     = excluded.compile_error, \
       updated_at        = excluded.updated_at";

const EVENT_INSERT: &str =
    "INSERT INTO events \
     (id, event_type, entity_id, actor, payload_json, version, \
      correlation_id, causation_id, workspace_id, session_id, \
      actor_id, parent_actor_id, elevated, created_at) \
     VALUES (?, ?, ?, 'ship', ?, 1, NULL, NULL, ?, NULL, ?, NULL, 1, ?)";

// ── owned bind parameter bundle ───────────────────────────────────────────────

struct WsBind {
    branch: String,
    workspace_id: String,
    workspace_type: String,
    status: String,
    active_agent: Option<String>,
    providers_json: String,
    mcp_servers_json: String,
    skills_json: String,
    is_worktree: i64,
    worktree_path: Option<String>,
    last_activated_at: Option<String>,
    context_hash: Option<String>,
    config_generation: i64,
    compiled_at: Option<String>,
    compile_error: Option<String>,
    now: String,
}

impl WsBind {
    fn from_upsert(record: WorkspaceUpsert<'_>) -> Result<Self> {
        let providers_json = serde_json::to_string(record.providers)
            .context("failed to serialise workspace providers")?;
        let mcp_servers_json = serde_json::to_string(record.mcp_servers)
            .context("failed to serialise workspace mcp servers")?;
        let skills_json = serde_json::to_string(record.skills)
            .context("failed to serialise workspace skills")?;
        Ok(Self {
            branch: record.branch.to_string(),
            workspace_id: record.workspace_id.to_string(),
            workspace_type: record.workspace_type.to_string(),
            status: record.status.to_string(),
            active_agent: record.active_agent.map(str::to_string),
            providers_json,
            mcp_servers_json,
            skills_json,
            is_worktree: if record.is_worktree { 1 } else { 0 },
            worktree_path: record.worktree_path.map(str::to_string),
            last_activated_at: record.last_activated_at.map(str::to_string),
            context_hash: record.context_hash.map(str::to_string),
            config_generation: record.config_generation,
            compiled_at: record.compiled_at.map(str::to_string),
            compile_error: record.compile_error.map(str::to_string),
            now: Utc::now().to_rfc3339(),
        })
    }
}

// ── core transactional write ──────────────────────────────────────────────────

/// Execute workspace upsert + event insert in one BEGIN/COMMIT block.
///
/// Both operations share the same `SqliteConnection`. If the event insert
/// fails, the explicit ROLLBACK ensures the workspace write is not persisted.
fn run_tx<P: serde::Serialize>(
    ws: WsBind,
    event_type: &'static str,
    payload: &P,
) -> Result<()> {
    let payload_json = serde_json::to_string(payload)
        .context("failed to serialise event payload")?;
    let event_id = Ulid::new().to_string();
    let event_ts = Utc::now().to_rfc3339();

    // entity_id, actor_id, and workspace_id all set to the branch name.
    let entity_id = ws.branch.clone();
    let actor_id = ws.branch.clone();
    let workspace_id = ws.branch.clone();

    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;

        let ws_result = sqlx::query(WORKSPACE_UPSERT)
            .bind(&ws.branch)
            .bind(&ws.workspace_id)
            .bind(&ws.workspace_type)
            .bind(&ws.status)
            .bind(&ws.active_agent)
            .bind(&ws.providers_json)
            .bind(&ws.mcp_servers_json)
            .bind(&ws.skills_json)
            .bind(ws.is_worktree)
            .bind(&ws.worktree_path)
            .bind(&ws.last_activated_at)
            .bind(&ws.context_hash)
            .bind(ws.config_generation)
            .bind(&ws.compiled_at)
            .bind(&ws.compile_error)
            .bind(&ws.now)
            .bind(&ws.now)
            .execute(&mut conn)
            .await;

        if let Err(e) = ws_result {
            let _ = sqlx::query("ROLLBACK").execute(&mut conn).await;
            return Err(e);
        }

        let ev_result = sqlx::query(EVENT_INSERT)
            .bind(&event_id)
            .bind(event_type)
            .bind(&entity_id)
            .bind(&payload_json)
            .bind(&workspace_id)
            .bind(&actor_id)
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

// ── public API ────────────────────────────────────────────────────────────────

/// Upsert workspace + emit `workspace.activated` atomically.
pub fn upsert_workspace_activated(
    record: WorkspaceUpsert<'_>,
    payload: &WorkspaceActivated,
) -> Result<()> {
    run_tx(WsBind::from_upsert(record)?, event_types::WORKSPACE_ACTIVATED, payload)
}

/// Upsert workspace + emit `workspace.compiled` atomically.
pub fn upsert_workspace_compiled(
    record: WorkspaceUpsert<'_>,
    payload: &WorkspaceCompiled,
) -> Result<()> {
    run_tx(WsBind::from_upsert(record)?, event_types::WORKSPACE_COMPILED, payload)
}

/// Upsert workspace + emit `workspace.compile_failed` atomically.
pub fn upsert_workspace_compile_failed(
    record: WorkspaceUpsert<'_>,
    payload: &WorkspaceCompileFailed,
) -> Result<()> {
    run_tx(WsBind::from_upsert(record)?, event_types::WORKSPACE_COMPILE_FAILED, payload)
}

/// Upsert workspace + emit `workspace.archived` atomically.
pub fn upsert_workspace_archived(
    record: WorkspaceUpsert<'_>,
    payload: &WorkspaceArchived,
) -> Result<()> {
    run_tx(WsBind::from_upsert(record)?, event_types::WORKSPACE_ARCHIVED, payload)
}
