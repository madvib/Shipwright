use crate::project::sanitize_file_name;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ADR {
    pub title: String,
    pub decision: String,
    pub status: String,
    pub date: DateTime<Utc>,
    pub links: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AdrEntry {
    pub file_name: String,
    pub path: String,
    pub adr: ADR,
}

pub fn create_adr(
    project_dir: PathBuf,
    title: &str,
    decision: &str,
    status: &str,
) -> Result<PathBuf> {
    let adr = ADR {
        title: title.to_string(),
        decision: decision.to_string(),
        status: status.to_string(),
        date: Utc::now(),
        links: Vec::new(),
    };

    let file_name = format!("{}.json", sanitize_file_name(title));
    let file_path = project_dir.join("ADR").join(&file_name);

    let json = serde_json::to_string_pretty(&adr)?;
    fs::write(&file_path, json).context("Failed to write ADR file")?;

    Ok(file_path)
}

pub fn list_adrs(project_dir: PathBuf) -> Result<Vec<AdrEntry>> {
    let mut entries = Vec::new();
    let adr_dir = project_dir.join("ADR");

    if !adr_dir.exists() {
        return Ok(entries);
    }

    for entry in fs::read_dir(adr_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "json") {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(adr) = serde_json::from_str::<ADR>(&content) {
                    entries.push(AdrEntry {
                        file_name,
                        path: path.to_string_lossy().to_string(),
                        adr,
                    });
                }
            }
        }
    }

    Ok(entries)
}
