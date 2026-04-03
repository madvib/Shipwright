#[cfg(test)]
mod tests {
    use crate::workspace::*;
    use anyhow::Result;
    use tempfile::tempdir;

    #[test]
    fn workspace_session_start_and_end_happy_path() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/session-flow".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let started = start_workspace_session(
            &ship_dir,
            "feature/session-flow",
            Some("Implement parser".to_string()),
            None,
            None,
        )?;
        assert_eq!(started.status, WorkspaceSessionStatus::Active);
        assert_eq!(started.goal.as_deref(), Some("Implement parser"));
        assert_eq!(started.primary_provider.as_deref(), Some("claude"));
        assert!(!started.stale_context);
        assert!(started.ended_at.is_none());

        let active = get_active_workspace_session(&ship_dir, "feature/session-flow")?
            .ok_or_else(|| anyhow::anyhow!("active session not found"))?;
        assert_eq!(active.id, started.id);
        assert!(!active.stale_context);

        let ended = end_workspace_session(
            &ship_dir,
            "feature/session-flow",
            EndWorkspaceSessionRequest {
                summary: Some("Implemented parser + tests".to_string()),
                updated_workspace_ids: vec!["feat-parser".to_string()],
                model: None,
                files_changed: None,
                gate_result: None,
            },
        )?;
        assert_eq!(ended.status, WorkspaceSessionStatus::Ended);
        assert!(ended.ended_at.is_some());
        assert_eq!(ended.summary.as_deref(), Some("Implemented parser + tests"));
        assert_eq!(ended.updated_workspace_ids, vec!["feat-parser".to_string()]);
        assert!(ended.session_record_id.is_some());
        assert!(get_active_workspace_session(&ship_dir, "feature/session-flow")?.is_none());

        Ok(())
    }

    #[test]
    fn session_record_captures_metrics_on_end() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/metrics".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let started = start_workspace_session(
            &ship_dir,
            "feature/metrics",
            Some("test metrics".to_string()),
            None,
            None,
        )?;

        let ended = end_workspace_session(
            &ship_dir,
            "feature/metrics",
            EndWorkspaceSessionRequest {
                summary: Some("done".to_string()),
                updated_workspace_ids: vec![],
                model: Some("claude-opus-4-20250514".to_string()),
                files_changed: Some(5),
                gate_result: Some("pass".to_string()),
            },
        )?;

        let record = get_workspace_session_record(&ship_dir, &ended.id)?
            .expect("session record should exist after end");

        assert_eq!(record.session_id, started.id);
        assert!(record.duration_secs.is_some());
        assert!(record.duration_secs.unwrap() >= 0);
        assert_eq!(record.provider.as_deref(), Some("claude"));
        assert_eq!(record.model.as_deref(), Some("claude-opus-4-20250514"));
        assert_eq!(record.agent_id, started.agent_id);
        assert_eq!(record.files_changed, Some(5));
        assert_eq!(record.gate_result.as_deref(), Some("pass"));
        Ok(())
    }

    #[test]
    fn session_record_metrics_default_to_none() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/metrics-defaults".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        start_workspace_session(&ship_dir, "feature/metrics-defaults", None, None, None)?;

        let ended = end_workspace_session(
            &ship_dir,
            "feature/metrics-defaults",
            EndWorkspaceSessionRequest::default(),
        )?;

        let record = get_workspace_session_record(&ship_dir, &ended.id)?
            .expect("session record should exist");

        assert!(record.duration_secs.is_some());
        assert!(record.duration_secs.unwrap() >= 0);
        assert!(record.provider.is_some());
        assert!(record.model.is_none());
        assert!(record.files_changed.is_none());
        assert!(record.gate_result.is_none());
        Ok(())
    }

    #[test]
    fn workspace_session_start_attaches_existing_active_session() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/session-dupe".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let first = start_workspace_session(
            &ship_dir,
            "feature/session-dupe",
            Some("one".into()),
            None,
            None,
        )?;
        let attached = start_workspace_session(
            &ship_dir,
            "feature/session-dupe",
            Some("two".into()),
            None,
            None,
        )?;

        assert_eq!(attached.id, first.id);
        let sessions = list_workspace_sessions(&ship_dir, Some("feature/session-dupe"), 10)?;
        assert_eq!(sessions.len(), 1);
        Ok(())
    }

    #[test]
    fn workspace_session_list_filters_by_branch_workspace() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/a".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/b".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let a = start_workspace_session(&ship_dir, "feature/a", None, None, None)?;
        end_workspace_session(
            &ship_dir,
            "feature/a",
            EndWorkspaceSessionRequest::default(),
        )?;
        let b = start_workspace_session(&ship_dir, "feature/b", None, None, None)?;

        let all = list_workspace_sessions(&ship_dir, None, 10)?;
        assert!(all.iter().any(|session| session.id == a.id));
        assert!(all.iter().any(|session| session.id == b.id));

        let only_a = list_workspace_sessions(&ship_dir, Some("feature/a"), 10)?;
        assert!(
            only_a
                .iter()
                .all(|session| session.workspace_branch == "feature/a")
        );
        assert_eq!(only_a.len(), 1);
        assert_eq!(only_a[0].id, a.id);
        Ok(())
    }

    #[test]
    fn workspace_session_start_allows_explicit_primary_provider() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-ok".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/provider-ok",
            Some("Pin provider".to_string()),
            None,
            Some("claude".to_string()),
        )?;
        assert_eq!(session.primary_provider.as_deref(), Some("claude"));
        Ok(())
    }

    #[test]
    fn workspace_session_start_rejects_provider_outside_allowed_targets() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-deny".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let err = start_workspace_session(
            &ship_dir,
            "feature/provider-deny",
            None,
            None,
            Some("gemini".to_string()),
        )
        .expect_err("provider outside allowed targets should be rejected");

        assert!(
            err.to_string()
                .contains("Provider 'gemini' is not allowed for workspace")
        );
        Ok(())
    }

    #[test]
    #[ignore = "provider resolution always falls back to claude; need workspace-level provider deny"]
    fn start_workspace_session_errors_when_no_valid_providers_resolve() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        // Ensure no providers are configured so the session start fails.
        let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
        config.providers = vec![];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/no-provider".to_string(),
                ..Default::default()
            },
        )?;

        let err = start_workspace_session(
            &ship_dir,
            "feature/no-provider",
            Some("should fail".to_string()),
            None,
            None,
        )
        .expect_err("session start should fail when no providers resolve");
        assert!(
            err.to_string()
                .contains("No providers resolved for workspace") || err.to_string().contains("No valid providers resolved for workspace"),
            "unexpected error: {}",
            err
        );
        Ok(())
    }

    #[test]
    #[ignore = "config_generation_at_start not populated after lean workspace refactor; needs projection read"]
    fn session_stale_context_turns_true_after_recompile() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let config = crate::config::ProjectConfig {
            providers: vec!["claude".to_string()],
            ..Default::default()
        };
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/stale-session".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let started = start_workspace_session(
            &ship_dir,
            "feature/stale-session",
            Some("test stale".to_string()),
            None,
            None,
        )?;
        let mut updated_config = crate::config::get_config(Some(ship_dir.clone()))?;
        updated_config.providers = vec!["codex".to_string()];
        crate::config::save_config(&updated_config, Some(ship_dir.clone()))?;

        let _ = set_workspace_active_agent(&ship_dir, "feature/stale-session", None)?;

        let active = get_active_workspace_session(&ship_dir, "feature/stale-session")?
            .ok_or_else(|| anyhow::anyhow!("active session missing"))?;
        assert!(active.stale_context);

        let sessions = list_workspace_sessions(&ship_dir, Some("feature/stale-session"), 10)?;
        assert!(!sessions.is_empty());
        assert!(sessions[0].stale_context);
        Ok(())
    }
}
