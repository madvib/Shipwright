//! Targets and capabilities — the north star layer above the job queue.
//! Targets are either milestones (v0.1.0) or surfaces (compiler, studio).
//! Capabilities are aspirational/actual states within a target.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use std::path::Path;

use crate::db::{block_on, open_db};
use crate::gen_nanoid;

#[derive(Debug, Clone)]
pub struct Target {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub goal: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct Capability {
    pub id: String,
    pub target_id: String,
    pub title: String,
    pub status: String,
    pub evidence: Option<String>,
    pub milestone_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

const T_COLS: &str = "id, kind, title, description, status, goal, created_at, updated_at";
const C_COLS: &str =
    "id, target_id, title, status, evidence, milestone_id, created_at, updated_at";

fn row_to_target(row: &sqlx::sqlite::SqliteRow) -> Target {
    Target {
        id: row.get("id"),
        kind: row.get("kind"),
        title: row.get("title"),
        description: row.get("description"),
        status: row.get("status"),
        goal: row.get("goal"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_capability(row: &sqlx::sqlite::SqliteRow) -> Capability {
    Capability {
        id: row.get("id"),
        target_id: row.get("target_id"),
        title: row.get("title"),
        status: row.get("status"),
        evidence: row.get("evidence"),
        milestone_id: row.get("milestone_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub fn create_target(
    ship_dir: &Path,
    kind: &str,
    title: &str,
    description: Option<&str>,
    goal: Option<&str>,
    status: Option<&str>,
) -> Result<Target> {
    let mut conn = open_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    let id = gen_nanoid();
    let status = status.unwrap_or("active");
    block_on(async {
        sqlx::query(
            "INSERT INTO target (id, kind, title, description, status, goal, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id).bind(kind).bind(title).bind(description)
        .bind(status).bind(goal).bind(&now).bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(Target {
        id,
        kind: kind.to_string(),
        title: title.to_string(),
        description: description.map(str::to_string),
        status: status.to_string(),
        goal: goal.map(str::to_string),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn list_targets(ship_dir: &Path, kind: Option<&str>) -> Result<Vec<Target>> {
    let mut conn = open_db(ship_dir)?;
    let rows = block_on(async {
        if let Some(k) = kind {
            sqlx::query(&format!("SELECT {T_COLS} FROM target WHERE kind = ? ORDER BY created_at ASC"))
                .bind(k)
                .fetch_all(&mut conn)
                .await
        } else {
            sqlx::query(&format!("SELECT {T_COLS} FROM target ORDER BY kind ASC, created_at ASC"))
                .fetch_all(&mut conn)
                .await
        }
    })?;
    Ok(rows.iter().map(row_to_target).collect())
}

pub fn get_target(ship_dir: &Path, id: &str) -> Result<Option<Target>> {
    let mut conn = open_db(ship_dir)?;
    let row = block_on(async {
        sqlx::query(&format!("SELECT {T_COLS} FROM target WHERE id = ?"))
            .bind(id)
            .fetch_optional(&mut conn)
            .await
    })?;
    Ok(row.as_ref().map(row_to_target))
}

pub fn create_capability(
    ship_dir: &Path,
    target_id: &str,
    title: &str,
    milestone_id: Option<&str>,
) -> Result<Capability> {
    let mut conn = open_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    let id = gen_nanoid();
    block_on(async {
        sqlx::query(
            "INSERT INTO capability (id, target_id, title, status, evidence, milestone_id, created_at, updated_at)
             VALUES (?, ?, ?, 'aspirational', NULL, ?, ?, ?)",
        )
        .bind(&id).bind(target_id).bind(title).bind(milestone_id)
        .bind(&now).bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(Capability {
        id,
        target_id: target_id.to_string(),
        title: title.to_string(),
        status: "aspirational".to_string(),
        evidence: None,
        milestone_id: milestone_id.map(str::to_string),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn mark_capability_actual(
    ship_dir: &Path,
    id: &str,
    evidence: &str,
) -> Result<()> {
    let mut conn = open_db(ship_dir)?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE capability SET status = 'actual', evidence = ?, updated_at = ? WHERE id = ?",
        )
        .bind(evidence).bind(&now).bind(id)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn list_capabilities(
    ship_dir: &Path,
    target_id: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<Capability>> {
    let mut conn = open_db(ship_dir)?;
    let rows = block_on(async {
        match (target_id, status) {
            (Some(t), Some(s)) => {
                sqlx::query(&format!("SELECT {C_COLS} FROM capability WHERE target_id = ? AND status = ? ORDER BY created_at ASC"))
                    .bind(t).bind(s).fetch_all(&mut conn).await
            }
            (Some(t), None) => {
                sqlx::query(&format!("SELECT {C_COLS} FROM capability WHERE target_id = ? ORDER BY status ASC, created_at ASC"))
                    .bind(t).fetch_all(&mut conn).await
            }
            (None, Some(s)) => {
                sqlx::query(&format!("SELECT {C_COLS} FROM capability WHERE status = ? ORDER BY target_id ASC, created_at ASC"))
                    .bind(s).fetch_all(&mut conn).await
            }
            (None, None) => {
                sqlx::query(&format!("SELECT {C_COLS} FROM capability ORDER BY target_id ASC, status ASC, created_at ASC"))
                    .fetch_all(&mut conn).await
            }
        }
    })?;
    Ok(rows.iter().map(row_to_capability).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::init_project;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        (tmp, ship_dir)
    }

    #[test]
    fn create_and_get_target() {
        let (_tmp, ship_dir) = setup();
        let t = create_target(&ship_dir, "milestone", "v0.1.0", Some("Funnel"), Some("ship in every project"), None).unwrap();
        assert_eq!(t.kind, "milestone");
        assert_eq!(t.status, "active");
        let fetched = get_target(&ship_dir, &t.id).unwrap().unwrap();
        assert_eq!(fetched.title, "v0.1.0");
    }

    #[test]
    fn list_targets_by_kind() {
        let (_tmp, ship_dir) = setup();
        create_target(&ship_dir, "milestone", "v0.1.0", None, None, None).unwrap();
        create_target(&ship_dir, "surface", "compiler", None, None, None).unwrap();
        let milestones = list_targets(&ship_dir, Some("milestone")).unwrap();
        assert_eq!(milestones.len(), 1);
        assert_eq!(milestones[0].title, "v0.1.0");
    }

    #[test]
    fn capability_lifecycle() {
        let (_tmp, ship_dir) = setup();
        let t = create_target(&ship_dir, "surface", "compiler", None, None, None).unwrap();
        let c = create_capability(&ship_dir, &t.id, "Profile compilation", None).unwrap();
        assert_eq!(c.status, "aspirational");

        mark_capability_actual(&ship_dir, &c.id, "test: profile_scaffold_parses").unwrap();
        let caps = list_capabilities(&ship_dir, Some(&t.id), Some("actual")).unwrap();
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].evidence.as_deref(), Some("test: profile_scaffold_parses"));
    }

    #[test]
    fn list_capabilities_all_filters() {
        let (_tmp, ship_dir) = setup();
        let t = create_target(&ship_dir, "milestone", "v0.1.0", None, None, None).unwrap();
        create_capability(&ship_dir, &t.id, "accounts", None).unwrap();
        let c2 = create_capability(&ship_dir, &t.id, "cli auth", None).unwrap();
        mark_capability_actual(&ship_dir, &c2.id, "ship login works").unwrap();

        let all = list_capabilities(&ship_dir, Some(&t.id), None).unwrap();
        assert_eq!(all.len(), 2);
        let actual = list_capabilities(&ship_dir, None, Some("actual")).unwrap();
        assert_eq!(actual.len(), 1);
    }
}
