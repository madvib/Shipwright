//! Workspace session CRUD — moved from state_db.

use anyhow::{Context, Result};
use sqlx::Row;

use super::types::{WorkspaceSessionDb, WorkspaceSessionRecordDb};
use super::{block_on, open_db};

fn parse_workspace_session_row(row: &sqlx::sqlite::SqliteRow) -> WorkspaceSessionDb {
    let updated_workspace_ids_json: String = row.get(10);
    let updated_workspace_ids =
        serde_json::from_str(&updated_workspace_ids_json).unwrap_or_default();
    WorkspaceSessionDb {
        id: row.get(0),
        workspace_id: row.get(1),
        workspace_branch: row.get(2),
        status: row.get(3),
        started_at: row.get(4),
        ended_at: row.get(5),
        agent_id: row.get(6),
        primary_provider: row.get(7),
        goal: row.get(8),
        summary: row.get(9),
        updated_workspace_ids,
        compiled_at: row.get(11),
        compile_error: row.get(12),
        config_generation_at_start: row.get(13),
        created_at: row.get(14),
        updated_at: row.get(15),
    }
}

pub fn get_workspace_session_db(session_id: &str) -> Result<Option<WorkspaceSessionDb>> {
    let mut conn = open_db()?;
    let row = block_on(async {
        sqlx::query(
            "SELECT id, workspace_id, workspace_branch, status, started_at, ended_at, agent_id, primary_provider, goal, summary, updated_workspace_ids_json, compiled_at, compile_error, config_generation_at_start, created_at, updated_at
             FROM workspace_session
             WHERE id = ?",
        )
        .bind(session_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(parse_workspace_session_row))
}

pub fn get_active_workspace_session_db(workspace_id: &str) -> Result<Option<WorkspaceSessionDb>> {
    let mut conn = open_db()?;
    let row = block_on(async {
        sqlx::query(
            "SELECT id, workspace_id, workspace_branch, status, started_at, ended_at, agent_id, primary_provider, goal, summary, updated_workspace_ids_json, compiled_at, compile_error, config_generation_at_start, created_at, updated_at
             FROM workspace_session
             WHERE workspace_id = ? AND status = 'active'
             ORDER BY started_at DESC
             LIMIT 1",
        )
        .bind(workspace_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(parse_workspace_session_row))
}

pub fn list_workspace_sessions_db(
    workspace_id: Option<&str>,
    limit: usize,
) -> Result<Vec<WorkspaceSessionDb>> {
    let mut conn = open_db()?;
    let clamped_limit = limit.clamp(1, 500) as i64;
    let rows = if let Some(workspace_id) = workspace_id {
        block_on(async {
            sqlx::query(
                "SELECT id, workspace_id, workspace_branch, status, started_at, ended_at, agent_id, primary_provider, goal, summary, updated_workspace_ids_json, compiled_at, compile_error, config_generation_at_start, created_at, updated_at
                 FROM workspace_session
                 WHERE workspace_id = ?
                 ORDER BY started_at DESC
                 LIMIT ?",
            )
            .bind(workspace_id)
            .bind(clamped_limit)
            .fetch_all(&mut conn)
            .await
        })?
    } else {
        block_on(async {
            sqlx::query(
                "SELECT id, workspace_id, workspace_branch, status, started_at, ended_at, agent_id, primary_provider, goal, summary, updated_workspace_ids_json, compiled_at, compile_error, config_generation_at_start, created_at, updated_at
                 FROM workspace_session
                 ORDER BY started_at DESC
                 LIMIT ?",
            )
            .bind(clamped_limit)
            .fetch_all(&mut conn)
            .await
        })?
    };

    Ok(rows.iter().map(parse_workspace_session_row).collect())
}

pub fn insert_workspace_session_db(session: &WorkspaceSessionDb) -> Result<()> {
    let mut conn = open_db()?;
    let updated_workspace_ids_json = serde_json::to_string(&session.updated_workspace_ids)
        .with_context(|| "Failed to serialize workspace session updated_workspace_ids")?;
    block_on(async {
        sqlx::query(
            "INSERT INTO workspace_session
             (id, workspace_id, workspace_branch, status, started_at, ended_at, agent_id, primary_provider, goal, summary, updated_workspace_ids_json, compiled_at, compile_error, config_generation_at_start, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&session.id)
        .bind(&session.workspace_id)
        .bind(&session.workspace_branch)
        .bind(&session.status)
        .bind(&session.started_at)
        .bind(&session.ended_at)
        .bind(&session.agent_id)
        .bind(&session.primary_provider)
        .bind(&session.goal)
        .bind(&session.summary)
        .bind(&updated_workspace_ids_json)
        .bind(&session.compiled_at)
        .bind(&session.compile_error)
        .bind(session.config_generation_at_start)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn update_workspace_session_db(session: &WorkspaceSessionDb) -> Result<()> {
    let mut conn = open_db()?;
    let updated_workspace_ids_json = serde_json::to_string(&session.updated_workspace_ids)
        .with_context(|| "Failed to serialize workspace session updated_workspace_ids")?;
    block_on(async {
        sqlx::query(
            "UPDATE workspace_session
             SET workspace_id = ?,
                 workspace_branch = ?,
                 status = ?,
                 started_at = ?,
                 ended_at = ?,
                 agent_id = ?,
                 primary_provider = ?,
                 goal = ?,
                 summary = ?,
                 updated_workspace_ids_json = ?,
                 compiled_at = ?,
                 compile_error = ?,
                 config_generation_at_start = ?,
                 created_at = ?,
                 updated_at = ?
             WHERE id = ?",
        )
        .bind(&session.workspace_id)
        .bind(&session.workspace_branch)
        .bind(&session.status)
        .bind(&session.started_at)
        .bind(&session.ended_at)
        .bind(&session.agent_id)
        .bind(&session.primary_provider)
        .bind(&session.goal)
        .bind(&session.summary)
        .bind(&updated_workspace_ids_json)
        .bind(&session.compiled_at)
        .bind(&session.compile_error)
        .bind(session.config_generation_at_start)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .bind(&session.id)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn insert_workspace_session_record_db(record: &WorkspaceSessionRecordDb) -> Result<()> {
    let mut conn = open_db()?;
    let updated_workspace_ids_json = serde_json::to_string(&record.updated_workspace_ids)
        .with_context(|| "Failed to serialize workspace session record updated_workspace_ids")?;
    block_on(async {
        sqlx::query(
            "INSERT INTO workspace_session_record
             (id, session_id, workspace_id, workspace_branch, summary,
              updated_workspace_ids_json, duration_secs, provider, model,
              agent_id, files_changed, gate_result, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(session_id) DO UPDATE SET
               id = excluded.id,
               workspace_id = excluded.workspace_id,
               workspace_branch = excluded.workspace_branch,
               summary = excluded.summary,
               updated_workspace_ids_json = excluded.updated_workspace_ids_json,
               duration_secs = excluded.duration_secs,
               provider = excluded.provider,
               model = excluded.model,
               agent_id = excluded.agent_id,
               files_changed = excluded.files_changed,
               gate_result = excluded.gate_result,
               created_at = excluded.created_at",
        )
        .bind(&record.id)
        .bind(&record.session_id)
        .bind(&record.workspace_id)
        .bind(&record.workspace_branch)
        .bind(&record.summary)
        .bind(&updated_workspace_ids_json)
        .bind(record.duration_secs)
        .bind(&record.provider)
        .bind(&record.model)
        .bind(&record.agent_id)
        .bind(record.files_changed)
        .bind(&record.gate_result)
        .bind(&record.created_at)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn get_workspace_session_record_db(
    session_id: &str,
) -> Result<Option<WorkspaceSessionRecordDb>> {
    let mut conn = open_db()?;
    let row = block_on(async {
        sqlx::query(
            "SELECT id, session_id, workspace_id, workspace_branch, summary,
                    updated_workspace_ids_json, duration_secs, provider, model,
                    agent_id, files_changed, gate_result, created_at
             FROM workspace_session_record
             WHERE session_id = ?",
        )
        .bind(session_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.map(|row| WorkspaceSessionRecordDb {
        id: row.get(0),
        session_id: row.get(1),
        workspace_id: row.get(2),
        workspace_branch: row.get(3),
        summary: row.get(4),
        updated_workspace_ids: serde_json::from_str::<Vec<String>>(&row.get::<String, _>(5))
            .unwrap_or_default(),
        duration_secs: row.get(6),
        provider: row.get(7),
        model: row.get(8),
        agent_id: row.get(9),
        files_changed: row.get(10),
        gate_result: row.get(11),
        created_at: row.get(12),
    }))
}
