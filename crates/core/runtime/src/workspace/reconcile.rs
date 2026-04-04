//! Workspace reconciliation — correct worktree state against git reality.
//!
//! Called on session start, workspace activation, and post-checkout hooks.
//! Compares the stored `is_worktree` / `worktree_path` against what
//! `git worktree list --porcelain` reports, and emits a `workspace.reconciled`
//! event when corrections are needed.

use anyhow::Result;
use std::path::Path;

use super::crud::get_workspace;
use super::event_upserts::upsert_workspace_on_reconciled;
use super::helpers::git_worktree_path_for_branch;
use super::types::Workspace;

/// Reconcile a workspace record against actual git worktree state.
///
/// Returns `None` if no workspace exists for the branch.
/// Returns `Some(workspace)` with corrected fields if a workspace exists.
/// Emits `workspace.reconciled` only when the stored state differs from git.
pub fn reconcile_workspace(
    ship_dir: &Path,
    branch: &str,
    connection_path: &Path,
) -> Result<Option<Workspace>> {
    let Some(mut workspace) = get_workspace(ship_dir, branch)? else {
        return Ok(None);
    };

    let git_path = git_worktree_path_for_branch(branch);
    let changes = detect_changes(&workspace, &git_path, connection_path);

    if let Some(change) = changes {
        workspace.is_worktree = change.is_worktree;
        workspace.worktree_path = change.worktree_path.clone();

        upsert_workspace_on_reconciled(
            ship_dir,
            branch,
            change.is_worktree,
            change.worktree_path.as_deref(),
            &change.reason,
        )?;
    }

    Ok(Some(workspace))
}

struct WorktreeChange {
    is_worktree: bool,
    worktree_path: Option<String>,
    reason: String,
}

fn detect_changes(
    workspace: &Workspace,
    git_path: &Option<String>,
    connection_path: &Path,
) -> Option<WorktreeChange> {
    match git_path {
        Some(actual_path) => {
            // Branch IS in a worktree according to git.
            let stored_path = workspace.worktree_path.as_deref();
            if !workspace.is_worktree {
                // DB says not a worktree, but git says it is.
                Some(WorktreeChange {
                    is_worktree: true,
                    worktree_path: Some(actual_path.clone()),
                    reason: "branch is in a worktree but workspace had is_worktree=false"
                        .to_string(),
                })
            } else if stored_path != Some(actual_path.as_str()) {
                // DB has wrong path.
                Some(WorktreeChange {
                    is_worktree: true,
                    worktree_path: Some(actual_path.clone()),
                    reason: format!(
                        "worktree path changed: {:?} -> {}",
                        stored_path, actual_path
                    ),
                })
            } else {
                None // Everything matches.
            }
        }
        None => {
            // Branch is NOT in any worktree according to git.
            if workspace.is_worktree {
                // Stale path detection: stored path doesn't exist on disk, or
                // branch was checked out in main repo.
                let stale = workspace
                    .worktree_path
                    .as_ref()
                    .map(|p| !Path::new(p).exists())
                    .unwrap_or(true);
                let reason = if stale {
                    "stored worktree path no longer exists on disk".to_string()
                } else {
                    format!(
                        "branch checked out in main repo at {}",
                        connection_path.display()
                    )
                };
                Some(WorktreeChange {
                    is_worktree: false,
                    worktree_path: None,
                    reason,
                })
            } else {
                None // Already correct.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::types::{Workspace, WorkspaceStatus};

    fn test_workspace(is_worktree: bool, worktree_path: Option<&str>) -> Workspace {
        Workspace {
            id: "test".to_string(),
            branch: "feat/test".to_string(),
            status: WorkspaceStatus::Active,
            is_worktree,
            worktree_path: worktree_path.map(str::to_string),
            active_agent: None,
            last_activated_at: None,
        }
    }

    #[test]
    fn no_change_when_not_worktree_and_git_agrees() {
        let ws = test_workspace(false, None);
        let conn = std::path::Path::new("/repo");
        assert!(detect_changes(&ws, &None, conn).is_none());
    }

    #[test]
    fn no_change_when_worktree_path_matches() {
        let ws = test_workspace(true, Some("/worktrees/feat-test"));
        let conn = std::path::Path::new("/worktrees/feat-test");
        let git_path = Some("/worktrees/feat-test".to_string());
        assert!(detect_changes(&ws, &git_path, conn).is_none());
    }

    #[test]
    fn detects_worktree_to_main_repo_transition() {
        let ws = test_workspace(true, Some("/nonexistent/path"));
        let conn = std::path::Path::new("/repo");
        let change = detect_changes(&ws, &None, conn).expect("should detect change");
        assert!(!change.is_worktree);
        assert!(change.worktree_path.is_none());
        assert!(change.reason.contains("no longer exists"));
    }

    #[test]
    fn detects_main_repo_to_worktree_transition() {
        let ws = test_workspace(false, None);
        let conn = std::path::Path::new("/repo");
        let git_path = Some("/worktrees/feat-test".to_string());
        let change = detect_changes(&ws, &git_path, conn).expect("should detect change");
        assert!(change.is_worktree);
        assert_eq!(change.worktree_path.as_deref(), Some("/worktrees/feat-test"));
        assert!(change.reason.contains("is_worktree=false"));
    }

    #[test]
    fn detects_worktree_path_change() {
        let ws = test_workspace(true, Some("/old/path"));
        let conn = std::path::Path::new("/new/path");
        let git_path = Some("/new/path".to_string());
        let change = detect_changes(&ws, &git_path, conn).expect("should detect change");
        assert!(change.is_worktree);
        assert_eq!(change.worktree_path.as_deref(), Some("/new/path"));
        assert!(change.reason.contains("path changed"));
    }

    #[test]
    fn detects_stale_worktree_with_no_path() {
        let ws = test_workspace(true, None);
        let conn = std::path::Path::new("/repo");
        let change = detect_changes(&ws, &None, conn).expect("should detect change");
        assert!(!change.is_worktree);
        assert!(change.worktree_path.is_none());
    }
}
