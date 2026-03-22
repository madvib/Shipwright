#[cfg(test)]
mod tests {
    use anyhow::Result;
    use crate::db::branch_context::get_branch_link;
    use crate::workspace::*;
    use tempfile::tempdir;

    #[test]
    fn create_workspace_hydrates_feature_link_from_branch_context() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;
        crate::db::branch_context::set_branch_link(
            &ship_dir,
            "feature/auth-redesign",
            "feature",
            "feat-auth",
        )?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/auth-redesign".to_string(),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        assert_eq!(workspace.feature_id.as_deref(), Some("feat-auth"));
        Ok(())
    }

    #[test]
    fn create_workspace_mixed_branch_links_preserve_target_context() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        crate::db::branch_context::set_branch_link(
            &ship_dir,
            "feature/mixed",
            "feature",
            "feat-mixed",
        )?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/mixed".to_string(),
                target_id: Some("target-direct".to_string()),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        assert_eq!(workspace.feature_id.as_deref(), Some("feat-mixed"));
        assert_eq!(workspace.target_id.as_deref(), Some("target-direct"));
        let stored_link = get_branch_link(&ship_dir, "feature/mixed")?;
        assert_eq!(
            stored_link,
            Some(("feature".to_string(), "feat-mixed".to_string()))
        );
        Ok(())
    }

    #[test]
    fn workspace_never_persists_target_as_branch_owner() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let workspace = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "service/target-context".to_string(),
                workspace_type: Some(ShipWorkspaceKind::Feature),
                target_id: Some("target-only".to_string()),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        assert_eq!(workspace.target_id.as_deref(), Some("target-only"));
        assert!(get_branch_link(&ship_dir, "service/target-context")?.is_none());
        Ok(())
    }

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
                feature_id: Some("feat-alpha".to_string()),
                ..CreateWorkspaceRequest::default()
            },
        )?;
        assert_eq!(first.status, WorkspaceStatus::Active);

        let second = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/beta".to_string(),
                status: Some(WorkspaceStatus::Active),
                feature_id: Some("feat-beta".to_string()),
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
        assert!(workspace.feature_id.is_none());
        assert!(workspace.target_id.is_none());
        assert!(get_branch_link(&ship_dir, "main")?.is_none());
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
                feature_id: Some("feat-delete".to_string()),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        let now = chrono::Utc::now().to_rfc3339();
        crate::db::session::insert_workspace_session_db(
            &ship_dir,
            &crate::db::types::WorkspaceSessionDb {
                id: "session-delete-me".to_string(),
                workspace_id: workspace.id.clone(),
                workspace_branch: workspace.branch.clone(),
                status: WorkspaceSessionStatus::Ended.to_string(),
                started_at: now.clone(),
                ended_at: Some(now.clone()),
                agent_id: None,
                primary_provider: None,
                goal: None,
                summary: Some("done".to_string()),
                updated_workspace_ids: Vec::new(),
                compiled_at: None,
                compile_error: None,
                config_generation_at_start: None,
                created_at: now.clone(),
                updated_at: now,
            },
        )?;
        assert_eq!(list_workspace_sessions(&ship_dir, None, 10)?.len(), 1);

        delete_workspace(&ship_dir, "feature/delete-me")?;

        assert!(get_workspace(&ship_dir, "feature/delete-me")?.is_none());
        assert!(get_branch_link(&ship_dir, "feature/delete-me")?.is_none());
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

        let workspace = activate_workspace(&ship_dir, "feature/worktree-export")?;
        assert!(workspace.compiled_at.is_some());
        assert!(workspace.compile_error.is_none());
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
    fn activate_workspace_compiles_and_bumps_generation() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let created = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/activation-compile".to_string(),
                ..Default::default()
            },
        )?;
        assert_eq!(created.config_generation, 0);

        let activated = activate_workspace(&ship_dir, "feature/activation-compile")?;
        assert_eq!(activated.status, WorkspaceStatus::Active);
        assert!(activated.config_generation >= 1);
        assert!(activated.compiled_at.is_some());
        assert!(activated.compile_error.is_none());
        Ok(())
    }

    #[test]
    fn activate_workspace_always_recompiles_and_bumps_generation() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/hash-short-circuit".to_string(),
                ..Default::default()
            },
        )?;

        let first = activate_workspace(&ship_dir, "feature/hash-short-circuit")?;
        let second = activate_workspace(&ship_dir, "feature/hash-short-circuit")?;

        assert!(second.config_generation > first.config_generation);
        assert!(second.compile_error.is_none());
        Ok(())
    }
}
