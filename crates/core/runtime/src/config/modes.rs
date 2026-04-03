use super::artifact_registry::{
    ARTIFACT_KIND_MCP, ARTIFACT_KIND_RULE, ARTIFACT_KIND_SKILL, resolve_external_ids_to_refs,
    resolve_refs_to_external_ids, sync_agent_artifact_registry,
};
use super::project::AgentProfile;
use super::types::{HookConfig, PermissionConfig};
use crate::db::kv;
use anyhow::Result;
use std::path::Path;

const NS: &str = "runtime";
const KEY: &str = "modes";

/// Intermediate row stored in kv_state — mirrors the old agent_config columns.
#[derive(serde::Serialize, serde::Deserialize)]
struct ModeRow {
    id: String,
    name: String,
    description: Option<String>,
    active_tools_json: String,
    mcp_refs_json: String,
    skill_refs_json: String,
    rule_refs_json: String,
    prompt_id: Option<String>,
    hooks_json: String,
    permissions_json: String,
    target_agents_json: String,
}

pub(super) fn get_modes_config(ship_dir: &Path) -> Result<Vec<AgentProfile>> {
    sync_agent_artifact_registry(ship_dir)?;

    let rows: Vec<ModeRow> = match kv::get(NS, KEY)? {
        Some(v) => serde_json::from_value(v).unwrap_or_default(),
        None => return Ok(Vec::new()),
    };

    let mut modes = Vec::new();
    for row in rows {
        let active_tools: Vec<String> =
            serde_json::from_str(&row.active_tools_json).unwrap_or_default();
        let mcp_refs: Vec<String> = serde_json::from_str(&row.mcp_refs_json).unwrap_or_default();
        let skill_refs: Vec<String> =
            serde_json::from_str(&row.skill_refs_json).unwrap_or_default();
        let rule_refs: Vec<String> = serde_json::from_str(&row.rule_refs_json).unwrap_or_default();
        let hooks: Vec<HookConfig> = serde_json::from_str(&row.hooks_json).unwrap_or_default();
        let permissions: PermissionConfig =
            serde_json::from_str(&row.permissions_json).unwrap_or_default();
        let target_agents: Vec<String> =
            serde_json::from_str(&row.target_agents_json).unwrap_or_default();

        modes.push(AgentProfile {
            id: row.id,
            name: row.name,
            description: row.description,
            active_tools,
            mcp_servers: resolve_refs_to_external_ids(ship_dir, ARTIFACT_KIND_MCP, &mcp_refs)?,
            skills: resolve_refs_to_external_ids(ship_dir, ARTIFACT_KIND_SKILL, &skill_refs)?,
            rules: resolve_refs_to_external_ids(ship_dir, ARTIFACT_KIND_RULE, &rule_refs)?,
            prompt_id: row.prompt_id,
            hooks,
            permissions,
            target_agents,
        });
    }
    modes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(modes)
}

pub(super) fn save_modes_config(ship_dir: &Path, modes: &[AgentProfile]) -> Result<()> {
    sync_agent_artifact_registry(ship_dir)?;

    let mut rows = Vec::with_capacity(modes.len());
    for mode in modes {
        rows.push(ModeRow {
            id: mode.id.clone(),
            name: mode.name.clone(),
            description: mode.description.clone(),
            active_tools_json: serde_json::to_string(&mode.active_tools)?,
            mcp_refs_json: serde_json::to_string(&resolve_external_ids_to_refs(
                ship_dir,
                ARTIFACT_KIND_MCP,
                &mode.mcp_servers,
            )?)?,
            skill_refs_json: serde_json::to_string(&resolve_external_ids_to_refs(
                ship_dir,
                ARTIFACT_KIND_SKILL,
                &mode.skills,
            )?)?,
            rule_refs_json: serde_json::to_string(&resolve_external_ids_to_refs(
                ship_dir,
                ARTIFACT_KIND_RULE,
                &mode.rules,
            )?)?,
            prompt_id: mode.prompt_id.clone(),
            hooks_json: serde_json::to_string(&mode.hooks)?,
            permissions_json: serde_json::to_string(&mode.permissions)?,
            target_agents_json: serde_json::to_string(&mode.target_agents)?,
        });
    }

    kv::set(NS, KEY, &serde_json::to_value(&rows)?)?;
    Ok(())
}
