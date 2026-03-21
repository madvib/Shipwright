//! Targets and capabilities — the north star layer above the job queue.
//! Targets are milestones (v0.1.0) or surfaces (compiler, studio).
//! Capabilities are required features/properties tracked from aspirational → actual.

use anyhow::Result;
use chrono::Utc;
use sqlx::{QueryBuilder, Row};
use std::path::Path;

use crate::db::{block_on, open_db};
use crate::gen_nanoid;

// ─── Structs ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Target {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub goal: Option<String>,
    pub phase: Option<String>,
    pub due_date: Option<String>,
    pub body_markdown: Option<String>,
    pub file_scope: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// status: aspirational | in_progress | actual
#[derive(Debug, Clone)]
pub struct Capability {
    pub id: String,
    pub target_id: String,
    pub title: String,
    pub status: String,
    pub evidence: Option<String>,
    pub milestone_id: Option<String>,
    pub phase: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub preset_hint: Option<String>,
    pub file_scope: Vec<String>,
    pub assigned_to: Option<String>,
    pub priority: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Default)]
pub struct TargetPatch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub status: Option<String>,
    pub phase: Option<String>,
    pub due_date: Option<String>,
    pub body_markdown: Option<String>,
    pub file_scope: Option<Vec<String>>,
}

#[derive(Debug, Default)]
pub struct CapabilityPatch {
    pub title: Option<String>,
    pub status: Option<String>,
    pub phase: Option<String>,
    pub acceptance_criteria: Option<Vec<String>>,
    pub preset_hint: Option<String>,
    pub file_scope: Option<Vec<String>>,
    pub assigned_to: Option<String>,
    pub priority: Option<i32>,
}

// ─── Column lists ─────────────────────────────────────────────────────────────

const T_COLS: &str =
    "id, kind, title, description, status, goal, phase, due_date, body_markdown, file_scope_json, created_at, updated_at";

const C_COLS: &str =
    "id, target_id, title, status, evidence, milestone_id, phase, acceptance_criteria, \
     preset_hint, file_scope, assigned_to, priority, created_at, updated_at";

// ─── Row mapping ──────────────────────────────────────────────────────────────

fn row_to_target(row: &sqlx::sqlite::SqliteRow) -> Target {
    let scope: Option<String> = row.get("file_scope_json");
    Target {
        id: row.get("id"),
        kind: row.get("kind"),
        title: row.get("title"),
        description: row.get("description"),
        status: row.get("status"),
        goal: row.get("goal"),
        phase: row.get("phase"),
        due_date: row.get("due_date"),
        body_markdown: row.get("body_markdown"),
        file_scope: scope.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_capability(row: &sqlx::sqlite::SqliteRow) -> Capability {
    let ac: Option<String> = row.get("acceptance_criteria");
    let scope: Option<String> = row.get("file_scope");
    Capability {
        id: row.get("id"),
        target_id: row.get("target_id"),
        title: row.get("title"),
        status: row.get("status"),
        evidence: row.get("evidence"),
        milestone_id: row.get("milestone_id"),
        phase: row.get("phase"),
        acceptance_criteria: ac.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default(),
        preset_hint: row.get("preset_hint"),
        file_scope: scope.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default(),
        assigned_to: row.get("assigned_to"),
        priority: row.try_get("priority").unwrap_or(0),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

// ─── Target operations ────────────────────────────────────────────────────────

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
        phase: None,
        due_date: None,
        body_markdown: None,
        file_scope: vec![],
        created_at: now.clone(),
        updated_at: now,
    })
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

pub fn list_targets(ship_dir: &Path, kind: Option<&str>) -> Result<Vec<Target>> {
    let mut conn = open_db(ship_dir)?;
    let rows = block_on(async {
        if let Some(k) = kind {
            sqlx::query(&format!("SELECT {T_COLS} FROM target WHERE kind = ? ORDER BY created_at ASC"))
                .bind(k).fetch_all(&mut conn).await
        } else {
            sqlx::query(&format!("SELECT {T_COLS} FROM target ORDER BY kind ASC, created_at ASC"))
                .fetch_all(&mut conn).await
        }
    })?;
    Ok(rows.iter().map(row_to_target).collect())
}

pub fn update_target(ship_dir: &Path, id: &str, patch: TargetPatch) -> Result<()> {
    let current = get_target(ship_dir, id)?
        .ok_or_else(|| anyhow::anyhow!("target {id} not found"))?;
    let now = Utc::now().to_rfc3339();
    let scope = serde_json::to_string(&patch.file_scope.unwrap_or(current.file_scope))?;
    let mut conn = open_db(ship_dir)?;
    block_on(async {
        sqlx::query(
            "UPDATE target SET title=?, description=?, goal=?, status=?, phase=?, \
             due_date=?, body_markdown=?, file_scope_json=?, updated_at=? WHERE id=?",
        )
        .bind(patch.title.unwrap_or(current.title))
        .bind(patch.description.or(current.description))
        .bind(patch.goal.or(current.goal))
        .bind(patch.status.unwrap_or(current.status))
        .bind(patch.phase.or(current.phase))
        .bind(patch.due_date.or(current.due_date))
        .bind(patch.body_markdown.or(current.body_markdown))
        .bind(&scope)
        .bind(&now)
        .bind(id)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

// ─── Capability operations ────────────────────────────────────────────────────

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
            "INSERT INTO capability \
             (id, target_id, title, status, evidence, milestone_id, created_at, updated_at) \
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
        phase: None,
        acceptance_criteria: vec![],
        preset_hint: None,
        file_scope: vec![],
        assigned_to: None,
        priority: 0,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_capability(ship_dir: &Path, id: &str) -> Result<Option<Capability>> {
    let mut conn = open_db(ship_dir)?;
    let row = block_on(async {
        sqlx::query(&format!("SELECT {C_COLS} FROM capability WHERE id = ?"))
            .bind(id)
            .fetch_optional(&mut conn)
            .await
    })?;
    Ok(row.as_ref().map(row_to_capability))
}

pub fn update_capability(ship_dir: &Path, id: &str, patch: CapabilityPatch) -> Result<()> {
    let current = get_capability(ship_dir, id)?
        .ok_or_else(|| anyhow::anyhow!("capability {id} not found"))?;
    let now = Utc::now().to_rfc3339();
    let ac = serde_json::to_string(&patch.acceptance_criteria.unwrap_or(current.acceptance_criteria))?;
    let scope = serde_json::to_string(&patch.file_scope.unwrap_or(current.file_scope))?;
    let mut conn = open_db(ship_dir)?;
    block_on(async {
        sqlx::query(
            "UPDATE capability SET title=?, status=?, phase=?, acceptance_criteria=?, \
             preset_hint=?, file_scope=?, assigned_to=?, priority=?, updated_at=? WHERE id=?",
        )
        .bind(patch.title.unwrap_or(current.title))
        .bind(patch.status.unwrap_or(current.status))
        .bind(patch.phase.or(current.phase))
        .bind(&ac)
        .bind(patch.preset_hint.or(current.preset_hint))
        .bind(&scope)
        .bind(patch.assigned_to.or(current.assigned_to))
        .bind(patch.priority.unwrap_or(current.priority))
        .bind(&now)
        .bind(id)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn delete_capability(ship_dir: &Path, id: &str) -> Result<bool> {
    let mut conn = open_db(ship_dir)?;
    let rows_affected = block_on(async {
        sqlx::query("DELETE FROM capability WHERE id = ?")
            .bind(id)
            .execute(&mut conn)
            .await
    })?
    .rows_affected();
    Ok(rows_affected > 0)
}

pub fn mark_capability_actual(ship_dir: &Path, id: &str, evidence: &str) -> Result<()> {
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
    phase: Option<&str>,
) -> Result<Vec<Capability>> {
    let mut conn = open_db(ship_dir)?;
    let mut qb: QueryBuilder<sqlx::Sqlite> =
        QueryBuilder::new(format!("SELECT {C_COLS} FROM capability"));
    let mut sep = " WHERE ";
    if let Some(t) = target_id {
        qb.push(sep).push("target_id = ").push_bind(t);
        sep = " AND ";
    }
    if let Some(s) = status {
        qb.push(sep).push("status = ").push_bind(s);
        sep = " AND ";
    }
    if let Some(p) = phase {
        qb.push(sep).push("phase = ").push_bind(p);
    }
    qb.push(" ORDER BY priority ASC, phase ASC, status ASC, created_at ASC");
    let rows = block_on(async { qb.build().fetch_all(&mut conn).await })?;
    Ok(rows.iter().map(row_to_capability).collect())
}

/// List capabilities linked to a milestone. Used by get_target on milestone targets.
pub fn list_capabilities_for_milestone(
    ship_dir: &Path,
    milestone_id: &str,
    status: Option<&str>,
) -> Result<Vec<Capability>> {
    let mut conn = open_db(ship_dir)?;
    let rows = block_on(async {
        if let Some(s) = status {
            sqlx::query(&format!(
                "SELECT {C_COLS} FROM capability \
                 WHERE milestone_id = ? AND status = ? \
                 ORDER BY priority ASC, target_id ASC, created_at ASC"
            ))
            .bind(milestone_id).bind(s).fetch_all(&mut conn).await
        } else {
            sqlx::query(&format!(
                "SELECT {C_COLS} FROM capability \
                 WHERE milestone_id = ? \
                 ORDER BY priority ASC, status ASC, target_id ASC, created_at ASC"
            ))
            .bind(milestone_id).fetch_all(&mut conn).await
        }
    })?;
    Ok(rows.iter().map(row_to_capability).collect())
}

#[cfg(test)]
mod tests;
