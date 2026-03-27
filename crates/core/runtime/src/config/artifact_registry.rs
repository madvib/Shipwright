use anyhow::Result;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub(super) const ARTIFACT_KIND_MCP: &str = "mcp";
pub(super) const ARTIFACT_KIND_SKILL: &str = "skill";
pub(super) const ARTIFACT_KIND_RULE: &str = "rule";

pub(super) fn stable_hash(value: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub(super) fn normalize_rule_external_id(id: &str) -> String {
    let normalized = id.trim().trim_end_matches(".md");
    if let Some((prefix, rest)) = normalized.split_once('-')
        && !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_digit())
    {
        return rest.to_string();
    }
    normalized.to_string()
}

pub(super) fn sync_agent_artifact_registry(ship_dir: &Path) -> Result<()> {
    for skill in crate::skill::list_skills(ship_dir)? {
        let path = crate::project::skills_dir(ship_dir)
            .join(&skill.id)
            .join("SKILL.md");
        let digest = stable_hash(&skill.content);
        crate::db::agents::upsert_agent_artifact_registry_db(
            ARTIFACT_KIND_SKILL,
            &skill.id,
            &skill.name,
            &path.to_string_lossy(),
            &digest,
        )?;
    }

    for rule in crate::rule::list_rules(ship_dir.to_path_buf())? {
        let external_id = normalize_rule_external_id(&rule.file_name);
        let digest = stable_hash(&rule.content);
        crate::db::agents::upsert_agent_artifact_registry_db(
            ARTIFACT_KIND_RULE,
            &external_id,
            &rule.file_name,
            &rule.path,
            &digest,
        )?;
    }

    for server in super::mcp::get_mcp_config(ship_dir)? {
        let digest = stable_hash(&toml::to_string(&server)?);
        crate::db::agents::upsert_agent_artifact_registry_db(
            ARTIFACT_KIND_MCP,
            &server.id,
            &server.name,
            &crate::project::mcp_config_path(ship_dir).to_string_lossy(),
            &digest,
        )?;
    }

    Ok(())
}

pub(super) fn resolve_refs_to_external_ids(
    _ship_dir: &Path,
    kind: &str,
    refs: &[String],
) -> Result<Vec<String>> {
    let mut resolved = Vec::new();
    let mut seen = HashSet::new();
    for reference in refs {
        if let Some(entry) =
            crate::db::agents::get_agent_artifact_registry_by_uuid_db(kind, reference)?
        {
            let external_id = if kind == ARTIFACT_KIND_RULE {
                normalize_rule_external_id(&entry.external_id)
            } else {
                entry.external_id
            };
            if seen.insert(external_id.clone()) {
                resolved.push(external_id);
            }
            continue;
        }

        let lookup = if kind == ARTIFACT_KIND_RULE {
            normalize_rule_external_id(reference)
        } else {
            reference.clone()
        };
        if let Some(entry) =
            crate::db::agents::get_agent_artifact_registry_by_external_id_db(kind, &lookup)?
            && seen.insert(entry.external_id.clone())
        {
            resolved.push(entry.external_id);
        }
    }
    Ok(resolved)
}

pub(super) fn resolve_external_ids_to_refs(
    _ship_dir: &Path,
    kind: &str,
    external_ids: &[String],
) -> Result<Vec<String>> {
    let mut refs = Vec::new();
    let mut seen = HashSet::new();
    for id in external_ids {
        let lookup = if kind == ARTIFACT_KIND_RULE {
            normalize_rule_external_id(id)
        } else {
            id.clone()
        };
        if let Some(entry) =
            crate::db::agents::get_agent_artifact_registry_by_external_id_db(kind, &lookup)?
            && seen.insert(entry.uuid.clone())
        {
            refs.push(entry.uuid);
        }
    }
    Ok(refs)
}
