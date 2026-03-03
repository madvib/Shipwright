use super::db::upsert_spec_db;
use super::types::{Spec, SpecStatus};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub fn import_specs_from_files(ship_dir: &Path) -> Result<usize> {
    let specs_dir = runtime::project::specs_dir(ship_dir);
    if !specs_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    let mut scan_dirs = vec![specs_dir.clone()];

    // Also scan status subdirectories
    for status in &["draft", "active", "archived"] {
        let status_dir = specs_dir.join(status);
        if status_dir.exists() {
            scan_dirs.push(status_dir);
        }
    }

    for dir in scan_dirs {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "md") {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name == "TEMPLATE.md" || file_name == "README.md" {
                    continue;
                }

                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read spec file: {}", path.display()))?;

                if let Ok(spec) = Spec::from_markdown(&content) {
                    // Determine status from directory name
                    let status = if path.parent() == Some(&specs_dir) {
                        SpecStatus::Draft
                    } else {
                        path.parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .and_then(|s| SpecStatus::from_str(s).ok())
                            .unwrap_or(SpecStatus::Draft)
                    };

                    upsert_spec_db(ship_dir, &spec, &status)?;
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}
