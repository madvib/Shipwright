use super::db::{delete_feature_db, get_feature_db, list_features_db, upsert_feature_db};
use super::types::{Feature, FeatureEntry, FeatureMetadata, FeatureStatus};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::path::{Path, PathBuf};

// ── File helpers ─────────────────────────────────────────────────────────────

fn feature_file_path(ship_dir: &Path, status: &FeatureStatus, title: &str) -> PathBuf {
    let base = runtime::project::sanitize_file_name(title);
    let dir = runtime::project::features_dir(ship_dir).join(status.to_string());
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

fn resolve_feature_id(ship_dir: &Path, reference: &str) -> Result<Option<String>> {
    let reference = reference.trim();
    if reference.is_empty() {
        return Ok(None);
    }

    if let Some(entry) = get_feature_db(ship_dir, reference)? {
        return Ok(Some(entry.id));
    }

    let without_ext = reference.trim_end_matches(".md");
    if without_ext != reference {
        if let Some(entry) = get_feature_db(ship_dir, without_ext)? {
            return Ok(Some(entry.id));
        }
    }

    let reference_file = if reference.ends_with(".md") {
        reference.to_string()
    } else {
        format!("{}.md", reference)
    };
    let reference_slug = runtime::project::sanitize_file_name(without_ext);

    for entry in list_features_db(ship_dir)? {
        let file_match = entry.file_name.eq_ignore_ascii_case(reference)
            || entry.file_name.eq_ignore_ascii_case(&reference_file);
        let slug_match = runtime::project::sanitize_file_name(&entry.feature.metadata.title)
            .eq_ignore_ascii_case(&reference_slug);
        if file_match || slug_match {
            return Ok(Some(entry.id));
        }
    }

    Ok(None)
}

fn require_feature_id(ship_dir: &Path, reference: &str) -> Result<String> {
    resolve_feature_id(ship_dir, reference)?
        .ok_or_else(|| anyhow!("Feature not found: {}", reference))
}

pub fn write_feature_file(
    ship_dir: &Path,
    feature: &Feature,
    status: &FeatureStatus,
) -> Result<PathBuf> {
    let path = feature_file_path(ship_dir, status, &feature.metadata.title);
    let content = feature.to_markdown()?;
    runtime::fs_util::write_atomic(&path, content)?;
    Ok(path)
}

pub fn remove_feature_files(ship_dir: &Path, id: &str, title: &str) {
    let base = runtime::project::sanitize_file_name(title);
    let features_dir = runtime::project::features_dir(ship_dir);

    // Check root and subdirs
    let mut scan_dirs = vec![features_dir.clone()];
    for status in &["planned", "in-progress", "implemented", "deprecated"] {
        scan_dirs.push(features_dir.join(status));
    }

    for dir in scan_dirs {
        if !dir.exists() {
            continue;
        }
        // Try exact match and common suffixes
        for suffix in &["", "-2", "-3", "-4", "-5"] {
            let file_name = if suffix.is_empty() {
                format!("{}.md", base)
            } else {
                format!("{}{}.md", base, suffix)
            };
            let p = dir.join(file_name);
            if p.exists() {
                if let Ok(content) = std::fs::read_to_string(&p) {
                    if content.contains(id) {
                        std::fs::remove_file(&p).ok();
                    }
                }
            }
        }
    }
}

// ── Public CRUD ──────────────────────────────────────────────────────────────

pub fn create_feature(
    ship_dir: &Path,
    title: &str,
    body: &str,
    release_id: Option<&str>,
    spec_id: Option<&str>,
    branch: Option<&str>,
) -> Result<FeatureEntry> {
    if title.trim().is_empty() {
        return Err(anyhow!("Feature title cannot be empty"));
    }
    let id = runtime::gen_nanoid();
    let now = Utc::now().to_rfc3339();

    let mut feature = Feature {
        metadata: FeatureMetadata {
            id: id.clone(),
            title: title.to_string(),
            description: None,
            created: now.clone(),
            updated: now,
            release_id: release_id.map(|s| s.to_string()),
            spec_id: spec_id.map(|s| s.to_string()),
            branch: branch.map(|s| s.to_string()),
            agent: None,
            tags: vec![],
        },
        body: body.to_string(),
        todos: vec![],
        criteria: vec![],
    };

    feature.extract_structured_data();

    let status = FeatureStatus::Planned;
    upsert_feature_db(ship_dir, &feature, &status)?;
    let file_path = write_feature_file(ship_dir, &feature, &status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Feature,
        runtime::EventAction::Create,
        id.clone(),
        Some(format!("title={}", title)),
    )?;

    Ok(FeatureEntry {
        id,
        file_name: file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
        path: file_path.to_string_lossy().to_string(),
        status,
        feature,
    })
}

pub fn get_feature_by_id(ship_dir: &Path, id: &str) -> Result<FeatureEntry> {
    let resolved_id = require_feature_id(ship_dir, id)?;
    get_feature_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Feature not found: {}", id))
}

pub fn update_feature(ship_dir: &Path, id: &str, mut feature: Feature) -> Result<FeatureEntry> {
    let resolved_id = require_feature_id(ship_dir, id)?;
    let existing = get_feature_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Feature not found: {}", id))?;
    feature.metadata.updated = Utc::now().to_rfc3339();

    upsert_feature_db(ship_dir, &feature, &existing.status)?;
    // Ensure updates replace the existing exported markdown file instead of
    // generating suffixed duplicates like `foo-2.md`.
    remove_feature_files(ship_dir, &resolved_id, &existing.feature.metadata.title);
    write_feature_file(ship_dir, &feature, &existing.status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Feature,
        runtime::EventAction::Update,
        resolved_id.clone(),
        Some(format!("title={}", feature.metadata.title)),
    )?;

    let mut entry = get_feature_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Feature not found after update"))?;
    entry.feature.body = feature.body;
    Ok(entry)
}

pub fn update_feature_content(ship_dir: &Path, id: &str, content: &str) -> Result<FeatureEntry> {
    let resolved_id = require_feature_id(ship_dir, id)?;
    let mut entry = get_feature_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Feature not found: {}", id))?;
    entry.feature.body = content.to_string();
    update_feature(ship_dir, &resolved_id, entry.feature)
}

pub fn move_feature(ship_dir: &Path, id: &str, new_status: FeatureStatus) -> Result<FeatureEntry> {
    let resolved_id = require_feature_id(ship_dir, id)?;
    let existing = get_feature_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Feature not found: {}", id))?;

    upsert_feature_db(ship_dir, &existing.feature, &new_status)?;
    remove_feature_files(ship_dir, &resolved_id, &existing.feature.metadata.title);
    write_feature_file(ship_dir, &existing.feature, &new_status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Feature,
        runtime::EventAction::Update,
        resolved_id.clone(),
        Some(format!("status={}", new_status)),
    )?;

    Ok(get_feature_db(ship_dir, &resolved_id)?.unwrap())
}

pub fn delete_feature(ship_dir: &Path, id: &str) -> Result<()> {
    let resolved_id = require_feature_id(ship_dir, id)?;
    let entry = get_feature_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Feature not found: {}", id))?;
    delete_feature_db(ship_dir, &resolved_id)?;
    remove_feature_files(ship_dir, &resolved_id, &entry.feature.metadata.title);

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Feature,
        runtime::EventAction::Delete,
        resolved_id,
        None,
    )?;
    Ok(())
}

pub fn feature_start(ship_dir: &Path, id: &str) -> Result<FeatureEntry> {
    move_feature(ship_dir, id, FeatureStatus::InProgress)
}

pub fn feature_done(ship_dir: &Path, id: &str) -> Result<FeatureEntry> {
    move_feature(ship_dir, id, FeatureStatus::Implemented)
}

pub fn list_features(ship_dir: &Path) -> Result<Vec<FeatureEntry>> {
    list_features_db(ship_dir)
}
