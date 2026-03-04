use super::db::{get_release_db, upsert_release_db};
use super::types::{Release, ReleaseStatus};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn import_releases_from_files(ship_dir: &Path) -> Result<usize> {
    let releases_dir = ship_dir.join("project").join("releases");
    if !releases_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    let mut scan_dirs = vec![releases_dir.clone()];

    // Also scan "upcoming" subdirectory if it exists (legacy pattern)
    let upcoming_dir = releases_dir.join("upcoming");
    if upcoming_dir.exists() {
        scan_dirs.push(upcoming_dir);
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
                    .with_context(|| format!("Failed to read release file: {}", path.display()))?;

                if let Ok(release) = Release::from_markdown(&content) {
                    let status = if path.starts_with(&releases_dir.join("upcoming")) {
                        ReleaseStatus::Planned
                    } else {
                        release.metadata.status.clone()
                    };

                    if get_release_db(ship_dir, &release.metadata.id)?.is_some() {
                        continue;
                    }
                    upsert_release_db(ship_dir, &release, &status)?;
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}
