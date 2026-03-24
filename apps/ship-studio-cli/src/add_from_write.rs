//! File writing logic for `ship add --from` — writes agent profiles, skill
//! files, and merges dependencies into the project manifest.

use std::path::Path;

use anyhow::{Context, Result};

use crate::add_from::{AgentBundle, SkillBundle, TransferBundle};

/// Write the transfer bundle to `.ship/`.
pub fn write_bundle(project_root: &Path, bundle: &TransferBundle) -> Result<()> {
    let ship_dir = project_root.join(".ship");

    // 1. Write agent profile as JSONC.
    write_agent(&ship_dir, &bundle.agent)?;

    // 2. Write inline skills.
    for (skill_id, skill) in &bundle.skills {
        write_skill(&ship_dir, skill_id, skill)?;
    }

    // 3. Merge dependencies into manifest.
    if !bundle.dependencies.is_empty() {
        merge_dependencies(&ship_dir, &bundle.dependencies)?;
    }

    Ok(())
}

/// Write an agent JSONC profile.
pub fn write_agent(ship_dir: &Path, agent: &AgentBundle) -> Result<()> {
    let agents_dir = ship_dir.join("agents");
    std::fs::create_dir_all(&agents_dir)?;

    let dest = agents_dir.join(format!("{}.jsonc", agent.id));
    if dest.exists() {
        eprintln!("warning: overwriting existing agent '{}'", agent.id);
    }

    let profile = build_agent_jsonc(agent);
    std::fs::write(&dest, profile)
        .with_context(|| format!("writing agent {}", dest.display()))?;

    Ok(())
}

/// Build JSONC content for an agent profile.
pub fn build_agent_jsonc(agent: &AgentBundle) -> String {
    let mut obj = serde_json::Map::new();
    obj.insert("id".into(), serde_json::json!(agent.id));
    if let Some(ref name) = agent.name {
        obj.insert("name".into(), serde_json::json!(name));
    }
    if let Some(ref desc) = agent.description {
        obj.insert("description".into(), serde_json::json!(desc));
    }
    if let Some(ref model) = agent.model {
        obj.insert("model".into(), serde_json::json!(model));
    }
    if !agent.skills.is_empty() {
        obj.insert("skills".into(), serde_json::json!(agent.skills));
    }
    if !agent.rules.is_empty() {
        obj.insert("rules".into(), serde_json::json!(agent.rules));
    }
    if !agent.mcp_servers.is_empty() {
        obj.insert("mcp_servers".into(), serde_json::json!(agent.mcp_servers));
    }

    serde_json::to_string_pretty(&obj).unwrap_or_else(|_| "{}".into())
}

/// Write inline skill files to `.ship/skills/<id>/`.
pub fn write_skill(ship_dir: &Path, skill_id: &str, skill: &SkillBundle) -> Result<()> {
    let skill_dir = ship_dir.join("skills").join(skill_id);
    std::fs::create_dir_all(&skill_dir)?;

    for (rel_path, content) in &skill.files {
        let dest = skill_dir.join(rel_path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest, content)
            .with_context(|| format!("writing skill file {}", dest.display()))?;
    }

    Ok(())
}

/// Merge dependencies into the existing manifest (ship.jsonc or ship.toml).
pub fn merge_dependencies(
    ship_dir: &Path,
    deps: &std::collections::HashMap<String, String>,
) -> Result<()> {
    let jsonc_path = ship_dir.join("ship.jsonc");
    let toml_path = ship_dir.join("ship.toml");

    let manifest_path = if jsonc_path.exists() {
        jsonc_path
    } else if toml_path.exists() {
        toml_path
    } else {
        anyhow::bail!("no ship.jsonc or ship.toml found to add dependencies to");
    };

    let raw = std::fs::read_to_string(&manifest_path)?;
    let is_jsonc = crate::paths::is_jsonc_ext(&manifest_path);

    let mut updated = raw;
    for (path, version) in deps {
        if updated.contains(path) {
            continue; // Already present.
        }
        updated = if is_jsonc {
            crate::add::append_dependency_jsonc(&updated, path, version)
        } else {
            crate::add::append_dependency(&updated, path, version)
        };
    }

    std::fs::write(&manifest_path, &updated)
        .with_context(|| format!("writing {}", manifest_path.display()))?;

    Ok(())
}
