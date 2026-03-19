use anyhow::{Context, Result};
use sqlx::SqliteConnection;

use super::compat_workspace::apply_workspace_schema_compat;
use super::util::{block_on, column_exists, table_exists};

pub(super) fn ensure_column(
    connection: &mut SqliteConnection,
    table: &str,
    column: &str,
    alter_sql: &str,
) -> Result<bool> {
    if !table_exists(connection, table)? {
        return Ok(false);
    }

    if column_exists(connection, table, column)? {
        return Ok(false);
    }

    block_on(async { sqlx::query(alter_sql).execute(&mut *connection).await })
        .with_context(|| format!("Failed applying compatibility column {}.{}", table, column))?;
    Ok(true)
}

pub(super) fn ensure_project_schema_compat(connection: &mut SqliteConnection) -> Result<()> {
    ensure_column(
        connection,
        "feature",
        "branch",
        "ALTER TABLE feature ADD COLUMN branch TEXT",
    )?;
    ensure_column(
        connection,
        "feature",
        "agent_json",
        "ALTER TABLE feature ADD COLUMN agent_json TEXT",
    )?;
    ensure_column(
        connection,
        "feature",
        "tags_json",
        "ALTER TABLE feature ADD COLUMN tags_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    let added_feature_active_target = ensure_column(
        connection,
        "feature",
        "active_target_id",
        "ALTER TABLE feature ADD COLUMN active_target_id TEXT",
    )?;
    if table_exists(connection, "feature")?
        && column_exists(connection, "feature", "active_target_id")?
    {
        block_on(async {
            sqlx::query(
                "CREATE INDEX IF NOT EXISTS feature_active_target_idx ON feature(active_target_id)",
            )
            .execute(&mut *connection)
            .await
        })?;
    }
    if added_feature_active_target
        && table_exists(connection, "feature")?
        && column_exists(connection, "feature", "release_id")?
    {
        block_on(async {
            sqlx::query(
                "UPDATE feature
                 SET active_target_id = release_id
                 WHERE (active_target_id IS NULL OR active_target_id = '')
                   AND release_id IS NOT NULL
                   AND release_id != ''",
            )
            .execute(&mut *connection)
            .await
        })?;
    }
    ensure_column(
        connection,
        "release",
        "target_date",
        "ALTER TABLE release ADD COLUMN target_date TEXT",
    )?;
    ensure_column(
        connection,
        "release",
        "supported",
        "ALTER TABLE release ADD COLUMN supported INTEGER",
    )?;
    ensure_column(
        connection,
        "release",
        "body",
        "ALTER TABLE release ADD COLUMN body TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        connection,
        "agent_runtime_settings",
        "statuses_json",
        "ALTER TABLE agent_runtime_settings ADD COLUMN statuses_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column(
        connection,
        "agent_runtime_settings",
        "ai_json",
        "ALTER TABLE agent_runtime_settings ADD COLUMN ai_json TEXT",
    )?;
    ensure_column(
        connection,
        "agent_runtime_settings",
        "git_json",
        "ALTER TABLE agent_runtime_settings ADD COLUMN git_json TEXT NOT NULL DEFAULT '{}'",
    )?;
    ensure_column(
        connection,
        "agent_runtime_settings",
        "namespaces_json",
        "ALTER TABLE agent_runtime_settings ADD COLUMN namespaces_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column(
        connection,
        "spec",
        "body",
        "ALTER TABLE spec ADD COLUMN body TEXT NOT NULL DEFAULT ''",
    )?;
    let added_spec_workspace_id = ensure_column(
        connection,
        "spec",
        "workspace_id",
        "ALTER TABLE spec ADD COLUMN workspace_id TEXT",
    )?;
    if table_exists(connection, "spec")? && column_exists(connection, "spec", "workspace_id")? {
        block_on(async {
            sqlx::query("CREATE INDEX IF NOT EXISTS spec_workspace_idx ON spec(workspace_id)")
                .execute(&mut *connection)
                .await
        })?;
    }
    if table_exists(connection, "event_log")? {
        block_on(async {
            sqlx::query(
                "CREATE INDEX IF NOT EXISTS event_log_lookup_idx
                 ON event_log(timestamp, actor, entity, action, subject)",
            )
            .execute(&mut *connection)
            .await
        })?;
    }

    let added_branch_link_type = ensure_column(
        connection,
        "branch_context",
        "link_type",
        "ALTER TABLE branch_context ADD COLUMN link_type TEXT",
    )?;
    let added_branch_link_id = ensure_column(
        connection,
        "branch_context",
        "link_id",
        "ALTER TABLE branch_context ADD COLUMN link_id TEXT",
    )?;
    if table_exists(connection, "branch_context")?
        && (added_branch_link_type || added_branch_link_id)
        && column_exists(connection, "branch_context", "doc_type")?
        && column_exists(connection, "branch_context", "doc_id")?
    {
        block_on(async {
            sqlx::query(
                "UPDATE branch_context
                 SET link_type = COALESCE(NULLIF(link_type, ''), doc_type),
                     link_id = COALESCE(NULLIF(link_id, ''), doc_id)",
            )
            .execute(&mut *connection)
            .await
        })?;
    }

    apply_workspace_schema_compat(connection, added_spec_workspace_id)
}
