//! Workspace state projection — derived from workspace.* events.
//!
//! This projection maintains the `workspace` table as a read model.
//! It handles every workspace event type and applies the state change.
//! On rebuild, the table is truncated and replayed from the event log.

use anyhow::Result;
use sqlx::SqliteConnection;

use super::registry::Projection;
use crate::db::block_on;
use crate::events::types::event_types;
use crate::events::EventEnvelope;

/// Projection that maintains the workspace table from workspace.* events.
pub struct WorkspaceProjection;

impl WorkspaceProjection {
    pub fn new() -> Self {
        Self
    }
}

const HANDLED: &[&str] = &[
    event_types::WORKSPACE_CREATED,
    event_types::WORKSPACE_ACTIVATED,
    event_types::WORKSPACE_COMPILED,
    event_types::WORKSPACE_COMPILE_FAILED,
    event_types::WORKSPACE_ARCHIVED,
    event_types::WORKSPACE_STATUS_CHANGED,
    event_types::WORKSPACE_AGENT_CHANGED,
    event_types::WORKSPACE_DELETED,
];

impl Projection for WorkspaceProjection {
    fn name(&self) -> &str {
        "workspace_state"
    }

    fn event_types(&self) -> &[&str] {
        HANDLED
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        let entity = &event.entity_id;
        match event.event_type.as_str() {
            event_types::WORKSPACE_CREATED => apply_created(entity, &event.payload_json, conn),
            event_types::WORKSPACE_ACTIVATED => apply_activated(entity, &event.payload_json, conn),
            event_types::WORKSPACE_COMPILED => apply_compiled(entity, &event.payload_json, &event.created_at, conn),
            event_types::WORKSPACE_COMPILE_FAILED => apply_compile_failed(entity, &event.payload_json, &event.created_at, conn),
            event_types::WORKSPACE_ARCHIVED => apply_status(entity, "archived", conn),
            event_types::WORKSPACE_STATUS_CHANGED => apply_status_changed(entity, &event.payload_json, conn),
            event_types::WORKSPACE_AGENT_CHANGED => apply_agent_changed(entity, &event.payload_json, conn),
            event_types::WORKSPACE_DELETED => apply_deleted(entity, conn),
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async {
            sqlx::query("DELETE FROM workspace")
                .execute(conn)
                .await?;
            Ok(())
        })
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// workspace.created payload must carry enough to INSERT the full row.
#[derive(serde::Deserialize)]
struct CreatedPayload {
    workspace_type: String,
    status: String,
    #[serde(default)]
    active_agent: Option<String>,
    #[serde(default)]
    providers: Vec<String>,
    #[serde(default)]
    mcp_servers: Vec<String>,
    #[serde(default)]
    skills: Vec<String>,
    #[serde(default)]
    is_worktree: bool,
    #[serde(default)]
    worktree_path: Option<String>,
}

fn apply_created(entity_id: &str, payload_json: &str, conn: &mut SqliteConnection) -> Result<()> {
    let p: CreatedPayload = serde_json::from_str(payload_json)?;
    let now = chrono::Utc::now().to_rfc3339();
    let providers = serde_json::to_string(&p.providers)?;
    let mcp_servers = serde_json::to_string(&p.mcp_servers)?;
    let skills = serde_json::to_string(&p.skills)?;
    block_on(async {
        sqlx::query(
            "INSERT OR IGNORE INTO workspace \
             (branch, id, workspace_type, status, active_agent, \
              providers_json, mcp_servers_json, skills_json, \
              is_worktree, worktree_path, config_generation, \
              created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?)",
        )
        .bind(entity_id)
        .bind(entity_id) // id = entity_id for now
        .bind(&p.workspace_type)
        .bind(&p.status)
        .bind(&p.active_agent)
        .bind(&providers)
        .bind(&mcp_servers)
        .bind(&skills)
        .bind(p.is_worktree)
        .bind(&p.worktree_path)
        .bind(&now)
        .bind(&now)
        .execute(conn)
        .await?;
        Ok(())
    })
}

#[derive(serde::Deserialize)]
struct ActivatedPayload {
    agent_id: Option<String>,
    providers: Vec<String>,
}

fn apply_activated(entity_id: &str, payload_json: &str, conn: &mut SqliteConnection) -> Result<()> {
    let p: ActivatedPayload = serde_json::from_str(payload_json)?;
    let now = chrono::Utc::now().to_rfc3339();
    let providers = serde_json::to_string(&p.providers)?;
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET status = 'active', active_agent = ?, \
             providers_json = ?, last_activated_at = ?, updated_at = ? \
             WHERE branch = ?",
        )
        .bind(&p.agent_id)
        .bind(&providers)
        .bind(&now)
        .bind(&now)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

#[derive(serde::Deserialize)]
struct CompiledPayload {
    config_generation: u32,
    #[allow(dead_code)]
    duration_ms: u64,
}

fn apply_compiled(
    entity_id: &str,
    payload_json: &str,
    event_time: &chrono::DateTime<chrono::Utc>,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: CompiledPayload = serde_json::from_str(payload_json)?;
    let compiled_at = event_time.to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET compiled_at = ?, compile_error = NULL, \
             config_generation = ?, updated_at = ? WHERE branch = ?",
        )
        .bind(&compiled_at)
        .bind(p.config_generation as i64)
        .bind(&compiled_at)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

#[derive(serde::Deserialize)]
struct CompileFailedPayload {
    error: String,
}

fn apply_compile_failed(
    entity_id: &str,
    payload_json: &str,
    event_time: &chrono::DateTime<chrono::Utc>,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: CompileFailedPayload = serde_json::from_str(payload_json)?;
    let compiled_at = event_time.to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET compiled_at = ?, compile_error = ?, updated_at = ? \
             WHERE branch = ?",
        )
        .bind(&compiled_at)
        .bind(&p.error)
        .bind(&compiled_at)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

fn apply_status(entity_id: &str, status: &str, conn: &mut SqliteConnection) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query("UPDATE workspace SET status = ?, updated_at = ? WHERE branch = ?")
            .bind(status)
            .bind(&now)
            .bind(entity_id)
            .execute(conn)
            .await?;
        Ok(())
    })
}

#[derive(serde::Deserialize)]
struct StatusChangedPayload {
    #[allow(dead_code)]
    old_status: String,
    new_status: String,
}

fn apply_status_changed(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: StatusChangedPayload = serde_json::from_str(payload_json)?;
    apply_status(entity_id, &p.new_status, conn)
}

#[derive(serde::Deserialize)]
struct AgentChangedPayload {
    agent_id: Option<String>,
}

fn apply_agent_changed(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: AgentChangedPayload = serde_json::from_str(payload_json)?;
    let now = chrono::Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET active_agent = ?, updated_at = ? WHERE branch = ?",
        )
        .bind(&p.agent_id)
        .bind(&now)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

fn apply_deleted(entity_id: &str, conn: &mut SqliteConnection) -> Result<()> {
    block_on(async {
        sqlx::query("DELETE FROM workspace WHERE branch = ?")
            .bind(entity_id)
            .execute(conn)
            .await?;
        Ok(())
    })
}
