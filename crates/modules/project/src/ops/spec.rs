use super::{OpsError, OpsResult, append_project_log};
use crate::{Spec, SpecEntry, SpecStatus};
use std::path::Path;

pub fn create_spec(
    ship_dir: &Path,
    title: &str,
    body: &str,
    feature_id: Option<String>,
    release_id: Option<String>,
) -> OpsResult<SpecEntry> {
    if title.trim().is_empty() {
        return Err(OpsError::Validation(
            "Spec title cannot be empty".to_string(),
        ));
    }
    let entry = crate::spec::create_spec(ship_dir, title, body, feature_id, release_id)
        .map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "spec create",
        &format!("Created spec: {}", entry.spec.metadata.title),
    )?;
    Ok(entry)
}

pub fn list_specs(ship_dir: &Path) -> OpsResult<Vec<SpecEntry>> {
    crate::spec::list_specs(ship_dir).map_err(OpsError::from)
}

pub fn get_spec_by_id(ship_dir: &Path, id: &str) -> OpsResult<SpecEntry> {
    crate::spec::get_spec_by_id(ship_dir, id).map_err(OpsError::from)
}

pub fn update_spec(ship_dir: &Path, id: &str, spec: Spec) -> OpsResult<SpecEntry> {
    if spec.metadata.title.trim().is_empty() {
        return Err(OpsError::Validation(
            "Spec title cannot be empty".to_string(),
        ));
    }
    let entry = crate::spec::update_spec(ship_dir, id, spec).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "spec update",
        &format!("Updated spec: {}", entry.spec.metadata.title),
    )?;
    Ok(entry)
}

pub fn move_spec(ship_dir: &Path, id: &str, status: SpecStatus) -> OpsResult<SpecEntry> {
    let entry = crate::spec::move_spec(ship_dir, id, status).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "spec move",
        &format!("Moved spec {} to {}", entry.id, entry.status),
    )?;
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_project;
    use runtime::read_log_entries;
    use tempfile::tempdir;

    #[test]
    fn create_spec_rejects_empty_title() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let err = create_spec(&project_dir, "   ", "", None, None)
            .expect_err("expected validation failure");
        assert!(matches!(err, OpsError::Validation(_)));
        Ok(())
    }

    #[test]
    fn move_spec_happy_path_writes_project_log() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;
        let created = create_spec(&project_dir, "ops-spec", "body", None, None)?;
        let moved = move_spec(&project_dir, &created.id, SpecStatus::Active)?;
        assert_eq!(moved.status, SpecStatus::Active);

        let logs = read_log_entries(&project_dir)?;
        assert!(
            logs.iter()
                .any(|entry| entry.action == "spec move" && entry.details.contains(&moved.id))
        );
        Ok(())
    }
}
