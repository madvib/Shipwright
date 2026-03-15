#[cfg(test)]
mod tests {
    use crate::{
        FeatureStatus, ReleaseStatus, create_feature, create_release, get_feature_by_id,
        get_feature_model, get_release_by_id, import_features_from_files,
        import_releases_from_files, init_demo_project, init_project, list_adrs, list_features,
        list_releases, move_feature, update_feature, update_feature_content, update_release,
        update_release_content,
    };
    use tempfile::tempdir;

    #[test]
    fn test_create_release_api() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_release(&project_dir, "v0.1.0-alpha", "")?;
        assert_eq!(entry.release.metadata.version, "v0.1.0-alpha");
        assert_eq!(entry.status, ReleaseStatus::Upcoming);

        let path = std::path::PathBuf::from(&entry.path);
        assert_eq!(
            path,
            runtime::project::releases_dir(&project_dir).join("v0.1.0-alpha.md")
        );
        assert!(
            !path.exists(),
            "release path should be projected, not written"
        );
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
        let reloaded = get_release_by_id(&project_dir, &entry.id)?;
        assert_eq!(reloaded.release.body, "updated");
        Ok(())
    }

    #[test]
    fn test_update_release_metadata_preserves_body() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let created = create_release(&project_dir, "v0.4.0-alpha", "release-body")?;

        let mut release = get_release_by_id(&project_dir, &created.id)?.release;
        release.metadata.target_date = Some("2026-03-31".to_string());
        // Simulate callers that hydrate from DB-only metadata and pass no body.
        release.body.clear();
        update_release(&project_dir, &created.id, release)?;

        let reloaded = get_release_by_id(&project_dir, &created.id)?;
        assert_eq!(reloaded.release.body, "release-body");
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
        assert_eq!(p1.id, p2.id);
        assert_eq!(p1.path, p2.path);
        assert!(!std::path::PathBuf::from(&p1.path).exists());
        assert!(!std::path::PathBuf::from(&p2.path).exists());
        assert_eq!(list_releases(&project_dir)?.len(), 1);
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
            None,
        )?;
        assert_eq!(entry.feature.metadata.title, "Agent Config");
        assert_eq!(entry.status, FeatureStatus::Planned);
        assert_eq!(
            entry.feature.metadata.release_id.as_deref(),
            Some("v0.1.0-alpha.md")
        );
        Ok(())
    }

    #[test]
    fn test_create_feature_empty_title_rejected() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let result = create_feature(&project_dir, "", "", None, None);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_get_and_update_feature() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_feature(&project_dir, "UI Agent Panel", "initial", None, None)?;
        let initial = get_feature_by_id(&project_dir, &entry.id)?;

        let updated = update_feature_content(&project_dir, &entry.id, "updated")?;
        assert_eq!(updated.feature.body, "updated");
        assert!(updated.feature.metadata.updated >= initial.feature.metadata.updated);
        Ok(())
    }

    #[test]
    fn test_update_feature_metadata_preserves_body() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let created = create_feature(
            &project_dir,
            "Feature Body Preserve",
            "feature-body",
            None,
            None,
        )?;

        let mut feature = get_feature_by_id(&project_dir, &created.id)?.feature;
        feature.metadata.description = Some("updated description".to_string());
        // Simulate callers that hydrate from DB-only metadata and pass no body.
        feature.body.clear();
        update_feature(&project_dir, &created.id, feature)?;

        let reloaded = get_feature_by_id(&project_dir, &created.id)?;
        assert_eq!(reloaded.feature.body, "feature-body");
        Ok(())
    }

    #[test]
    fn test_update_feature_content_rewrites_in_place_without_suffix_files() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_feature(
            &project_dir,
            "Pre-defined Agent Modes",
            "initial",
            None,
            None,
        )?;

        let canonical_path = runtime::project::features_dir(&project_dir)
            .join("planned")
            .join("pre-defined-agent-modes.md");
        let suffixed_path = runtime::project::features_dir(&project_dir)
            .join("planned")
            .join("pre-defined-agent-modes-2.md");
        assert_eq!(entry.path, canonical_path.to_string_lossy().to_string());

        update_feature_content(&project_dir, &entry.id, "updated")?;
        assert!(!suffixed_path.exists());
        assert!(!canonical_path.exists());
        let reloaded = get_feature_by_id(&project_dir, &entry.id)?;
        assert_eq!(reloaded.path, canonical_path.to_string_lossy().to_string());
        assert_eq!(reloaded.feature.body, "updated");
        Ok(())
    }

    #[test]
    fn test_update_feature_content_syncs_metadata_title_from_markdown_h1() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_feature(&project_dir, "Old Title", "initial", None, None)?;

        update_feature_content(
            &project_dir,
            &entry.id,
            "# New Title\n\n## Intent\n\nUpdated intent body.",
        )?;

        let reloaded = get_feature_by_id(&project_dir, &entry.id)?;
        assert_eq!(reloaded.feature.metadata.title, "New Title");
        assert!(reloaded.feature.body.starts_with("# New Title"));
        Ok(())
    }

    #[test]
    fn test_feature_model_computes_delta_from_declaration_and_status() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let entry = create_feature(
            &project_dir,
            "Feature Model Delta",
            r#"
## Declaration
Build a robust skill export path.

## Acceptance Criteria
- [ ] PASS: SKILL.md starts with YAML frontmatter FAIL: header missing
- [ ] Exported skill metadata is present

## Status
Codex path patched; full matrix pending.

## Status Checks
- [x] codex export test
- [ ] claude/gemini export tests
"#,
            None,
            None,
        )?;

        let initial_model = get_feature_model(&project_dir, &entry.id)?;
        assert!(initial_model.delta.drift_score > 0);
        assert_eq!(initial_model.delta.unmet_acceptance_criteria.len(), 2);
        assert_eq!(initial_model.delta.failing_checks.len(), 1);
        assert_eq!(initial_model.delta.missing_pass_fail_criteria.len(), 1);

        update_feature_content(
            &project_dir,
            &entry.id,
            r#"
## Declaration
Build a robust skill export path.

## Acceptance Criteria
- [x] PASS: SKILL.md starts with YAML frontmatter FAIL: header missing

## Status
All provider exports validated.

## Status Checks
- [x] codex export test
- [x] claude export test
- [x] gemini export test
"#,
        )?;

        let resolved_model = get_feature_model(&project_dir, &entry.id)?;
        assert_eq!(resolved_model.delta.drift_score, 0);
        Ok(())
    }

    #[test]
    fn test_move_feature_preserves_body() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let created = create_feature(&project_dir, "Move Preserve", "move-body", None, None)?;

        let moved = move_feature(&project_dir, &created.id, FeatureStatus::InProgress)?;
        let moved_path = runtime::project::features_dir(&project_dir)
            .join("in-progress")
            .join("move-preserve.md");
        assert_eq!(moved.path, moved_path.to_string_lossy().to_string());
        let reloaded = get_feature_by_id(&project_dir, &created.id)?;
        assert_eq!(reloaded.status, FeatureStatus::InProgress);
        assert_eq!(reloaded.feature.body, "move-body");
        Ok(())
    }

    #[test]
    fn test_list_features() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        create_feature(&project_dir, "Feature One", "", None, None)?;
        create_feature(&project_dir, "Feature Two", "", None, None)?;
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
        let p1 = create_feature(&project_dir, "Ship Agents", "", None, None)?;
        let p2 = create_feature(&project_dir, "Ship Agents!", "", None, None)?;
        assert_ne!(p1.id, p2.id);
        assert_eq!(p1.path, p2.path);
        assert!(!std::path::PathBuf::from(&p1.path).exists());
        assert!(!std::path::PathBuf::from(&p2.path).exists());
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
    fn test_init_demo_project_seeds_correctly() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_demo_project(tmp.path().to_path_buf())?;

        // Verify ADRs
        let adrs = list_adrs(&project_dir)?;
        assert!(
            adrs.iter()
                .any(|a| a.adr.metadata.title == "Use PostgreSQL as primary database")
        );

        Ok(())
    }
}
