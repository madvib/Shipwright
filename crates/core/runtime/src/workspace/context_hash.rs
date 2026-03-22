use crate::agents::config::ProviderSettings;
use anyhow::Result;
use std::hash::{Hash, Hasher};
use std::path::Path;

use super::helpers::normalize_nonempty_id_list;
use super::types::Workspace;

pub(super) fn stable_hash(value: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub(super) fn compute_workspace_context_hash(
    ship_dir: &Path,
    workspace: &Workspace,
    resolved_agent: &ProviderSettings,
) -> Result<String> {
    let config = crate::config::get_effective_config(Some(ship_dir.to_path_buf()))?;
    let config_hash = stable_hash(&toml::to_string(&config)?);
    let permissions_hash = stable_hash(&toml::to_string(&resolved_agent.permissions)?);

    let mut skill_hashes = resolved_agent
        .skills
        .iter()
        .map(|skill| (skill.id.clone(), stable_hash(&skill.content)))
        .collect::<Vec<_>>();
    skill_hashes.sort_by(|left, right| left.0.cmp(&right.0));

    let mut rule_hashes = resolved_agent
        .rules
        .iter()
        .map(|rule| (rule.file_name.clone(), stable_hash(&rule.content)))
        .collect::<Vec<_>>();
    rule_hashes.sort_by(|left, right| left.0.cmp(&right.0));

    let mut mcp_hashes = resolved_agent
        .mcp_servers
        .iter()
        .map(|server| -> Result<(String, String)> {
            let digest = stable_hash(&toml::to_string(&server)?);
            Ok((server.id.clone(), digest))
        })
        .collect::<Result<Vec<_>>>()?;
    mcp_hashes.sort_by(|left, right| left.0.cmp(&right.0));

    let mut normalized_providers = resolved_agent
        .providers
        .iter()
        .map(|provider| provider.trim().to_ascii_lowercase())
        .filter(|provider| !provider.is_empty())
        .collect::<Vec<_>>();
    normalized_providers.sort();
    normalized_providers.dedup();

    let fingerprint = serde_json::json!({
        "workspace": {
            "branch": workspace.branch,
            "workspace_type": workspace.workspace_type.to_string(),
            "environment_id": workspace.environment_id,
            "feature_id": workspace.feature_id,
            "target_id": workspace.target_id,
            "agent_id": resolved_agent.active_agent,
            "provider_overrides": normalize_nonempty_id_list(&workspace.providers),
            "mcp_server_overrides": normalize_nonempty_id_list(&workspace.mcp_servers),
            "skill_overrides": normalize_nonempty_id_list(&workspace.skills),
        },
        "providers": normalized_providers,
        "model": resolved_agent.model,
        "config_hash": config_hash,
        "permissions_hash": permissions_hash,
        "skill_hashes": skill_hashes,
        "rule_hashes": rule_hashes,
        "mcp_hashes": mcp_hashes,
    });
    Ok(stable_hash(&serde_json::to_string(&fingerprint)?))
}
