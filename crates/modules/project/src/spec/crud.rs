use super::db::{delete_spec_db, get_spec_db, list_specs_db, upsert_spec_db};
use super::types::{Spec, SpecEntry, SpecMetadata, SpecStatus};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::path::Path;

fn resolve_spec_id(ship_dir: &Path, reference: &str) -> Result<Option<String>> {
    let reference = reference.trim();
    if reference.is_empty() {
        return Ok(None);
    }

    if let Some(entry) = get_spec_db(ship_dir, reference)? {
        return Ok(Some(entry.id));
    }

    let without_ext = reference.trim_end_matches(".md");
    if without_ext != reference
        && let Some(entry) = get_spec_db(ship_dir, without_ext)?
    {
        return Ok(Some(entry.id));
    }

    let reference_file = if reference.ends_with(".md") {
        reference.to_string()
    } else {
        format!("{}.md", reference)
    };
    let reference_slug = runtime::project::sanitize_file_name(without_ext);

    for entry in list_specs_db(ship_dir)? {
        let file_match = entry.file_name.eq_ignore_ascii_case(reference)
            || entry.file_name.eq_ignore_ascii_case(&reference_file);
        let slug_match = runtime::project::sanitize_file_name(&entry.spec.metadata.title)
            .eq_ignore_ascii_case(&reference_slug);
        if file_match || slug_match {
            return Ok(Some(entry.id));
        }
    }

    Ok(None)
}

fn require_spec_id(ship_dir: &Path, reference: &str) -> Result<String> {
    resolve_spec_id(ship_dir, reference)?.ok_or_else(|| anyhow!("Spec not found: {}", reference))
}

pub fn create_spec(
    ship_dir: &Path,
    title: &str,
    body: &str,
    feature_id: Option<String>,
    release_id: Option<String>,
) -> Result<SpecEntry> {
    if title.trim().is_empty() {
        return Err(anyhow!("Spec title cannot be empty"));
    }
    let id = runtime::gen_nanoid();
    let now = Utc::now().to_rfc3339();

    let spec = Spec {
        metadata: SpecMetadata {
            id: id.clone(),
            title: title.to_string(),
            created: now.clone(),
            updated: now,
            feature_id,
            release_id,
            ..Default::default()
        },
        body: body.to_string(),
    };

    let status = SpecStatus::Draft;
    upsert_spec_db(ship_dir, &spec, &status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Spec,
        runtime::EventAction::Create,
        id.clone(),
        Some(format!("title={}", title)),
    )?;

    get_spec_db(ship_dir, &id)?.ok_or_else(|| anyhow!("Spec not found after create: {}", id))
}

pub fn get_spec_by_id(ship_dir: &Path, id: &str) -> Result<SpecEntry> {
    let resolved_id = require_spec_id(ship_dir, id)?;
    get_spec_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Spec not found: {}", id))
}

pub fn update_spec(ship_dir: &Path, id: &str, mut spec: Spec) -> Result<SpecEntry> {
    let resolved_id = require_spec_id(ship_dir, id)?;
    let existing =
        get_spec_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Spec not found: {}", id))?;
    spec.metadata.updated = Utc::now().to_rfc3339();

    upsert_spec_db(ship_dir, &spec, &existing.status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Spec,
        runtime::EventAction::Update,
        resolved_id.clone(),
        Some(format!("title={}", spec.metadata.title)),
    )?;

    get_spec_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Spec not found after update: {}", resolved_id))
}

pub fn move_spec(ship_dir: &Path, id: &str, new_status: SpecStatus) -> Result<SpecEntry> {
    let resolved_id = require_spec_id(ship_dir, id)?;
    let existing =
        get_spec_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Spec not found: {}", id))?;

    upsert_spec_db(ship_dir, &existing.spec, &new_status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Spec,
        runtime::EventAction::Update,
        resolved_id.clone(),
        Some(format!("status={}", new_status)),
    )?;

    get_spec_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Spec not found after move: {}", resolved_id))
}

pub fn delete_spec(ship_dir: &Path, id: &str) -> Result<()> {
    let resolved_id = require_spec_id(ship_dir, id)?;
    get_spec_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Spec not found: {}", id))?;
    delete_spec_db(ship_dir, &resolved_id)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Spec,
        runtime::EventAction::Delete,
        resolved_id,
        None,
    )?;
    Ok(())
}

pub fn list_specs(ship_dir: &Path) -> Result<Vec<SpecEntry>> {
    list_specs_db(ship_dir)
}
