use crate::db::session::{
    get_active_workspace_session_db, get_workspace_session_record_db, list_workspace_sessions_db,
};
use crate::db::types::WorkspaceSessionDb;
use crate::events::{EventAction, EventEntity, append_event};
use crate::project::{get_global_dir, project_slug_from_ship_dir, sanitize_file_name};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::crud::{get_workspace, list_workspaces};
use super::helpers::*;
use super::types_session::*;

// ---- Hydration helpers -----------------------------------------------------

pub(super) fn hydrate_workspace_session(row: WorkspaceSessionDb) -> WorkspaceSession {
    WorkspaceSession {
        id: row.id,
        workspace_id: row.workspace_id,
        workspace_branch: row.workspace_branch,
        status: row.status.parse().unwrap_or(WorkspaceSessionStatus::Active),
        started_at: parse_datetime(&row.started_at),
        ended_at: parse_datetime_opt(row.ended_at),
        agent_id: row.agent_id,
        primary_provider: row.primary_provider,
        goal: row.goal,
        summary: row.summary,
        updated_workspace_ids: row.updated_workspace_ids,
        session_record_id: None,
        compiled_at: parse_datetime_opt(row.compiled_at),
        compile_error: row.compile_error,
        config_generation_at_start: row.config_generation_at_start,
        stale_context: false,
        created_at: parse_datetime(&row.created_at),
        updated_at: parse_datetime(&row.updated_at),
    }
}

pub(super) fn hydrate_workspace_session_record(
    row: crate::db::types::WorkspaceSessionRecordDb,
) -> WorkspaceSessionRecord {
    WorkspaceSessionRecord {
        id: row.id,
        session_id: row.session_id,
        workspace_id: row.workspace_id,
        workspace_branch: row.workspace_branch,
        summary: row.summary,
        updated_workspace_ids: row.updated_workspace_ids,
        duration_secs: row.duration_secs,
        provider: row.provider,
        model: row.model,
        agent_id: row.agent_id,
        files_changed: row.files_changed,
        gate_result: row.gate_result,
        created_at: parse_datetime(&row.created_at),
    }
}

pub(super) fn annotate_session_stale_state(
    session: &mut WorkspaceSession,
    workspace_generation_by_branch: &HashMap<String, i64>,
) {
    session.stale_context = session
        .config_generation_at_start
        .is_some_and(|session_generation| {
            workspace_generation_by_branch
                .get(&session.workspace_branch)
                .is_some_and(|workspace_generation| *workspace_generation > session_generation)
        });
}

pub(super) fn annotate_session_record(
    _ship_dir: &Path,
    session: &mut WorkspaceSession,
) -> Result<()> {
    session.session_record_id =
        get_workspace_session_record_db(&session.id)?.map(|record| record.id);
    Ok(())
}

// ---- Session artifacts -----------------------------------------------------

pub(super) fn session_artifacts_dir(ship_dir: &Path, session_id: &str) -> Option<PathBuf> {
    let slug = project_slug_from_ship_dir(ship_dir);
    let global_dir = get_global_dir().ok()?;
    let root = global_dir.join("projects").join(slug).join("sessions");
    Some(root.join(sanitize_file_name(session_id)))
}

pub(super) fn persist_session_artifact(
    ship_dir: &Path,
    session: &WorkspaceSession,
    phase: &str,
) -> Result<()> {
    let Some(session_dir) = session_artifacts_dir(ship_dir, &session.id) else {
        return Ok(());
    };
    std::fs::create_dir_all(&session_dir)?;

    let snapshot_path = session_dir.join("session.json");
    let snapshot = serde_json::to_string_pretty(session)?;
    std::fs::write(snapshot_path, snapshot)?;

    let timeline_path = session_dir.join("timeline.ndjson");
    let entry = serde_json::json!({
        "phase": phase,
        "timestamp": Utc::now().to_rfc3339(),
        "session_id": session.id,
        "branch": session.workspace_branch,
        "status": session.status.to_string(),
        "provider": session.primary_provider,
        "goal": session.goal,
        "summary": session.summary,
        "updated_workspace_ids": session.updated_workspace_ids,
        "session_record_id": session.session_record_id,
    });
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(timeline_path)?;
    writeln!(file, "{}", entry)?;

    Ok(())
}

fn append_session_note_artifact(ship_dir: &Path, session_id: &str, note: &str) -> Result<()> {
    let Some(session_dir) = session_artifacts_dir(ship_dir, session_id) else {
        return Ok(());
    };
    std::fs::create_dir_all(&session_dir)?;

    let notes_path = session_dir.join("notes.ndjson");
    let entry = serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "note": note,
    });
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(notes_path)?;
    writeln!(file, "{}", entry)?;

    Ok(())
}

// ---- Public session query/progress API -------------------------------------

pub fn get_active_workspace_session(
    ship_dir: &Path,
    branch: &str,
) -> Result<Option<WorkspaceSession>> {
    let branch = ensure_branch_key(branch)?;
    let workspace = match get_workspace(ship_dir, branch)? {
        Some(workspace) => workspace,
        None => return Ok(None),
    };
    let mut generation_by_branch = HashMap::new();
    generation_by_branch.insert(workspace.branch.clone(), workspace.config_generation);
    Ok(
        get_active_workspace_session_db(&workspace.id)?.map(|row| {
            let mut session = hydrate_workspace_session(row);
            annotate_session_stale_state(&mut session, &generation_by_branch);
            let _ = annotate_session_record(ship_dir, &mut session);
            session
        }),
    )
}

pub fn list_workspace_sessions(
    ship_dir: &Path,
    branch: Option<&str>,
    limit: usize,
) -> Result<Vec<WorkspaceSession>> {
    let mut workspace_generation_by_branch = HashMap::new();
    let workspace_id = if let Some(branch) = branch {
        let branch = ensure_branch_key(branch)?;
        match get_workspace(ship_dir, branch)? {
            Some(workspace) => {
                workspace_generation_by_branch
                    .insert(workspace.branch.clone(), workspace.config_generation);
                Some(workspace.id)
            }
            None => return Ok(Vec::new()),
        }
    } else {
        for workspace in list_workspaces(ship_dir)? {
            workspace_generation_by_branch.insert(workspace.branch, workspace.config_generation);
        }
        None
    };

    let rows = list_workspace_sessions_db(workspace_id.as_deref(), limit)?;
    let mut sessions: Vec<WorkspaceSession> =
        rows.into_iter().map(hydrate_workspace_session).collect();
    for session in &mut sessions {
        annotate_session_stale_state(session, &workspace_generation_by_branch);
        annotate_session_record(ship_dir, session)?;
    }
    Ok(sessions)
}

pub fn get_workspace_session_record(
    _ship_dir: &Path,
    session_id: &str,
) -> Result<Option<WorkspaceSessionRecord>> {
    let session_id = session_id.trim();
    if session_id.is_empty() {
        return Err(anyhow!("Session ID cannot be empty"));
    }
    Ok(
        get_workspace_session_record_db(session_id)?
            .map(hydrate_workspace_session_record),
    )
}

pub fn record_workspace_session_progress(ship_dir: &Path, branch: &str, note: &str) -> Result<()> {
    let branch = ensure_branch_key(branch)?;
    let workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow::anyhow!("Workspace not found for branch '{}'", branch))?;
    let active = get_active_workspace_session_db(&workspace.id)?
        .ok_or_else(|| anyhow::anyhow!("No active workspace session for '{}'", workspace.branch))?;

    let normalized_note = note.trim();
    if normalized_note.is_empty() {
        return Err(anyhow!("Session note cannot be empty"));
    }

    append_event(
        ship_dir,
        "agent",
        EventEntity::Session,
        EventAction::Note,
        active.id.clone(),
        Some(format!("branch={} {}", workspace.branch, normalized_note)),
    )?;
    if let Err(error) = append_session_note_artifact(ship_dir, &active.id, normalized_note) {
        eprintln!("Failed to persist session note artifact: {}", error);
    }
    Ok(())
}
