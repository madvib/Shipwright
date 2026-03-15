use super::db::{delete_release_db, get_release_db, list_releases_db, upsert_release_db};
use super::types::{Release, ReleaseEntry, ReleaseMetadata, ReleaseStatus};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::path::Path;

// ── Identity helpers ─────────────────────────────────────────────────────────

fn resolve_release_id(ship_dir: &Path, reference: &str) -> Result<Option<String>> {
    let reference = reference.trim();
    if reference.is_empty() {
        return Ok(None);
    }

    if let Some(entry) = get_release_db(ship_dir, reference)? {
        return Ok(Some(entry.id));
    }

    let without_ext = reference.trim_end_matches(".md");
    if without_ext != reference {
        if let Some(entry) = get_release_db(ship_dir, without_ext)? {
            return Ok(Some(entry.id));
        }
    }

    if let Some(file_name) = Path::new(reference)
        .file_name()
        .and_then(|name| name.to_str())
    {
        if file_name != reference {
            if let Some(entry) = get_release_db(ship_dir, file_name.trim_end_matches(".md"))? {
                return Ok(Some(entry.id));
            }
        }
    }

    let reference_file = if reference.ends_with(".md") {
        reference.to_string()
    } else {
        format!("{}.md", reference)
    };

    for entry in list_releases_db(ship_dir)? {
        let file_match = entry.file_name.eq_ignore_ascii_case(reference)
            || entry.file_name.eq_ignore_ascii_case(&reference_file);
        let id_match =
            entry.id.eq_ignore_ascii_case(reference) || entry.id.eq_ignore_ascii_case(without_ext);
        let version_match = entry.version.eq_ignore_ascii_case(reference)
            || entry.version.eq_ignore_ascii_case(without_ext);
        let dashed_version_match = entry
            .version
            .replace('.', "-")
            .eq_ignore_ascii_case(without_ext);
        if file_match || id_match || version_match || dashed_version_match {
            return Ok(Some(entry.id));
        }
    }

    Ok(None)
}

fn require_release_id(ship_dir: &Path, reference: &str) -> Result<String> {
    resolve_release_id(ship_dir, reference)?
        .ok_or_else(|| anyhow!("Release not found: {}", reference))
}

pub fn write_release_file(ship_dir: &Path, release: &Release) -> Result<std::path::PathBuf> {
    let path =
        runtime::project::releases_dir(ship_dir).join(format!("{}.md", release.metadata.version));
    Ok(path)
}

// ── Public CRUD ──────────────────────────────────────────────────────────────

pub fn create_release(ship_dir: &Path, version: &str, body: &str) -> Result<ReleaseEntry> {
    create_release_with_metadata(ship_dir, version, body, None, None, None, Vec::new())
}

pub fn create_release_with_metadata(
    ship_dir: &Path,
    version: &str,
    body: &str,
    status: Option<ReleaseStatus>,
    target_date: Option<String>,
    supported: Option<bool>,
    tags: Vec<String>,
) -> Result<ReleaseEntry> {
    if version.trim().is_empty() {
        return Err(anyhow!("Release version cannot be empty"));
    }
    let id = version.to_string(); // version is the ID for releases
    let now = Utc::now().to_rfc3339();
    let status = status.unwrap_or(ReleaseStatus::Upcoming);
    let target_date = target_date.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });
    let tags = tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();

    let mut release = Release {
        metadata: ReleaseMetadata {
            id: id.clone(),
            version: version.to_string(),
            status: status.clone(),
            created: now.clone(),
            updated: now,
            supported,
            target_date,
            tags,
        },
        body: body.to_string(),
        breaking_changes: vec![],
    };

    release.extract_breaking_changes();

    upsert_release_db(ship_dir, &release, &status)?;
    let file_path = write_release_file(ship_dir, &release)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Release,
        runtime::EventAction::Create,
        id.clone(),
        Some(format!("version={}", version)),
    )?;

    Ok(ReleaseEntry {
        id,
        file_name: file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
        path: file_path.to_string_lossy().to_string(),
        version: version.to_string(),
        status,
        release,
    })
}

pub fn get_release_by_id(ship_dir: &Path, id: &str) -> Result<ReleaseEntry> {
    let resolved_id = require_release_id(ship_dir, id)?;
    get_release_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Release not found: {}", id))
}

pub fn update_release(ship_dir: &Path, id: &str, mut release: Release) -> Result<ReleaseEntry> {
    let resolved_id = require_release_id(ship_dir, id)?;
    let existing = get_release_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Release not found: {}", id))?;
    release.metadata.updated = Utc::now().to_rfc3339();
    if release.body.trim().is_empty() {
        release.body = existing.release.body;
    }

    let persisted_status = release.metadata.status;
    upsert_release_db(ship_dir, &release, &persisted_status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Release,
        runtime::EventAction::Update,
        resolved_id.clone(),
        Some(format!("version={}", release.metadata.version)),
    )?;

    let mut entry = get_release_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Release not found after update"))?;
    entry.release.body = release.body;
    entry.status = persisted_status;
    entry.release.metadata.status = persisted_status;
    Ok(entry)
}

pub fn update_release_content(ship_dir: &Path, id: &str, content: &str) -> Result<ReleaseEntry> {
    let resolved_id = require_release_id(ship_dir, id)?;
    let mut entry = get_release_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Release not found: {}", id))?;
    entry.release.body = content.to_string();
    update_release(ship_dir, &resolved_id, entry.release)
}

pub fn delete_release(ship_dir: &Path, id: &str) -> Result<()> {
    let resolved_id = require_release_id(ship_dir, id)?;
    get_release_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Release not found: {}", id))?;
    delete_release_db(ship_dir, &resolved_id)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Release,
        runtime::EventAction::Delete,
        resolved_id,
        None,
    )?;
    Ok(())
}

pub fn list_releases(ship_dir: &Path) -> Result<Vec<ReleaseEntry>> {
    list_releases_db(ship_dir)
}
