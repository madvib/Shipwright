use crate::fs_util::write_atomic;
use crate::project::sanitize_file_name;
use crate::{EventAction, EventEntity, append_event};
use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

// ─── Data types ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct SpecMetadata {
    #[serde(default)]
    pub id: String,
    pub title: String,
    #[serde(default = "default_status")]
    pub status: String, // draft | active | archived
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_status() -> String {
    "draft".to_string()
}

impl Default for SpecMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: String::new(),
            status: default_status(),
            created: now,
            updated: now,
            author: None,
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Type)]
pub struct Spec {
    pub metadata: SpecMetadata,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct SpecEntry {
    pub file_name: String,
    pub path: String,
    pub title: String,
    pub status: String,
    pub updated: DateTime<Utc>,
}

fn ship_dir_from_spec_path(path: &Path) -> Option<PathBuf> {
    // .ship/specs/<file>.md -> go up two levels
    path.parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
}

// ─── Validation ───────────────────────────────────────────────────────────────

fn validate_title(title: &str) -> Result<()> {
    if title.trim().is_empty() {
        return Err(anyhow!("Spec title cannot be empty"));
    }
    Ok(())
}

// ─── Serialisation ────────────────────────────────────────────────────────────

impl Spec {
    pub fn to_markdown(&self) -> Result<String> {
        let toml_str =
            toml::to_string(&self.metadata).context("Failed to serialise spec metadata as TOML")?;
        Ok(format!("+++\n{}+++\n\n{}", toml_str, self.body))
    }

    pub fn from_markdown(content: &str) -> Result<Self> {
        if content.starts_with("+++\n") {
            Self::from_toml_markdown(content)
        } else {
            // Legacy: raw markdown with no frontmatter — synthesise metadata from first heading
            let title = content
                .lines()
                .find(|l| l.starts_with("# "))
                .map(|l| l.trim_start_matches("# ").trim().to_string())
                .unwrap_or_default();
            let now = Utc::now();
            Ok(Spec {
                metadata: SpecMetadata {
                    id: String::new(), // will be backfilled
                    title,
                    status: default_status(),
                    created: now,
                    updated: now,
                    author: None,
                    tags: Vec::new(),
                },
                body: content.to_string(),
            })
        }
    }

    fn from_toml_markdown(content: &str) -> Result<Self> {
        let rest = &content[4..]; // skip "+++\n"
        let end = rest
            .find("\n+++")
            .ok_or_else(|| anyhow!("Invalid spec format: missing closing +++"))?;
        let toml_str = &rest[..end];
        let body = rest[end + 4..].trim_start_matches('\n').to_string();
        let metadata: SpecMetadata =
            toml::from_str(toml_str).context("Failed to parse spec TOML frontmatter")?;
        Ok(Spec { metadata, body })
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

/// Create a new spec file in `.ship/specs/`.
pub fn create_spec(project_dir: PathBuf, title: &str, body: &str) -> Result<PathBuf> {
    validate_title(title)?;

    let specs_dir = project_dir.join("specs");
    fs::create_dir_all(&specs_dir)?;

    let now = Utc::now();
    let spec = Spec {
        metadata: SpecMetadata {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            status: default_status(),
            created: now,
            updated: now,
            author: None,
            tags: Vec::new(),
        },
        body: if body.is_empty() {
            format!(
                "## Overview\n\n\n## Goals\n\n\n## Non-Goals\n\n\n## Approach\n\n\n## Open Questions\n\n"
            )
        } else {
            body.to_string()
        },
    };

    let base = sanitize_file_name(title);
    let file_path = unique_path(&specs_dir, &base);
    write_atomic(&file_path, spec.to_markdown()?)?;
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    append_event(
        &project_dir,
        "logic",
        EventEntity::Spec,
        EventAction::Create,
        file_name,
        Some(format!("title={}", title)),
    )?;
    Ok(file_path)
}

/// Read and parse a spec file.
pub fn get_spec(path: PathBuf) -> Result<Spec> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read spec: {}", path.display()))?;
    Spec::from_markdown(&content)
}

/// Read the raw markdown content of a spec (for MCP/AI consumption).
pub fn get_spec_raw(path: PathBuf) -> Result<String> {
    fs::read_to_string(&path).with_context(|| format!("Failed to read spec: {}", path.display()))
}

/// Overwrite a spec's body content, updating the `updated` timestamp.
pub fn update_spec(path: PathBuf, body: &str) -> Result<()> {
    let mut spec = get_spec(path.clone())?;
    spec.metadata.updated = Utc::now();
    let title = spec.metadata.title.clone();
    spec.body = body.to_string();
    write_atomic(&path, spec.to_markdown()?)
        .with_context(|| format!("Failed to write spec: {}", path.display()))?;
    if let Some(project_dir) = ship_dir_from_spec_path(&path) {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        append_event(
            &project_dir,
            "logic",
            EventEntity::Spec,
            EventAction::Update,
            file_name,
            Some(format!("title={}", title)),
        )?;
    }
    Ok(())
}

pub fn delete_spec(path: PathBuf) -> Result<()> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    fs::remove_file(&path).with_context(|| format!("Failed to delete spec: {}", path.display()))?;
    if let Some(project_dir) = ship_dir_from_spec_path(&path) {
        append_event(
            &project_dir,
            "logic",
            EventEntity::Spec,
            EventAction::Delete,
            file_name,
            None,
        )?;
    }
    Ok(())
}

/// List all spec files in `.ship/specs/`.
pub fn list_specs(project_dir: PathBuf) -> Result<Vec<SpecEntry>> {
    let specs_dir = project_dir.join("specs");
    if !specs_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&specs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "md") {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if let Ok(spec) = get_spec(path.clone()) {
                entries.push(SpecEntry {
                    file_name,
                    path: path.to_string_lossy().to_string(),
                    title: spec.metadata.title,
                    status: spec.metadata.status,
                    updated: spec.metadata.updated,
                });
            }
        }
    }
    Ok(entries)
}
