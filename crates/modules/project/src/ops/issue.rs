use super::{OpsError, OpsResult, append_project_log};
use crate::{Issue, IssueEntry, IssuePriority, IssueStatus};
use std::path::Path;

pub fn create_issue(
    ship_dir: &Path,
    title: &str,
    description: &str,
    status: IssueStatus,
    assignee: Option<String>,
    priority: Option<IssuePriority>,
    feature_id: Option<String>,
    spec_id: Option<String>,
) -> OpsResult<IssueEntry> {
    if title.trim().is_empty() {
        return Err(OpsError::Validation(
            "Issue title cannot be empty".to_string(),
        ));
    }

    let entry = crate::issue::create_issue(
        ship_dir,
        title,
        description,
        status,
        assignee,
        priority,
        feature_id,
        spec_id,
    )
    .map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "issue create",
        &format!("Created issue: {}", entry.issue.metadata.title),
    )?;
    Ok(entry)
}

pub fn list_issues(ship_dir: &Path) -> OpsResult<Vec<IssueEntry>> {
    crate::issue::list_issues(ship_dir).map_err(OpsError::from)
}

pub fn get_issue_by_id(ship_dir: &Path, id: &str) -> OpsResult<IssueEntry> {
    crate::issue::get_issue_by_id(ship_dir, id).map_err(OpsError::from)
}

pub fn update_issue(ship_dir: &Path, id: &str, issue: Issue) -> OpsResult<IssueEntry> {
    if issue.metadata.title.trim().is_empty() {
        return Err(OpsError::Validation(
            "Issue title cannot be empty".to_string(),
        ));
    }

    let entry = crate::issue::update_issue(ship_dir, id, issue).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "issue update",
        &format!("Updated issue: {}", entry.issue.metadata.title),
    )?;
    Ok(entry)
}

pub fn move_issue_with_from(
    ship_dir: &Path,
    id: &str,
    from_status: IssueStatus,
    to_status: IssueStatus,
) -> OpsResult<IssueEntry> {
    let existing = crate::issue::get_issue_by_id(ship_dir, id).map_err(OpsError::from)?;
    if existing.status != from_status {
        return Err(OpsError::InvalidTransition(
            existing.status.to_string(),
            to_status.to_string(),
        ));
    }
    move_issue(ship_dir, id, to_status)
}

pub fn move_issue(ship_dir: &Path, id: &str, new_status: IssueStatus) -> OpsResult<IssueEntry> {
    let entry = crate::issue::move_issue(ship_dir, id, new_status).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "issue move",
        &format!("Moved {} to {}", entry.file_name, entry.status),
    )?;
    Ok(entry)
}

pub fn delete_issue(ship_dir: &Path, id: &str) -> OpsResult<()> {
    let existing = crate::issue::get_issue_by_id(ship_dir, id).map_err(OpsError::from)?;
    crate::issue::delete_issue(ship_dir, id).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "issue delete",
        &format!("Deleted issue: {}", existing.issue.metadata.title),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_project;
    use runtime::read_log_entries;
    use tempfile::tempdir;

    #[test]
    fn move_issue_with_from_rejects_state_mismatch() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let issue = create_issue(
            &project_dir,
            "Mismatch",
            "state",
            IssueStatus::Backlog,
            None,
            None,
            None,
            None,
        )?;

        let err = move_issue_with_from(
            &project_dir,
            &issue.id,
            IssueStatus::InProgress,
            IssueStatus::Done,
        )
        .expect_err("expected transition validation failure");
        assert!(matches!(err, OpsError::InvalidTransition(_, _)));
        let unchanged = get_issue_by_id(&project_dir, &issue.id)?;
        assert_eq!(unchanged.status, IssueStatus::Backlog);
        Ok(())
    }

    #[test]
    fn move_issue_with_from_happy_path_writes_project_log() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let issue = create_issue(
            &project_dir,
            "happy-path",
            "state",
            IssueStatus::Backlog,
            None,
            None,
            None,
            None,
        )?;

        let moved = move_issue_with_from(
            &project_dir,
            &issue.id,
            IssueStatus::Backlog,
            IssueStatus::InProgress,
        )?;
        assert_eq!(moved.status, IssueStatus::InProgress);

        let logs = read_log_entries(&project_dir)?;
        assert!(logs.iter().any(|entry| {
            entry.action == "issue move" && entry.details.contains(&moved.file_name)
        }));
        Ok(())
    }
}
