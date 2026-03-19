use anyhow::{Result, anyhow};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, Row, SqliteConnection};
use std::str::FromStr;
use tempfile::tempdir;

use crate::state_db::init::ensure_database;
use crate::state_db::migrations::PROJECT_MIGRATIONS;
use crate::state_db::util::{sqlite_url, table_exists, column_exists};
use crate::state_db::block_on;

#[test]
fn runtime_primitives_migration_creates_projection_tables() -> Result<()> {
    let tmp = tempdir()?;
    let db_path = tmp.path().join("runtime-primitives.db");
    ensure_database(&db_path, PROJECT_MIGRATIONS)?;

    let db_url = sqlite_url(&db_path);
    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&options).await })?;

    assert!(table_exists(&mut conn, "environment")?);
    assert!(table_exists(&mut conn, "runtime_process")?);
    assert!(table_exists(&mut conn, "git_workspace")?);

    assert!(column_exists(&mut conn, "workspace", "environment_id")?);
    Ok(())
}

#[test]
fn workspace_runtime_contract_migration_normalizes_status_and_casing_only() -> Result<()> {
    let tmp = tempdir()?;
    let db_path = tmp.path().join("workspace-contract.db");
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
            "INSERT INTO workspace (branch, id, workspace_type, status, resolved_at)
             VALUES
             ('legacy/patch', 'w-patch', 'PATCH', 'planned', '2026-01-01T00:00:00Z'),
               ('legacy/service', 'w-service', 'SERVICE', 'ARCHIVED', '2026-01-01T00:00:00Z'),
               ('legacy/unknown', 'w-unknown', 'spike', 'review', '2026-01-01T00:00:00Z'),
               ('legacy/empty-status', 'w-empty', 'feature', '', '2026-01-01T00:00:00Z')",
        )
        .execute(&mut conn)
        .await?;
        Ok::<_, sqlx::Error>(())
    })?;

    let ddl_0017 = PROJECT_MIGRATIONS
        .iter()
        .find(|(version, _)| *version == "0017_workspace_runtime_contract")
        .map(|(_, ddl)| *ddl)
        .ok_or_else(|| anyhow!("migration 0017 not found"))?;
    block_on(async { sqlx::query(ddl_0017).execute(&mut conn).await })?;

    let patch_row = block_on(async {
        sqlx::query(
            "SELECT workspace_type, status FROM workspace WHERE branch = 'legacy/patch'",
        )
        .fetch_one(&mut conn)
        .await
    })?;
    assert_eq!(patch_row.get::<String, _>(0), "patch");
    assert_eq!(patch_row.get::<String, _>(1), "archived");

    let service_row = block_on(async {
        sqlx::query(
            "SELECT workspace_type, status FROM workspace WHERE branch = 'legacy/service'",
        )
        .fetch_one(&mut conn)
        .await
    })?;
    assert_eq!(service_row.get::<String, _>(0), "service");
    assert_eq!(service_row.get::<String, _>(1), "archived");

    let unknown_row = block_on(async {
        sqlx::query(
            "SELECT workspace_type, status FROM workspace WHERE branch = 'legacy/unknown'",
        )
        .fetch_one(&mut conn)
        .await
    })?;
    assert_eq!(unknown_row.get::<String, _>(0), "spike");
    assert_eq!(unknown_row.get::<String, _>(1), "archived");

    let empty_row = block_on(async {
        sqlx::query(
            "SELECT workspace_type, status FROM workspace WHERE branch = 'legacy/empty-status'",
        )
        .fetch_one(&mut conn)
        .await
    })?;
    assert_eq!(empty_row.get::<String, _>(0), "feature");
    assert_eq!(empty_row.get::<String, _>(1), "active");
    Ok(())
}

#[test]
fn migrations_create_capability_and_target_link_tables() -> Result<()> {
    let tmp = tempdir()?;
    let db_path = tmp.path().join("capability-target-links.db");
    ensure_database(&db_path, PROJECT_MIGRATIONS)?;

    let db_url = sqlite_url(&db_path);
    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&options).await })?;

    assert!(table_exists(&mut conn, "capability_map")?);
    assert!(table_exists(&mut conn, "capability")?);
    assert!(table_exists(&mut conn, "feature_capability")?);
    assert!(table_exists(&mut conn, "target_feature")?);
    Ok(())
}

#[test]
fn migration_meta_project_roundtrip() -> Result<()> {
    let tmp = tempdir()?;
    let db_path = tmp.path().join("migration-meta.db");
    ensure_database(&db_path, PROJECT_MIGRATIONS)?;

    let db_url = sqlite_url(&db_path);
    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&options).await })?;

    use crate::state_db::init::{clear_migration_meta, mark_migration_meta_complete, migration_meta_complete};

    assert!(!migration_meta_complete(&mut conn, "feature")?);
    assert!(!migration_meta_complete(&mut conn, "release")?);

    mark_migration_meta_complete(&mut conn, "feature", 3)?;
    mark_migration_meta_complete(&mut conn, "release", 1)?;

    assert!(migration_meta_complete(&mut conn, "feature")?);
    assert!(migration_meta_complete(&mut conn, "release")?);

    let cleared = clear_migration_meta(&mut conn)?;
    assert_eq!(cleared, 2);
    assert!(!migration_meta_complete(&mut conn, "feature")?);
    assert!(!migration_meta_complete(&mut conn, "release")?);
    Ok(())
}
