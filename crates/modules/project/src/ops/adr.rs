use super::{OpsError, OpsResult, append_project_log};
use crate::{ADR, AdrEntry, AdrStatus};
use std::path::{Path, PathBuf};

pub fn create_adr(
    ship_dir: &Path,
    title: &str,
    context: &str,
    decision: &str,
    status: &str,
) -> OpsResult<AdrEntry> {
    if title.trim().is_empty() {
        return Err(OpsError::Validation(
            "ADR title cannot be empty".to_string(),
        ));
    }
    if decision.trim().is_empty() {
        return Err(OpsError::Validation(
            "ADR decision cannot be empty".to_string(),
        ));
    }
    let entry = crate::adr::create_adr(ship_dir, title, context, decision, status)
        .map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "adr create",
        &format!("Created ADR: {}", entry.adr.metadata.title),
    )?;
    Ok(entry)
}

pub fn list_adrs(ship_dir: &Path) -> OpsResult<Vec<AdrEntry>> {
    crate::adr::list_adrs(ship_dir).map_err(OpsError::from)
}

pub fn get_adr_by_id(ship_dir: &Path, id: &str) -> OpsResult<AdrEntry> {
    crate::adr::get_adr_by_id(ship_dir, id).map_err(OpsError::from)
}

pub fn update_adr(ship_dir: &Path, id: &str, adr: ADR) -> OpsResult<AdrEntry> {
    let entry = crate::adr::update_adr(ship_dir, id, adr).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "adr update",
        &format!("Updated ADR: {}", entry.adr.metadata.title),
    )?;
    Ok(entry)
}

pub fn move_adr(ship_dir: &Path, id: &str, status: AdrStatus) -> OpsResult<AdrEntry> {
    let entry = crate::adr::move_adr(ship_dir, id, status).map_err(OpsError::from)?;
    append_project_log(
        ship_dir,
        "adr move",
        &format!("Moved ADR {} to {}", entry.id, entry.status),
    )?;
    Ok(entry)
}

pub fn find_adr_path(ship_dir: &Path, file_name: &str) -> OpsResult<PathBuf> {
    crate::adr::find_adr_path(ship_dir, file_name).map_err(OpsError::from)
}
