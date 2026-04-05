#[cfg(test)]
mod tests {
    use crate::workspace::*;
    use crate::workspace::reconcile::reconcile_workspace_with_git_path;
    use anyhow::Result;
    use tempfile::tempdir;

    fn setup_workspace(branch: &str, is_worktree: bool, worktree_path: Option<&str>) -> Result<(tempfile::TempDir, std::path::PathBuf)> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                is_worktree: Some(is_worktree),
                worktree_path: worktree_path.map(String::from),
                status: Some(WorkspaceStatus::Active),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        Ok((tmp, ship_dir))
    }

    #[test]
    fn noop_when_state_matches_no_worktree() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace("main", false, None)?;
        let ws = reconcile_workspace_with_git_path(&ship_dir, "main", &None)?
            .expect("workspace should exist");
        assert!(!ws.is_worktree);
        assert!(ws.worktree_path.is_none());
        Ok(())
    }

    #[test]
    fn noop_when_state_matches_worktree() -> Result<()> {
        let stored_path = "/tmp/ship-test-wt-match".to_string();
        let (_tmp, ship_dir) = setup_workspace(
            "feature/x",
            true,
            Some(&stored_path),
        )?;
        let ws = reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/x",
            &Some(stored_path.clone()),
        )?
        .expect("workspace should exist");
        assert!(ws.is_worktree);
        assert_eq!(ws.worktree_path.as_deref(), Some(stored_path.as_str()));
        Ok(())
    }

    #[test]
    fn worktree_to_main_repo_transition() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace(
            "feature/a",
            true,
            Some("/tmp/ship-test-wt-a"),
        )?;
        // Git says branch is NOT in any worktree anymore.
        let ws = reconcile_workspace_with_git_path(&ship_dir, "feature/a", &None)?
            .expect("workspace should exist");
        assert!(!ws.is_worktree);
        assert!(ws.worktree_path.is_none());
        Ok(())
    }

    #[test]
    fn main_repo_to_worktree_transition() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace("feature/b", false, None)?;
        // Git now says the branch IS in a worktree.
        let new_path = "/tmp/ship-test-worktree-b".to_string();
        let ws = reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/b",
            &Some(new_path.clone()),
        )?
        .expect("workspace should exist");
        assert!(ws.is_worktree);
        assert_eq!(ws.worktree_path.as_deref(), Some(new_path.as_str()));
        Ok(())
    }

    #[test]
    fn worktree_path_update() -> Result<()> {
        let (_tmp, ship_dir) = setup_workspace(
            "feature/c",
            true,
            Some("/tmp/ship-test-wt-c-old"),
        )?;
        let new_path = "/tmp/ship-test-moved-worktree".to_string();
        let ws = reconcile_workspace_with_git_path(
            &ship_dir,
            "feature/c",
            &Some(new_path.clone()),
        )?
        .expect("workspace should exist");
        assert!(ws.is_worktree);
        assert_eq!(ws.worktree_path.as_deref(), Some(new_path.as_str()));
        Ok(())
    }

    #[test]
    fn stale_path_cleared() -> Result<()> {
        // Workspace claims a worktree at a path that doesn't exist on disk,
        // and git confirms the branch is not in any worktree.
        let (_tmp, ship_dir) = setup_workspace(
            "feature/stale",
            true,
            Some("/nonexistent/stale/path"),
        )?;
        let ws = reconcile_workspace_with_git_path(&ship_dir, "feature/stale", &None)?
            .expect("workspace should exist");
        assert!(!ws.is_worktree);
        assert!(ws.worktree_path.is_none());
        Ok(())
    }

    #[test]
    fn returns_none_for_unknown_branch() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;
        let result = reconcile_workspace_with_git_path(
            &ship_dir,
            "nonexistent-branch",
            &None,
        )?;
        assert!(result.is_none());
        Ok(())
    }

}
