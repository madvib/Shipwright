use super::{OpsError, OpsResult, append_project_log};
use crate::{
    Feature, FeatureDocStatus, FeatureDocumentation, FeatureEntry, FeatureStatus,
    record_feature_session_update,
};
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

pub fn ensure_feature_documentation(ship_dir: &Path, id: &str) -> OpsResult<FeatureDocumentation> {
    crate::feature::ensure_feature_documentation(ship_dir, id).map_err(OpsError::from)
}

pub fn get_feature_documentation(ship_dir: &Path, id: &str) -> OpsResult<FeatureDocumentation> {
    crate::feature::get_feature_documentation(ship_dir, id).map_err(OpsError::from)
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

pub fn update_feature_documentation(
    ship_dir: &Path,
    id: &str,
    content: String,
    status: Option<FeatureDocStatus>,
    verify_now: bool,
    actor: Option<&str>,
) -> OpsResult<FeatureDocumentation> {
    crate::feature::update_feature_documentation(ship_dir, id, content, status, verify_now, actor)
        .map_err(OpsError::from)
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

    let docs = get_feature_documentation(ship_dir, &existing.id).map_err(OpsError::from)?;
    if docs.status == FeatureDocStatus::NotStarted {
        return Err(OpsError::Validation(
            "Feature documentation must be started before marking done".to_string(),
        ));
    }
    if docs.content.trim().is_empty() {
        return Err(OpsError::Validation(
            "Feature documentation cannot be empty before marking done".to_string(),
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

pub fn delete_feature(ship_dir: &Path, id: &str) -> OpsResult<()> {
    // Resolve first so logs include canonical ID.
    let existing = crate::feature::get_feature_by_id(ship_dir, id).map_err(OpsError::from)?;
    crate::feature::delete_feature(ship_dir, id).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "feature delete",
        &format!("Deleted feature: {}", existing.id),
    )?;
    Ok(())
}

pub fn sync_feature_docs_after_session(
    ship_dir: &Path,
    feature_ids: &[String],
    summary: Option<&str>,
) -> OpsResult<Vec<FeatureDocumentation>> {
    let mut updated = Vec::new();
    for feature_id in feature_ids {
        let feature_id = feature_id.trim();
        if feature_id.is_empty() {
            continue;
        }
        let doc = record_feature_session_update(ship_dir, feature_id, summary, Some("session"))
            .map_err(OpsError::from)?;
        updated.push(doc);
    }
    if !updated.is_empty() {
        append_project_log(
            ship_dir,
            "feature docs sync",
            &format!("Updated docs for {} feature(s)", updated.len()),
        )?;
    }
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FeatureDocStatus, init_project, update_feature};
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

    #[test]
    fn feature_create_scaffolds_documentation() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let created = create_feature(
            &project_dir,
            "docs-scaffold",
            "",
            None,
            None,
            Some("feature/docs-scaffold"),
        )?;

        let docs = get_feature_documentation(&project_dir, &created.id)?;
        assert_eq!(docs.feature_id, created.id);
        assert_eq!(docs.status, FeatureDocStatus::NotStarted);
        assert!(docs.content.contains("## Capability Summary"));
        Ok(())
    }

    #[test]
    fn sync_feature_docs_after_session_appends_summary() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let created = create_feature(
            &project_dir,
            "session-doc-sync",
            "",
            None,
            None,
            Some("feature/session-doc-sync"),
        )?;

        let updated = sync_feature_docs_after_session(
            &project_dir,
            std::slice::from_ref(&created.id),
            Some("Implemented auth guard for token refresh."),
        )?;
        assert_eq!(updated.len(), 1);
        assert!(
            updated[0]
                .content
                .contains("Implemented auth guard for token refresh.")
        );
        assert_eq!(updated[0].status, FeatureDocStatus::Draft);
        Ok(())
    }

    #[test]
    fn feature_done_requires_started_documentation() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let created = create_feature(
            &project_dir,
            "done-doc-gate",
            "",
            None,
            None,
            Some("feature/done-doc-gate"),
        )?;
        let _ = feature_start(&project_dir, &created.id)?;

        let docs = get_feature_documentation(&project_dir, &created.id)?;
        let _ = update_feature_documentation(
            &project_dir,
            &created.id,
            docs.content,
            Some(FeatureDocStatus::NotStarted),
            false,
            Some("test"),
        )?;

        let err = feature_done(&project_dir, &created.id).expect_err("expected docs gate");
        assert!(matches!(err, OpsError::Validation(_)));
        Ok(())
    }
}
