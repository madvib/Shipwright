//! Agent runtime settings, artifact registry, and mode CRUD.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::Row;

use super::types::{AgentArtifactRegistryDb, AgentConfigDb, AgentRuntimeSettingsDb};
use super::{block_on, open_db};

pub fn get_agent_runtime_settings_db() -> Result<Option<AgentRuntimeSettingsDb>> {
    let mut conn = match open_db() {
        Ok(c) => c,
        Err(e) if e.to_string().contains("no id field") => return Ok(None),
        Err(e) => return Err(e),
    };
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT providers_json, active_agent, hooks_json, statuses_json, ai_json, git_json, namespaces_json
             FROM agent_runtime_settings
             WHERE id = 1",
        )
        .fetch_optional(&mut conn)
        .await
    })?;

    let Some(row) = row_opt else {
        return Ok(None);
    };

    let providers_json: String = row.get(0);
    let active_agent: Option<String> = row.get(1);
    let hooks_json: String = row.get(2);
    let statuses_json: String = row.get(3);
    let ai_json: Option<String> = row.get(4);
    let git_json: String = row.get(5);
    let namespaces_json: String = row.get(6);
    let providers: Vec<String> = serde_json::from_str(&providers_json).unwrap_or_default();

    Ok(Some(AgentRuntimeSettingsDb {
        providers,
        active_agent,
        hooks_json,
        statuses_json,
        ai_json,
        git_json,
        namespaces_json,
    }))
}

#[allow(clippy::too_many_arguments)]
pub fn set_agent_runtime_settings_db(
    providers: &[String],
    active_agent: Option<&str>,
    hooks_json: &str,
    statuses_json: &str,
    ai_json: Option<&str>,
    git_json: &str,
    namespaces_json: &str,
) -> Result<()> {
    let mut conn = open_db()?;
    let providers_json = serde_json::to_string(providers)
        .with_context(|| "Failed to serialize providers for agent runtime settings")?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO agent_runtime_settings
             (id, providers_json, active_agent, hooks_json, statuses_json, ai_json, git_json, namespaces_json, updated_at)
             VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               providers_json = excluded.providers_json,
               active_agent = excluded.active_agent,
               hooks_json = excluded.hooks_json,
               statuses_json = excluded.statuses_json,
               ai_json = excluded.ai_json,
               git_json = excluded.git_json,
               namespaces_json = excluded.namespaces_json,
               updated_at = excluded.updated_at",
        )
        .bind(&providers_json)
        .bind(active_agent)
        .bind(hooks_json)
        .bind(statuses_json)
        .bind(ai_json)
        .bind(git_json)
        .bind(namespaces_json)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn upsert_agent_artifact_registry_db(
    kind: &str,
    external_id: &str,
    name: &str,
    source_path: &str,
    content_hash: &str,
) -> Result<String> {
    let mut conn = open_db()?;
    let existing_uuid = block_on(async {
        sqlx::query(
            "SELECT uuid
             FROM agent_artifact_registry
             WHERE kind = ? AND external_id = ?",
        )
        .bind(kind)
        .bind(external_id)
        .fetch_optional(&mut conn)
        .await
    })?
    .map(|row| row.get::<String, _>(0));

    let uuid = existing_uuid.unwrap_or_else(crate::gen_nanoid);
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO agent_artifact_registry
                (uuid, kind, external_id, name, source_path, content_hash, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(kind, external_id) DO UPDATE SET
               name = excluded.name,
               source_path = excluded.source_path,
               content_hash = excluded.content_hash,
               updated_at = excluded.updated_at",
        )
        .bind(&uuid)
        .bind(kind)
        .bind(external_id)
        .bind(name)
        .bind(source_path)
        .bind(content_hash)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;

    Ok(uuid)
}

pub fn get_agent_artifact_registry_by_uuid_db(
    kind: &str,
    uuid: &str,
) -> Result<Option<AgentArtifactRegistryDb>> {
    let mut conn = open_db()?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT uuid, kind, external_id, name, source_path, content_hash
             FROM agent_artifact_registry
             WHERE kind = ? AND uuid = ?",
        )
        .bind(kind)
        .bind(uuid)
        .fetch_optional(&mut conn)
        .await
    })?;

    let Some(row) = row_opt else {
        return Ok(None);
    };

    Ok(Some(AgentArtifactRegistryDb {
        uuid: row.get(0),
        kind: row.get(1),
        external_id: row.get(2),
        name: row.get(3),
        source_path: row.get(4),
        content_hash: row.get(5),
    }))
}

pub fn get_agent_artifact_registry_by_external_id_db(
    kind: &str,
    external_id: &str,
) -> Result<Option<AgentArtifactRegistryDb>> {
    let mut conn = open_db()?;
    let row_opt = block_on(async {
        sqlx::query(
            "SELECT uuid, kind, external_id, name, source_path, content_hash
             FROM agent_artifact_registry
             WHERE kind = ? AND external_id = ?",
        )
        .bind(kind)
        .bind(external_id)
        .fetch_optional(&mut conn)
        .await
    })?;

    let Some(row) = row_opt else {
        return Ok(None);
    };

    Ok(Some(AgentArtifactRegistryDb {
        uuid: row.get(0),
        kind: row.get(1),
        external_id: row.get(2),
        name: row.get(3),
        source_path: row.get(4),
        content_hash: row.get(5),
    }))
}

pub fn list_agent_configs_db() -> Result<Vec<AgentConfigDb>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT id, name, description, active_tools_json, mcp_refs_json, skill_refs_json, rule_refs_json, prompt_id, hooks_json, permissions_json, target_agents_json
             FROM agent_config
             ORDER BY id ASC",
        )
        .fetch_all(&mut conn)
        .await
    })?;

    let mut modes = Vec::with_capacity(rows.len());
    for row in rows {
        modes.push(AgentConfigDb {
            id: row.get(0),
            name: row.get(1),
            description: row.get(2),
            active_tools_json: row.get(3),
            mcp_refs_json: row.get(4),
            skill_refs_json: row.get(5),
            rule_refs_json: row.get(6),
            prompt_id: row.get(7),
            hooks_json: row.get(8),
            permissions_json: row.get(9),
            target_agents_json: row.get(10),
        });
    }
    Ok(modes)
}

pub fn upsert_agent_config_db(mode: &AgentConfigDb) -> Result<()> {
    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO agent_config
                (id, name, description, active_tools_json, mcp_refs_json, skill_refs_json, rule_refs_json, prompt_id, hooks_json, permissions_json, target_agents_json, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name,
               description = excluded.description,
               active_tools_json = excluded.active_tools_json,
               mcp_refs_json = excluded.mcp_refs_json,
               skill_refs_json = excluded.skill_refs_json,
               rule_refs_json = excluded.rule_refs_json,
               prompt_id = excluded.prompt_id,
               hooks_json = excluded.hooks_json,
               permissions_json = excluded.permissions_json,
               target_agents_json = excluded.target_agents_json,
               updated_at = excluded.updated_at",
        )
        .bind(&mode.id)
        .bind(&mode.name)
        .bind(&mode.description)
        .bind(&mode.active_tools_json)
        .bind(&mode.mcp_refs_json)
        .bind(&mode.skill_refs_json)
        .bind(&mode.rule_refs_json)
        .bind(&mode.prompt_id)
        .bind(&mode.hooks_json)
        .bind(&mode.permissions_json)
        .bind(&mode.target_agents_json)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn delete_agent_config_db(id: &str) -> Result<()> {
    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("DELETE FROM agent_config WHERE id = ?")
            .bind(id)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}
