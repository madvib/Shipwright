use super::types::{Release, ReleaseBreakingChange, ReleaseEntry, ReleaseMetadata, ReleaseStatus};
use anyhow::Result;
use chrono::Utc;
use sqlx::{Connection, Row};
use std::path::Path;

pub fn upsert_release_db(ship_dir: &Path, release: &Release, status: &ReleaseStatus) -> Result<()> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    let now = Utc::now().to_rfc3339();

    runtime::state_db::block_on(async {
        let mut tx = conn.begin().await?;

        // Upsert release
        sqlx::query(
            "INSERT INTO release
               (id, version, status, target_date, supported, body, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               version     = excluded.version,
               status      = excluded.status,
               target_date = excluded.target_date,
               supported   = excluded.supported,
               body        = excluded.body,
               updated_at  = excluded.updated_at",
        )
        .bind(&release.metadata.id)
        .bind(&release.metadata.version)
        .bind(status.to_string())
        .bind(&release.metadata.target_date)
        .bind(release.metadata.supported.map(|s| if s { 1 } else { 0 }))
        .bind(&release.body)
        .bind(&release.metadata.created)
        .bind(&now)
        .execute(&mut *tx)
        .await?;

        // Delete existing breaking changes to replace
        sqlx::query("DELETE FROM release_breaking_change WHERE release_id = ?")
            .bind(&release.metadata.id)
            .execute(&mut *tx)
            .await?;

        // Insert breaking changes
        for (i, bc) in release.breaking_changes.iter().enumerate() {
            sqlx::query(
                "INSERT INTO release_breaking_change (id, release_id, text, ord) VALUES (?, ?, ?, ?)",
            )
            .bind(&bc.id)
            .bind(&release.metadata.id)
            .bind(&bc.text)
            .bind(i as i64)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    })?;
    Ok(())
}

pub fn get_release_db(ship_dir: &Path, id: &str) -> Result<Option<ReleaseEntry>> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        let row_opt = sqlx::query(
            "SELECT id, version, status, target_date, supported, body, created_at, updated_at
             FROM release WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut conn)
        .await?;

        if let Some(r) = row_opt {
            let id: String = r.get(0);
            let version: String = r.get(1);
            let status_str: String = r.get(2);
            let target_date: Option<String> = r.get(3);
            let supported: Option<i64> = r.get(4);
            let body: String = r.get(5);
            let created: String = r.get(6);
            let updated: String = r.get(7);

            let status = status_str.parse::<ReleaseStatus>().unwrap_or_default();

            // Fetch breaking changes
            let bc_rows = sqlx::query(
                "SELECT id, text FROM release_breaking_change WHERE release_id = ? ORDER BY ord ASC",
            )
            .bind(&id)
            .fetch_all(&mut conn)
            .await?;

            let breaking_changes = bc_rows
                .into_iter()
                .map(|br| ReleaseBreakingChange {
                    id: br.get(0),
                    text: br.get(1),
                })
                .collect();

            let file_name = format!("{}.md", version);
            let path = runtime::project::releases_dir(ship_dir)
                .join(&file_name)
                .to_string_lossy()
                .to_string();

            Ok(Some(ReleaseEntry {
                id: id.clone(),
                file_name,
                path,
                version: version.clone(),
                status,
                release: Release {
                    metadata: ReleaseMetadata {
                        id,
                        version,
                        status: status.clone(),
                        created,
                        updated,
                        supported: supported.map(|s| s != 0),
                        target_date,
                        tags: Vec::new(),
                    },
                    body,
                    breaking_changes,
                },
            }))
        } else {
            Ok(None)
        }
    })
}

pub fn list_releases_db(ship_dir: &Path) -> Result<Vec<ReleaseEntry>> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        let rows = sqlx::query(
            "SELECT id, version, status, target_date, supported, body, created_at, updated_at
             FROM release ORDER BY version DESC",
        )
        .fetch_all(&mut conn)
        .await?;

        let mut entries = Vec::new();
        for r in rows {
            let id: String = r.get(0);
            let version: String = r.get(1);
            let status_str: String = r.get(2);
            let status = status_str.parse::<ReleaseStatus>().unwrap_or_default();
            let body: String = r.get(5);
            let file_name = format!("{}.md", version);
            let path = runtime::project::releases_dir(ship_dir)
                .join(&file_name)
                .to_string_lossy()
                .to_string();

            entries.push(ReleaseEntry {
                id: id.clone(),
                file_name,
                path,
                version: version.clone(),
                status: status.clone(),
                release: Release {
                    metadata: ReleaseMetadata {
                        id,
                        version,
                        status,
                        created: r.get(6),
                        updated: r.get(7),
                        supported: r.get::<Option<i64>, _>(4).map(|s| s != 0),
                        target_date: r.get(3),
                        tags: Vec::new(),
                    },
                    body,
                    breaking_changes: Vec::new(),
                },
            });
        }
        Ok(entries)
    })
}

pub fn delete_release_db(ship_dir: &Path, id: &str) -> Result<()> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        sqlx::query("DELETE FROM release WHERE id = ?")
            .bind(id)
            .execute(&mut conn)
            .await?;
        Ok(())
    })?;
    Ok(())
}
