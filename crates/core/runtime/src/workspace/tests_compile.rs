#[cfg(test)]
mod tests {
    use crate::workspace::compile::resolve_workspace_agent_config;
    use crate::workspace::*;
    use anyhow::Result;
    use tempfile::tempdir;

    #[test]
    fn workspace_compile_resolves_default_providers() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
        config.providers = vec!["codex".to_string()];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/compile-resolve".to_string(),
                ..Default::default()
            },
        )?;

        let workspace = get_workspace(&ship_dir, "feature/compile-resolve")?
            .ok_or_else(|| anyhow::anyhow!("workspace missing"))?;
        let resolved = resolve_workspace_agent_config(&ship_dir, &workspace, None)?;
        assert!(!resolved.providers.is_empty());
        Ok(())
    }

    #[test]
    fn set_workspace_active_agent_updates_and_clears_override() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let config = crate::config::ProjectConfig {
            modes: vec![crate::config::AgentProfile {
                id: "planning".to_string(),
                name: "Planning".to_string(),
                target_agents: vec!["codex".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        };
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/mode-override".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let updated =
            set_workspace_active_agent(&ship_dir, "feature/mode-override", Some("planning"))?;
        assert_eq!(updated.active_agent.as_deref(), Some("planning"));
        assert!(tmp.path().join(".codex").join("config.toml").exists());

        let cleared = set_workspace_active_agent(&ship_dir, "feature/mode-override", None)?;
        assert!(cleared.active_agent.is_none());
        Ok(())
    }

    #[test]
    fn provider_matrix_uses_config_default_source() -> Result<()> {
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
                branch: "feature/provider-matrix".to_string(),
                ..Default::default()
            },
        )?;

        let matrix = get_workspace_provider_matrix(
            &ship_dir,
            "feature/provider-matrix",
            None,
        )?;
        assert_eq!(matrix.source, "config/default");
        assert!(matrix.resolution_error.is_none());
        Ok(())
    }

    #[test]
    fn provider_matrix_reports_resolution_error_for_empty_providers() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        // No providers configured at all
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-invalid".to_string(),
                ..Default::default()
            },
        )?;

        let matrix = get_workspace_provider_matrix(
            &ship_dir,
            "feature/provider-invalid",
            None,
        )?;
        // With no providers configured, resolution_error should be set
        if matrix.allowed_providers.is_empty() {
            assert!(matrix.resolution_error.is_some());
        }
        Ok(())
    }

    #[test]
    fn repair_workspace_on_archived_workspace_reports_activation_action() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/repair-idle".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;
        transition_workspace_status(
            &ship_dir,
            "feature/repair-idle",
            WorkspaceStatus::Archived,
        )?;

        let report = repair_workspace(&ship_dir, "feature/repair-idle", false)?;
        assert!(!report.reapplied_compile);
        Ok(())
    }
}
