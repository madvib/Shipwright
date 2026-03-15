pub mod agents;
pub mod catalog;
pub mod config;
pub mod events;
pub mod fs_util;
pub mod hooks;
pub mod log;
pub mod migration;
pub mod plugin;
pub mod project;
pub mod state_db;
pub mod workspace;

// Backward-compatible module aliases.
// Canonical implementation lives under `runtime::agents::*`.
pub use agents::config as agent_config;
pub use agents::export as agent_export;
pub use agents::permissions;
pub use agents::rule;
pub use agents::skill;

pub use agent_config::{AgentConfig, resolve_agent_config};
pub use agent_export::{
    ModelInfo, ProviderInfo, autodetect_providers, detect_binary, detect_version, disable_provider,
    enable_provider, export_to, import_from_claude, list_models, list_providers, sync_active_mode,
};
pub use catalog::{CatalogEntry, CatalogKind, list_catalog, list_catalog_by_kind, search_catalog};
pub use config::{
    AgentLayerConfig, AiConfig, GitConfig, HookConfig, HookTrigger, McpServerConfig, McpServerType,
    ModeConfig, NamespaceConfig, PermissionConfig, ProjectConfig, StatusConfig, add_hook,
    add_mcp_server, add_mode, add_status, ensure_registered_namespaces, generate_gitignore,
    get_active_mode, get_config, get_effective_config, get_git_config, get_project_statuses,
    is_category_committed, list_hooks, list_mcp_servers, migrate_json_config_file, remove_hook,
    remove_mcp_server, remove_mode, remove_status, save_config, set_active_mode,
    set_category_committed, set_git_config,
};

pub use events::{
    EVENTS_FILE_NAME, EventAction, EventEntity, EventRecord, append_event, ensure_event_log,
    export_events_ndjson, ingest_external_events, latest_event_seq, list_events_since, read_events,
    sync_event_snapshot,
};
pub use hooks::{DefaultRuntimeHooks, RuntimeHooks};
pub use log::{LogEntry, log_action, log_action_by, read_log, read_log_entries};
pub use migration::{
    GlobalStateMigrationReport, ProjectFileMigrationReport, ProjectStateMigrationReport,
    migrate_global_state, migrate_project_state,
};
pub use permissions::{
    AgentLimits, CommandPermissions, FsPermissions, NetworkPermissions, NetworkPolicy, Permissions,
    ToolPermissions, get_permissions, permission_tool_ids_for_provider, save_permissions,
};
pub use plugin::{Plugin, PluginRegistry};
// NOTE: ship-specific project primitives stay under `runtime::project`.
// Do not re-export them from the runtime root; this keeps the root API closer
// to domain-agnostic runtime/engine concerns.
pub use rule::{Rule, create_rule, delete_rule, get_rule, list_rules, update_rule};
pub use skill::{
    Skill, SkillInstallScope, SkillSource, create_skill, create_user_skill, delete_skill,
    delete_user_skill, get_effective_skill, get_skill, get_user_skill, install_skill_from_source,
    list_effective_skills, list_skills, list_user_skills, update_skill, update_user_skill,
};
pub use state_db::{
    CapabilityDb, CapabilityMapDb, DatabaseMigrationReport, WorkspaceSessionRecordDb,
    clear_branch_doc, clear_branch_link, clear_global_migration_meta, clear_project_migration_meta,
    ensure_global_database, ensure_project_database, get_branch_doc, get_branch_link,
    get_feature_primary_capability_db, get_managed_state_db, get_workspace_session_record_db,
    list_capabilities_db, list_capability_maps_db, list_target_features_db,
    mark_migration_meta_complete_global, mark_migration_meta_complete_project,
    migration_meta_complete_global, migration_meta_complete_project, replace_target_features_db,
    set_branch_doc, set_branch_link, set_feature_primary_capability_db, set_managed_state_db,
    upsert_capability_db, upsert_capability_map_db, upsert_workspace_db,
};
pub use workspace::{
    CreateWorkspaceRequest, EndWorkspaceSessionRequest, Environment, Process, ProcessStatus,
    ShipWorkspaceKind, Workspace, WorkspaceProviderMatrix, WorkspaceRepairReport, WorkspaceSession,
    WorkspaceSessionRecord, WorkspaceSessionStatus, WorkspaceStatus, activate_workspace,
    create_workspace, delete_workspace, end_workspace_session, get_active_workspace_session,
    get_workspace, get_workspace_provider_matrix, get_workspace_session_record,
    list_workspace_sessions, list_workspaces, record_workspace_session_progress, repair_workspace,
    set_workspace_active_mode, start_workspace_session, sync_workspace,
    transition_workspace_status, upsert_workspace, validate_workspace_transition,
};

pub fn gen_nanoid() -> String {
    let alphabet: [char; 56] = [
        '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K',
        'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd',
        'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
        'y', 'z',
    ];
    nanoid::format(nanoid::rngs::default, &alphabet, 8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{features_dir, get_project_dir, init_project, sanitize_file_name};
    use std::fs;
    use tempfile::tempdir;

    // ── Config tests ────────────────────────────────────────────────────────────

    #[test]
    fn test_add_and_remove_status() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        add_status(Some(project_dir.clone()), "testing")?;
        let statuses = get_project_statuses(Some(project_dir.clone()))?;
        assert!(statuses.contains(&"testing".to_string()));

        remove_status(Some(project_dir.clone()), "testing")?;
        let statuses = get_project_statuses(Some(project_dir))?;
        assert!(!statuses.contains(&"testing".to_string()));
        Ok(())
    }

    #[test]
    fn test_remove_status_without_issue_guard() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        add_status(Some(project_dir.clone()), "review")?;
        let result = remove_status(Some(project_dir), "review");
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_git_config_roundtrip() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        set_category_committed(&project_dir, "features", true)?;
        set_category_committed(&project_dir, "notes", false)?;
        let git = get_git_config(&project_dir)?;
        assert!(is_category_committed(&git, "features"));
        assert!(!is_category_committed(&git, "notes"));
        Ok(())
    }

    #[test]
    fn test_legacy_agents_category_maps_to_rules_mcp_permissions() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;

        set_category_committed(&project_dir, "agents", false)?;
        let gitignore = fs::read_to_string(project_dir.join(".gitignore"))?;
        assert!(gitignore.contains("agents/rules"));
        assert!(gitignore.contains("agents/mcp.toml"));
        assert!(gitignore.contains("agents/permissions.toml"));

        set_category_committed(&project_dir, "agents", true)?;
        let gitignore = fs::read_to_string(project_dir.join(".gitignore"))?;
        assert!(!gitignore.contains("agents/rules"));
        assert!(!gitignore.contains("agents/mcp.toml"));
        assert!(!gitignore.contains("agents/permissions.toml"));
        Ok(())
    }

    #[test]
    fn test_agent_layer_is_not_file_backed() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;

        let mut config = get_config(Some(project_dir.clone()))?;
        config.ai = Some(AiConfig {
            provider: Some("codex".into()),
            model: Some("gpt-5".into()),
            cli_path: None,
        });
        config.agent.skills = vec!["backend-rust".into(), "frontend-react".into()];
        config.agent.prompts = vec!["Summarize risks first".into()];
        config.agent.context = vec!["AGENTS.md".into(), "specs/".into()];
        save_config(&config, Some(project_dir.clone()))?;

        let loaded = get_config(Some(project_dir))?;
        assert_eq!(loaded.ai.and_then(|ai| ai.provider), Some("codex".into()));
        assert!(loaded.agent.skills.is_empty());
        assert!(loaded.agent.prompts.is_empty());
        assert!(loaded.agent.context.is_empty());
        Ok(())
    }

    #[test]
    fn test_generate_gitignore() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let gitignore = fs::read_to_string(project_dir.join(".gitignore"))?;
        // Default config keeps project docs local unless explicitly included.
        assert!(gitignore.contains("generated/"));
        assert!(gitignore.contains(".tmp-global/"));
        assert!(gitignore.contains("project/releases"));
        assert!(gitignore.contains("project/features"));
        assert!(gitignore.contains("project/specs"));
        assert!(gitignore.contains("project/adrs"));
        assert!(gitignore.contains("project/notes"));
        assert!(gitignore.contains("vision.md"));
        assert!(gitignore.contains("agents/skills"));
        assert!(gitignore.contains("agents/README.md"));
        assert!(!gitignore.contains("agents/rules"));
        assert!(!gitignore.contains("agents/mcp.toml"));
        assert!(!gitignore.contains("agents/permissions.toml"));
        // DB is now at ~/.ship/state/<slug>/ship.db — not inside .ship/
        assert!(!gitignore.contains("ship.db"));
        assert!(!gitignore.contains("log.md"));
        Ok(())
    }

    // ── Log tests ───────────────────────────────────────────────────────────────

    #[test]
    fn test_log_action() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        log_action(&project_dir, "test", "details")?;
        let entries = read_log_entries(&project_dir)?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].actor, "ship");
        assert_eq!(entries[0].action, "test");
        assert_eq!(entries[0].details, "details");
        assert!(!project_dir.join("log.md").exists());
        Ok(())
    }

    #[test]
    fn test_log_action_by() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        log_action_by(&project_dir, "agent", "create", "issue-abc.md")?;
        let entries = read_log_entries(&project_dir)?;
        assert_eq!(entries[0].actor, "agent");
        assert_eq!(entries[0].action, "create");
        assert_eq!(entries[0].details, "issue-abc.md");
        Ok(())
    }

    #[test]
    fn test_read_log_entries() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        log_action(&project_dir, "create", "first entry")?;
        log_action(&project_dir, "update", "second entry")?;
        let entries = read_log_entries(&project_dir)?;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].actor, "ship");
        assert_eq!(entries[0].action, "update");
        assert_eq!(entries[1].actor, "ship");
        assert_eq!(entries[1].action, "create");
        Ok(())
    }

    // ── Project tests ───────────────────────────────────────────────────────────

    #[test]
    fn test_init_project() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        assert!(ship_path.exists());
        // project/ namespace
        assert!(ship_path.join("project/specs").is_dir());
        assert!(ship_path.join("project/features").is_dir());
        assert!(ship_path.join("project/releases").is_dir());
        assert!(ship_path.join("project/adrs").is_dir());
        assert!(ship_path.join("project/notes").is_dir());
        assert!(ship_path.join("vision.md").is_file());
        assert!(ship_path.join("generated").is_dir());
        let project_skills_dir = crate::project::skills_dir(&ship_path);
        assert!(project_skills_dir.is_dir());
        // shared
        assert!(ship_path.join("project/releases/TEMPLATE.md").is_file());
        assert!(ship_path.join("project/features/TEMPLATE.md").is_file());
        assert!(ship_path.join("project/notes/TEMPLATE.md").is_file());
        let cfg = crate::config::get_config(Some(ship_path.clone()))?;
        assert!(
            cfg.modes.is_empty(),
            "new projects should not seed legacy planning/code/config modes by default"
        );
        assert!(!ship_path.join("events.ndjson").is_file());
        assert!(ship_path.join("ship.toml").is_file());
        // default skill seeded
        assert!(project_skills_dir.join("task-policy/SKILL.md").is_file());
        let skill_content = fs::read_to_string(project_skills_dir.join("task-policy/SKILL.md"))?;
        assert!(skill_content.contains("task-policy"));
        assert!(skill_content.contains("Ship Workflow Policy"));
        Ok(())
    }

    #[test]
    fn test_init_project_idempotent() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        // First init
        let ship_path = init_project(tmp.path().to_path_buf())?;
        let project_skills_dir = crate::project::skills_dir(&ship_path);
        // Write a custom skill so we can verify it isn't clobbered
        let custom_skill = project_skills_dir.join("custom/SKILL.md");
        fs::create_dir_all(custom_skill.parent().expect("custom skill dir"))?;
        fs::write(
            &custom_skill,
            "---\nname: custom\ndescription: Custom test skill.\n---\n\nmy content",
        )?;
        // Second init on the same directory
        let ship_path2 = init_project(tmp.path().to_path_buf())?;
        assert_eq!(ship_path, ship_path2);
        // Custom skill must still be present and unchanged
        assert!(custom_skill.exists());
        assert_eq!(
            fs::read_to_string(&custom_skill)?,
            "---\nname: custom\ndescription: Custom test skill.\n---\n\nmy content"
        );
        // Default skill still present
        assert!(project_skills_dir.join("task-policy/SKILL.md").is_file());
        Ok(())
    }

    #[test]
    fn test_init_project_feature_template_has_rich_fields() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        let template = fs::read_to_string(ship_path.join("project/features/TEMPLATE.md"))?;
        // New lifecycle fields
        // Status is directory-based, not in frontmatter
        assert!(template.contains("release_id"));
        assert!(template.contains("## Why"));
        assert!(template.contains("## Delivery Todos"));
        Ok(())
    }

    struct DemoPlugin;
    impl Plugin for DemoPlugin {
        fn name(&self) -> &str {
            "demo-plugin"
        }
        fn description(&self) -> &str {
            "Demo plugin for tests"
        }
    }

    #[test]
    fn test_plugin_activation_creates_namespace() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        let mut registry = PluginRegistry::new();
        registry.register_with_project(&ship_path, Box::new(DemoPlugin))?;

        let namespaces = get_config(Some(ship_path.clone()))?.namespaces;
        assert!(namespaces.iter().any(|ns| ns.id == "plugin:demo-plugin"));
        Ok(())
    }

    #[test]
    fn test_event_stream_since() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        assert!(!ship_path.join("events.ndjson").exists());
        let seq0 = latest_event_seq(&ship_path)?;
        append_event(
            &ship_path,
            "ship",
            EventEntity::Feature,
            EventAction::Create,
            "feat-1",
            Some("Created feature".to_string()),
        )?;
        append_event(
            &ship_path,
            "ship",
            EventEntity::Feature,
            EventAction::Create,
            "feat-2",
            Some("Created another feature".to_string()),
        )?;
        let events = list_events_since(&ship_path, seq0, None)?;
        assert!(events.len() >= 2);
        assert!(events.iter().all(|e| e.seq > seq0));
        assert!(events.windows(2).all(|w| w[0].seq < w[1].seq));
        Ok(())
    }

    #[test]
    fn test_event_export_on_demand() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        append_event(
            &ship_path,
            "ship",
            EventEntity::Project,
            EventAction::Log,
            "export",
            Some("ndjson".to_string()),
        )?;
        let export_path = ship_path.join("generated").join("events-export.ndjson");
        let count = export_events_ndjson(&ship_path, &export_path)?;
        assert!(count >= 1);
        let content = fs::read_to_string(&export_path)?;
        assert!(content.contains("\"action\":\"log\""));
        Ok(())
    }

    #[test]
    fn test_ingest_external_events_detects_filesystem_changes() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;

        // Ensure snapshot is synced to current state.
        let _ = ingest_external_events(&ship_path)?;

        let manual = features_dir(&ship_path).join("manual-sync.md");
        fs::write(&manual, "+++\ntitle = \"Manual\"\n+++\n\nbody\n")?;
        let created = ingest_external_events(&ship_path)?;
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].actor, "filesystem");
        assert_eq!(created[0].entity, EventEntity::Feature);
        assert_eq!(created[0].action, EventAction::Create);

        fs::write(&manual, "+++\ntitle = \"Manual\"\n+++\n\nchanged\n")?;
        let updated = ingest_external_events(&ship_path)?;
        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].actor, "filesystem");
        assert_eq!(updated[0].entity, EventEntity::Feature);
        assert_eq!(updated[0].action, EventAction::Update);

        fs::remove_file(&manual)?;
        let deleted = ingest_external_events(&ship_path)?;
        assert_eq!(deleted.len(), 1);
        assert_eq!(deleted[0].actor, "filesystem");
        assert_eq!(deleted[0].entity, EventEntity::Feature);
        assert_eq!(deleted[0].action, EventAction::Delete);
        Ok(())
    }

    #[test]
    fn test_legacy_migration() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_path = tmp.path().join(".project");
        fs::create_dir_all(&project_path)?;
        let project_dir = get_project_dir(Some(tmp.path().to_path_buf()))?;
        assert!(!project_path.exists());
        assert_eq!(
            fs::canonicalize(project_dir)?,
            fs::canonicalize(tmp.path().join(".ship"))?
        );
        Ok(())
    }

    #[test]
    fn test_get_project_dir_resolves_main_ship_from_worktree() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let main_root = tmp.path().join("main");
        let main_ship = main_root.join(".ship");
        let common_git = main_root.join(".git");
        let worktree_git = common_git.join("worktrees").join("feature-auth");
        let worktree_root = tmp.path().join("worktrees").join("feature-auth");
        let worktree_nested = worktree_root.join("src").join("ui");

        fs::create_dir_all(&main_ship)?;
        fs::create_dir_all(&worktree_git)?;
        fs::create_dir_all(&worktree_nested)?;
        fs::write(
            worktree_root.join(".git"),
            format!("gitdir: {}\n", worktree_git.display()),
        )?;

        let resolved = get_project_dir(Some(worktree_nested))?;
        assert_eq!(
            fs::canonicalize(resolved)?,
            fs::canonicalize(main_ship)?,
            "worktree paths should resolve to the main checkout .ship"
        );
        Ok(())
    }

    #[test]
    fn test_get_project_dir_prefers_main_ship_when_worktree_has_local_copy() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let main_root = tmp.path().join("main");
        let main_ship = main_root.join(".ship");
        let common_git = main_root.join(".git");
        let worktree_git = common_git.join("worktrees").join("feature-auth");
        let worktree_root = tmp.path().join("worktrees").join("feature-auth");
        let worktree_nested = worktree_root.join("src").join("ui");

        fs::create_dir_all(&main_ship)?;
        fs::create_dir_all(&worktree_git)?;
        fs::create_dir_all(worktree_root.join(".ship"))?;
        fs::create_dir_all(&worktree_nested)?;
        fs::write(
            worktree_root.join(".git"),
            format!("gitdir: {}\n", worktree_git.display()),
        )?;

        let resolved = get_project_dir(Some(worktree_nested))?;
        assert_eq!(
            fs::canonicalize(resolved)?,
            fs::canonicalize(main_ship)?,
            "worktree paths should resolve to the main checkout .ship even when the worktree has a local .ship copy"
        );
        Ok(())
    }

    #[test]
    fn test_env_override() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let override_dir = tmp.path().join("override");
        fs::create_dir_all(&override_dir)?;
        unsafe {
            std::env::set_var("SHIP_DIR", override_dir.to_str().unwrap());
        }
        let project_dir = get_project_dir(None)?;
        assert_eq!(project_dir, override_dir);
        unsafe {
            std::env::remove_var("SHIP_DIR");
        }
        Ok(())
    }

    #[test]
    fn test_sanitize_file_name() {
        assert_eq!(sanitize_file_name("My Issue Title!"), "my-issue-title");
        assert_eq!(
            sanitize_file_name("Already_Sanitized-123"),
            "already_sanitized-123"
        );
    }
}
