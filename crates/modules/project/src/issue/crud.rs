use super::db::{delete_issue_db, get_issue_db, list_issues_db, upsert_issue_db};
use super::types::{Issue, IssueEntry, IssueMetadata, IssueStatus};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::path::{Path, PathBuf};

// ── File helpers ─────────────────────────────────────────────────────────────

fn issue_file_path(ship_dir: &Path, status: &IssueStatus, title: &str) -> PathBuf {
    let base = runtime::project::sanitize_file_name(title);
    let dir = runtime::project::issues_dir(ship_dir).join(status.to_string());
    std::fs::create_dir_all(&dir).ok();
    let candidate = dir.join(format!("{}.md", base));
    if !candidate.exists() {
        return candidate;
    }
    let mut n = 2u32;
    loop {
        let candidate = dir.join(format!("{}-{}.md", base, n));
        if !candidate.exists() {
            return candidate;
        }
        n += 1;
    }
}

fn resolve_issue_id(ship_dir: &Path, reference: &str) -> Result<Option<String>> {
    let reference = reference.trim();
    if reference.is_empty() {
        return Ok(None);
    }

    if let Some(entry) = get_issue_db(ship_dir, reference)? {
        return Ok(Some(entry.id));
    }

    let without_ext = reference.trim_end_matches(".md");
    if without_ext != reference {
        if let Some(entry) = get_issue_db(ship_dir, without_ext)? {
            return Ok(Some(entry.id));
        }
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

pub fn write_issue_file(ship_dir: &Path, issue: &Issue, status: &IssueStatus) -> Result<PathBuf> {
    let path = issue_file_path(ship_dir, status, &issue.metadata.title);
    let content = issue.to_markdown()?;
    runtime::fs_util::write_atomic(&path, content)?;
    Ok(path)
}

pub fn remove_issue_files(ship_dir: &Path, id: &str, title: &str) {
    let base = runtime::project::sanitize_file_name(title);
    let issues_dir = runtime::project::issues_dir(ship_dir);

    // Check root and subdirs
    let mut scan_dirs = vec![issues_dir.clone()];
    for status in &["backlog", "in-progress", "blocked", "done"] {
        scan_dirs.push(issues_dir.join(status));
    }

    for dir in scan_dirs {
        if !dir.exists() {
            continue;
        }
        for suffix in &["", "-2", "-3", "-4", "-5"] {
            let file_name = if suffix.is_empty() {
                format!("{}.md", base)
            } else {
                format!("{}{}.md", base, suffix)
            };
            let p = dir.join(file_name);
            if p.exists() {
                if let Ok(content) = std::fs::read_to_string(&p) {
                    if content.contains(id) {
                        std::fs::remove_file(&p).ok();
                        return;
                    }
                }
            }
        }
    }
}

// ── Public CRUD ──────────────────────────────────────────────────────────────

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
    let file_path = write_issue_file(ship_dir, &issue, &status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Issue,
        runtime::EventAction::Create,
        id.clone(),
        Some(format!("title={}", title)),
    )?;

    Ok(IssueEntry {
        id,
        file_name: file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
        path: file_path.to_string_lossy().to_string(),
        status,
        issue,
    })
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
    write_issue_file(ship_dir, &issue, &existing.status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Issue,
        runtime::EventAction::Update,
        resolved_id.clone(),
        Some(format!("title={}", issue.metadata.title)),
    )?;

    let mut entry = get_issue_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Issue not found after update"))?;
    entry.issue.description = issue.description;
    Ok(entry)
}

pub fn move_issue(ship_dir: &Path, id: &str, new_status: IssueStatus) -> Result<IssueEntry> {
    let resolved_id = require_issue_id(ship_dir, id)?;
    let existing =
        get_issue_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Issue not found: {}", id))?;

    upsert_issue_db(ship_dir, &existing.issue, &new_status)?;
    remove_issue_files(ship_dir, &resolved_id, &existing.issue.metadata.title);
    write_issue_file(ship_dir, &existing.issue, &new_status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Issue,
        runtime::EventAction::Update,
        resolved_id.clone(),
        Some(format!("status={}", new_status)),
    )?;

    Ok(get_issue_db(ship_dir, &resolved_id)?.unwrap())
}

pub fn delete_issue(ship_dir: &Path, id: &str) -> Result<()> {
    let resolved_id = require_issue_id(ship_dir, id)?;
    let entry =
        get_issue_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Issue not found: {}", id))?;
    delete_issue_db(ship_dir, &resolved_id)?;
    remove_issue_files(ship_dir, &resolved_id, &entry.issue.metadata.title);

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
