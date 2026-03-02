/// Idempotent import of existing ADR markdown files into SQLite.
use super::{
    db::{adr_to_db_row, upsert_adr_db},
    export::status_dir_name,
    types::{ADR, AdrStatus},
};
use anyhow::Result;
use std::path::Path;

pub fn import_adrs_from_files(ship_dir: &Path) -> Result<usize> {
    let adrs_dir = runtime::project::adrs_dir(ship_dir);
    if !adrs_dir.exists() {
        return Ok(0);
    }

    let mut imported = 0usize;

    // 1. Check root adrs dir for any loose .md files
    imported += import_from_dir(ship_dir, &adrs_dir, AdrStatus::Proposed)?;

    // 2. Check status subdirectories
    let statuses = [
        AdrStatus::Proposed,
        AdrStatus::Accepted,
        AdrStatus::Rejected,
        AdrStatus::Superseded,
        AdrStatus::Deprecated,
    ];
    for status in &statuses {
        let status_str = status_dir_name(status);
        let dir = adrs_dir.join(status_str);
        if !dir.exists() {
            continue;
        }
        imported += import_from_dir(ship_dir, &dir, status.clone())?;
    }
    Ok(imported)
}

fn import_from_dir(ship_dir: &Path, dir: &Path, status: AdrStatus) -> Result<usize> {
    let status_str = status_dir_name(&status);
    let mut count = 0;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let fname = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if fname == "TEMPLATE.md" || fname == "README.md" {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut adr = match ADR::from_markdown(&content) {
            Ok(a) => a,
            Err(_) => continue,
        };

        // Normalize ID: if it's a UUID (contains '-' and length > 20), change to Nanoid
        let old_id = adr.metadata.id.clone();
        if old_id.contains('-') && old_id.len() > 20 {
            let new_id = runtime::gen_nanoid();
            adr.metadata.id = new_id.clone();
            // Update the file frontmatter
            if let Ok(updated_markdown) = adr.to_markdown() {
                let _ = std::fs::write(&path, updated_markdown);
                println!(
                    "[ship] Normalized ADR ID for '{}': {} -> {}",
                    adr.metadata.title, old_id, new_id
                );
            }
            // Delete old record from DB if it exists
            let _ = super::db::delete_adr_db(ship_dir, &old_id);
        }

        if super::db::get_adr_db(ship_dir, &adr.metadata.id)?.is_some() {
            continue;
        }
        let row = adr_to_db_row(&adr, status_str, None);
        upsert_adr_db(ship_dir, &row)?;
        count += 1;
    }
    Ok(count)
}
