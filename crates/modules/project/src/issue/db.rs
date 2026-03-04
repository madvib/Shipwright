use super::types::{Issue, IssueEntry, IssueMetadata, IssuePriority, IssueStatus};
use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use std::path::Path;
use std::str::FromStr;

fn virtual_issue_file_name(id: &str) -> String {
    format!("{}.md", id)
}

fn virtual_issue_path(ship_dir: &Path, status: &IssueStatus, id: &str) -> String {
    runtime::project::issues_dir(ship_dir)
        .join(status.to_string())
        .join(virtual_issue_file_name(id))
        .to_string_lossy()
        .to_string()
}

pub fn upsert_issue_db(ship_dir: &Path, issue: &Issue, status: &IssueStatus) -> Result<()> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    let now = Utc::now().to_rfc3339();

    runtime::state_db::block_on(async {
        sqlx::query(
            "INSERT INTO issue
               (id, title, description, status, assignee, priority, release_id, feature_id, spec_id, tags_json, links_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               title       = excluded.title,
               description = excluded.description,
               status      = excluded.status,
               assignee    = excluded.assignee,
               priority    = excluded.priority,
               release_id  = excluded.release_id,
               feature_id  = excluded.feature_id,
               spec_id     = excluded.spec_id,
               tags_json   = excluded.tags_json,
               links_json  = excluded.links_json,
               updated_at  = excluded.updated_at",
        )
        .bind(&issue.metadata.id)
        .bind(&issue.metadata.title)
        .bind(&issue.description)
        .bind(status.to_string())
        .bind(&issue.metadata.assignee)
        .bind(issue.metadata.priority.as_ref().map(|p| p.to_string()))
        .bind(&issue.metadata.release_id)
        .bind(&issue.metadata.feature_id)
        .bind(&issue.metadata.spec_id)
        .bind(serde_json::to_string(&issue.metadata.tags).unwrap_or_else(|_| "[]".to_string()))
        .bind(serde_json::to_string(&issue.metadata.links).unwrap_or_else(|_| "[]".to_string()))
        .bind(&issue.metadata.created)
        .bind(&now)
        .execute(&mut conn)
        .await?;
        Ok(())
    })?;
    Ok(())
}

pub fn get_issue_db(ship_dir: &Path, id: &str) -> Result<Option<IssueEntry>> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        let row_opt = sqlx::query(
            "SELECT id, title, description, status, assignee, priority, release_id, feature_id, spec_id, tags_json, links_json, created_at, updated_at
             FROM issue WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut conn)
        .await?;

        if let Some(r) = row_opt {
            let id: String = r.get(0);
            let title: String = r.get(1);
            let description: String = r.get(2);
            let status_str: String = r.get(3);
            let assignee: Option<String> = r.get(4);
            let priority_str: Option<String> = r.get(5);
            let release_id: Option<String> = r.get(6);
            let feature_id: Option<String> = r.get(7);
            let spec_id: Option<String> = r.get(8);
            let tags_json: String = r.get(9);
            let links_json: String = r.get(10);
            let created: String = r.get(11);
            let updated: String = r.get(12);

            let status = IssueStatus::from_str(&status_str).ok().unwrap_or_default();
            let priority = priority_str.and_then(|s| match s.as_str() {
                "critical" => Some(IssuePriority::Critical),
                "high" => Some(IssuePriority::High),
                "medium" => Some(IssuePriority::Medium),
                "low" => Some(IssuePriority::Low),
                _ => None,
            });
            let tags = serde_json::from_str(&tags_json).unwrap_or_default();
            let links = serde_json::from_str(&links_json).unwrap_or_default();
            let file_name = virtual_issue_file_name(&id);
            let path = virtual_issue_path(ship_dir, &status, &id);

            Ok(Some(IssueEntry {
                id: id.clone(),
                file_name,
                path,
                status,
                issue: Issue {
                    metadata: IssueMetadata {
                        id,
                        title,
                        created,
                        updated,
                        assignee,
                        priority,
                        tags,
                        spec_id,
                        feature_id,
                        release_id,
                        links,
                    },
                    description,
                },
            }))
        } else {
            Ok(None)
        }
    })
}

pub fn list_issues_db(ship_dir: &Path) -> Result<Vec<IssueEntry>> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        let rows = sqlx::query(
            "SELECT id, title, description, status, assignee, priority, release_id, feature_id, spec_id, tags_json, links_json, created_at, updated_at
             FROM issue ORDER BY updated_at DESC",
        )
        .fetch_all(&mut conn)
        .await?;

        let mut entries = Vec::new();
        for r in rows {
            let id: String = r.get(0);
            let title: String = r.get(1);
            let description: String = r.get(2);
            let status_str: String = r.get(3);
            let assignee: Option<String> = r.get(4);
            let priority_str: Option<String> = r.get(5);
            let release_id: Option<String> = r.get(6);
            let feature_id: Option<String> = r.get(7);
            let spec_id: Option<String> = r.get(8);
            let tags_json: String = r.get(9);
            let links_json: String = r.get(10);
            let created: String = r.get(11);
            let updated: String = r.get(12);

            let status = IssueStatus::from_str(&status_str).ok().unwrap_or_default();
            let priority = priority_str.and_then(|s| match s.as_str() {
                "critical" => Some(IssuePriority::Critical),
                "high" => Some(IssuePriority::High),
                "medium" => Some(IssuePriority::Medium),
                "low" => Some(IssuePriority::Low),
                _ => None,
            });
            let tags = serde_json::from_str(&tags_json).unwrap_or_default();
            let links = serde_json::from_str(&links_json).unwrap_or_default();
            let file_name = virtual_issue_file_name(&id);
            let path = virtual_issue_path(ship_dir, &status, &id);

            entries.push(IssueEntry {
                id: id.clone(),
                file_name,
                path,
                status,
                issue: Issue {
                    metadata: IssueMetadata {
                        id,
                        title,
                        created,
                        updated,
                        assignee,
                        priority,
                        tags,
                        spec_id,
                        feature_id,
                        release_id,
                        links,
                    },
                    description,
                },
            });
        }
        Ok(entries)
    })
}

pub fn delete_issue_db(ship_dir: &Path, id: &str) -> Result<()> {
    let mut conn = runtime::state_db::open_project_connection(ship_dir)?;
    runtime::state_db::block_on(async {
        sqlx::query("DELETE FROM issue WHERE id = ?")
            .bind(id)
            .execute(&mut conn)
            .await?;
        Ok(())
    })?;
    Ok(())
}
