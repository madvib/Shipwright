use super::db::{
    delete_feature_db, get_feature_db, get_feature_doc_db, list_features_db, upsert_feature_db,
    upsert_feature_doc_db,
};
use super::types::{
    Feature, FeatureDocStatus, FeatureDocumentation, FeatureEntry, FeatureMetadata, FeatureModel,
    FeatureStatus,
};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::path::Path;

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
) -> Result<std::path::PathBuf> {
    let path = runtime::project::features_dir(ship_dir)
        .join(status.to_string())
        .join(format!(
            "{}.md",
            runtime::project::sanitize_file_name(&feature.metadata.title)
        ));
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

pub fn remove_feature_files(_ship_dir: &Path, _id: &str, _title: &str) {}

// ── Public CRUD ──────────────────────────────────────────────────────────────

pub fn create_feature(
    ship_dir: &Path,
    title: &str,
    body: &str,
    release_id: Option<&str>,
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
            branch: branch.map(|s| s.to_string()),
            agent: None,
            tags: vec![],
        },
        body: body.to_string(),
        todos: vec![],
        criteria: vec![],
    };

    feature.extract_structured_data();
    let delta = feature.compute_delta();

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
        Some(format!("title={};drift_score={}", title, delta.drift_score)),
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
    if feature.body.trim().is_empty() {
        // Preserve existing body rather than blanking it
        feature.body = existing.feature.body.clone();
    }
    feature.extract_structured_data();
    let delta = feature.compute_delta();

    upsert_feature_db(ship_dir, &feature, &existing.status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Feature,
        runtime::EventAction::Update,
        resolved_id.clone(),
        Some(format!(
            "title={};drift_score={}",
            feature.metadata.title, delta.drift_score
        )),
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
    if let Ok(parsed) = Feature::from_markdown(content) {
        let parsed_title = parsed.metadata.title.trim();
        if !parsed_title.is_empty() {
            entry.feature.metadata.title = parsed_title.to_string();
        }
    }
    update_feature(ship_dir, &resolved_id, entry.feature)
}

pub fn move_feature(ship_dir: &Path, id: &str, new_status: FeatureStatus) -> Result<FeatureEntry> {
    let resolved_id = require_feature_id(ship_dir, id)?;
    let existing = get_feature_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Feature not found: {}", id))?;

    let mut feature = existing.feature.clone();
    feature.extract_structured_data();
    upsert_feature_db(ship_dir, &feature, &new_status)?;

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
    get_feature_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Feature not found: {}", id))?;
    delete_feature_db(ship_dir, &resolved_id)?;

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

pub fn get_feature_model(ship_dir: &Path, id: &str) -> Result<FeatureModel> {
    let mut entry = get_feature_by_id(ship_dir, id)?;
    entry.feature.extract_structured_data();
    Ok(entry.feature.model())
}
