#[cfg(test)]
mod tests {
    use crate::workspace::compile::resolve_workspace_agent_config;
    use crate::workspace::*;
    use anyhow::Result;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn stdio_server(id: &str, command: &str) -> crate::config::McpServerConfig {
        crate::config::McpServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            command: command.to_string(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: crate::config::McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        }
    }

    #[test]
    fn workspace_invalid_provider_override_reports_error() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
        config.providers = vec!["codex".to_string()];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-fallback".to_string(),
                providers: Some(vec!["totally-unknown-provider".to_string()]),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        let matrix = get_workspace_provider_matrix(&ship_dir, "feature/provider-fallback", None)?;
        assert!(matrix.allowed_providers.is_empty());
        assert!(matrix.resolution_error.is_some());
        Ok(())
    }

    #[test]
    fn workspace_compile_exports_only_workspace_filtered_skills() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
        config.providers = vec!["codex".to_string()];
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        crate::skill::create_skill(&ship_dir, "selected-skill", "Selected", "selected content")?;
        crate::skill::create_skill(&ship_dir, "other-skill", "Other", "other content")?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/skill-filter".to_string(),
                status: Some(WorkspaceStatus::Active),
                providers: Some(vec!["codex".to_string()]),
                skills: Some(vec!["selected-skill".to_string()]),
                ..CreateWorkspaceRequest::default()
            },
        )?;

        let workspace = activate_workspace(&ship_dir, "feature/skill-filter")?;
        assert!(workspace.compile_error.is_none());

        let project_root = ship_dir.parent().unwrap_or(&ship_dir).to_path_buf();
        assert!(
            project_root
                .join(".agents")
                .join("skills")
                .join("selected-skill")
                .join("SKILL.md")
                .exists(),
            "selected workspace skill should be exported"
        );
        assert!(
            !project_root
                .join(".agents")
                .join("skills")
                .join("other-skill")
                .join("SKILL.md")
                .exists(),
            "non-selected workspace skill should not be exported"
        );
        Ok(())
    }

    #[test]
    fn workspace_agent_overrides_persist_and_round_trip() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/agent-overrides".to_string(),
                providers: Some(vec!["codex".to_string()]),
                mcp_servers: Some(vec!["github".to_string()]),
                skills: Some(vec!["task-policy".to_string()]),
                ..Default::default()
            },
        )?;

        let workspace = get_workspace(&ship_dir, "feature/agent-overrides")?
            .ok_or_else(|| anyhow::anyhow!("workspace missing"))?;
        assert_eq!(workspace.providers, vec!["codex".to_string()]);
        assert_eq!(workspace.mcp_servers, vec!["github".to_string()]);
        assert_eq!(workspace.skills, vec!["task-policy".to_string()]);
        Ok(())
    }

    #[test]
    fn workspace_agent_overrides_resolve_in_provider_config() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let config = crate::config::ProjectConfig {
            providers: vec!["claude".to_string()],
            mcp_servers: vec![
                stdio_server("github", "gh"),
                stdio_server("linear", "linear"),
            ],
            ..Default::default()
        };
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        crate::skill::create_skill(
            &ship_dir,
            "workspace-skill",
            "Workspace",
            "workspace content",
        )?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/workspace-agent-override".to_string(),
                providers: Some(vec!["codex".to_string()]),
                mcp_servers: Some(vec!["linear".to_string()]),
                skills: Some(vec!["workspace-skill".to_string()]),
                ..Default::default()
            },
        )?;

        let workspace = get_workspace(&ship_dir, "feature/workspace-agent-override")?
            .ok_or_else(|| anyhow::anyhow!("workspace missing"))?;
        let resolved = resolve_workspace_agent_config(&ship_dir, &workspace, None)?;

        assert_eq!(resolved.providers, vec!["codex".to_string()]);
        assert_eq!(
            resolved
                .mcp_servers
                .iter()
                .map(|server| server.id.as_str())
                .collect::<Vec<_>>(),
            vec!["linear"]
        );
        assert_eq!(
            resolved
                .skills
                .iter()
                .map(|skill| skill.id.as_str())
                .collect::<Vec<_>>(),
            vec!["workspace-skill"]
        );
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
        assert!(updated.config_generation >= 1);
        assert!(updated.compiled_at.is_some());
        assert!(updated.compile_error.is_none());
        assert!(tmp.path().join(".codex").join("config.toml").exists());

        let cleared = set_workspace_active_agent(&ship_dir, "feature/mode-override", None)?;
        assert!(cleared.active_agent.is_none());
        assert!(cleared.config_generation > updated.config_generation);
        Ok(())
    }

    #[test]
    fn provider_matrix_prefers_workspace_provider_overrides() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let config = crate::config::ProjectConfig {
            providers: vec!["claude".to_string()],
            modes: vec![crate::config::AgentProfile {
                id: "planning".to_string(),
                name: "Planning".to_string(),
                target_agents: vec!["gemini".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        };
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-matrix".to_string(),
                providers: Some(vec!["codex".to_string()]),
                active_agent: Some("planning".to_string()),
                ..Default::default()
            },
        )?;

        let matrix = get_workspace_provider_matrix(&ship_dir, "feature/provider-matrix", None)?;
        assert_eq!(matrix.source, "workspace");
        assert_eq!(matrix.allowed_providers, vec!["codex".to_string()]);
        assert!(matrix.resolution_error.is_none());
        Ok(())
    }

    #[test]
    fn provider_matrix_prefers_workspace_providers_over_agent_and_config() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let config = crate::config::ProjectConfig {
            providers: vec!["claude".to_string()],
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
                branch: "feature/provider-ws".to_string(),
                providers: Some(vec!["gemini".to_string()]),
                active_agent: Some("planning".to_string()),
                ..Default::default()
            },
        )?;

        let matrix = get_workspace_provider_matrix(&ship_dir, "feature/provider-ws", None)?;
        assert_eq!(matrix.source, "workspace");
        assert_eq!(matrix.allowed_providers, vec!["gemini".to_string()]);
        assert!(matrix.resolution_error.is_none());
        Ok(())
    }

    #[test]
    fn provider_matrix_reports_resolution_error_for_invalid_candidates() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/provider-invalid".to_string(),
                providers: Some(vec!["ghost-provider".to_string()]),
                ..Default::default()
            },
        )?;

        let matrix = get_workspace_provider_matrix(&ship_dir, "feature/provider-invalid", None)?;
        assert!(matrix.allowed_providers.is_empty());
        assert!(matrix.resolution_error.is_some());
        Ok(())
    }

    #[test]
    fn repair_workspace_dry_run_reports_missing_provider_config() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/repair-dry-run".to_string(),
                providers: Some(vec!["codex".to_string()]),
                ..Default::default()
            },
        )?;

        let report = repair_workspace(&ship_dir, "feature/repair-dry-run", true)?;
        assert_eq!(report.workspace_branch, "feature/repair-dry-run");
        assert!(report.dry_run);
        assert!(report.needs_recompile);
        assert!(report.missing_provider_configs.iter().any(|p| p == "codex"));
        assert!(!report.reapplied_compile);
        Ok(())
    }

    #[test]
    fn repair_workspace_apply_recompiles_active_workspace() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        let created = create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/repair-apply".to_string(),
                providers: Some(vec!["codex".to_string()]),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;
        assert_eq!(created.status, WorkspaceStatus::Active);

        let codex_config = tmp.path().join(".codex").join("config.toml");
        if codex_config.exists() {
            std::fs::remove_file(&codex_config)?;
        }

        let report = repair_workspace(&ship_dir, "feature/repair-apply", false)?;
        assert!(report.reapplied_compile);
        assert!(!report.needs_recompile);
        assert!(report.missing_provider_configs.is_empty());
        assert!(codex_config.exists());
        Ok(())
    }

    #[test]
    fn repair_workspace_apply_on_idle_workspace_reports_activation_action() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf())?;

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/repair-idle".to_string(),
                providers: Some(vec!["codex".to_string()]),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;
        transition_workspace_status(&ship_dir, "feature/repair-idle", WorkspaceStatus::Archived)?;

        let report = repair_workspace(&ship_dir, "feature/repair-idle", false)?;
        assert!(report.needs_recompile);
        assert!(!report.reapplied_compile);
        assert!(
            report
                .actions
                .iter()
                .any(|action| action.contains("activate workspace"))
        );
        Ok(())
    }
}
