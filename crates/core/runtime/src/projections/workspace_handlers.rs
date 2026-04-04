//! Handler functions for workspace projection events.
//!
//! Each function applies a single event type to the workspace table.

use anyhow::Result;
use sqlx::SqliteConnection;

use crate::db::block_on;

// ── Payloads ────────────────────────────────────────────────────────────────

/// workspace.created payload must carry enough to INSERT the full row.
#[derive(serde::Deserialize)]
pub(super) struct CreatedPayload {
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub workspace_type: String,
    pub status: String,
    #[serde(default)]
    pub active_agent: Option<String>,
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub is_worktree: bool,
    #[serde(default)]
    pub worktree_path: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct ActivatedPayload {
    pub agent_id: Option<String>,
    pub providers: Vec<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct CompiledPayload {
    #[allow(dead_code)]
    pub config_generation: u32,
    #[allow(dead_code)]
    pub duration_ms: u64,
}

#[derive(serde::Deserialize)]
pub(super) struct CompileFailedPayload {
    pub error: String,
}

#[derive(serde::Deserialize)]
pub(super) struct StatusChangedPayload {
    #[allow(dead_code)]
    pub old_status: String,
    pub new_status: String,
}

#[derive(serde::Deserialize)]
pub(super) struct AgentChangedPayload {
    pub agent_id: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct ReconciledPayload {
    pub is_worktree: bool,
    pub worktree_path: Option<String>,
    #[allow(dead_code)]
    pub reason: String,
}

// ── Handlers ────────────────────────────────────────────────────────────────

pub(super) fn apply_created(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: CreatedPayload = serde_json::from_str(payload_json)?;
    let now = chrono::Utc::now().to_rfc3339();
    let workspace_id = p.workspace_id.as_deref().unwrap_or(entity_id);
    let providers = serde_json::to_string(&p.providers)?;
    let mcp_servers = serde_json::to_string(&p.mcp_servers)?;
    let skills = serde_json::to_string(&p.skills)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO workspace \
             (branch, id, workspace_type, status, active_agent, \
              providers_json, mcp_servers_json, skills_json, \
              is_worktree, worktree_path, config_generation, \
              created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?) \
             ON CONFLICT(branch) DO UPDATE SET \
               id               = excluded.id, \
               workspace_type   = excluded.workspace_type, \
               status           = excluded.status, \
               active_agent     = excluded.active_agent, \
               providers_json   = excluded.providers_json, \
               mcp_servers_json = excluded.mcp_servers_json, \
               skills_json      = excluded.skills_json, \
               is_worktree      = excluded.is_worktree, \
               worktree_path    = excluded.worktree_path, \
               updated_at       = excluded.updated_at",
        )
        .bind(entity_id)
        .bind(workspace_id)
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

pub(super) fn apply_activated(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
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

pub(super) fn apply_compiled(
    entity_id: &str,
    payload_json: &str,
    event_time: &chrono::DateTime<chrono::Utc>,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let _p: CompiledPayload = serde_json::from_str(payload_json)?;
    let compiled_at = event_time.to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET compiled_at = ?, compile_error = NULL, \
             config_generation = config_generation + 1, updated_at = ? WHERE branch = ?",
        )
        .bind(&compiled_at)
        .bind(&compiled_at)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

pub(super) fn apply_compile_failed(
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

pub(super) fn apply_status(
    entity_id: &str,
    status: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
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

pub(super) fn apply_status_changed(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: StatusChangedPayload = serde_json::from_str(payload_json)?;
    apply_status(entity_id, &p.new_status, conn)
}

pub(super) fn apply_agent_changed(
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

pub(super) fn apply_reconciled(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: ReconciledPayload = serde_json::from_str(payload_json)?;
    let now = chrono::Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET is_worktree = ?, worktree_path = ?, updated_at = ? \
             WHERE branch = ?",
        )
        .bind(p.is_worktree)
        .bind(&p.worktree_path)
        .bind(&now)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

pub(super) fn apply_deleted(entity_id: &str, conn: &mut SqliteConnection) -> Result<()> {
    block_on(async {
        sqlx::query("DELETE FROM workspace WHERE branch = ?")
            .bind(entity_id)
            .execute(conn)
            .await?;
        Ok(())
    })
}
