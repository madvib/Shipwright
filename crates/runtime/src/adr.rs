use crate::fs_util::write_atomic;
use crate::project::sanitize_file_name;
use crate::{EventAction, EventEntity, append_event};
use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

// ─── Data types ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
#[serde(rename_all = "lowercase")]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Rejected,
    Superseded,
    Deprecated,
}

impl Default for AdrStatus {
    fn default() -> Self {
        AdrStatus::Proposed
    }
}

impl std::fmt::Display for AdrStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdrStatus::Proposed => write!(f, "proposed"),
            AdrStatus::Accepted => write!(f, "accepted"),
            AdrStatus::Rejected => write!(f, "rejected"),
            AdrStatus::Superseded => write!(f, "superseded"),
            AdrStatus::Deprecated => write!(f, "deprecated"),
        }
    }
}

impl std::str::FromStr for AdrStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proposed" => Ok(AdrStatus::Proposed),
            "accepted" => Ok(AdrStatus::Accepted),
            "rejected" => Ok(AdrStatus::Rejected),
            "superseded" => Ok(AdrStatus::Superseded),
            "deprecated" => Ok(AdrStatus::Deprecated),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AdrMetadata {
    #[serde(default)]
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub status: AdrStatus,
    pub date: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ADR {
    pub metadata: AdrMetadata,
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AdrEntry {
    pub file_name: String,
    pub path: String,
    pub adr: ADR,
}

fn ship_dir_from_adr_path(path: &Path) -> Option<PathBuf> {
    crate::project::ship_dir_from_path(path)
}

// ─── Validation ───────────────────────────────────────────────────────────────

fn validate_title(title: &str) -> Result<()> {
    if title.trim().is_empty() {
        return Err(anyhow!("ADR title cannot be empty"));
    }
    Ok(())
}

// ─── Serialisation ────────────────────────────────────────────────────────────

impl ADR {
    pub fn to_markdown(&self) -> Result<String> {
        let toml_str =
            toml::to_string(&self.metadata).context("Failed to serialise ADR metadata as TOML")?;
        Ok(format!("+++\n{}+++\n\n{}", toml_str, self.body))
    }

    pub fn from_markdown(content: &str) -> Result<Self> {
        if !content.starts_with("+++\n") {
            return Err(anyhow!("Invalid ADR format: missing TOML frontmatter"));
        }
        let rest = &content[4..];
        let end = rest
            .find("\n+++")
            .ok_or_else(|| anyhow!("Invalid ADR format: missing closing +++"))?;
        let toml_str = &rest[..end];
        let body = rest[end + 4..].trim_start_matches('\n').to_string();
        let metadata: AdrMetadata =
            toml::from_str(toml_str).context("Failed to parse ADR TOML frontmatter")?;
        Ok(ADR { metadata, body })
    }
}

// ─── File helpers ─────────────────────────────────────────────────────────────

fn unique_path(dir: &Path, base: &str) -> PathBuf {
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

// ─── CRUD ─────────────────────────────────────────────────────────────────────

pub fn create_adr(
    project_dir: PathBuf,
    title: &str,
    decision: &str,
    status: &str,
) -> Result<PathBuf> {
    validate_title(title)?;

    let adrs_dir = crate::project::adrs_dir(&project_dir);
    fs::create_dir_all(&adrs_dir)?;

    let adr_status = status.parse::<AdrStatus>().unwrap_or_default();
    let metadata = AdrMetadata {
        id: Uuid::new_v4().to_string(),
        title: title.to_string(),
        status: adr_status,
        date: Utc::now().format("%Y-%m-%d").to_string(),
        tags: Vec::new(),
        spec_id: None,
        supersedes_id: None,
    };

    let body = format!("## Decision\n\n{}\n", decision);
    let adr = ADR { metadata, body };

    let base = sanitize_file_name(title);
    let file_path = unique_path(&adrs_dir, &base);

    write_atomic(&file_path, adr.to_markdown()?)?;
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    append_event(
        &project_dir,
        "logic",
        EventEntity::Adr,
        EventAction::Create,
        file_name,
        Some(format!("title={} status={}", title, status)),
    )?;
    Ok(file_path)
}

pub fn get_adr(path: PathBuf) -> Result<ADR> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read ADR: {}", path.display()))?;
    ADR::from_markdown(&content)
}

pub fn update_adr(path: PathBuf, adr: ADR) -> Result<()> {
    let title = adr.metadata.title.clone();
    let content = adr.to_markdown()?;
    write_atomic(&path, content)
        .with_context(|| format!("Failed to write ADR: {}", path.display()))?;
    if let Some(project_dir) = ship_dir_from_adr_path(&path) {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        append_event(
            &project_dir,
            "logic",
            EventEntity::Adr,
            EventAction::Update,
            file_name,
            Some(format!("title={}", title)),
        )?;
    }
    Ok(())
}

pub fn delete_adr(path: PathBuf) -> Result<()> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    fs::remove_file(&path).with_context(|| format!("Failed to delete ADR: {}", path.display()))?;
    if let Some(project_dir) = ship_dir_from_adr_path(&path) {
        append_event(
            &project_dir,
            "logic",
            EventEntity::Adr,
            EventAction::Delete,
            file_name,
            None,
        )?;
    }
    Ok(())
}

pub fn list_adrs(project_dir: PathBuf) -> Result<Vec<AdrEntry>> {
    let mut entries = Vec::new();
    let adrs_dir = crate::project::adrs_dir(&project_dir);
    if !adrs_dir.exists() {
        return Ok(entries);
    }

    for entry in fs::read_dir(&adrs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "md") {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if file_name == "TEMPLATE.md" || file_name == "README.md" {
                continue;
            }
            if let Ok(adr) = get_adr(path.clone()) {
                entries.push(AdrEntry {
                    file_name,
                    path: path.to_string_lossy().to_string(),
                    adr,
                });
            }
        }
    }
    Ok(entries)
}
