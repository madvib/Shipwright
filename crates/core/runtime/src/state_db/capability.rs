use anyhow::Result;
use sqlx::Row;
use std::path::Path;

use super::init::open_project_db;
use super::types::{CapabilityDb, CapabilityMapDb};
use super::util::block_on;

pub fn upsert_capability_map_db(ship_dir: &Path, map: &CapabilityMapDb) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO capability_map (id, vision_ref, created_at, updated_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(id)
             DO UPDATE SET
               vision_ref = excluded.vision_ref,
               updated_at = excluded.updated_at",
        )
        .bind(&map.id)
        .bind(&map.vision_ref)
        .bind(&map.created_at)
        .bind(&map.updated_at)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn list_capability_maps_db(ship_dir: &Path) -> Result<Vec<CapabilityMapDb>> {
    let mut conn = open_project_db(ship_dir)?;
    let rows = block_on(async {
        sqlx::query(
            "SELECT id, vision_ref, created_at, updated_at
             FROM capability_map
             ORDER BY updated_at DESC, id ASC",
        )
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows
        .into_iter()
        .map(|row| CapabilityMapDb {
            id: row.get(0),
            vision_ref: row.get(1),
            created_at: row.get(2),
            updated_at: row.get(3),
        })
        .collect())
}

pub fn upsert_capability_db(ship_dir: &Path, capability: &CapabilityDb) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO capability
             (id, map_id, title, description, parent_capability_id, status, ord, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id)
             DO UPDATE SET
               map_id = excluded.map_id,
               title = excluded.title,
               description = excluded.description,
               parent_capability_id = excluded.parent_capability_id,
               status = excluded.status,
               ord = excluded.ord,
               updated_at = excluded.updated_at",
        )
        .bind(&capability.id)
        .bind(&capability.map_id)
        .bind(&capability.title)
        .bind(&capability.description)
        .bind(&capability.parent_capability_id)
        .bind(&capability.status)
        .bind(capability.ord)
        .bind(&capability.created_at)
        .bind(&capability.updated_at)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn list_capabilities_db(ship_dir: &Path, map_id: Option<&str>) -> Result<Vec<CapabilityDb>> {
    let mut conn = open_project_db(ship_dir)?;
    let rows = if let Some(map_id) = map_id {
        block_on(async {
            sqlx::query(
                "SELECT id, map_id, title, description, parent_capability_id, status, ord, created_at, updated_at
                 FROM capability
                 WHERE map_id = ?
                 ORDER BY ord ASC, updated_at DESC",
            )
            .bind(map_id)
            .fetch_all(&mut conn)
            .await
        })?
    } else {
        block_on(async {
            sqlx::query(
                "SELECT id, map_id, title, description, parent_capability_id, status, ord, created_at, updated_at
                 FROM capability
                 ORDER BY map_id ASC, ord ASC, updated_at DESC",
            )
            .fetch_all(&mut conn)
            .await
        })?
    };

    Ok(rows
        .into_iter()
        .map(|row| CapabilityDb {
            id: row.get(0),
            map_id: row.get(1),
            title: row.get(2),
            description: row.get(3),
            parent_capability_id: row.get(4),
            status: row.get(5),
            ord: row.get(6),
            created_at: row.get(7),
            updated_at: row.get(8),
        })
        .collect())
}
