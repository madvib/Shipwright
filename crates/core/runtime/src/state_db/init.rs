use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::compat::ensure_project_schema_compat;
use super::migrations::{GLOBAL_MIGRATIONS, PROJECT_MIGRATIONS};
use super::types::DatabaseMigrationReport;
use super::util::{block_on, sqlite_url};

/// Returns `~/.ship/state/<project-slug>/ship.db` for the given ship_dir.
///
/// The state key is a human-readable, stable slug (`<project-name>-<project-id>`), so
/// operators can identify project DBs at a glance while still staying path-independent.
///
/// For compatibility, if an older ID-only directory exists (`state/<id>/ship.db`) and the
/// slug directory does not, Ship promotes the old directory to the slug directory once.
pub fn project_db_path(ship_dir: &Path) -> Result<PathBuf> {
    let global_dir = ship_global_dir()?;
    ensure_global_dir_outside_project(ship_dir, &global_dir)?;

    let key = project_db_key(ship_dir)?;
    promote_legacy_project_state_dir(ship_dir, &global_dir, &key)?;
    Ok(global_dir.join("state").join(key).join("ship.db"))
}

/// Stable key used for the project's state directory.
/// Uses a stable, human-readable slug derived from `ship.toml`.
///
/// This avoids calling `get_config` here to prevent dependency loops.
fn project_db_key(ship_dir: &Path) -> Result<String> {
    Ok(crate::project::project_slug_from_ship_dir(ship_dir))
}

fn legacy_project_db_key(ship_dir: &Path) -> Result<String> {
    crate::project::ensure_project_id(ship_dir)
}

fn promote_legacy_project_state_dir(
    ship_dir: &Path,
    global_dir: &Path,
    project_key: &str,
) -> Result<()> {
    let legacy_key = legacy_project_db_key(ship_dir)?;
    if legacy_key == project_key {
        return Ok(());
    }

    let state_root = global_dir.join("state");
    let project_dir = state_root.join(project_key);
    let project_db = project_dir.join("ship.db");
    if project_db.exists() {
        return Ok(());
    }

    let legacy_dir = state_root.join(&legacy_key);
    let legacy_db = legacy_dir.join("ship.db");
    if !legacy_db.exists() {
        return Ok(());
    }

    if let Some(parent) = project_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if std::fs::rename(&legacy_dir, &project_dir).is_ok() {
        return Ok(());
    }

    std::fs::create_dir_all(&project_dir)?;
    for suffix in ["", "-wal", "-shm"] {
        let src = legacy_dir.join(format!("ship.db{}", suffix));
        if !src.exists() {
            continue;
        }
        let dst = project_dir.join(format!("ship.db{}", suffix));
        let _ = std::fs::rename(&src, &dst).or_else(|_| {
            std::fs::copy(&src, &dst)?;
            std::fs::remove_file(&src)
        });
    }
    let _ = std::fs::remove_dir(&legacy_dir);
    Ok(())
}

pub fn ensure_project_database(ship_dir: &Path) -> Result<DatabaseMigrationReport> {
    let db_path = project_db_path(ship_dir)?;
    ensure_database(&db_path, PROJECT_MIGRATIONS)
}

pub fn ensure_global_database(global_dir: &Path) -> Result<DatabaseMigrationReport> {
    let db_path = global_dir.join("ship.db");
    ensure_database(&db_path, GLOBAL_MIGRATIONS)
}

pub(super) fn open_project_db(ship_dir: &Path) -> Result<SqliteConnection> {
    ensure_project_database(ship_dir)?;
    let db_path = project_db_path(ship_dir)?;
    let db_url = sqlite_url(&db_path);
    let options = SqliteConnectOptions::from_str(&db_url)
        .with_context(|| format!("Invalid sqlite url {}", db_url))?;
    let mut connection = block_on(async { SqliteConnection::connect_with(&options).await })?;
    ensure_project_schema_compat(&mut connection)?;
    Ok(connection)
}

/// Exposed for use by module crates (e.g. `ship-module-project`).
pub fn open_project_connection(ship_dir: &Path) -> Result<SqliteConnection> {
    open_project_db(ship_dir)
}

/// Exposed for use by module crates (e.g. `ship-module-project`).
pub fn open_global_connection() -> Result<SqliteConnection> {
    let global_dir = crate::project::get_global_dir()?;
    ensure_global_database(&global_dir)?;
    let db_path = global_dir.join("ship.db");
    let db_url = sqlite_url(&db_path);
    let options = SqliteConnectOptions::from_str(&db_url)
        .with_context(|| format!("Invalid sqlite url {}", db_url))?;
    block_on(async { SqliteConnection::connect_with(&options).await })
}

pub fn migration_meta_complete_project(ship_dir: &Path, entity_type: &str) -> Result<bool> {
    let mut conn = open_project_db(ship_dir)?;
    migration_meta_complete(&mut conn, entity_type)
}

pub fn migration_meta_complete_global(entity_type: &str) -> Result<bool> {
    let mut conn = open_global_connection()?;
    migration_meta_complete(&mut conn, entity_type)
}

pub fn mark_migration_meta_complete_project(
    ship_dir: &Path,
    entity_type: &str,
    file_count: usize,
) -> Result<()> {
    let mut conn = open_project_db(ship_dir)?;
    mark_migration_meta_complete(&mut conn, entity_type, file_count)
}

pub fn mark_migration_meta_complete_global(entity_type: &str, file_count: usize) -> Result<()> {
    let mut conn = open_global_connection()?;
    mark_migration_meta_complete(&mut conn, entity_type, file_count)
}

pub fn clear_project_migration_meta(ship_dir: &Path) -> Result<usize> {
    let mut conn = open_project_db(ship_dir)?;
    clear_migration_meta(&mut conn)
}

pub fn clear_global_migration_meta() -> Result<usize> {
    let mut conn = open_global_connection()?;
    clear_migration_meta(&mut conn)
}

pub(super) fn migration_meta_complete(conn: &mut SqliteConnection, entity_type: &str) -> Result<bool> {
    let row_opt = block_on(async {
        sqlx::query("SELECT entity_type FROM migration_meta WHERE entity_type = ?")
            .bind(entity_type)
            .fetch_optional(&mut *conn)
            .await
    })?;
    Ok(row_opt.is_some())
}

pub(super) fn mark_migration_meta_complete(
    conn: &mut SqliteConnection,
    entity_type: &str,
    file_count: usize,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO migration_meta (entity_type, migrated_at, file_count)
             VALUES (?, ?, ?)
             ON CONFLICT(entity_type) DO UPDATE SET
               migrated_at = excluded.migrated_at,
               file_count = excluded.file_count",
        )
        .bind(entity_type)
        .bind(&now)
        .bind(file_count as i64)
        .execute(&mut *conn)
        .await
    })?;
    Ok(())
}

pub(super) fn clear_migration_meta(conn: &mut SqliteConnection) -> Result<usize> {
    let result = block_on(async {
        sqlx::query("DELETE FROM migration_meta")
            .execute(&mut *conn)
            .await
    })?;
    Ok(result.rows_affected() as usize)
}

pub(super) fn ensure_database(
    db_path: &Path,
    migrations: &[(&str, &str)],
) -> Result<DatabaseMigrationReport> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let created = !db_path.exists();

    let db_url = sqlite_url(db_path);
    let options = SqliteConnectOptions::from_str(&db_url)
        .with_context(|| format!("Invalid sqlite url {}", db_url))?
        .create_if_missing(true);
    let mut connection = block_on(async { SqliteConnection::connect_with(&options).await })?;

    // Keep write behavior stable across CLI/UI/MCP call paths.
    block_on(async {
        sqlx::query("PRAGMA journal_mode = WAL;")
            .execute(&mut connection)
            .await
    })?;
    block_on(async {
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&mut connection)
            .await
    })?;

    let mut applied = 0usize;
    for (version, ddl) in migrations {
        let has_row = block_on(async {
            sqlx::query("SELECT version FROM schema_migrations WHERE version = ?")
                .bind(*version)
                .fetch_optional(&mut connection)
                .await
        })
        .ok()
        .flatten()
        .is_some();
        if has_row {
            continue;
        }

        block_on(async { sqlx::query(ddl).execute(&mut connection).await }).with_context(|| {
            format!(
                "Failed applying schema migration {} for {}",
                version,
                db_path.display()
            )
        })?;
        block_on(async {
            sqlx::query("INSERT INTO schema_migrations (version, applied_at) VALUES (?, ?)")
                .bind(*version)
                .bind(Utc::now().to_rfc3339())
                .execute(&mut connection)
                .await
        })
        .with_context(|| {
            format!(
                "Failed recording schema migration {} for {}",
                version,
                db_path.display()
            )
        })?;
        applied += 1;
    }

    Ok(DatabaseMigrationReport {
        db_path: db_path.to_path_buf(),
        created,
        applied_migrations: applied,
    })
}

fn ship_global_dir() -> Result<PathBuf> {
    crate::project::get_global_dir()
}

pub(super) fn ensure_global_dir_outside_project(ship_dir: &Path, global_dir: &Path) -> Result<()> {
    use anyhow::anyhow;
    let project_root = ship_dir.parent().unwrap_or(ship_dir);
    let normalize =
        |path: &Path| std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    let ship_dir_norm = normalize(ship_dir);
    let project_root_norm = normalize(project_root);
    let global_dir_norm = normalize(global_dir);

    if global_dir_norm.starts_with(&ship_dir_norm)
        || global_dir_norm.starts_with(&project_root_norm)
    {
        return Err(anyhow!(
            "Resolved global state directory {} is inside project {}. Refusing to write project state locally; expected ~/.ship (or another external absolute path).",
            global_dir_norm.display(),
            project_root_norm.display()
        ));
    }
    Ok(())
}
