use anyhow::Result;
use chrono::Utc;
use sqlx::{Connection, Row};
use tempfile::tempdir;

use crate::state_db::{
    ensure_project_database, open_project_connection,
    list_target_features_db, replace_target_features_db,
    list_capability_maps_db, list_capabilities_db, upsert_capability_db,
    upsert_capability_map_db, set_feature_primary_capability_db,
    get_feature_primary_capability_db,
    CapabilityDb, CapabilityMapDb,
};
use crate::state_db::block_on;

#[test]
fn capability_and_target_link_helpers_round_trip() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;
    ensure_project_database(&ship_dir)?;

    let now = Utc::now().to_rfc3339();
    let mut conn = open_project_connection(&ship_dir)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO release (id, version, status, created_at, updated_at)
             VALUES (?, ?, 'planned', ?, ?)",
        )
        .bind("target-q2")
        .bind("v0.2.0")
        .bind(&now)
        .bind(&now)
        .execute(&mut conn)
        .await?;

        sqlx::query(
            "INSERT INTO feature (id, title, created_at, updated_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind("feat-auth")
        .bind("Auth")
        .bind(&now)
        .bind(&now)
        .execute(&mut conn)
        .await?;

        Ok::<(), sqlx::Error>(())
    })?;
    block_on(async { conn.close().await })?;

    replace_target_features_db(&ship_dir, "target-q2", &["feat-auth".to_string()])?;
    let target_features = list_target_features_db(&ship_dir, "target-q2")?;
    assert_eq!(target_features, vec!["feat-auth".to_string()]);

    upsert_capability_map_db(
        &ship_dir,
        &CapabilityMapDb {
            id: "cap-map-main".to_string(),
            vision_ref: Some("vision.md".to_string()),
            created_at: now.clone(),
            updated_at: now.clone(),
        },
    )?;
    let maps = list_capability_maps_db(&ship_dir)?;
    assert!(maps.iter().any(|entry| entry.id == "cap-map-main"));

    upsert_capability_db(
        &ship_dir,
        &CapabilityDb {
            id: "cap-auth".to_string(),
            map_id: "cap-map-main".to_string(),
            title: "Authentication".to_string(),
            description: "Identity and auth flows".to_string(),
            parent_capability_id: None,
            status: "active".to_string(),
            ord: 0,
            created_at: now.clone(),
            updated_at: now,
        },
    )?;

    let capabilities = list_capabilities_db(&ship_dir, Some("cap-map-main"))?;
    assert!(capabilities.iter().any(|entry| entry.id == "cap-auth"));

    set_feature_primary_capability_db(&ship_dir, "feat-auth", Some("cap-auth"))?;
    assert_eq!(
        get_feature_primary_capability_db(&ship_dir, "feat-auth")?.as_deref(),
        Some("cap-auth")
    );
    Ok(())
}
