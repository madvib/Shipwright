#[cfg(test)]
mod tests {
    use crate::{
        FeatureStatus, IssueStatus, ReleaseStatus, SpecStatus, create_feature, create_issue,
        create_release, create_spec, delete_issue, delete_spec, get_feature_by_id, get_issue_by_id,
        get_release_by_id, get_spec_by_id, import_features_from_files, import_releases_from_files,
        init_demo_project, init_project, list_adrs, list_features, list_issues, list_releases,
        list_specs, move_issue, move_spec, update_feature_content, update_issue,
        update_release_content, update_spec,
    };
    use tempfile::tempdir;

    #[test]
    fn test_create_release_api() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_release(&project_dir, "v0.1.0-alpha", "")?;
        assert_eq!(entry.release.metadata.version, "v0.1.0-alpha");
        assert_eq!(entry.status, ReleaseStatus::Planned);

        let path = std::path::PathBuf::from(&entry.path);
        assert!(path.exists());
        let content = std::fs::read_to_string(&path)?;
        assert!(content.contains("version = \"v0.1.0-alpha\""));
        Ok(())
    }

    #[test]
    fn test_create_release_empty_version_rejected() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let result = create_release(&project_dir, "", "");
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_get_and_update_release() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_release(&project_dir, "v0.2.0-alpha", "initial")?;
        let initial = get_release_by_id(&project_dir, &entry.id)?;
        assert_eq!(initial.release.metadata.version, "v0.2.0-alpha");

        let updated = update_release_content(&project_dir, &entry.id, "updated")?;
        assert_eq!(updated.release.body, "updated");
        assert!(updated.release.metadata.updated >= initial.release.metadata.updated);
        Ok(())
    }

    #[test]
    fn test_list_releases() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        create_release(&project_dir, "v0.1.0-alpha", "")?;
        create_release(&project_dir, "v0.2.0-alpha", "")?;
        let releases = list_releases(&project_dir)?;
        assert_eq!(releases.len(), 2);
        let versions: Vec<&str> = releases
            .iter()
            .map(|r| r.release.metadata.version.as_str())
            .collect();
        assert!(versions.contains(&"v0.1.0-alpha"));
        assert!(versions.contains(&"v0.2.0-alpha"));
        Ok(())
    }

    #[test]
    fn test_release_collision_gets_suffix() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let p1 = create_release(&project_dir, "v0.1.0-tmp", "")?;
        let p2 = create_release(&project_dir, "v0.1.0-tmp", "")?;
        assert_ne!(p1.path, p2.path);
        assert!(std::path::PathBuf::from(&p1.path).exists());
        assert!(std::path::PathBuf::from(&p2.path).exists());
        Ok(())
    }

    #[test]
    fn test_create_feature_api() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_feature(
            &project_dir,
            "Agent Config",
            "",
            Some("v0.1.0-alpha.md"),
            Some("agent-config.md"),
            None,
        )?;
        assert_eq!(entry.feature.metadata.title, "Agent Config");
        assert_eq!(entry.status, FeatureStatus::Planned);
        assert_eq!(
            entry.feature.metadata.release_id.as_deref(),
            Some("v0.1.0-alpha.md")
        );
        assert_eq!(
            entry.feature.metadata.spec_id.as_deref(),
            Some("agent-config.md")
        );
        Ok(())
    }

    #[test]
    fn test_create_feature_empty_title_rejected() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let result = create_feature(&project_dir, "", "", None, None, None);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_get_and_update_feature() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_feature(&project_dir, "UI Agent Panel", "initial", None, None, None)?;
        let initial = get_feature_by_id(&project_dir, &entry.id)?;

        let updated = update_feature_content(&project_dir, &entry.id, "updated")?;
        assert_eq!(updated.feature.body, "updated");
        assert!(updated.feature.metadata.updated >= initial.feature.metadata.updated);
        Ok(())
    }

    #[test]
    fn test_list_features() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        create_feature(&project_dir, "Feature One", "", None, None, None)?;
        create_feature(&project_dir, "Feature Two", "", None, None, None)?;
        let features = list_features(&project_dir)?;
        assert_eq!(features.len(), 2);
        let titles: Vec<&str> = features
            .iter()
            .map(|f| f.feature.metadata.title.as_str())
            .collect();
        assert!(titles.contains(&"Feature One"));
        assert!(titles.contains(&"Feature Two"));
        Ok(())
    }

    #[test]
    fn test_feature_collision_gets_suffix() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let p1 = create_feature(&project_dir, "Ship Agents", "", None, None, None)?;
        let p2 = create_feature(&project_dir, "Ship Agents!", "", None, None, None)?;
        assert_ne!(p1.path, p2.path);
        assert!(std::path::PathBuf::from(&p1.path).exists());
        assert!(std::path::PathBuf::from(&p2.path).exists());
        Ok(())
    }

    #[test]
    fn test_import_release_supports_upcoming_and_legacy_locations() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;

        // New layout: upcoming/
        let upcoming_path =
            runtime::project::upcoming_releases_dir(&project_dir).join("v0-3-0-alpha.md");
        std::fs::create_dir_all(upcoming_path.parent().unwrap())?;
        std::fs::write(
            &upcoming_path,
            "+++\nid = \"v0.3.0-alpha\"\nversion = \"v0.3.0-alpha\"\nstatus = \"planned\"\ncreated = \"2026-01-01T00:00:00Z\"\nupdated = \"2026-01-01T00:00:00Z\"\nfeature_ids = []\nadr_ids = []\nbreaking_changes = []\ntags = []\n+++\n\nnew\n",
        )?;

        // Legacy layout: top-level project/releases/
        let legacy_path = runtime::project::releases_dir(&project_dir).join("v0-0-9-alpha.md");
        std::fs::create_dir_all(legacy_path.parent().unwrap())?;
        std::fs::write(
            &legacy_path,
            "+++\nid = \"v0.0.9-alpha\"\nversion = \"v0.0.9-alpha\"\nstatus = \"shipped\"\ncreated = \"2026-01-01T00:00:00Z\"\nupdated = \"2026-01-01T00:00:00Z\"\nfeature_ids = []\nadr_ids = []\nbreaking_changes = []\ntags = []\n+++\n\nlegacy\n",
        )?;

        import_releases_from_files(&project_dir)?;

        let releases = list_releases(&project_dir)?;
        let versions: Vec<&str> = releases
            .iter()
            .map(|r| r.release.metadata.version.as_str())
            .collect();
        assert!(versions.contains(&"v0.3.0-alpha"));
        assert!(versions.contains(&"v0.0.9-alpha"));
        Ok(())
    }

    #[test]
    fn test_import_features_from_files_is_idempotent() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;

        let feature_path =
            runtime::project::features_dir(&project_dir).join("planned/import-idempotent.md");
        std::fs::create_dir_all(feature_path.parent().unwrap())?;
        std::fs::write(
            &feature_path,
            "+++\nid = \"feature-import-idempotent\"\ntitle = \"Import Idempotent\"\ncreated = \"2026-01-01T00:00:00Z\"\nupdated = \"2026-01-01T00:00:00Z\"\ntags = []\n+++\n\nbody\n",
        )?;

        let first = import_features_from_files(&project_dir)?;
        let second = import_features_from_files(&project_dir)?;
        assert_eq!(first, 1);
        assert_eq!(second, 0);
        assert_eq!(list_features(&project_dir)?.len(), 1);
        Ok(())
    }

    #[test]
    fn test_import_releases_from_files_is_idempotent() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;

        let release_path = runtime::project::releases_dir(&project_dir).join("v0.4.0-alpha.md");
        std::fs::create_dir_all(release_path.parent().unwrap())?;
        std::fs::write(
            &release_path,
            "+++\nid = \"v0.4.0-alpha\"\nversion = \"v0.4.0-alpha\"\nstatus = \"planned\"\ncreated = \"2026-01-01T00:00:00Z\"\nupdated = \"2026-01-01T00:00:00Z\"\ntags = []\n+++\n\nbody\n",
        )?;

        let first = import_releases_from_files(&project_dir)?;
        let second = import_releases_from_files(&project_dir)?;
        assert_eq!(first, 1);
        assert_eq!(second, 0);
        assert_eq!(list_releases(&project_dir)?.len(), 1);
        Ok(())
    }

    #[test]
    fn test_create_issue_api() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_issue(
            &project_dir,
            "Fix login bug",
            "Broken for SSO",
            IssueStatus::Backlog,
            None,
            None,
            None,
            None,
        )?;
        assert_eq!(entry.issue.metadata.title, "Fix login bug");
        assert_eq!(entry.status, IssueStatus::Backlog);
        assert!(!std::path::PathBuf::from(&entry.path).exists());
        let fetched = get_issue_by_id(&project_dir, &entry.id)?;
        assert_eq!(fetched.issue.description, "Broken for SSO");
        Ok(())
    }

    #[test]
    fn test_create_spec_api() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_spec(&project_dir, "Auth Spec", "Spec content", None, None)?;
        assert_eq!(entry.spec.metadata.title, "Auth Spec");
        assert_eq!(entry.status, SpecStatus::Draft);
        assert!(!std::path::PathBuf::from(&entry.path).exists());
        let fetched = get_spec_by_id(&project_dir, &entry.id)?;
        assert_eq!(fetched.spec.body, "Spec content");
        Ok(())
    }

    #[test]
    fn test_create_issue_collision_gets_suffix() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let p1 = create_issue(
            &project_dir,
            "Fix Bug",
            "a",
            IssueStatus::Backlog,
            None,
            None,
            None,
            None,
        )?;
        let p2 = create_issue(
            &project_dir,
            "Fix Bug!",
            "b",
            IssueStatus::Backlog,
            None,
            None,
            None,
            None,
        )?;
        assert_ne!(p1.path, p2.path);
        assert!(!std::path::PathBuf::from(&p1.path).exists());
        assert!(!std::path::PathBuf::from(&p2.path).exists());
        Ok(())
    }

    #[test]
    fn test_list_issues_full() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        create_issue(
            &project_dir,
            "Full Issue",
            "Detailed desc",
            IssueStatus::Backlog,
            None,
            None,
            None,
            None,
        )?;
        let entries = list_issues(&project_dir)?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].issue.metadata.title, "Full Issue");
        assert_eq!(entries[0].issue.description, "Detailed desc");
        Ok(())
    }

    #[test]
    fn test_get_and_update_issue() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_issue(
            &project_dir,
            "Update Me",
            "original",
            IssueStatus::Backlog,
            None,
            None,
            None,
            None,
        )?;
        let initial = get_issue_by_id(&project_dir, &entry.id)?;

        let mut issue = initial.issue.clone();
        issue.description = "updated".to_string();
        let updated = update_issue(&project_dir, &entry.id, issue)?;
        assert_eq!(updated.issue.description, "updated");
        assert!(updated.issue.metadata.updated >= initial.issue.metadata.updated);
        Ok(())
    }

    #[test]
    fn test_move_issue_api() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_issue(
            &project_dir,
            "Test Issue",
            "Desc",
            IssueStatus::Backlog,
            None,
            None,
            None,
            None,
        )?;
        let moved = move_issue(&project_dir, &entry.id, IssueStatus::InProgress)?;
        assert!(moved.path.contains("in-progress"));
        assert_eq!(moved.status, IssueStatus::InProgress);
        Ok(())
    }

    #[test]
    fn test_delete_issue_api() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_issue(
            &project_dir,
            "Delete Me",
            "bye",
            IssueStatus::Backlog,
            None,
            None,
            None,
            None,
        )?;
        assert!(!std::path::PathBuf::from(&entry.path).exists());
        delete_issue(&project_dir, &entry.id)?;
        assert!(get_issue_by_id(&project_dir, &entry.id).is_err());
        Ok(())
    }

    #[test]
    fn test_get_and_update_spec() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_spec(&project_dir, "Spec Update", "original body", None, None)?;
        let initial = get_spec_by_id(&project_dir, &entry.id)?;

        let mut spec = initial.spec.clone();
        spec.body = "updated body".to_string();
        let updated = update_spec(&project_dir, &entry.id, spec)?;
        assert_eq!(updated.spec.body, "updated body");
        assert!(updated.spec.metadata.updated >= initial.spec.metadata.updated);
        Ok(())
    }

    #[test]
    fn test_move_spec_api() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_spec(&project_dir, "Move Spec", "content", None, None)?;
        let moved = move_spec(&project_dir, &entry.id, SpecStatus::Active)?;
        assert!(moved.path.contains("active"));
        assert_eq!(moved.status, SpecStatus::Active);
        Ok(())
    }

    #[test]
    fn test_delete_spec_api() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_spec(&project_dir, "Delete Spec", "content", None, None)?;
        assert!(!std::path::PathBuf::from(&entry.path).exists());
        delete_spec(&project_dir, &entry.id)?;
        assert!(get_spec_by_id(&project_dir, &entry.id).is_err());
        Ok(())
    }

    #[test]
    fn test_init_demo_project_seeds_correctly() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_demo_project(tmp.path().to_path_buf())?;

        // Verify issues
        let issues = list_issues(&project_dir)?;
        assert!(issues.len() >= 6);

        // Verify specs
        let specs = list_specs(&project_dir)?;
        assert!(
            specs
                .iter()
                .any(|s| s.spec.metadata.title == "Agent Configuration and Modes")
        );

        // Verify ADRs
        let adrs = list_adrs(&project_dir)?;
        assert!(
            adrs.iter()
                .any(|a| a.adr.metadata.title == "Use PostgreSQL as primary database")
        );

        Ok(())
    }
}
