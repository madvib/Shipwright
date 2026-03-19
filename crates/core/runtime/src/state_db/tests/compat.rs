use anyhow::Result;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, Row, SqliteConnection};
use std::str::FromStr;
use tempfile::tempdir;

use crate::state_db::compat::ensure_project_schema_compat;
use crate::state_db::util::sqlite_url;
use crate::state_db::block_on;

#[test]
fn compat_workspace_backfills_only_when_columns_are_added() -> Result<()> {
    let tmp = tempdir()?;
    let db_path = tmp.path().join("compat.db");
    let db_url = sqlite_url(&db_path);
    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&options).await })?;

    block_on(async {
        sqlx::query(
            "CREATE TABLE workspace (
               branch TEXT PRIMARY KEY,
               feature_id TEXT,
               spec_id TEXT,
               active_agent TEXT,
               providers_json TEXT NOT NULL DEFAULT '[]',
               resolved_at TEXT NOT NULL,
               is_worktree INTEGER NOT NULL DEFAULT 0,
               worktree_path TEXT
             )",
        )
        .execute(&mut conn)
        .await
    })?;
    block_on(async {
        sqlx::query(
            "INSERT INTO workspace (branch, resolved_at) VALUES ('feature/demo', '2026-01-01T00:00:00Z')",
        )
        .execute(&mut conn)
        .await
    })?;

    ensure_project_schema_compat(&mut conn)?;

    let row = block_on(async {
        sqlx::query("SELECT id, status FROM workspace WHERE branch = 'feature/demo'")
            .fetch_one(&mut conn)
            .await
    })?;
    let id: Option<String> = row.get(0);
    let status: Option<String> = row.get(1);
    assert_eq!(id.as_deref(), Some("feature/demo"));
    assert_eq!(status.as_deref(), Some("active"));
    Ok(())
}

#[test]
fn compat_workspace_contract_enforces_canonical_workspace_values() -> Result<()> {
    let tmp = tempdir()?;
    let db_path = tmp.path().join("compat-contract.db");
    let db_url = sqlite_url(&db_path);
    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&options).await })?;

    block_on(async {
        sqlx::query(
            "CREATE TABLE workspace (
               branch TEXT PRIMARY KEY,
               id TEXT,
               workspace_type TEXT NOT NULL DEFAULT 'feature',
               status TEXT NOT NULL DEFAULT 'active',
               feature_id TEXT,
               spec_id TEXT,
               release_id TEXT,
               active_agent TEXT,
               providers_json TEXT NOT NULL DEFAULT '[]',
               resolved_at TEXT NOT NULL,
               is_worktree INTEGER NOT NULL DEFAULT 0,
               worktree_path TEXT,
               last_activated_at TEXT,
               context_hash TEXT
             )",
        )
        .execute(&mut conn)
        .await?;
        sqlx::query(
            "INSERT INTO workspace (branch, id, workspace_type, status, resolved_at)
             VALUES
               ('legacy/patch', 'legacy-patch', 'PATCH', 'planned', '2026-01-01T00:00:00Z'),
               ('legacy/service', 'legacy-service', 'SERVICE', 'ARCHIVED', '2026-01-01T00:00:00Z'),
               ('legacy/unknown', 'legacy-unknown', 'spike', 'review', '2026-01-01T00:00:00Z'),
               ('legacy/empty', 'legacy-empty', 'feature', '', '2026-01-01T00:00:00Z')",
        )
        .execute(&mut conn)
        .await?;
        Ok::<_, sqlx::Error>(())
    })?;

    ensure_project_schema_compat(&mut conn)?;

    let rows = block_on(async {
        sqlx::query("SELECT branch, workspace_type, status FROM workspace ORDER BY branch")
            .fetch_all(&mut conn)
            .await
    })?;
    let mut normalized = std::collections::HashMap::new();
    for row in rows {
        let branch: String = row.get(0);
        let workspace_type: String = row.get(1);
        let status: String = row.get(2);
        normalized.insert(branch, (workspace_type, status));
    }

    assert_eq!(
        normalized.get("legacy/patch"),
        Some(&("patch".to_string(), "archived".to_string()))
    );
    assert_eq!(
        normalized.get("legacy/service"),
        Some(&("service".to_string(), "archived".to_string()))
    );
    assert_eq!(
        normalized.get("legacy/unknown"),
        Some(&("spike".to_string(), "archived".to_string()))
    );
    assert_eq!(
        normalized.get("legacy/empty"),
        Some(&("feature".to_string(), "active".to_string()))
    );

    Ok(())
}

#[test]
fn compat_branch_context_backfills_link_columns_from_legacy_doc_columns() -> Result<()> {
    let tmp = tempdir()?;
    let db_path = tmp.path().join("branch-context-compat.db");
    let db_url = sqlite_url(&db_path);
    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&options).await })?;

    block_on(async {
        sqlx::query(
            "CREATE TABLE branch_context (
               branch TEXT PRIMARY KEY,
               doc_type TEXT NOT NULL,
               doc_id TEXT NOT NULL,
               last_synced TEXT NOT NULL
             )",
        )
        .execute(&mut conn)
        .await
    })?;
    block_on(async {
        sqlx::query(
            "INSERT INTO branch_context (branch, doc_type, doc_id, last_synced)
             VALUES ('feature/auth', 'feature', 'ABC123', '2026-01-01T00:00:00Z')",
        )
        .execute(&mut conn)
        .await
    })?;

    ensure_project_schema_compat(&mut conn)?;

    let row = block_on(async {
        sqlx::query(
            "SELECT link_type, link_id
             FROM branch_context
             WHERE branch = 'feature/auth'",
        )
        .fetch_one(&mut conn)
        .await
    })?;
    let link_type: Option<String> = row.get(0);
    let link_id: Option<String> = row.get(1);
    assert_eq!(link_type.as_deref(), Some("feature"));
    assert_eq!(link_id.as_deref(), Some("ABC123"));
    Ok(())
}

#[test]
fn compat_adds_spec_body_and_workspace_columns() -> Result<()> {
    let tmp = tempdir()?;
    let db_path = tmp.path().join("spec-compat.db");
    let db_url = sqlite_url(&db_path);
    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&options).await })?;

    block_on(async {
        sqlx::query(
            "CREATE TABLE spec (
               id TEXT PRIMARY KEY,
               title TEXT NOT NULL,
               status TEXT NOT NULL DEFAULT 'draft',
               author TEXT,
               branch TEXT,
               feature_id TEXT,
               release_id TEXT,
               tags_json TEXT NOT NULL DEFAULT '[]',
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL
             )",
        )
        .execute(&mut conn)
        .await
    })?;

    ensure_project_schema_compat(&mut conn)?;

    use crate::state_db::util::column_exists;
    assert!(column_exists(&mut conn, "spec", "body")?);
    assert!(column_exists(&mut conn, "spec", "workspace_id")?);
    Ok(())
}

#[test]
fn compat_backfills_spec_workspace_id_from_workspace_branch_or_feature() -> Result<()> {
    let tmp = tempdir()?;
    let db_path = tmp.path().join("spec-workspace-backfill.db");
    let db_url = sqlite_url(&db_path);
    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&options).await })?;

    block_on(async {
        sqlx::query(
            "CREATE TABLE workspace (
               branch TEXT PRIMARY KEY,
               id TEXT,
               workspace_type TEXT NOT NULL DEFAULT 'feature',
               status TEXT NOT NULL DEFAULT 'planned',
               feature_id TEXT,
               spec_id TEXT,
               release_id TEXT,
               active_agent TEXT,
               providers_json TEXT NOT NULL DEFAULT '[]',
               resolved_at TEXT NOT NULL DEFAULT '',
               is_worktree INTEGER NOT NULL DEFAULT 0,
               worktree_path TEXT,
               last_activated_at TEXT,
               context_hash TEXT
             )",
        )
        .execute(&mut conn)
        .await?;
        sqlx::query(
            "INSERT INTO workspace (branch, id, status, feature_id, resolved_at)
             VALUES ('feature/auth', 'feature-auth', 'active', 'feat-auth', '2026-01-01T00:00:00Z')",
        )
        .execute(&mut conn)
        .await?;
        Ok::<_, sqlx::Error>(())
    })?;

    block_on(async {
        sqlx::query(
            "CREATE TABLE spec (
               id TEXT PRIMARY KEY,
               title TEXT NOT NULL,
               status TEXT NOT NULL DEFAULT 'draft',
               author TEXT,
               branch TEXT,
               feature_id TEXT,
               release_id TEXT,
               tags_json TEXT NOT NULL DEFAULT '[]',
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL
             )",
        )
        .execute(&mut conn)
        .await?;
        sqlx::query(
            "INSERT INTO spec (id, title, branch, feature_id, tags_json, created_at, updated_at)
             VALUES ('spec-auth', 'Auth Spec', 'feature/auth', 'feat-auth', '[]', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        )
        .execute(&mut conn)
        .await?;
        Ok::<_, sqlx::Error>(())
    })?;

    ensure_project_schema_compat(&mut conn)?;

    let workspace_id: Option<String> = block_on(async {
        sqlx::query("SELECT workspace_id FROM spec WHERE id = 'spec-auth'")
            .fetch_one(&mut conn)
            .await
            .map(|row| row.get(0))
    })?;
    assert_eq!(workspace_id.as_deref(), Some("feature-auth"));

    Ok(())
}
