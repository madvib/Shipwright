use super::db::upsert_issue_db;
use super::types::{Issue, IssueStatus};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub fn import_issues_from_files(ship_dir: &Path) -> Result<usize> {
    let issues_dir = runtime::project::issues_dir(ship_dir);
    if !issues_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    let mut scan_dirs = vec![issues_dir.clone()];

    // Also scan status subdirectories
    for status in &["backlog", "in-progress", "blocked", "done"] {
        let status_dir = issues_dir.join(status);
        if status_dir.exists() {
            scan_dirs.push(status_dir);
        }
    }

    for dir in scan_dirs {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "md") {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name == "TEMPLATE.md" || file_name == "README.md" {
                    continue;
                }

                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read issue file: {}", path.display()))?;

                if let Ok(issue) = Issue::from_markdown(&content) {
                    // Determine status from directory name
                    let status = if path.parent() == Some(&issues_dir) {
                        IssueStatus::Backlog
                    } else {
                        path.parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .and_then(|s| IssueStatus::from_str(s).ok())
                            .unwrap_or(IssueStatus::Backlog)
                    };

                    upsert_issue_db(ship_dir, &issue, &status)?;
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}
