#[cfg(test)]
mod tests {
    use crate::db::branch_context::get_branch_link;
    use crate::projections::{Projection, SessionProjection};
    use crate::workspace::*;
    use anyhow::Result;
    use tempfile::tempdir;

    #[test]
    fn activating_workspace_keeps_other_workspace_status_when_both_are_feature_workspaces()
    -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let first = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/alpha".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        assert_eq!(first.status, WorkspaceStatus::Active);

        let second = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/beta".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        assert_eq!(second.status, WorkspaceStatus::Active);
        let _ = activate_workspace(&ship_dir, "feature/beta")?;

        let first_after = get_workspace(&ship_dir, "feature/alpha")?
            .ok_or_else(|| anyhow::anyhow!("feature/alpha workspace missing"))?;
        let second_after = get_workspace(&ship_dir, "feature/beta")?
            .ok_or_else(|| anyhow::anyhow!("feature/beta workspace missing"))?;
        assert_eq!(first_after.status, WorkspaceStatus::Active);
        assert_eq!(second_after.status, WorkspaceStatus::Active);
        assert!(second_after.last_activated_at.is_some());
        Ok(())
    }

    #[test]
    fn activate_workspace_main_branch_stays_unlinked() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let workspace = activate_workspace(&ship_dir, "main")?;
        assert_eq!(workspace.status, WorkspaceStatus::Active);
        assert!(get_branch_link("main")?.is_none());
        Ok(())
    }

    #[test]
    fn delete_workspace_removes_workspace_links_and_sessions() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/delete-me".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        let started = crate::events::types::SessionStarted {
            goal: None,
            workspace_id: workspace.id.clone(),
            workspace_branch: workspace.branch.clone(),
            ..Default::default()
        };
        let start_envelope = crate::db::session_events::insert_session_with_started_event(
            "session-delete-me",
            &workspace.id,
            &started,
        )?;
        if let Ok(mut conn) = crate::db::open_db() {
            let _ = SessionProjection::new().apply(&start_envelope, &mut conn);
        }
        let ended = crate::events::types::SessionEnded {
            summary: Some("done".to_string()),
            duration_secs: Some(0),
            gate_result: None,
            updated_workspace_ids: Vec::new(),
            compile_error: None,
        };
        let end_envelope = crate::db::session_events::update_session_with_ended_event(
            "session-delete-me",
            &workspace.id,
            &ended,
        )?;
        if let Ok(mut conn) = crate::db::open_db() {
            let _ = SessionProjection::new().apply(&end_envelope, &mut conn);
        }
        assert_eq!(list_workspace_sessions(&ship_dir, None, 10)?.len(), 1);

        delete_workspace(&ship_dir, "feature/delete-me")?;

        assert!(get_workspace(&ship_dir, "feature/delete-me")?.is_none());
        assert!(get_branch_link("feature/delete-me")?.is_none());
        assert!(list_workspace_sessions(&ship_dir, None, 10)?.is_empty());
        Ok(())
    }

    #[test]
    fn create_workspace_clears_worktree_metadata_when_switched_to_non_worktree() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let branch = "feature/worktree-cleanup";
        let initial = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                is_worktree: Some(true),
                worktree_path: Some("../worktrees/worktree-cleanup".to_string()),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        assert!(initial.is_worktree);
        assert_eq!(
            initial.worktree_path.as_deref(),
            Some("../worktrees/worktree-cleanup")
        );

        let updated = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                is_worktree: Some(false),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        assert!(!updated.is_worktree);
        assert!(updated.worktree_path.is_none());

        let stored = get_workspace(&ship_dir, branch)?
            .ok_or_else(|| anyhow::anyhow!("workspace missing after update"))?;
        assert!(!stored.is_worktree);
        assert!(stored.worktree_path.is_none());
        Ok(())
    }

    #[test]
    fn create_workspace_auto_populates_worktree_path_when_missing() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/missing-path".to_string(),
                is_worktree: Some(true),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        assert!(workspace.is_worktree);
        let worktree_path = workspace
            .worktree_path
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("expected auto-generated worktree path"))?;
        assert!(worktree_path.contains("feature-missing-path"));
        Ok(())
    }

    #[test]
    fn activate_worktree_workspace_compiles_agent_config_into_worktree_root() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
        config.providers = vec!["claude".to_string()];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        let worktree_root = tmp
            .path()
            .join(".worktrees")
            .join("feature-worktree-export");
        let worktree_path = worktree_root.to_string_lossy().to_string();
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/worktree-export".to_string(),
                status: Some(WorkspaceStatus::Active),
                is_worktree: Some(true),
                worktree_path: Some(worktree_path),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        let _workspace = activate_workspace(&ship_dir, "feature/worktree-export")?;
        assert!(
            worktree_root.join(".mcp.json").exists(),
            "expected provider config to be written to worktree root"
        );
        assert!(
            !tmp.path().join(".mcp.json").exists(),
            "main checkout root should not receive worktree provider config"
        );
        Ok(())
    }

    #[test]
    fn create_workspace_rejects_unknown_active_agent() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let err = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/no-mode".to_string(),
                active_agent: Some("ghost".to_string()),
                ..Default::default()
            },
        )
        .expect_err("expected invalid agent to be rejected");

        assert!(err.to_string().contains("Agent 'ghost' not found"));
        Ok(())
    }

    #[test]
    fn activate_workspace_succeeds() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/activation-compile".to_string(),
                ..Default::default()
            },
        )?;

        let activated = activate_workspace(&ship_dir, "feature/activation-compile")?;
        assert_eq!(activated.status, WorkspaceStatus::Active);
        Ok(())
    }

    #[test]
    fn set_workspace_tmux_session_write_and_read_back() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feat/tmux-test".to_string(),
                ..Default::default()
            },
        )?;

        // Set a session name.
        let _updated = set_workspace_tmux_session(
            &ship_dir,
            "feat/tmux-test",
            Some("my-tmux-session"),
        )?;

        // Clear the session name.
        let _cleared = set_workspace_tmux_session(&ship_dir, "feat/tmux-test", None)?;

        Ok(())
    }

    #[test]
    fn set_workspace_tmux_session_errors_for_missing_workspace() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let err = set_workspace_tmux_session(&ship_dir, "nonexistent", Some("session"))
            .expect_err("expected error for nonexistent workspace");
        assert!(
            err.to_string().contains("Workspace not found"),
            "unexpected error: {err}"
        );
        Ok(())
    }
}
