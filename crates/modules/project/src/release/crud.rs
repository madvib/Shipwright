use super::db::{delete_release_db, get_release_db, list_releases_db, upsert_release_db};
use super::types::{Release, ReleaseEntry, ReleaseMetadata, ReleaseStatus};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::path::{Path, PathBuf};

// ── File helpers ─────────────────────────────────────────────────────────────

fn release_file_path(ship_dir: &Path, version: &str) -> PathBuf {
    let dir = runtime::project::releases_dir(ship_dir);
    std::fs::create_dir_all(&dir).ok();
    let candidate = dir.join(format!("{}.md", version));
    if !candidate.exists() {
        return candidate;
    }
    let mut n = 2u32;
    loop {
        let candidate = dir.join(format!("{}-{}.md", version, n));
        if !candidate.exists() {
            return candidate;
        }
        n += 1;
    }
}

fn release_update_path(ship_dir: &Path, version: &str) -> PathBuf {
    let releases_dir = runtime::project::releases_dir(ship_dir);
    let primary = releases_dir.join(format!("{}.md", version));
    if primary.exists() {
        return primary;
    }

    let upcoming_dir = runtime::project::upcoming_releases_dir(ship_dir);
    let legacy = upcoming_dir.join(format!("{}.md", version));
    if legacy.exists() {
        return legacy;
    }

    primary
}

fn find_release_file_by_id(ship_dir: &Path, id: &str) -> Option<PathBuf> {
    let releases_dir = runtime::project::releases_dir(ship_dir);
    if !releases_dir.exists() {
        return None;
    }
    let legacy_marker = format!("id = \"{}\"", id);
    let generated_marker = format!("ship:release id={}", id);
    for entry in std::fs::read_dir(&releases_dir).ok()?.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if content.contains(&legacy_marker) || content.contains(&generated_marker) {
            return Some(path);
        }
    }
    None
}

fn load_release_body_from_file(ship_dir: &Path, id: &str, version: &str) -> Option<String> {
    let path = find_release_file_by_id(ship_dir, id)
        .unwrap_or_else(|| release_update_path(ship_dir, version));
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    let release = Release::from_markdown(&content).ok()?;
    Some(release.body)
}

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

pub fn write_release_file(ship_dir: &Path, release: &Release) -> Result<PathBuf> {
    let path = release_file_path(ship_dir, &release.metadata.version);
    let content = release.to_markdown()?;
    runtime::fs_util::write_atomic(&path, content)?;
    Ok(path)
}

fn remove_release_files(ship_dir: &Path, version: &str) {
    let releases_dir = runtime::project::releases_dir(ship_dir);
    let upcoming_dir = runtime::project::upcoming_releases_dir(ship_dir);

    for dir in &[releases_dir, upcoming_dir] {
        if !dir.exists() {
            continue;
        }
        for suffix in &["", "-2", "-3", "-4", "-5"] {
            let file_name = if suffix.is_empty() {
                format!("{}.md", version)
            } else {
                format!("{}{}.md", version, suffix)
            };
            let p = dir.join(file_name);
            if p.exists() {
                if let Ok(content) = std::fs::read_to_string(&p) {
                    if content.contains(version) {
                        std::fs::remove_file(&p).ok();
                    }
                }
            }
        }
    }
}

// ── Public CRUD ──────────────────────────────────────────────────────────────

pub fn create_release(ship_dir: &Path, version: &str, body: &str) -> Result<ReleaseEntry> {
    if version.trim().is_empty() {
        return Err(anyhow!("Release version cannot be empty"));
    }
    let id = version.to_string(); // version is the ID for releases
    let now = Utc::now().to_rfc3339();

    let mut release = Release {
        metadata: ReleaseMetadata {
            id: id.clone(),
            version: version.to_string(),
            status: ReleaseStatus::Planned,
            created: now.clone(),
            updated: now,
            supported: None,
            target_date: None,
            tags: vec![],
        },
        body: body.to_string(),
        breaking_changes: vec![],
    };

    release.extract_breaking_changes();

    upsert_release_db(ship_dir, &release, &ReleaseStatus::Planned)?;
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
        status: ReleaseStatus::Planned,
        release,
    })
}

pub fn get_release_by_id(ship_dir: &Path, id: &str) -> Result<ReleaseEntry> {
    let resolved_id = require_release_id(ship_dir, id)?;
    let mut entry =
        get_release_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Release not found: {}", id))?;
    if entry.release.body.trim().is_empty()
        && let Some(body) = load_release_body_from_file(ship_dir, &resolved_id, &entry.version)
    {
        entry.release.body = body;
    }
    Ok(entry)
}

pub fn update_release(ship_dir: &Path, id: &str, mut release: Release) -> Result<ReleaseEntry> {
    let resolved_id = require_release_id(ship_dir, id)?;
    let existing = get_release_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Release not found: {}", id))?;
    release.metadata.updated = Utc::now().to_rfc3339();
    if release.body.trim().is_empty()
        && let Some(body) =
            load_release_body_from_file(ship_dir, &resolved_id, &release.metadata.version)
    {
        release.body = body;
    }

    upsert_release_db(ship_dir, &release, &existing.status)?;
    let path = release_update_path(ship_dir, &release.metadata.version);
    let content = release.to_markdown()?;
    runtime::fs_util::write_atomic(&path, content)?;

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
    let entry = get_release_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Release not found: {}", id))?;
    delete_release_db(ship_dir, &resolved_id)?;
    remove_release_files(ship_dir, &entry.version);

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
