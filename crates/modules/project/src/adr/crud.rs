use super::{
    db::{
        adr_to_db_row, delete_adr_db, get_adr_db, list_adrs_db, update_adr_status_db, upsert_adr_db,
    },
    export::status_dir_name,
    types::{ADR, AdrEntry, AdrMetadata, AdrStatus},
};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::path::{Path, PathBuf};

// ── File helpers ─────────────────────────────────────────────────────────────

fn adr_file_path(ship_dir: &Path, status: &AdrStatus, title: &str) -> PathBuf {
    let base = runtime::project::sanitize_file_name(title);
    let dir = runtime::project::adrs_dir(ship_dir).join(status_dir_name(status));
    dir.join(format!("{}.md", base))
}

fn write_adr_file(ship_dir: &Path, adr: &ADR, status: &AdrStatus) -> Result<PathBuf> {
    Ok(adr_file_path(ship_dir, status, &adr.metadata.title))
}

fn entry_with_projected_path(ship_dir: &Path, row: super::db::AdrDbRow) -> AdrEntry {
    let mut entry = row.into_entry();
    let path = adr_file_path(ship_dir, &entry.status, &entry.adr.metadata.title);
    entry.path = path.to_string_lossy().to_string();
    entry
}

// ── Public CRUD ──────────────────────────────────────────────────────────────

pub fn create_adr(
    ship_dir: &Path,
    title: &str,
    context: &str,
    decision: &str,
    status: &str,
) -> Result<AdrEntry> {
    if title.trim().is_empty() {
        return Err(anyhow!("ADR title cannot be empty"));
    }
    let adr_status = status.parse::<AdrStatus>().unwrap_or_default();
    let id = runtime::gen_nanoid();
    let now = Utc::now().to_rfc3339();
    let adr = ADR {
        metadata: AdrMetadata {
            id: id.clone(),
            title: title.to_string(),
            date: now.clone(),
            tags: vec![],
            supersedes_id: None,
        },
        context: context.to_string(),
        decision: decision.to_string(),
    };
    let status_str = status_dir_name(&adr_status);
    let row = adr_to_db_row(&adr, status_str, Some(&now));
    upsert_adr_db(ship_dir, &row)?;

    let file_path = write_adr_file(ship_dir, &adr, &adr_status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Adr,
        runtime::EventAction::Create,
        id.clone(),
        Some(format!("title={} status={}", title, status_str)),
    )?;

    Ok(AdrEntry {
        id,
        file_name: file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
        path: file_path.to_string_lossy().to_string(),
        status: adr_status,
        adr,
    })
}

pub fn get_adr_by_id(ship_dir: &Path, id: &str) -> Result<AdrEntry> {
    get_adr_db(ship_dir, id)?
        .map(|row| entry_with_projected_path(ship_dir, row))
        .ok_or_else(|| anyhow!("ADR not found: {}", id))
}

pub fn update_adr(ship_dir: &Path, id: &str, adr: ADR) -> Result<AdrEntry> {
    let existing_row = get_adr_db(ship_dir, id)?.ok_or_else(|| anyhow!("ADR not found: {}", id))?;
    let status_str = existing_row.status.clone();
    let row = adr_to_db_row(&adr, &status_str, Some(&existing_row.created_at));
    upsert_adr_db(ship_dir, &row)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Adr,
        runtime::EventAction::Update,
        id.to_string(),
        Some(format!("title={}", adr.metadata.title)),
    )?;

    Ok(entry_with_projected_path(
        ship_dir,
        get_adr_db(ship_dir, id)?.unwrap(),
    ))
}

pub fn move_adr(ship_dir: &Path, id: &str, new_status: AdrStatus) -> Result<AdrEntry> {
    let existing_row = get_adr_db(ship_dir, id)?.ok_or_else(|| anyhow!("ADR not found: {}", id))?;
    let _old_status: AdrStatus = existing_row.status.parse().unwrap_or_default();
    let new_status_str = status_dir_name(&new_status);
    update_adr_status_db(ship_dir, id, new_status_str)?;

    let entry = entry_with_projected_path(ship_dir, get_adr_db(ship_dir, id)?.unwrap());

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Adr,
        runtime::EventAction::Update,
        id.to_string(),
        Some(format!("status={}", new_status_str)),
    )?;

    Ok(entry)
}

pub fn delete_adr(ship_dir: &Path, id: &str) -> Result<()> {
    let row = get_adr_db(ship_dir, id)?.ok_or_else(|| anyhow!("ADR not found: {}", id))?;
    let title = row.title.clone();
    delete_adr_db(ship_dir, id)?;
    let _ = title;
    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Adr,
        runtime::EventAction::Delete,
        id.to_string(),
        None,
    )?;
    Ok(())
}

pub fn list_adrs(ship_dir: &Path) -> Result<Vec<AdrEntry>> {
    Ok(list_adrs_db(ship_dir)?
        .into_iter()
        .map(|r| entry_with_projected_path(ship_dir, r))
        .collect())
}

pub fn find_adr_path(ship_dir: &Path, file_name: &str) -> Result<PathBuf> {
    for entry in list_adrs(ship_dir)? {
        if entry.file_name.eq_ignore_ascii_case(file_name) {
            return Ok(PathBuf::from(entry.path));
        }
    }
    Err(anyhow!("ADR file not found: {}", file_name))
}
