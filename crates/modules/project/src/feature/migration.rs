use super::db::{get_feature_db, upsert_feature_db};
use super::types::{Feature, FeatureStatus};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn import_features_from_files(ship_dir: &Path) -> Result<usize> {
    let features_dir = ship_dir.join("project").join("features");
    if !features_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    let mut scan_dirs = vec![features_dir.clone()];

    // Also scan status subdirectories
    for status in &["planned", "in-progress", "implemented", "deprecated"] {
        let status_dir = features_dir.join(status);
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
                    .with_context(|| format!("Failed to read feature file: {}", path.display()))?;

                if let Ok(feature) = Feature::from_markdown(&content) {
                    // Determine status from directory name
                    let status = if path.parent() == Some(&features_dir) {
                        FeatureStatus::Planned
                    } else {
                        path.parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .and_then(|s| s.parse::<FeatureStatus>().ok())
                            .unwrap_or(FeatureStatus::Planned)
                    };

                    if get_feature_db(ship_dir, &feature.metadata.id)?.is_some() {
                        continue;
                    }
                    upsert_feature_db(ship_dir, &feature, &status)?;
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}
