//! Artifact registry CRUD — content-addressed registry of compiled artifacts.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;

use super::types::AgentArtifactRegistryDb;
use super::{block_on, open_db};

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
