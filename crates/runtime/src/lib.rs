pub mod adr;
pub mod agent_config;
pub mod agent_export;
pub mod catalog;
pub mod config;
pub mod demo;
pub mod events;
pub mod feature;
mod fs_util;
pub mod issue;
pub mod log;
pub mod migration;
pub mod note;
pub mod permissions;
pub mod plugin;
pub mod project;
pub mod prompt;
pub mod release;
pub mod rule;
pub mod skill;
pub mod spec;
pub mod state_db;
pub mod vision;
pub mod workspace;

pub use adr::{
    ADR, AdrEntry, AdrMetadata, AdrStatus, create_adr, delete_adr, find_adr_path, get_adr,
    list_adrs, move_adr, update_adr,
};
pub use agent_config::{AgentConfig, resolve_agent_config};
pub use agent_export::{
    ModelInfo, ProviderInfo, autodetect_providers, detect_binary, detect_version,
    disable_provider, enable_provider, export_to, import_from_claude, list_models, list_providers,
    sync_active_mode,
};
pub use catalog::{CatalogEntry, CatalogKind, list_catalog, list_catalog_by_kind, search_catalog};
pub use config::{
    AgentLayerConfig, AiConfig, GitConfig, HookConfig, HookTrigger, McpServerConfig, McpServerType,
    ModeConfig, NamespaceConfig, PermissionConfig, ProjectConfig, StatusConfig, add_hook,
    add_mcp_server, add_mode, add_status, generate_gitignore, get_active_mode, get_config,
    get_effective_config, get_git_config, get_project_statuses, is_category_committed, list_hooks,
    list_mcp_servers, migrate_json_config_file, remove_hook, remove_mcp_server, remove_mode,
    remove_status, save_config, set_active_mode, set_category_committed, set_git_config,
};
pub use demo::init_demo_project;
pub use events::{
    EVENTS_FILE_NAME, EventAction, EventEntity, EventRecord, append_event, ensure_event_log,
    event_log_path, ingest_external_events, latest_event_seq, list_events_since, read_events,
    sync_event_snapshot,
};
pub use feature::{
    Feature, FeatureAgentConfig, FeatureEntry, FeatureMcpRef, FeatureMetadata, FeatureSkillRef,
    FeatureStatus, create_feature, feature_done, feature_start, find_feature_path, get_feature,
    get_feature_raw, list_features, update_feature,
};
pub use issue::{
    Issue, IssueEntry, IssueLink, IssueMetadata, IssuePriority, add_link, append_note,
    backfill_issue_ids, create_issue, delete_issue, get_issue, list_issues, list_issues_full,
    migrate_yaml_issues, move_issue, update_issue,
};
pub use log::{LogEntry, log_action, log_action_by, read_log, read_log_entries};
pub use migration::{
    GlobalStateMigrationReport, ProjectFileMigrationReport, ProjectStateMigrationReport,
    migrate_global_state, migrate_project_state,
};
pub use note::{
    Note, NoteEntry, NoteMetadata, NoteScope, create_note, get_note, get_note_raw, list_notes,
    note_path_for_scope, update_note,
};
pub use permissions::{
    AgentLimits, CommandPermissions, FsPermissions, NetworkPermissions, NetworkPolicy, Permissions,
    ToolPermissions, get_permissions, save_permissions,
};
pub use plugin::{Plugin, PluginRegistry};
pub use project::{
    AppState as GlobalAppState, DEFAULT_STATUSES, ISSUE_STATUSES, ProjectEntry, ProjectRegistry,
    SHIP_DIR_NAME, get_active_project_global, get_global_dir, get_project_dir, get_project_name,
    get_recent_projects_global, get_registry_path, init_project, list_registered_namespaces,
    list_registered_projects, load_app_state, load_registry, read_template, register_project,
    register_ship_namespace, sanitize_file_name, save_app_state, save_registry,
    set_active_project_global, unregister_project,
};
pub use prompt::{Prompt, create_prompt, delete_prompt, get_prompt, list_prompts, update_prompt};
pub use release::{
    Release, ReleaseEntry, ReleaseMetadata, ReleaseStatus, create_release, find_release_path,
    get_release, get_release_raw, list_releases, update_release,
};
pub use rule::{Rule, create_rule, delete_rule, get_rule, list_rules, update_rule};
pub use skill::{
    Skill, SkillSource, create_skill, create_user_skill, delete_skill, delete_user_skill,
    get_effective_skill, get_skill, get_user_skill, list_effective_skills, list_skills,
    list_user_skills, update_skill, update_user_skill,
};
pub use spec::{
    Spec, SpecEntry, SpecMetadata, SpecStatus, create_spec, delete_spec, get_spec, get_spec_raw,
    list_specs, update_spec,
};
pub use state_db::{
    DatabaseMigrationReport, ensure_global_database, ensure_project_database, get_branch_doc,
    get_managed_state_db, set_branch_doc, set_managed_state_db, upsert_workspace_db,
};
pub use vision::{Vision, get_vision, update_vision};
pub use workspace::{Workspace, get_workspace, upsert_workspace};

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
    use std::fs;
    use tempfile::tempdir;

    // ── Issue tests ─────────────────────────────────────────────────────────────

    #[test]
    fn test_create_issue() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let issue_path = create_issue(project_dir, "Test Issue", "Desc", "backlog")?;
        assert!(issue_path.exists());
        assert_eq!(issue_path.extension().unwrap(), "md");
        let content = fs::read_to_string(issue_path)?;
        assert!(content.contains("title = \"Test Issue\""));
        assert!(content.contains("Desc"));
        Ok(())
    }

    #[test]
    fn test_create_issue_empty_title_rejected() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let result = create_issue(project_dir, "", "Desc", "backlog");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .to_lowercase()
                .contains("empty")
        );
        Ok(())
    }

    #[test]
    fn test_create_issue_invalid_status_rejected() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let result = create_issue(project_dir, "Title", "Desc", "../evil");
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_create_issue_collision_gets_suffix() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let p1 = create_issue(project_dir.clone(), "Fix Bug", "a", "backlog")?;
        let p2 = create_issue(project_dir.clone(), "Fix Bug!", "b", "backlog")?; // same slug
        assert_ne!(p1, p2);
        assert!(p1.exists());
        assert!(p2.exists());
        // Both files should be readable
        assert_eq!(get_issue(p1)?.metadata.title, "Fix Bug");
        assert_eq!(get_issue(p2)?.metadata.title, "Fix Bug!");
        Ok(())
    }

    #[test]
    fn test_create_issue_has_uuid() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_issue(project_dir, "UUID Test", "", "backlog")?;
        let issue = get_issue(path)?;
        assert!(!issue.metadata.id.is_empty(), "id should be populated");
        assert_eq!(issue.metadata.id.len(), 8, "ID should be 8 chars (nanoid)");
        Ok(())
    }

    #[test]
    fn test_list_issues() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        create_issue(project_dir.clone(), "Issue 1", "Desc 1", "backlog")?;
        create_issue(project_dir.clone(), "Issue 2", "Desc 2", "in-progress")?;
        let issues = list_issues(project_dir)?;
        assert_eq!(issues.len(), 2);
        let titles: Vec<String> = issues.iter().map(|(n, _)| n.clone()).collect();
        assert!(titles.contains(&"issue-1.md".to_string()));
        assert!(titles.contains(&"issue-2.md".to_string()));
        Ok(())
    }

    #[test]
    fn test_list_issues_full() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        create_issue(
            project_dir.clone(),
            "Full Issue",
            "Detailed desc",
            "backlog",
        )?;
        let entries = list_issues_full(project_dir)?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].issue.metadata.title, "Full Issue");
        assert_eq!(entries[0].issue.description, "Detailed desc");
        Ok(())
    }

    #[test]
    fn test_get_issue() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_issue(project_dir, "Get Issue", "Some desc", "backlog")?;
        let issue = get_issue(path)?;
        assert_eq!(issue.metadata.title, "Get Issue");
        Ok(())
    }

    #[test]
    fn test_update_issue() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_issue(project_dir, "Update Me", "original", "backlog")?;
        let mut issue = get_issue(path.clone())?;
        issue.description = "updated".to_string();
        update_issue(path.clone(), issue)?;
        let reloaded = get_issue(path)?;
        assert_eq!(reloaded.description, "updated");
        Ok(())
    }

    #[test]
    fn test_delete_issue() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_issue(project_dir, "Delete Me", "bye", "backlog")?;
        assert!(path.exists());
        delete_issue(path.clone())?;
        assert!(!path.exists());
        Ok(())
    }

    #[test]
    fn test_move_issue() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_issue(project_dir.clone(), "Test Issue", "Desc", "backlog")?;
        let new_path = move_issue(project_dir.clone(), path, "backlog", "in-progress")?;
        assert!(new_path.exists());
        assert!(new_path.to_str().unwrap().contains("in-progress"));
        assert_eq!(new_path.extension().unwrap(), "md");
        let issues = list_issues(project_dir)?;
        assert_eq!(issues[0].1, "in-progress");
        Ok(())
    }

    #[test]
    fn test_issue_note_event_writes_to_root_stream() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let issue_path = create_issue(project_dir.clone(), "Event Note", "Desc", "backlog")?;
        append_note(issue_path, "Implementation summary")?;

        let root_events = fs::read_to_string(project_dir.join("events.ndjson"))?;
        assert!(root_events.contains("\"entity\":\"issue\""));
        assert!(root_events.contains("\"action\":\"note\""));
        assert!(!project_dir.join("workflow/events.ndjson").exists());
        Ok(())
    }

    #[test]
    fn test_add_link_typed() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_issue(project_dir, "Issue Link", "Desc", "backlog")?;
        add_link(path.clone(), "blocks", "other-issue.md")?;
        let issue = get_issue(path)?;
        assert_eq!(issue.metadata.links.len(), 1);
        assert_eq!(issue.metadata.links[0].type_, "blocks");
        assert_eq!(issue.metadata.links[0].target, "other-issue.md");
        Ok(())
    }

    #[test]
    fn test_yaml_migration() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let legacy = "---\ntitle: Legacy Issue\nstatus: backlog\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nlinks: []\n---\n\nOld body.\n";
        fs::write(
            project_dir.join("workflow/issues/backlog/legacy-issue.md"),
            legacy,
        )?;
        let migrated = migrate_yaml_issues(&project_dir)?;
        assert_eq!(migrated, 1);
        let content =
            fs::read_to_string(project_dir.join("workflow/issues/backlog/legacy-issue.md"))?;
        assert!(content.starts_with("+++\n"));
        assert!(content.contains("title = \"Legacy Issue\""));
        Ok(())
    }

    // ── ADR tests ───────────────────────────────────────────────────────────────

    #[test]
    fn test_create_adr() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let adr_path = create_adr(
            project_dir,
            "Use PostgreSQL",
            "Chosen for robustness",
            "accepted",
        )?;
        assert!(adr_path.exists());
        let content = fs::read_to_string(adr_path)?;
        assert!(content.contains("title = \"Use PostgreSQL\""));
        Ok(())
    }

    #[test]
    fn test_create_adr_empty_title_rejected() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let result = create_adr(project_dir, "", "decision", "accepted");
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_create_adr_has_uuid() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_adr(project_dir, "Use Redis", "Fast in-memory store", "accepted")?;
        let adr = get_adr(path)?;
        assert!(!adr.metadata.id.is_empty());
        assert_eq!(adr.metadata.id.len(), 8);
        Ok(())
    }

    #[test]
    fn test_get_adr() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_adr(project_dir, "Use SQLite", "Embedded", "proposed")?;
        let adr = get_adr(path.clone())?;
        assert_eq!(adr.metadata.title, "Use SQLite");
        assert!(path.to_string_lossy().contains("/proposed/"));
        assert!(adr.body.contains("Embedded"));
        Ok(())
    }

    #[test]
    fn test_update_adr() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_adr(project_dir.clone(), "Update ADR", "original", "proposed")?;
        let mut adr = get_adr(path.clone())?;
        adr.body = "## Decision\n\nupdated body\n".to_string();
        update_adr(path.clone(), adr)?;
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let path = move_adr(project_dir, &file_name, AdrStatus::Accepted)?;
        let reloaded = get_adr(path.clone())?;
        assert!(path.to_string_lossy().contains("/accepted/"));
        assert!(reloaded.body.contains("updated body"));
        Ok(())
    }

    #[test]
    fn test_list_adrs() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        create_adr(project_dir.clone(), "ADR One", "decision one", "accepted")?;
        create_adr(project_dir.clone(), "ADR Two", "decision two", "proposed")?;
        let adrs = list_adrs(project_dir)?;
        assert_eq!(adrs.len(), 2);
        let titles: Vec<&str> = adrs.iter().map(|a| a.adr.metadata.title.as_str()).collect();
        assert!(titles.contains(&"ADR One"));
        assert!(titles.contains(&"ADR Two"));
        Ok(())
    }

    #[test]
    fn test_delete_adr() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_adr(project_dir.clone(), "Delete ADR", "decision", "accepted")?;
        assert!(path.exists());
        delete_adr(path.clone())?;
        assert!(!path.exists());
        Ok(())
    }

    #[test]
    fn test_adr_collision_gets_suffix() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let p1 = create_adr(project_dir.clone(), "Use Postgres", "reason a", "accepted")?;
        let p2 = create_adr(project_dir.clone(), "Use Postgres!", "reason b", "proposed")?;
        assert_ne!(p1, p2);
        assert!(p1.exists());
        assert!(p2.exists());
        Ok(())
    }

    // ── Spec tests ──────────────────────────────────────────────────────────────

    #[test]
    fn test_create_spec() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_spec(project_dir, "Auth Flow", "", "draft")?;
        assert!(path.exists());
        let content = fs::read_to_string(&path)?;
        assert!(content.starts_with("+++\n"));
        assert!(content.contains("title = \"Auth Flow\""));
        assert!(path.to_string_lossy().contains("/draft/"));
        Ok(())
    }

    #[test]
    fn test_create_spec_empty_title_rejected() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let result = create_spec(project_dir, "", "", "draft");
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_create_spec_has_uuid() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_spec(project_dir, "My Spec", "", "draft")?;
        let spec = get_spec(path)?;
        assert!(!spec.metadata.id.is_empty());
        assert_eq!(spec.metadata.id.len(), 8);
        Ok(())
    }

    #[test]
    fn test_get_spec() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_spec(
            project_dir,
            "Feature Spec",
            "## Overview\n\nCustom body.",
            "draft",
        )?;
        let spec = get_spec(path)?;
        assert_eq!(spec.metadata.title, "Feature Spec");
        assert!(spec.body.contains("Custom body."));
        Ok(())
    }

    #[test]
    fn test_update_spec() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_spec(project_dir, "Spec Update", "original body", "draft")?;
        let original = get_spec(path.clone())?;
        update_spec(path.clone(), "updated body")?;
        let updated = get_spec(path)?;
        assert_eq!(updated.body, "updated body");
        assert!(updated.metadata.updated >= original.metadata.updated);
        Ok(())
    }

    #[test]
    fn test_delete_spec() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_spec(project_dir, "Delete Spec", "content", "draft")?;
        assert!(path.exists());
        delete_spec(path.clone())?;
        assert!(!path.exists());
        Ok(())
    }

    #[test]
    fn test_list_specs() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        create_spec(project_dir.clone(), "Spec Alpha", "", "draft")?;
        create_spec(project_dir.clone(), "Spec Beta", "", "active")?;
        let specs = list_specs(project_dir)?;
        assert!(specs.len() >= 2); // vision.md moved to project/ namespace, no longer a spec
        let titles: Vec<&str> = specs.iter().map(|s| s.title.as_str()).collect();
        assert!(titles.contains(&"Spec Alpha"));
        assert!(titles.contains(&"Spec Beta"));
        Ok(())
    }

    #[test]
    fn test_spec_collision_gets_suffix() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let p1 = create_spec(project_dir.clone(), "Auth Flow", "", "draft")?;
        let p2 = create_spec(project_dir.clone(), "Auth Flow!", "", "draft")?;
        assert_ne!(p1, p2);
        assert!(p1.exists());
        assert!(p2.exists());
        Ok(())
    }

    // ── Release tests ───────────────────────────────────────────────────────────

    #[test]
    fn test_create_release() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_release(project_dir.clone(), "v0.1.0-alpha", "")?;
        assert!(path.exists());
        assert!(path.starts_with(project::upcoming_releases_dir(&project_dir)));
        let content = fs::read_to_string(&path)?;
        assert!(content.starts_with("+++\n"));
        assert!(content.contains("version = \"v0.1.0-alpha\""));
        assert!(content.contains("status = \"planned\""));
        assert!(content.contains("supported = false"));
        Ok(())
    }

    #[test]
    fn test_create_release_empty_version_rejected() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let result = create_release(project_dir, "", "");
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_get_and_update_release() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_release(project_dir, "v0.2.0-alpha", "initial")?;
        let initial = get_release(path.clone())?;
        assert!(!initial.metadata.id.is_empty());
        assert_eq!(initial.metadata.version, "v0.2.0-alpha");
        update_release(path.clone(), "updated")?;
        let updated = get_release(path)?;
        assert_eq!(updated.body, "updated");
        assert!(updated.metadata.updated >= initial.metadata.updated);
        Ok(())
    }

    #[test]
    fn test_list_releases() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        create_release(project_dir.clone(), "v0.1.0-alpha", "")?;
        create_release(project_dir.clone(), "v0.2.0-alpha", "")?;
        let releases = list_releases(project_dir)?;
        assert_eq!(releases.len(), 2);
        let versions: Vec<&str> = releases.iter().map(|r| r.version.as_str()).collect();
        assert!(versions.contains(&"v0.1.0-alpha"));
        assert!(versions.contains(&"v0.2.0-alpha"));
        Ok(())
    }

    #[test]
    fn test_find_release_path_supports_upcoming_and_legacy_locations() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;

        // New layout: upcoming/
        let upcoming = create_release(project_dir.clone(), "v0.3.0-alpha", "")?;
        let upcoming_name = upcoming.file_name().unwrap().to_string_lossy().to_string();
        let resolved_upcoming = find_release_path(&project_dir, &upcoming_name)?;
        assert_eq!(resolved_upcoming, upcoming);

        // Legacy layout: top-level project/releases/
        let legacy_path = project::releases_dir(&project_dir).join("v0-0-9-alpha.md");
        fs::write(
            &legacy_path,
            "+++\nid = \"\"\nversion = \"v0.0.9-alpha\"\nstatus = \"shipped\"\ncreated = \"2026-01-01T00:00:00Z\"\nupdated = \"2026-01-01T00:00:00Z\"\nfeature_ids = []\nadr_ids = []\ntags = []\n+++\n\nlegacy\n",
        )?;
        let resolved_legacy = find_release_path(&project_dir, "v0-0-9-alpha.md")?;
        assert_eq!(resolved_legacy, legacy_path);
        Ok(())
    }

    #[test]
    fn test_release_collision_gets_suffix() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let p1 = create_release(project_dir.clone(), "v0.1.0-alpha", "")?;
        let p2 = create_release(project_dir.clone(), "v0.1.0-alpha", "")?;
        assert_ne!(p1, p2);
        assert!(p1.exists());
        assert!(p2.exists());
        Ok(())
    }

    // ── Feature tests ───────────────────────────────────────────────────────────

    #[test]
    fn test_create_feature() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_feature(
            project_dir,
            "Agent Config",
            "",
            Some("v0.1.0-alpha.md"),
            Some("agent-config.md"),
            None,
        )?;
        assert!(path.exists());
        let feature = get_feature(path.clone())?;
        assert_eq!(feature.metadata.title, "Agent Config");
        assert!(path.to_string_lossy().contains("/planned/"));
        assert_eq!(
            feature.metadata.release_id,
            Some("v0.1.0-alpha.md".to_string())
        );
        assert_eq!(
            feature.metadata.spec_id,
            Some("agent-config.md".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_create_feature_empty_title_rejected() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let result = create_feature(project_dir, "", "", None, None, None);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_get_and_update_feature() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_feature(project_dir, "UI Agent Panel", "initial", None, None, None)?;
        let initial = get_feature(path.clone())?;
        assert!(!initial.metadata.id.is_empty());
        update_feature(path.clone(), "updated")?;
        let updated = get_feature(path)?;
        assert_eq!(updated.body, "updated");
        assert!(updated.metadata.updated >= initial.metadata.updated);
        Ok(())
    }

    #[test]
    fn test_list_features() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        create_feature(project_dir.clone(), "Feature One", "", None, None, None)?;
        create_feature(project_dir.clone(), "Feature Two", "", None, None, None)?;
        let features = list_features(project_dir, None)?;
        assert_eq!(features.len(), 2);
        let titles: Vec<&str> = features.iter().map(|f| f.title.as_str()).collect();
        assert!(titles.contains(&"Feature One"));
        assert!(titles.contains(&"Feature Two"));
        Ok(())
    }

    #[test]
    fn test_feature_collision_gets_suffix() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let p1 = create_feature(project_dir.clone(), "Ship Agents", "", None, None, None)?;
        let p2 = create_feature(project_dir.clone(), "Ship Agents!", "", None, None, None)?;
        assert_ne!(p1, p2);
        assert!(p1.exists());
        assert!(p2.exists());
        Ok(())
    }

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
    fn test_remove_status_blocked_by_issues() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        add_status(Some(project_dir.clone()), "review")?;
        create_issue(project_dir.clone(), "Stuck Issue", "desc", "review")?;
        let result = remove_status(Some(project_dir), "review");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("1 issue(s)"));
        Ok(())
    }

    #[test]
    fn test_git_config_roundtrip() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        set_category_committed(&project_dir, "issues", true)?;
        set_category_committed(&project_dir, "events.ndjson", false)?;
        let git = get_git_config(&project_dir)?;
        assert!(is_category_committed(&git, "issues"));
        assert!(!is_category_committed(&git, "events.ndjson"));
        Ok(())
    }

    #[test]
    fn test_agent_layer_roundtrip() -> anyhow::Result<()> {
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
        assert_eq!(loaded.agent.skills.len(), 2);
        assert_eq!(loaded.agent.prompts.len(), 1);
        assert_eq!(loaded.agent.context.len(), 2);
        Ok(())
    }

    #[test]
    fn test_generate_gitignore() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let gitignore = fs::read_to_string(project_dir.join(".gitignore"))?;
        // Default config: issues/events stay local; features/specs/adrs/releases committed.
        assert!(gitignore.contains("workflow/issues"));
        assert!(gitignore.contains("events.ndjson"));
        assert!(gitignore.contains("generated/"));
        // DB is now at ~/.ship/state/<slug>/ship.db — not inside .ship/
        assert!(!gitignore.contains("ship.db"));
        assert!(!gitignore.contains("log.md"));
        assert!(!gitignore.contains("project/releases"));
        assert!(!gitignore.contains("project/features"));
        assert!(!gitignore.contains("workflow/specs"));
        assert!(!gitignore.contains("project/adrs"));
        assert!(!gitignore.contains("project/notes"));
        assert!(!gitignore.contains("agents"));
        Ok(())
    }

    // ── Log tests ───────────────────────────────────────────────────────────────

    #[test]
    fn test_log_action() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        log_action(project_dir.clone(), "test", "details")?;
        let entries = read_log_entries(project_dir.clone())?;
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
        log_action_by(project_dir.clone(), "agent", "create", "issue-abc.md")?;
        let entries = read_log_entries(project_dir)?;
        assert_eq!(entries[0].actor, "agent");
        assert_eq!(entries[0].action, "create");
        assert_eq!(entries[0].details, "issue-abc.md");
        Ok(())
    }

    #[test]
    fn test_read_log_entries() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        log_action(project_dir.clone(), "create", "first entry")?;
        log_action(project_dir.clone(), "update", "second entry")?;
        let entries = read_log_entries(project_dir)?;
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
        // workflow/ namespace
        assert!(ship_path.join("workflow/issues/backlog").is_dir());
        assert!(ship_path.join("workflow/issues/in-progress").is_dir());
        assert!(ship_path.join("workflow/specs").is_dir());
        assert!(ship_path.join("project/features").is_dir());
        // project/ namespace
        assert!(ship_path.join("project/releases").is_dir());
        assert!(ship_path.join("project/adrs").is_dir());
        assert!(ship_path.join("project/notes").is_dir());
        assert!(ship_path.join("project/vision.md").is_file());
        // agents/ namespace
        assert!(ship_path.join("agents/modes").is_dir());
        assert!(ship_path.join("agents/skills").is_dir());
        assert!(ship_path.join("agents/prompts").is_dir());
        assert!(ship_path.join("generated").is_dir());
        // shared
        assert!(ship_path.join("project/releases/TEMPLATE.md").is_file());
        assert!(ship_path.join("project/features/TEMPLATE.md").is_file());
        assert!(ship_path.join("project/TEMPLATE.md").is_file());
        assert!(ship_path.join("project/notes/TEMPLATE.md").is_file());
        assert!(ship_path.join("README.md").is_file());
        assert!(ship_path.join("project/README.md").is_file());
        assert!(ship_path.join("workflow/README.md").is_file());
        assert!(ship_path.join("agents/modes/planning.toml").is_file());
        assert!(ship_path.join("agents/modes/execution.toml").is_file());
        assert!(ship_path.join("events.ndjson").is_file());
        assert!(ship_path.join("ship.toml").is_file());
        // default skill seeded
        assert!(
            ship_path
                .join("agents/skills/task-policy/index.md")
                .is_file()
        );
        assert!(
            ship_path
                .join("agents/skills/task-policy/skill.toml")
                .is_file()
        );
        let skill_content =
            fs::read_to_string(ship_path.join("agents/skills/task-policy/index.md"))?;
        assert!(skill_content.contains("task-policy"));
        assert!(skill_content.contains("Shipwright Workflow Policy"));
        Ok(())
    }

    #[test]
    fn test_init_project_idempotent() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        // First init
        let ship_path = init_project(tmp.path().to_path_buf())?;
        // Write a custom skill so we can verify it isn't clobbered
        let custom_skill = ship_path.join("agents/skills/custom.md");
        fs::write(
            &custom_skill,
            "+++\nid = \"custom\"\nname = \"Custom\"\n+++\nmy content",
        )?;
        // Second init on the same directory
        let ship_path2 = init_project(tmp.path().to_path_buf())?;
        assert_eq!(ship_path, ship_path2);
        // Custom skill must still be present and unchanged
        assert!(custom_skill.exists());
        assert_eq!(
            fs::read_to_string(&custom_skill)?,
            "+++\nid = \"custom\"\nname = \"Custom\"\n+++\nmy content"
        );
        // Default skill still present
        assert!(
            ship_path
                .join("agents/skills/task-policy/index.md")
                .is_file()
        );
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

    #[test]
    fn test_project_notes_round_trip() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;

        let project_note = create_note(
            NoteScope::Project,
            Some(project_dir.clone()),
            "Project Summary",
            "Project content",
        )?;
        assert!(project_note.starts_with(project::notes_dir(&project_dir)));

        let project_notes = list_notes(NoteScope::Project, Some(project_dir.clone()))?;
        assert!(!project_notes.is_empty());
        Ok(())
    }

    #[test]
    fn test_plugin_namespace_registration() -> anyhow::Result<()> {
        struct DemoPlugin;

        impl Plugin for DemoPlugin {
            fn name(&self) -> &str {
                "demo-plugin"
            }

            fn description(&self) -> &str {
                "Demo plugin for namespace registration tests"
            }
        }

        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        let mut registry = PluginRegistry::new();
        registry.register_with_project(&ship_path, Box::new(DemoPlugin))?;

        let namespaces = list_registered_namespaces(&ship_path)?;
        assert!(namespaces.iter().any(|ns| ns.id == "plugin:demo-plugin"));
        assert!(ship_path.join("demo-plugin").is_dir());
        Ok(())
    }

    #[test]
    fn test_event_stream_since() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;
        let seq0 = latest_event_seq(&ship_path)?;
        create_release(ship_path.clone(), "v0.1.0-alpha", "")?;
        create_feature(
            ship_path.clone(),
            "Event Stream Feature",
            "",
            None,
            None,
            None,
        )?;
        let events = list_events_since(&ship_path, seq0, None)?;
        assert!(events.len() >= 2);
        assert!(events.iter().all(|e| e.seq > seq0));
        assert!(events.windows(2).all(|w| w[0].seq < w[1].seq));
        Ok(())
    }

    #[test]
    fn test_ingest_external_events_detects_filesystem_changes() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_project(tmp.path().to_path_buf())?;

        // Ensure snapshot is synced to current state.
        let baseline = ingest_external_events(&ship_path)?;
        assert!(baseline.is_empty());

        let manual = crate::project::features_dir(&ship_path).join("manual-sync.md");
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
    fn test_init_demo_project_seeds_alpha_primitives() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let ship_path = init_demo_project(tmp.path().to_path_buf())?;
        let releases = list_releases(ship_path.clone())?;
        let features = list_features(ship_path.clone(), None)?;
        let specs = list_specs(ship_path)?;
        assert!(!releases.is_empty());
        assert!(!features.is_empty());
        assert!(!specs.is_empty()); // vision.md moved to project/ namespace
        Ok(())
    }

    #[test]
    fn test_legacy_migration() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_path = tmp.path().join(".project");
        fs::create_dir_all(&project_path)?;
        let project_dir = get_project_dir(Some(tmp.path().to_path_buf()))?;
        assert!(!project_path.exists());
        assert_eq!(project_dir, tmp.path().join(".ship"));
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
