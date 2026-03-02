use super::{
    db::{
        adr_to_db_row, delete_adr_db, get_adr_db, list_adrs_db, update_adr_status_db, upsert_adr_db,
    },
    export::status_dir_name,
    types::{ADR, AdrEntry, AdrMetadata, AdrStatus},
};
use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use std::path::{Path, PathBuf};

// ── File helpers ─────────────────────────────────────────────────────────────

fn adr_file_path(ship_dir: &Path, status: &AdrStatus, title: &str) -> PathBuf {
    let base = runtime::project::sanitize_file_name(title);
    let dir = runtime::project::adrs_dir(ship_dir).join(status_dir_name(status));
    std::fs::create_dir_all(&dir).ok();
    let candidate = dir.join(format!("{}.md", base));
    if !candidate.exists() {
        return candidate;
    }
    let mut n = 2u32;
    loop {
        let candidate = dir.join(format!("{}-{}.md", base, n));
        if !candidate.exists() {
            return candidate;
        }
        n += 1;
    }
}

fn write_adr_file(ship_dir: &Path, adr: &ADR, status: &AdrStatus) -> Result<PathBuf> {
    let path = adr_file_path(ship_dir, status, &adr.metadata.title);
    let content = adr.to_markdown()?;
    runtime::fs_util::write_atomic(&path, content)?;
    Ok(path)
}

fn remove_adr_files(ship_dir: &Path, id: &str, title: &str) {
    let base = runtime::project::sanitize_file_name(title);
    let adrs_dir = runtime::project::adrs_dir(ship_dir);
    for status_name in &[
        "proposed",
        "accepted",
        "rejected",
        "superseded",
        "deprecated",
    ] {
        let dir = adrs_dir.join(status_name);
        for suffix in &[format!("{}.md", base), format!("{}-2.md", base)] {
            let p = dir.join(suffix);
            if p.exists() {
                if let Ok(content) = std::fs::read_to_string(&p) {
                    if content.contains(id) {
                        std::fs::remove_file(&p).ok();
                        return;
                    }
                }
            }
        }
    }
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
            spec_id: None,
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
        .map(|row| row.into_entry())
        .ok_or_else(|| anyhow!("ADR not found: {}", id))
}

pub fn update_adr(ship_dir: &Path, id: &str, adr: ADR) -> Result<AdrEntry> {
    let existing_row = get_adr_db(ship_dir, id)?.ok_or_else(|| anyhow!("ADR not found: {}", id))?;
    let status_str = existing_row.status.clone();
    let adr_status = status_str.parse::<AdrStatus>().unwrap_or_default();
    let row = adr_to_db_row(&adr, &status_str, Some(&existing_row.created_at));
    upsert_adr_db(ship_dir, &row)?;

    write_adr_file(ship_dir, &adr, &adr_status)
        .with_context(|| "Failed to regenerate ADR file after update")?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Adr,
        runtime::EventAction::Update,
        id.to_string(),
        Some(format!("title={}", adr.metadata.title)),
    )?;

    Ok(get_adr_db(ship_dir, id)?.unwrap().into_entry())
}

pub fn move_adr(ship_dir: &Path, id: &str, new_status: AdrStatus) -> Result<AdrEntry> {
    let existing_row = get_adr_db(ship_dir, id)?.ok_or_else(|| anyhow!("ADR not found: {}", id))?;
    let _old_status: AdrStatus = existing_row.status.parse().unwrap_or_default();
    let new_status_str = status_dir_name(&new_status);
    update_adr_status_db(ship_dir, id, new_status_str)?;

    let entry = get_adr_db(ship_dir, id)?.unwrap().into_entry();

    // Always remove from old status dir (if it was there) and write to new one
    remove_adr_files(ship_dir, id, &entry.adr.metadata.title);
    write_adr_file(ship_dir, &entry.adr, &new_status)?;

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
    remove_adr_files(ship_dir, id, &title);
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
        .map(|r| r.into_entry())
        .collect())
}

pub fn find_adr_path(ship_dir: &Path, file_name: &str) -> Result<PathBuf> {
    let adrs_dir = runtime::project::adrs_dir(ship_dir);
    for status in &[
        "proposed",
        "accepted",
        "rejected",
        "superseded",
        "deprecated",
    ] {
        let candidate = adrs_dir.join(status).join(file_name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    let candidate = adrs_dir.join(file_name);
    if candidate.exists() {
        return Ok(candidate);
    }
    Err(anyhow!("ADR file not found: {}", file_name))
}
