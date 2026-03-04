use super::{OpsError, OpsResult, append_project_log};
use crate::{Feature, FeatureEntry, FeatureStatus};
use std::path::Path;

pub fn create_feature(
    ship_dir: &Path,
    title: &str,
    body: &str,
    release_id: Option<&str>,
    spec_id: Option<&str>,
    branch: Option<&str>,
) -> OpsResult<FeatureEntry> {
    if title.trim().is_empty() {
        return Err(OpsError::Validation(
            "Feature title cannot be empty".to_string(),
        ));
    }

    let entry = crate::feature::create_feature(ship_dir, title, body, release_id, spec_id, branch)
        .map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "feature create",
        &format!(
            "Created feature: {} ({})",
            entry.feature.metadata.title, entry.id
        ),
    )?;
    Ok(entry)
}

pub fn list_features(ship_dir: &Path) -> OpsResult<Vec<FeatureEntry>> {
    crate::feature::list_features(ship_dir).map_err(OpsError::from)
}

pub fn get_feature_by_id(ship_dir: &Path, id: &str) -> OpsResult<FeatureEntry> {
    crate::feature::get_feature_by_id(ship_dir, id).map_err(OpsError::from)
}

pub fn update_feature(ship_dir: &Path, id: &str, feature: Feature) -> OpsResult<FeatureEntry> {
    if feature.metadata.title.trim().is_empty() {
        return Err(OpsError::Validation(
            "Feature title cannot be empty".to_string(),
        ));
    }
    let entry = crate::feature::update_feature(ship_dir, id, feature).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "feature update",
        &format!("Updated feature: {}", entry.id),
    )?;
    Ok(entry)
}

pub fn update_feature_content(ship_dir: &Path, id: &str, content: &str) -> OpsResult<FeatureEntry> {
    let entry =
        crate::feature::update_feature_content(ship_dir, id, content).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "feature update",
        &format!("Updated feature: {}", entry.id),
    )?;
    Ok(entry)
}

pub fn feature_start(ship_dir: &Path, id: &str) -> OpsResult<FeatureEntry> {
    let existing = crate::feature::get_feature_by_id(ship_dir, id).map_err(OpsError::from)?;
    if existing.status != FeatureStatus::Planned {
        return Err(OpsError::InvalidTransition(
            existing.status.to_string(),
            FeatureStatus::InProgress.to_string(),
        ));
    }
    let entry = crate::feature::feature_start(ship_dir, id).map_err(OpsError::from)?;
    append_project_log(ship_dir, "feature start", &format!("Started feature: {id}"))?;
    Ok(entry)
}

pub fn feature_done(ship_dir: &Path, id: &str) -> OpsResult<FeatureEntry> {
    let existing = crate::feature::get_feature_by_id(ship_dir, id).map_err(OpsError::from)?;
    if existing.status != FeatureStatus::InProgress {
        return Err(OpsError::InvalidTransition(
            existing.status.to_string(),
            FeatureStatus::Implemented.to_string(),
        ));
    }
    let branch_set = existing
        .feature
        .metadata
        .branch
        .as_ref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    if !branch_set {
        return Err(OpsError::Validation(
            "Feature must have a branch before marking done".to_string(),
        ));
    }
    let entry = crate::feature::feature_done(ship_dir, id).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "feature done",
        &format!("Completed feature: {}", entry.id),
    )?;
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{init_project, update_feature};
    use runtime::read_log_entries;
    use tempfile::tempdir;

    #[test]
    fn feature_start_rejects_invalid_transition() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let created = create_feature(
            &project_dir,
            "ops-start",
            "",
            None,
            None,
            Some("feature/ops"),
        )?;

        let mut raw = crate::get_feature_by_id(&project_dir, &created.id)?;
        raw.feature.metadata.branch = Some("feature/ops".to_string());
        update_feature(&project_dir, &created.id, raw.feature)?;
        crate::feature_done(&project_dir, &created.id)?;

        let err =
            feature_start(&project_dir, &created.id).expect_err("expected invalid transition");
        assert!(matches!(err, OpsError::InvalidTransition(_, _)));
        Ok(())
    }

    #[test]
    fn feature_done_requires_branch() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let created = create_feature(&project_dir, "ops-done", "", None, None, None)?;
        crate::feature_start(&project_dir, &created.id)?;

        let err = feature_done(&project_dir, &created.id).expect_err("expected validation failure");
        assert!(matches!(err, OpsError::Validation(_)));
        Ok(())
    }

    #[test]
    fn feature_done_happy_path_transitions_and_logs() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let created = create_feature(
            &project_dir,
            "ops-done-happy",
            "",
            None,
            None,
            Some("feature/ops-done-happy"),
        )?;

        let started = feature_start(&project_dir, &created.id)?;
        assert_eq!(started.status, FeatureStatus::InProgress);

        let done = feature_done(&project_dir, &created.id)?;
        assert_eq!(done.status, FeatureStatus::Implemented);

        let logs = read_log_entries(&project_dir)?;
        assert!(
            logs.iter()
                .any(|entry| entry.action == "feature done" && entry.details.contains(&done.id))
        );
        Ok(())
    }
}
