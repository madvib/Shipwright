use anyhow::Result;
use chrono::Utc;
use std::path::Path;

use super::init::open_project_db;
use super::util::block_on;

/// Set/clear the primary capability for a feature.
pub fn set_feature_primary_capability_db(
    ship_dir: &Path,
    feature_id: &str,
    capability_id: Option<&str>,
) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query("DELETE FROM feature_capability WHERE feature_id = ? AND is_primary = 1")
            .bind(feature_id)
            .execute(&mut conn)
            .await?;

        if let Some(capability_id) = capability_id {
            sqlx::query(
                "INSERT INTO feature_capability (feature_id, capability_id, is_primary, created_at)
                 VALUES (?, ?, 1, ?)
                 ON CONFLICT(feature_id, capability_id)
                 DO UPDATE SET is_primary = 1",
            )
            .bind(feature_id)
            .bind(capability_id)
            .bind(&now)
            .execute(&mut conn)
            .await?;
        }

        Ok::<(), sqlx::Error>(())
    })?;
    Ok(())
}

/// Get the primary capability id for a feature when present.
pub fn get_feature_primary_capability_db(
    ship_dir: &Path,
    feature_id: &str,
) -> Result<Option<String>> {
    let mut conn = open_project_db(ship_dir)?;
    block_on(async {
        sqlx::query_scalar::<_, String>(
            "SELECT capability_id
             FROM feature_capability
             WHERE feature_id = ? AND is_primary = 1
             ORDER BY created_at DESC
             LIMIT 1",
        )
        .bind(feature_id)
        .fetch_optional(&mut conn)
        .await
    })
}
