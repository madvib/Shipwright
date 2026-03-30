use crate::db::session::{
    get_active_workspace_session_db, get_workspace_session_db,
    insert_workspace_session_record_db,
};
use crate::db::session_events::{
    insert_session_with_started_event, update_session_with_ended_event,
};
use crate::events::types::{SessionEnded, SessionStarted};
use crate::projections::{Projection, SessionProjection};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::path::Path;
use std::process::Command;

use super::compile::{compile_workspace_context, resolve_session_providers};
use super::crud::get_workspace;
use super::helpers::*;
use super::lifecycle::{activate_workspace, set_workspace_active_agent};
use super::session::{
    annotate_session_stale_state, hydrate_workspace_session, persist_session_artifact,
};
use super::types::*;
use super::types_session::*;

// ---- Post-session hooks ----------------------------------------------------

fn run_post_session_hooks(ship_dir: &Path, session: &WorkspaceSession) -> Result<()> {
    let effective = crate::config::get_effective_config(Some(ship_dir.to_path_buf()))?;
    let hooks: Vec<_> = effective
        .hooks
        .into_iter()
        .filter(|hook| {
            hook.trigger == crate::config::HookTrigger::SessionEnd
                || hook.trigger == crate::config::HookTrigger::Stop
        })
        .collect();

    for hook in hooks {
        let command = hook.command.trim();
        if command.is_empty() {
            continue;
        }

        let mut process = if cfg!(windows) {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", command]);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.args(["-lc", command]);
            cmd
        };

        let output = process
            .current_dir(ship_dir)
            .env("SHIP_SESSION_ID", &session.id)
            .env("SHIP_SESSION_BRANCH", &session.workspace_branch)
            .output();

        match output {
            Ok(out) => {
                if !out.status.success() {
                    eprintln!(
                        "Post-session hook '{}' failed with status {:?}",
                        hook.id,
                        out.status.code()
                    );
                }
            }
            Err(error) => {
                eprintln!(
                    "Failed to execute post-session hook '{}': {}",
                    hook.id, error
                );
            }
        }
    }

    Ok(())
}

// ---- Start / end -----------------------------------------------------------

pub fn start_workspace_session(
    ship_dir: &Path,
    branch: &str,
    goal: Option<String>,
    agent_id: Option<String>,
    primary_provider: Option<String>,
) -> Result<WorkspaceSession> {
    let branch = ensure_branch_key(branch)?;
    let mut workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow::anyhow!("Workspace not found for branch '{}'", branch))?;

    if workspace.status != WorkspaceStatus::Active {
        workspace = activate_workspace(ship_dir, branch)?;
    }

    if let Some(agent_id) = agent_id.as_deref() {
        workspace = set_workspace_active_agent(ship_dir, branch, Some(agent_id))?;
    }

    if let Some(active) = get_active_workspace_session_db(&workspace.id)? {
        let mut existing = hydrate_workspace_session(active);
        annotate_session_stale_state(&mut existing, &std::collections::HashMap::new());
        if let Err(error) = persist_session_artifact(ship_dir, &existing, "attach") {
            eprintln!("Failed to persist attached session artifact: {}", error);
        }
        return Ok(existing);
    }

    let agent_id = agent_id
        .or(workspace.active_agent.clone())
        .and_then(|value| normalize_optional_text(Some(value)));
    let providers = resolve_session_providers(ship_dir, &workspace, agent_id.as_deref())?;
    let primary_provider = if let Some(requested_provider) = primary_provider {
        let normalized = normalize_provider_ref(&requested_provider)
            .ok_or_else(|| anyhow!("Session provider cannot be empty"))?;
        if !providers.iter().any(|provider| provider == &normalized) {
            return Err(anyhow!(
                "Provider '{}' is not allowed for workspace '{}' (allowed: {})",
                normalized,
                workspace.branch,
                providers.join(", ")
            ));
        }
        normalized
    } else {
        providers
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("No providers resolved for workspace '{}'", workspace.branch))?
    };

    compile_workspace_context(ship_dir, &mut workspace, agent_id.as_deref())?;

    let session_id = crate::gen_nanoid();
    let started_payload = SessionStarted {
        goal: normalize_optional_text(goal),
        workspace_id: workspace.id.clone(),
        workspace_branch: workspace.branch.clone(),
        agent_id,
        primary_provider: Some(primary_provider),
        config_generation_at_start: Some(workspace.config_generation),
        compiled_at: workspace.compiled_at.as_ref().map(|ts| ts.to_rfc3339()),
    };
    let start_envelope = insert_session_with_started_event(&session_id, &workspace.id, &started_payload)?;
    // Apply session projection synchronously so get_workspace_session_db finds the row.
    if let Ok(mut conn) = crate::db::open_db() {
        let _ = SessionProjection::new().apply(&start_envelope, &mut conn);
    }
    let created = get_workspace_session_db(&session_id)?
        .ok_or_else(|| anyhow::anyhow!("Failed to load created workspace session"))?;
    let started = hydrate_workspace_session(created);

    if let Err(error) = persist_session_artifact(ship_dir, &started, "start") {
        eprintln!("Failed to persist session artifact on start: {}", error);
    }

    Ok(started)
}

pub fn end_workspace_session(
    ship_dir: &Path,
    branch: &str,
    request: EndWorkspaceSessionRequest,
) -> Result<WorkspaceSession> {
    let branch = ensure_branch_key(branch)?;
    let workspace = get_workspace(ship_dir, branch)?
        .ok_or_else(|| anyhow::anyhow!("Workspace not found for branch '{}'", branch))?;

    let mut active = get_active_workspace_session_db(&workspace.id)?
        .ok_or_else(|| anyhow::anyhow!("No active workspace session for '{}'", workspace.branch))?;

    let now = Utc::now().to_rfc3339();
    active.status = WorkspaceSessionStatus::Ended.to_string();
    active.ended_at = Some(now.clone());
    active.summary = normalize_optional_text(request.summary);
    active.updated_workspace_ids = request.updated_workspace_ids;
    active.updated_at = now;

    // Compute duration before the transactional update so we can include it in
    // the session.ended event payload.
    let started_at = active.started_at.parse::<chrono::DateTime<Utc>>()
        .map_err(|e| anyhow::anyhow!("invalid started_at '{}': {}", active.started_at, e))?;
    let ended_ts = active.ended_at.as_deref()
        .ok_or_else(|| anyhow::anyhow!("ended_at must be set before emitting session.ended"))?
        .parse::<chrono::DateTime<Utc>>()
        .map_err(|e| anyhow::anyhow!("invalid ended_at: {}", e))?;
    let duration_secs_val = (ended_ts - started_at).num_seconds();

    let ended_payload = SessionEnded {
        summary: active.summary.clone(),
        duration_secs: Some(duration_secs_val as u64),
        gate_result: request.gate_result.clone(),
        updated_workspace_ids: active.updated_workspace_ids.clone(),
        compile_error: active.compile_error.clone(),
    };
    let end_envelope = update_session_with_ended_event(&active.id, &active.workspace_id, &ended_payload)?;
    // Apply session projection synchronously so get_workspace_session_db finds the updated row.
    if let Ok(mut conn) = crate::db::open_db() {
        let _ = SessionProjection::new().apply(&end_envelope, &mut conn);
    }

    let ended = get_workspace_session_db(&active.id)?
        .ok_or_else(|| anyhow::anyhow!("Failed to load ended workspace session"))?;
    let ended = hydrate_workspace_session(ended);

    let duration_secs = ended
        .ended_at
        .map(|end| (end - ended.started_at).num_seconds());

    let record = crate::db::types::WorkspaceSessionRecordDb {
        id: crate::gen_nanoid(),
        session_id: ended.id.clone(),
        workspace_id: ended.workspace_id.clone(),
        workspace_branch: ended.workspace_branch.clone(),
        summary: ended.summary.clone(),
        updated_workspace_ids: ended.updated_workspace_ids.clone(),
        duration_secs,
        provider: ended.primary_provider.clone(),
        model: request.model,
        agent_id: ended.agent_id.clone(),
        files_changed: request.files_changed,
        gate_result: request.gate_result,
        created_at: Utc::now().to_rfc3339(),
    };
    insert_workspace_session_record_db(&record)?;

    let mut ended = ended;
    ended.session_record_id = Some(record.id);

    if let Err(error) = persist_session_artifact(ship_dir, &ended, "end") {
        eprintln!("Failed to persist session artifact on end: {}", error);
    }
    if let Err(error) = run_post_session_hooks(ship_dir, &ended) {
        eprintln!("Failed to run post-session hooks: {}", error);
    }

    Ok(ended)
}
