use crate::agents::config::{
    ProviderSettings, resolve_provider_settings_with_agent_override,
};
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::event_upserts::{upsert_workspace_on_compile_failed, upsert_workspace_on_compiled};
use super::helpers::*;
use super::types::*;
use super::types_session::*;

// ---- Agent config resolution -----------------------------------------------

pub(crate) fn resolve_workspace_agent_config(
    ship_dir: &Path,
    workspace: &Workspace,
    agent_id: Option<&str>,
) -> Result<ProviderSettings> {
    let agent_id = agent_id
        .and_then(normalize_agent_ref)
        .or_else(|| {
            workspace
                .active_agent
                .as_deref()
                .and_then(normalize_agent_ref)
        })
        .map(|a| a.to_string());

    resolve_provider_settings_with_agent_override(
        ship_dir,
        None,
        agent_id.as_deref(),
    )
}

pub(crate) fn resolve_session_providers(
    ship_dir: &Path,
    workspace: &Workspace,
    agent_id: Option<&str>,
) -> Result<Vec<String>> {
    let resolved = resolve_workspace_agent_config(ship_dir, workspace, agent_id)?;
    if resolved.providers.is_empty() {
        return Err(anyhow!(
            "No valid providers resolved for workspace '{}'",
            workspace.branch
        ));
    }
    Ok(resolved.providers)
}

// ---- Provider matrix -------------------------------------------------------

pub(crate) fn build_workspace_provider_matrix(
    ship_dir: &Path,
    workspace: &Workspace,
    agent_id: Option<&str>,
) -> Result<WorkspaceProviderMatrix> {
    let resolved_agent_id = agent_id.and_then(normalize_agent_ref).or_else(|| {
        workspace
            .active_agent
            .as_deref()
            .and_then(normalize_agent_ref)
    });
    let resolved = resolve_workspace_agent_config(ship_dir, workspace, agent_id)?;
    let providers = resolved.providers;

    let supported_providers = crate::agents::export::list_providers(ship_dir)?
        .into_iter()
        .map(|provider| provider.id)
        .collect::<Vec<_>>();

    let resolution_error = if providers.is_empty() {
        Some(format!(
            "No valid providers resolved for workspace '{}'",
            workspace.branch
        ))
    } else {
        None
    };

    Ok(WorkspaceProviderMatrix {
        workspace_branch: workspace.branch.clone(),
        agent_id: resolved_agent_id,
        source: if resolved.active_agent.is_some() {
            "agent/config".to_string()
        } else {
            "config/default".to_string()
        },
        allowed_providers: providers,
        supported_providers,
        resolution_error,
    })
}

// ---- Context root and missing configs --------------------------------------

pub(crate) fn resolve_workspace_context_root(
    ship_dir: &Path,
    workspace: &Workspace,
) -> PathBuf {
    if workspace.is_worktree
        && let Some(path) = workspace.worktree_path.as_deref()
    {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    ship_dir.parent().unwrap_or(ship_dir).to_path_buf()
}

pub(crate) fn missing_provider_configs_for_workspace(
    context_root: &Path,
    providers: &[String],
) -> Vec<String> {
    providers
        .iter()
        .filter_map(|provider| {
            let desc = crate::agents::export::get_provider(provider)?;
            let target = context_root.join(desc.project_config);
            if target.exists() {
                None
            } else {
                Some(provider.clone())
            }
        })
        .collect()
}

// ---- Compile workspace context ---------------------------------------------

/// Compile workspace context and return the resolved provider list.
///
/// Events (compiled / compile_failed) are emitted as side effects.
/// The Workspace struct is read-only -- compile state lives in the DB
/// via the projection.
pub(crate) fn compile_workspace_context(
    ship_dir: &Path,
    workspace: &Workspace,
    agent_id_override: Option<&str>,
) -> Result<Vec<String>> {
    let started = Instant::now();
    let agent_id = agent_id_override
        .map(|a| a.to_string())
        .or_else(|| workspace.active_agent.clone());
    let agent_id = agent_id.and_then(|value| normalize_optional_text(Some(value)));
    let resolved_agent =
        match resolve_workspace_agent_config(ship_dir, workspace, agent_id.as_deref()) {
            Ok(agent) => agent,
            Err(error) => {
                upsert_workspace_on_compile_failed(
                    ship_dir,
                    &workspace.branch,
                    &error.to_string(),
                )?;
                return Err(error);
            }
        };
    let providers = resolved_agent.providers.clone();
    if providers.is_empty() {
        let error = anyhow!(
            "No valid providers resolved for workspace '{}'",
            workspace.branch
        );
        upsert_workspace_on_compile_failed(
            ship_dir,
            &workspace.branch,
            &error.to_string(),
        )?;
        return Err(error);
    }

    let mcp_server_filter = resolved_agent
        .mcp_servers
        .iter()
        .map(|server| server.id.clone())
        .collect::<Vec<_>>();
    let skill_filter = resolved_agent
        .skills
        .iter()
        .map(|skill| skill.id.clone())
        .collect::<Vec<_>>();

    let context_root = resolve_workspace_context_root(ship_dir, workspace);
    for provider in &providers {
        if let Err(error) =
            crate::agents::export::export_to_filtered_with_mode_override_and_skills_at_root(
                ship_dir.to_path_buf(),
                provider,
                Some(mcp_server_filter.as_slice()),
                Some(skill_filter.as_slice()),
                agent_id.as_deref(),
                &context_root,
            )
        {
            let contextual = error.context(format!(
                "Failed to compile provider '{}' for workspace '{}'",
                provider, workspace.branch
            ));
            upsert_workspace_on_compile_failed(
                ship_dir,
                &workspace.branch,
                &contextual.to_string(),
            )?;
            return Err(contextual);
        }
    }

    let duration_ms = started.elapsed().as_millis() as u64;
    // config_generation is tracked by the projection; pass 0 here as a
    // signal that the projection should increment.
    upsert_workspace_on_compiled(ship_dir, &workspace.branch, 0, duration_ms)?;
    Ok(providers)
}
