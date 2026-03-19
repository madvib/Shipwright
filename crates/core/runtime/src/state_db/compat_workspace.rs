use anyhow::Result;
use sqlx::SqliteConnection;

use super::compat::ensure_column;
use super::util::{block_on, column_exists, table_exists};

/// Applies workspace and session schema compatibility adjustments.
/// `added_spec_workspace_id` indicates whether the spec.workspace_id column was
/// just added in this pass; used to trigger the spec backfill.
pub(super) fn apply_workspace_schema_compat(
    connection: &mut SqliteConnection,
    added_spec_workspace_id: bool,
) -> Result<()> {
    let added_workspace_id = ensure_column(
        connection,
        "workspace",
        "id",
        "ALTER TABLE workspace ADD COLUMN id TEXT",
    )?;
    ensure_column(
        connection,
        "workspace",
        "workspace_type",
        "ALTER TABLE workspace ADD COLUMN workspace_type TEXT NOT NULL DEFAULT 'feature'",
    )?;
    let added_workspace_status = ensure_column(
        connection,
        "workspace",
        "status",
        "ALTER TABLE workspace ADD COLUMN status TEXT NOT NULL DEFAULT 'active'",
    )?;
    ensure_column(
        connection,
        "workspace",
        "environment_id",
        "ALTER TABLE workspace ADD COLUMN environment_id TEXT",
    )?;
    ensure_column(
        connection,
        "workspace",
        "target_id",
        "ALTER TABLE workspace ADD COLUMN target_id TEXT",
    )?;
    ensure_column(
        connection,
        "workspace",
        "mcp_servers_json",
        "ALTER TABLE workspace ADD COLUMN mcp_servers_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column(
        connection,
        "workspace",
        "skills_json",
        "ALTER TABLE workspace ADD COLUMN skills_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column(
        connection,
        "workspace",
        "last_activated_at",
        "ALTER TABLE workspace ADD COLUMN last_activated_at TEXT",
    )?;
    ensure_column(
        connection,
        "workspace",
        "context_hash",
        "ALTER TABLE workspace ADD COLUMN context_hash TEXT",
    )?;
    ensure_column(
        connection,
        "workspace",
        "config_generation",
        "ALTER TABLE workspace ADD COLUMN config_generation INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        connection,
        "workspace",
        "compiled_at",
        "ALTER TABLE workspace ADD COLUMN compiled_at TEXT",
    )?;
    ensure_column(
        connection,
        "workspace",
        "compile_error",
        "ALTER TABLE workspace ADD COLUMN compile_error TEXT",
    )?;
    ensure_column(
        connection,
        "workspace_session",
        "primary_provider",
        "ALTER TABLE workspace_session ADD COLUMN primary_provider TEXT",
    )?;
    ensure_column(
        connection,
        "workspace_session",
        "compiled_at",
        "ALTER TABLE workspace_session ADD COLUMN compiled_at TEXT",
    )?;
    ensure_column(
        connection,
        "workspace_session",
        "compile_error",
        "ALTER TABLE workspace_session ADD COLUMN compile_error TEXT",
    )?;
    ensure_column(
        connection,
        "workspace_session",
        "config_generation_at_start",
        "ALTER TABLE workspace_session ADD COLUMN config_generation_at_start INTEGER",
    )?;
    if table_exists(connection, "workspace_session")? {
        block_on(async {
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS workspace_session_record (
                   id                 TEXT PRIMARY KEY,
                   session_id         TEXT NOT NULL UNIQUE REFERENCES workspace_session(id) ON DELETE CASCADE,
                   workspace_id       TEXT NOT NULL,
                   workspace_branch   TEXT NOT NULL,
                   summary            TEXT,
                   updated_feature_ids_json TEXT NOT NULL DEFAULT '[]',
                   created_at         TEXT NOT NULL
                 )",
            )
            .execute(&mut *connection)
            .await
        })?;
        block_on(async {
            sqlx::query(
                "CREATE INDEX IF NOT EXISTS workspace_session_record_workspace_idx
                 ON workspace_session_record(workspace_id, created_at DESC)",
            )
            .execute(&mut *connection)
            .await
        })?;
    }

    if table_exists(connection, "workspace")? {
        if column_exists(connection, "workspace", "target_id")?
            && column_exists(connection, "workspace", "release_id")?
        {
            block_on(async {
                sqlx::query(
                    "UPDATE workspace
                     SET target_id = release_id
                     WHERE (target_id IS NULL OR target_id = '')
                       AND release_id IS NOT NULL
                       AND release_id != ''",
                )
                .execute(&mut *connection)
                .await
            })?;
        }
        block_on(async {
            sqlx::query(
                "UPDATE workspace
                 SET workspace_type = lower(trim(workspace_type))
                 WHERE workspace_type IS NOT NULL
                   AND trim(workspace_type) != '';",
            )
            .execute(&mut *connection)
            .await
        })?;
        block_on(async {
            sqlx::query(
                "UPDATE workspace
                 SET workspace_type = 'feature'
                 WHERE workspace_type IS NULL
                    OR trim(workspace_type) = '';",
            )
            .execute(&mut *connection)
            .await
        })?;
        block_on(async {
            sqlx::query(
                "UPDATE workspace
                 SET status = 'active'
                 WHERE lower(trim(status)) = 'active';",
            )
            .execute(&mut *connection)
            .await
        })?;
        block_on(async {
            sqlx::query(
                "UPDATE workspace
                 SET status = 'archived'
                 WHERE lower(trim(status)) = 'archived';",
            )
            .execute(&mut *connection)
            .await
        })?;
        block_on(async {
            sqlx::query(
                "UPDATE workspace
                 SET status = 'archived'
                 WHERE status IS NOT NULL
                   AND trim(status) != ''
                   AND lower(trim(status)) NOT IN ('active', 'archived');",
            )
            .execute(&mut *connection)
            .await
        })?;
        block_on(async {
            sqlx::query(
                "UPDATE workspace SET status = 'active' WHERE status IS NULL OR trim(status) = '';",
            )
            .execute(&mut *connection)
            .await
        })?;

        if added_workspace_id {
            block_on(async {
                sqlx::query(
                    "UPDATE workspace
                     SET id = branch
                     WHERE id IS NULL OR id = ''",
                )
                .execute(&mut *connection)
                .await
            })?;
        }
        if added_workspace_status {
            // Existing pre-lifecycle rows represented currently checked-out work.
            // Preserve that behavior once when the status column is introduced.
            block_on(async {
                sqlx::query(
                    "UPDATE workspace
                     SET status = 'active'
                     WHERE status IS NULL OR status = ''",
                )
                .execute(&mut *connection)
                .await
            })?;
        }
    }

    if table_exists(connection, "spec")?
        && table_exists(connection, "workspace")?
        && (added_spec_workspace_id || column_exists(connection, "spec", "workspace_id")?)
    {
        block_on(async {
            sqlx::query(
                "UPDATE spec
                 SET workspace_id = (
                   SELECT w.id
                   FROM workspace w
                   WHERE (spec.branch IS NOT NULL AND spec.branch != '' AND w.branch = spec.branch)
                      OR (spec.feature_id IS NOT NULL AND spec.feature_id != '' AND w.feature_id = spec.feature_id)
                   ORDER BY
                     CASE WHEN w.status = 'active' THEN 0 ELSE 1 END,
                     COALESCE(w.last_activated_at, w.resolved_at) DESC
                   LIMIT 1
                 )
                 WHERE (workspace_id IS NULL OR workspace_id = '')",
            )
            .execute(&mut *connection)
            .await
        })?;
    }

    Ok(())
}
