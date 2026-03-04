use super::{OpsError, OpsResult, append_project_log};
use crate::{Release, ReleaseEntry};
use std::path::Path;

pub fn create_release(ship_dir: &Path, version: &str, body: &str) -> OpsResult<ReleaseEntry> {
    if version.trim().is_empty() {
        return Err(OpsError::Validation(
            "Release version cannot be empty".to_string(),
        ));
    }
    let entry = crate::release::create_release(ship_dir, version, body).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "release create",
        &format!("Created release: {}", entry.version),
    )?;
    Ok(entry)
}

pub fn list_releases(ship_dir: &Path) -> OpsResult<Vec<ReleaseEntry>> {
    crate::release::list_releases(ship_dir).map_err(OpsError::from)
}

pub fn get_release_by_id(ship_dir: &Path, id: &str) -> OpsResult<ReleaseEntry> {
    crate::release::get_release_by_id(ship_dir, id).map_err(OpsError::from)
}

pub fn update_release(ship_dir: &Path, id: &str, release: Release) -> OpsResult<ReleaseEntry> {
    let entry = crate::release::update_release(ship_dir, id, release).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "release update",
        &format!("Updated release: {}", entry.version),
    )?;
    Ok(entry)
}

pub fn update_release_content(ship_dir: &Path, id: &str, content: &str) -> OpsResult<ReleaseEntry> {
    let entry =
        crate::release::update_release_content(ship_dir, id, content).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "release update",
        &format!("Updated release: {}", entry.version),
    )?;
    Ok(entry)
}
