use crate::agents::config::{
    ProviderSettings, WorkspaceAgentSettings, resolve_provider_settings_with_agent_override,
};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::path::{Path, PathBuf};

use super::context_hash::compute_workspace_context_hash;
use super::crud::upsert_workspace;
use super::helpers::*;
use super::types::*;
use super::types_session::*;

// ---- Agent config resolution -----------------------------------------------

fn merge_feature_agent_with_workspace(
    feature_agent: Option<WorkspaceAgentSettings>,
    workspace: &Workspace,
) -> Option<WorkspaceAgentSettings> {
    let mut has_override = feature_agent.is_some();
    let mut merged = feature_agent.unwrap_or_default();

    let workspace_providers = normalize_nonempty_id_list(&workspace.providers);
    if !workspace_providers.is_empty() {
        merged.providers = workspace_providers;
        has_override = true;
    }

    let workspace_mcp_servers = normalize_nonempty_id_list(&workspace.mcp_servers);
    if !workspace_mcp_servers.is_empty() {
        merged.mcp_servers = workspace_mcp_servers;
        has_override = true;
    }

    let workspace_skills = normalize_nonempty_id_list(&workspace.skills);
    if !workspace_skills.is_empty() {
        merged.skills = workspace_skills;
        has_override = true;
    }

    if has_override { Some(merged) } else { None }
}

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

    let workspace_agent = merge_feature_agent_with_workspace(None, workspace);

    resolve_provider_settings_with_agent_override(
        ship_dir,
        workspace_agent.as_ref(),
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
        source: if !workspace.providers.is_empty() {
            "workspace".to_string()
        } else if workspace.feature_id.is_some() {
            "feature".to_string()
        } else if resolved.active_agent.is_some() {
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

pub(crate) fn compile_workspace_context(
    ship_dir: &Path,
    workspace: &mut Workspace,
    agent_id_override: Option<&str>,
) -> Result<()> {
    let agent_id = agent_id_override
        .map(|a| a.to_string())
        .or_else(|| workspace.active_agent.clone());
    let agent_id = agent_id.and_then(|value| normalize_optional_text(Some(value)));
    let resolved_agent =
        match resolve_workspace_agent_config(ship_dir, workspace, agent_id.as_deref()) {
            Ok(agent) => agent,
            Err(error) => {
                let now = Utc::now();
                workspace.compiled_at = Some(now);
                workspace.compile_error = Some(error.to_string());
                workspace.resolved_at = now;
                upsert_workspace(ship_dir, workspace)?;
                return Err(error);
            }
        };
    let providers = resolved_agent.providers.clone();
    if providers.is_empty() {
        let error = anyhow!(
            "No valid providers resolved for workspace '{}'",
            workspace.branch
        );
        let now = Utc::now();
        workspace.compiled_at = Some(now);
        workspace.compile_error = Some(error.to_string());
        workspace.resolved_at = now;
        upsert_workspace(ship_dir, workspace)?;
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

    let now = Utc::now();
    let next_context_hash =
        match compute_workspace_context_hash(ship_dir, workspace, &resolved_agent) {
            Ok(hash) => hash,
            Err(error) => {
                workspace.compiled_at = Some(now);
                workspace.compile_error = Some(error.to_string());
                workspace.resolved_at = now;
                upsert_workspace(ship_dir, workspace)?;
                return Err(error);
            }
        };

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
            workspace.compiled_at = Some(now);
            workspace.compile_error = Some(contextual.to_string());
            workspace.context_hash = Some(next_context_hash.clone());
            workspace.resolved_at = now;
            upsert_workspace(ship_dir, workspace)?;
            return Err(contextual);
        }
    }

    workspace.config_generation = workspace.config_generation.saturating_add(1);
    workspace.compiled_at = Some(now);
    workspace.compile_error = None;
    workspace.context_hash = Some(next_context_hash);
    workspace.resolved_at = now;
    upsert_workspace(ship_dir, workspace)?;
    Ok(())
}
