use super::db::{delete_spec_db, get_spec_db, list_specs_db, upsert_spec_db};
use super::types::{Spec, SpecEntry, SpecMetadata, SpecStatus};
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::path::{Path, PathBuf};

// ── File helpers ─────────────────────────────────────────────────────────────

fn spec_file_path(ship_dir: &Path, status: &SpecStatus, title: &str) -> PathBuf {
    let base = runtime::project::sanitize_file_name(title);
    let dir = runtime::project::specs_dir(ship_dir).join(status.to_string());
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

fn resolve_spec_id(ship_dir: &Path, reference: &str) -> Result<Option<String>> {
    let reference = reference.trim();
    if reference.is_empty() {
        return Ok(None);
    }

    if let Some(entry) = get_spec_db(ship_dir, reference)? {
        return Ok(Some(entry.id));
    }

    let without_ext = reference.trim_end_matches(".md");
    if without_ext != reference {
        if let Some(entry) = get_spec_db(ship_dir, without_ext)? {
            return Ok(Some(entry.id));
        }
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

pub fn write_spec_file(ship_dir: &Path, spec: &Spec, status: &SpecStatus) -> Result<PathBuf> {
    let path = spec_file_path(ship_dir, status, &spec.metadata.title);
    let content = spec.to_markdown()?;
    runtime::fs_util::write_atomic(&path, content)?;
    Ok(path)
}

pub fn remove_spec_files(ship_dir: &Path, id: &str, title: &str) {
    let base = runtime::project::sanitize_file_name(title);
    let specs_dir = runtime::project::specs_dir(ship_dir);

    // Check root and subdirs
    let mut scan_dirs = vec![specs_dir.clone()];
    for status in &["draft", "active", "archived"] {
        scan_dirs.push(specs_dir.join(status));
    }

    for dir in scan_dirs {
        if !dir.exists() {
            continue;
        }
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
                        return;
                    }
                }
            }
        }
    }
}

// ── Public CRUD ──────────────────────────────────────────────────────────────

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
    let file_path = write_spec_file(ship_dir, &spec, &status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Spec,
        runtime::EventAction::Create,
        id.clone(),
        Some(format!("title={}", title)),
    )?;

    Ok(SpecEntry {
        id,
        file_name: file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
        path: file_path.to_string_lossy().to_string(),
        status,
        spec,
    })
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
    write_spec_file(ship_dir, &spec, &existing.status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Spec,
        runtime::EventAction::Update,
        resolved_id.clone(),
        Some(format!("title={}", spec.metadata.title)),
    )?;

    let mut entry = get_spec_db(ship_dir, &resolved_id)?
        .ok_or_else(|| anyhow!("Spec not found after update"))?;
    entry.spec.body = spec.body;
    Ok(entry)
}

pub fn move_spec(ship_dir: &Path, id: &str, new_status: SpecStatus) -> Result<SpecEntry> {
    let resolved_id = require_spec_id(ship_dir, id)?;
    let existing =
        get_spec_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Spec not found: {}", id))?;

    upsert_spec_db(ship_dir, &existing.spec, &new_status)?;
    remove_spec_files(ship_dir, &resolved_id, &existing.spec.metadata.title);
    write_spec_file(ship_dir, &existing.spec, &new_status)?;

    runtime::append_event(
        ship_dir,
        "logic",
        runtime::EventEntity::Spec,
        runtime::EventAction::Update,
        resolved_id.clone(),
        Some(format!("status={}", new_status)),
    )?;

    Ok(get_spec_db(ship_dir, &resolved_id)?.unwrap())
}

pub fn delete_spec(ship_dir: &Path, id: &str) -> Result<()> {
    let resolved_id = require_spec_id(ship_dir, id)?;
    let entry =
        get_spec_db(ship_dir, &resolved_id)?.ok_or_else(|| anyhow!("Spec not found: {}", id))?;
    delete_spec_db(ship_dir, &resolved_id)?;
    remove_spec_files(ship_dir, &resolved_id, &entry.spec.metadata.title);

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
