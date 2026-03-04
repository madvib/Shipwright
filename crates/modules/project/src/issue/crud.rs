use super::db::{delete_issue_db, get_issue_db, list_issues_db, upsert_issue_db};
use super::types::{Issue, IssueEntry, IssueMetadata, IssueStatus};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::path::Path;

fn resolve_issue_id(ship_dir: &Path, reference: &str) -> Result<Option<String>> {
    let reference = reference.trim();
    if reference.is_empty() {
        return Ok(None);
    }

    if let Some(entry) = get_issue_db(ship_dir, reference)? {
        return Ok(Some(entry.id));
    }

    let without_ext = reference.trim_end_matches(".md");
    if without_ext != reference
        && let Some(entry) = get_issue_db(ship_dir, without_ext)?
    {
        return Ok(Some(entry.id));
    }

    let reference_file = if reference.ends_with(".md") {
        reference.to_string()
    } else {
        format!("{}.md", reference)
    };
    let reference_slug = runtime::project::sanitize_file_name(without_ext);

    for entry in list_issues_db(ship_dir)? {
        let file_match = entry.file_name.eq_ignore_ascii_case(reference)
            || entry.file_name.eq_ignore_ascii_case(&reference_file);
        let slug_match = runtime::project::sanitize_file_name(&entry.issue.metadata.title)
            .eq_ignore_ascii_case(&reference_slug);
        if file_match || slug_match {
            return Ok(Some(entry.id));
        }
    }

    Ok(None)
}

fn require_issue_id(ship_dir: &Path, reference: &str) -> Result<String> {
    resolve_issue_id(ship_dir, reference)?.ok_or_else(|| anyhow!("Issue not found: {}", reference))
}

pub fn create_issue(
    ship_dir: &Path,
    title: &str,
    description: &str,
    status: IssueStatus,
    assignee: Option<String>,
    priority: Option<super::types::IssuePriority>,
    feature_id: Option<String>,
    spec_id: Option<String>,
) -> Result<IssueEntry> {
    if title.trim().is_empty() {
        return Err(anyhow!("Issue title cannot be empty"));
    }
    let id = runtime::gen_nanoid();
    let now = Utc::now().to_rfc3339();

    let issue = Issue {
        metadata: IssueMetadata {
            id: id.clone(),
            title: title.to_string(),
            created: now.clone(),
            updated: now,
            assignee,
            priority,
            tags: vec![],
            spec_id,
            feature_id,
            release_id: None,
            links: vec![],
        },
        description: description.to_string(),
    };

    upsert_issue_db(ship_dir, &issue, &status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Issue,
        runtime::EventAction::Create,
        id.clone(),
        Some(format!("title={}", title)),
    )?;

    get_issue_db(ship_dir, &id)?.ok_or_else(|| anyhow!("Issue not found after create: {}", id))
}

pub fn get_issue_by_id(ship_dir: &Path, id: &str) -> Result<IssueEntry> {
    let resolved_id = require_issue_id(ship_dir, id)?;
    get_issue_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Issue not found: {}", id))
}

pub fn update_issue(ship_dir: &Path, id: &str, mut issue: Issue) -> Result<IssueEntry> {
    let resolved_id = require_issue_id(ship_dir, id)?;
    let existing =
        get_issue_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Issue not found: {}", id))?;
    issue.metadata.updated = Utc::now().to_rfc3339();

    upsert_issue_db(ship_dir, &issue, &existing.status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Issue,
        runtime::EventAction::Update,
        resolved_id.clone(),
        Some(format!("title={}", issue.metadata.title)),
    )?;

    get_issue_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Issue not found after update: {}", resolved_id))
}

pub fn move_issue(ship_dir: &Path, id: &str, new_status: IssueStatus) -> Result<IssueEntry> {
    let resolved_id = require_issue_id(ship_dir, id)?;
    let existing =
        get_issue_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Issue not found: {}", id))?;

    upsert_issue_db(ship_dir, &existing.issue, &new_status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Issue,
        runtime::EventAction::Update,
        resolved_id.clone(),
        Some(format!("status={}", new_status)),
    )?;

    get_issue_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Issue not found after move: {}", resolved_id))
}

pub fn delete_issue(ship_dir: &Path, id: &str) -> Result<()> {
    let resolved_id = require_issue_id(ship_dir, id)?;
    get_issue_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Issue not found: {}", id))?;
    delete_issue_db(ship_dir, &resolved_id)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Issue,
        runtime::EventAction::Delete,
        resolved_id,
        None,
    )?;
    Ok(())
}

pub fn list_issues(ship_dir: &Path) -> Result<Vec<IssueEntry>> {
    list_issues_db(ship_dir)
}
