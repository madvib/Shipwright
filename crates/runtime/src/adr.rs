use crate::fs_util::write_atomic;
use crate::project::sanitize_file_name;
use crate::{EventAction, EventEntity, append_event};
use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};

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
    pub id: String,
    pub title: String,
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
    pub status: AdrStatus,
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

/// Create a new ADR file in `.ship/project/adrs/`.
pub fn create_adr(
    project_dir: PathBuf,
    title: &str,
    decision: &str,
    status: &str,
) -> Result<PathBuf> {
    validate_title(title)?;

    let ship_path = crate::project::get_project_dir(Some(project_dir.clone()))?;
    let template_str = crate::project::read_template(&ship_path, "adr")?;
    let mut adr = ADR::from_markdown(&template_str).context("Failed to parse ADR template")?;

    let adr_status = status.parse::<AdrStatus>().unwrap_or_default();
    adr.metadata.id = crate::gen_nanoid();
    adr.metadata.title = title.to_string();
    adr.metadata.date = Utc::now().to_rfc3339();

    if !decision.trim().is_empty() {
        adr.body = format!("## Decision\n\n{}\n", decision);
    }

    let adrs_dir = crate::project::adrs_dir(&project_dir);
    let status_dir = adrs_dir.join(adr_status.to_string());
    fs::create_dir_all(&status_dir)?;

    let base = sanitize_file_name(title);
    let file_path = unique_path(&status_dir, &base);

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

pub fn find_adr_path(project_dir: &Path, file_name: &str) -> Result<PathBuf> {
    let adrs_dir = crate::project::adrs_dir(project_dir);
    for status in &[
        "proposed",
        "accepted",
        "rejected",
        "superseded",
        "deprecated",
    ] {
        let candidate = adrs_dir.join(status).join(file_name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    let candidate = adrs_dir.join(file_name);
    if candidate.exists() {
        return Ok(candidate);
    }
    Err(anyhow!("ADR not found: {}", file_name))
}

pub fn move_adr(project_dir: PathBuf, file_name: &str, new_status: AdrStatus) -> Result<PathBuf> {
    let path = find_adr_path(&project_dir, file_name)?;

    let target_dir = crate::project::adrs_dir(&project_dir).join(new_status.to_string());
    fs::create_dir_all(&target_dir)?;

    let target_path = target_dir.join(file_name);
    if path != target_path {
        fs::rename(&path, &target_path)
            .with_context(|| format!("Failed to move ADR to {}", new_status))?;
    }

    append_event(
        &project_dir,
        "logic",
        EventEntity::Adr,
        EventAction::Update,
        file_name.to_string(),
        Some(format!("status changed to {}", new_status)),
    )?;
    Ok(target_path)
}

pub fn list_adrs(project_dir: PathBuf) -> Result<Vec<AdrEntry>> {
    let mut entries = Vec::new();
    let adrs_dir = crate::project::adrs_dir(&project_dir);
    if !adrs_dir.exists() {
        return Ok(entries);
    }

    // Check subdirectories
    for status_name in &[
        "proposed",
        "accepted",
        "rejected",
        "superseded",
        "deprecated",
    ] {
        let status_dir = adrs_dir.join(status_name);
        if status_dir.exists() {
            let status = status_name.parse::<AdrStatus>().unwrap_or_default();
            search_adr_dir(&status_dir, status, &mut entries)?;
        }
    }

    Ok(entries)
}

fn search_adr_dir(dir: &Path, status: AdrStatus, entries: &mut Vec<AdrEntry>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
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
                    status: status.clone(),
                    adr,
                });
            }
        }
    }
    Ok(())
}
