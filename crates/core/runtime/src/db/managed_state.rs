//! Managed MCP state — server ids and last mode per provider.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::Row;
use std::path::Path;

use super::{block_on, open_db};

/// Returns `(server_ids, last_mode)` for the given provider, or empty defaults.
pub fn get_managed_state_db(
    _ship_dir: &Path,
    provider: &str,
) -> Result<(Vec<String>, Option<String>)> {
    let mut conn = open_db()?;
    let row_opt = block_on(async {
        sqlx::query("SELECT server_ids_json, last_mode FROM managed_mcp_state WHERE provider = ?")
            .bind(provider)
            .fetch_optional(&mut conn)
            .await
    })?;
    if let Some(row) = row_opt {
        let ids_json: String = row.get(0);
        let last_mode: Option<String> = row.get(1);
        let ids: Vec<String> = serde_json::from_str(&ids_json).unwrap_or_default();
        Ok((ids, last_mode))
    } else {
        Ok((Vec::new(), None))
    }
}

/// Persist the managed server ids and last mode for the given provider.
pub fn set_managed_state_db(
    _ship_dir: &Path,
    provider: &str,
    ids: &[String],
    last_mode: Option<&str>,
) -> Result<()> {
    let mut conn = open_db()?;
    let ids_json = serde_json::to_string(ids)
        .with_context(|| format!("Failed to serialize server ids for provider {}", provider))?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO managed_mcp_state (provider, server_ids_json, last_mode, updated_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(provider) DO UPDATE SET
               server_ids_json = excluded.server_ids_json,
               last_mode = excluded.last_mode,
               updated_at = excluded.updated_at",
        )
        .bind(provider)
        .bind(&ids_json)
        .bind(last_mode)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}
