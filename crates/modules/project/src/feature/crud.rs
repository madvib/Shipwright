use super::db::{
    delete_feature_db, get_feature_db, get_feature_doc_db, list_features_db, upsert_feature_db,
    upsert_feature_doc_db,
};
use super::types::{
    Feature, FeatureDocStatus, FeatureDocumentation, FeatureEntry, FeatureMetadata, FeatureStatus,
};
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

fn find_feature_file_by_id(ship_dir: &Path, id: &str) -> Option<PathBuf> {
    let features_dir = runtime::project::features_dir(ship_dir);
    let mut scan_dirs = vec![features_dir.clone()];
    for status in &["planned", "in-progress", "implemented", "deprecated"] {
        scan_dirs.push(features_dir.join(status));
    }

    for dir in scan_dirs {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
            if file_name == "README.md" || file_name == "TEMPLATE.md" {
                continue;
            }
            let content = match std::fs::read_to_string(&path) {
                Ok(content) => content,
                Err(_) => continue,
            };
            let legacy_marker = format!("id = \"{}\"", id);
            let generated_marker = format!("ship:feature id={}", id);
            if content.contains(&legacy_marker) || content.contains(&generated_marker) {
                return Some(path);
            }
        }
    }

    None
}

fn load_feature_body_from_file(ship_dir: &Path, id: &str) -> Option<String> {
    let path = find_feature_file_by_id(ship_dir, id)?;
    let content = std::fs::read_to_string(path).ok()?;
    let feature = Feature::from_markdown(&content).ok()?;
    Some(feature.body)
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

fn default_feature_doc_content(feature: &Feature) -> String {
    format!(
        "# {} Documentation\n\n## Capability Summary\n\nDescribe what this feature does in production terms.\n\n## User Workflows\n\n- Primary flow:\n- Edge cases:\n\n## Implementation Notes\n\nKey technical decisions and integration points.\n\n## Validation\n\nHow this capability is validated (tests, checks, manual QA).\n\n## Session Updates\n\n- _Use session summaries to keep this section current._\n",
        feature.metadata.title
    )
}

pub fn ensure_feature_documentation(
    ship_dir: &Path,
    feature_id: &str,
) -> Result<FeatureDocumentation> {
    let entry = get_feature_by_id(ship_dir, feature_id)?;
    if let Some(existing) = get_feature_doc_db(ship_dir, &entry.id)? {
        return Ok(existing);
    }

    let now = Utc::now().to_rfc3339();
    let doc = FeatureDocumentation {
        feature_id: entry.id.clone(),
        status: FeatureDocStatus::NotStarted,
        content: default_feature_doc_content(&entry.feature),
        revision: 1,
        last_verified_at: None,
        created_at: now.clone(),
        updated_at: now,
    };
    upsert_feature_doc_db(ship_dir, &doc, Some("ship"))
}

pub fn get_feature_documentation(
    ship_dir: &Path,
    feature_id: &str,
) -> Result<FeatureDocumentation> {
    ensure_feature_documentation(ship_dir, feature_id)
}

pub fn update_feature_documentation(
    ship_dir: &Path,
    feature_id: &str,
    content: String,
    status: Option<FeatureDocStatus>,
    verify_now: bool,
    actor: Option<&str>,
) -> Result<FeatureDocumentation> {
    let mut doc = ensure_feature_documentation(ship_dir, feature_id)?;
    doc.content = content;
    if let Some(next_status) = status {
        doc.status = next_status;
    } else if doc.status == FeatureDocStatus::NotStarted {
        doc.status = FeatureDocStatus::Draft;
    }
    if verify_now {
        let now = Utc::now().to_rfc3339();
        doc.last_verified_at = Some(now.clone());
        if doc.status == FeatureDocStatus::Draft {
            doc.status = FeatureDocStatus::Reviewed;
        }
    }

    let updated = upsert_feature_doc_db(ship_dir, &doc, actor)?;
    runtime::append_event(
        ship_dir,
        actor.unwrap_or("logic"),
        runtime::EventEntity::Feature,
        runtime::EventAction::Update,
        feature_id.to_string(),
        Some("feature-doc-update".to_string()),
    )?;
    Ok(updated)
}

pub fn record_feature_session_update(
    ship_dir: &Path,
    feature_id: &str,
    session_summary: Option<&str>,
    actor: Option<&str>,
) -> Result<FeatureDocumentation> {
    let mut doc = ensure_feature_documentation(ship_dir, feature_id)?;
    let summary = session_summary
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Session completed.");
    let timestamp = Utc::now().to_rfc3339();
    let mut content = doc.content.trim_end().to_string();
    if !content.contains("## Session Updates") {
        content.push_str("\n\n## Session Updates\n");
    }
    content.push_str(&format!("\n- {} — {}", timestamp, summary));
    doc.content = content;
    if doc.status == FeatureDocStatus::NotStarted {
        doc.status = FeatureDocStatus::Draft;
    }
    upsert_feature_doc_db(ship_dir, &doc, actor)
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
            active_target_id: release_id.map(|s| s.to_string()),
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
    let _ = ensure_feature_documentation(ship_dir, &id);
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
    let mut entry =
        get_feature_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Feature not found: {}", id))?;
    if entry.feature.body.trim().is_empty()
        && let Some(body) = load_feature_body_from_file(ship_dir, &resolved_id)
    {
        entry.feature.body = body;
    }
    Ok(entry)
}

pub fn update_feature(ship_dir: &Path, id: &str, mut feature: Feature) -> Result<FeatureEntry> {
    let resolved_id = require_feature_id(ship_dir, id)?;
    let existing = get_feature_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Feature not found: {}", id))?;
    feature.metadata.updated = Utc::now().to_rfc3339();
    if feature.body.trim().is_empty() {
        if let Some(body) = load_feature_body_from_file(ship_dir, &resolved_id) {
            feature.body = body;
        }
    }
    feature.extract_structured_data();

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

    let mut feature = existing.feature.clone();
    if feature.body.trim().is_empty() {
        if let Some(body) = load_feature_body_from_file(ship_dir, &resolved_id) {
            feature.body = body;
        }
    }
    feature.extract_structured_data();
    upsert_feature_db(ship_dir, &feature, &new_status)?;
    remove_feature_files(ship_dir, &resolved_id, &existing.feature.metadata.title);
    write_feature_file(ship_dir, &feature, &new_status)?;

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
    let entry = move_feature(ship_dir, id, FeatureStatus::InProgress)?;
    let doc = ensure_feature_documentation(ship_dir, &entry.id)?;
    if doc.status == FeatureDocStatus::NotStarted {
        let _ = update_feature_documentation(
            ship_dir,
            &entry.id,
            doc.content,
            Some(FeatureDocStatus::Draft),
            false,
            Some("ship"),
        )?;
    }
    Ok(entry)
}

pub fn feature_done(ship_dir: &Path, id: &str) -> Result<FeatureEntry> {
    move_feature(ship_dir, id, FeatureStatus::Implemented)
}

pub fn list_features(ship_dir: &Path) -> Result<Vec<FeatureEntry>> {
    list_features_db(ship_dir)
}
