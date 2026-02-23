pub mod adr;
pub mod config;
pub mod demo;
mod fs_util;
pub mod issue;
pub mod log;
pub mod plugin;
pub mod project;
pub mod spec;

pub use adr::{ADR, AdrEntry, AdrMetadata, create_adr, get_adr, list_adrs, update_adr};
pub use spec::{Spec, SpecEntry, SpecMetadata, create_spec, get_spec, get_spec_raw, list_specs, update_spec};
pub use issue::{
    Issue, IssueEntry, IssueLink, IssueMetadata, add_link, append_note, backfill_issue_ids,
    create_issue, delete_issue, get_issue, list_issues, list_issues_full, migrate_yaml_issues,
    move_issue, update_issue,
};
pub use demo::init_demo_project;
pub use log::{LogEntry, log_action, log_action_by, read_log, read_log_entries};
pub use plugin::{Plugin, PluginRegistry};
pub use config::{
    AiConfig, GitConfig, ProjectConfig, StatusConfig, add_status, generate_gitignore, get_config,
    get_git_config, get_project_statuses, is_category_committed, migrate_json_config_file,
    remove_status, save_config, set_category_committed, set_git_config,
};
pub use project::{
    DEFAULT_STATUSES, ISSUE_STATUSES, ProjectEntry, ProjectRegistry, SHIP_DIR_NAME, get_global_dir,
    get_project_dir, get_project_name, get_registry_path, init_project, list_registered_projects,
    load_registry, register_project, sanitize_file_name, save_registry, unregister_project,
};

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
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
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
        assert_eq!(issue.metadata.id.len(), 36, "UUID should be 36 chars");
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
        create_issue(project_dir.clone(), "Full Issue", "Detailed desc", "backlog")?;
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
        fs::write(project_dir.join("issues/backlog/legacy-issue.md"), legacy)?;
        let migrated = migrate_yaml_issues(&project_dir)?;
        assert_eq!(migrated, 1);
        let content = fs::read_to_string(project_dir.join("issues/backlog/legacy-issue.md"))?;
        assert!(content.starts_with("+++\n"));
        assert!(content.contains("title = \"Legacy Issue\""));
        Ok(())
    }

    // ── ADR tests ───────────────────────────────────────────────────────────────

    #[test]
    fn test_create_adr() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let adr_path = create_adr(project_dir, "Use PostgreSQL", "Chosen for robustness", "accepted")?;
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
        assert_eq!(adr.metadata.id.len(), 36);
        Ok(())
    }

    #[test]
    fn test_get_adr() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_adr(project_dir, "Use SQLite", "Embedded", "proposed")?;
        let adr = get_adr(path)?;
        assert_eq!(adr.metadata.title, "Use SQLite");
        assert_eq!(adr.metadata.status, "proposed");
        assert!(adr.body.contains("Embedded"));
        Ok(())
    }

    #[test]
    fn test_update_adr() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_adr(project_dir, "Update ADR", "original", "proposed")?;
        let mut adr = get_adr(path.clone())?;
        adr.metadata.status = "accepted".to_string();
        adr.body = "## Decision\n\nupdated body\n".to_string();
        update_adr(path.clone(), adr)?;
        let reloaded = get_adr(path)?;
        assert_eq!(reloaded.metadata.status, "accepted");
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
        let path = create_spec(project_dir, "Auth Flow", "")?;
        assert!(path.exists());
        let content = fs::read_to_string(&path)?;
        assert!(content.starts_with("+++\n"));
        assert!(content.contains("title = \"Auth Flow\""));
        assert!(content.contains("status = \"draft\""));
        Ok(())
    }

    #[test]
    fn test_create_spec_empty_title_rejected() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let result = create_spec(project_dir, "", "");
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_create_spec_has_uuid() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_spec(project_dir, "My Spec", "")?;
        let spec = get_spec(path)?;
        assert!(!spec.metadata.id.is_empty());
        assert_eq!(spec.metadata.id.len(), 36);
        Ok(())
    }

    #[test]
    fn test_get_spec() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_spec(project_dir, "Feature Spec", "## Overview\n\nCustom body.")?;
        let spec = get_spec(path)?;
        assert_eq!(spec.metadata.title, "Feature Spec");
        assert_eq!(spec.metadata.status, "draft");
        assert!(spec.body.contains("Custom body."));
        Ok(())
    }

    #[test]
    fn test_update_spec() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let path = create_spec(project_dir, "Spec Update", "original body")?;
        let original = get_spec(path.clone())?;
        update_spec(path.clone(), "updated body")?;
        let updated = get_spec(path)?;
        assert_eq!(updated.body, "updated body");
        assert!(updated.metadata.updated >= original.metadata.updated);
        Ok(())
    }

    #[test]
    fn test_list_specs() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        create_spec(project_dir.clone(), "Spec Alpha", "")?;
        create_spec(project_dir.clone(), "Spec Beta", "")?;
        let specs = list_specs(project_dir)?;
        assert_eq!(specs.len(), 2);
        let titles: Vec<&str> = specs.iter().map(|s| s.title.as_str()).collect();
        assert!(titles.contains(&"Spec Alpha"));
        assert!(titles.contains(&"Spec Beta"));
        Ok(())
    }

    #[test]
    fn test_spec_collision_gets_suffix() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let p1 = create_spec(project_dir.clone(), "Auth Flow", "")?;
        let p2 = create_spec(project_dir.clone(), "Auth Flow!", "")?;
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
        set_category_committed(&project_dir, "log.md", false)?;
        let git = get_git_config(&project_dir)?;
        assert!(is_category_committed(&git, "issues"));
        assert!(!is_category_committed(&git, "log.md"));
        Ok(())
    }

    #[test]
    fn test_generate_gitignore() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let gitignore = fs::read_to_string(project_dir.join(".gitignore"))?;
        // Default config: nothing in commit list, so everything is ignored
        assert!(gitignore.contains("issues"));
        assert!(gitignore.contains("log.md"));
        Ok(())
    }

    // ── Log tests ───────────────────────────────────────────────────────────────

    #[test]
    fn test_log_action() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        log_action(project_dir.clone(), "test", "details")?;
        let content = fs::read_to_string(project_dir.join("log.md"))?;
        assert!(content.contains("[ship] test: details"));
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
        assert!(ship_path.join("issues/backlog").is_dir());
        assert!(ship_path.join("issues/review").is_dir());
        assert!(ship_path.join("adrs").is_dir());
        assert!(ship_path.join("specs").is_dir());
        assert!(ship_path.join("templates").is_dir());
        assert!(ship_path.join("log.md").is_file());
        assert!(ship_path.join("config.toml").is_file());
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
        assert_eq!(sanitize_file_name("My Issue Title!"), "my-issue-title-");
        assert_eq!(
            sanitize_file_name("Already_Sanitized-123"),
            "already_sanitized-123"
        );
    }
}
