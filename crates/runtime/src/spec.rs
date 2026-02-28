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
pub enum SpecStatus {
    Draft,
    Active,
    Archived,
}

impl Default for SpecStatus {
    fn default() -> Self {
        SpecStatus::Draft
    }
}

impl std::fmt::Display for SpecStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecStatus::Draft => write!(f, "draft"),
            SpecStatus::Active => write!(f, "active"),
            SpecStatus::Archived => write!(f, "archived"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct SpecMetadata {
    #[serde(default)]
    pub id: String,
    pub title: String,
    pub created: String,
    pub updated: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Default for SpecMetadata {
    fn default() -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: crate::gen_nanoid(),
            title: String::new(),
            created: now.clone(),
            updated: now,
            author: None,
            branch: None,
            feature_id: None,
            release_id: None,
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
    pub status: SpecStatus,
    pub updated: String,
}

fn ship_dir_from_spec_path(path: &Path) -> Option<PathBuf> {
    crate::project::ship_dir_from_path(path)
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
            let now = Utc::now().to_rfc3339();
            Ok(Spec {
                metadata: SpecMetadata {
                    id: crate::gen_nanoid(),
                    title,
                    created: now.clone(),
                    updated: now,
                    author: None,
                    branch: None,
                    feature_id: None,
                    release_id: None,
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

/// Create a new spec file in `.ship/workflow/specs/<status>/`.
pub fn create_spec(project_dir: PathBuf, title: &str, body: &str, status: &str) -> Result<PathBuf> {
    validate_title(title)?;
    crate::project::validate_status(status)?;

    let status_dir = crate::project::specs_dir(&project_dir).join(status);
    fs::create_dir_all(&status_dir)?;

    let template = crate::project::read_template(&project_dir, "spec")?;
    let mut spec = Spec::from_markdown(&template)?;
    let now = Utc::now().to_rfc3339();

    spec.metadata.id = crate::gen_nanoid();
    spec.metadata.title = title.to_string();
    spec.metadata.created = now.clone();
    spec.metadata.updated = now;

    if !body.is_empty() {
        spec.body = body.to_string();
    }

    let base = sanitize_file_name(title);
    let file_path = unique_path(&status_dir, &base);
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
        Some(format!("title={} status={}", title, status)),
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
    spec.metadata.updated = Utc::now().to_rfc3339();
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

/// List all spec files in `.ship/workflow/specs/`.
pub fn list_specs(project_dir: PathBuf) -> Result<Vec<SpecEntry>> {
    let specs_dir = crate::project::specs_dir(&project_dir);
    if !specs_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    for status_entry in fs::read_dir(&specs_dir)? {
        let status_entry = status_entry?;
        let status_path = status_entry.path();
        if !status_path.is_dir() {
            continue;
        }

        let status_str = status_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let status = status_str.parse::<SpecStatus>().unwrap_or_default();

        for entry in fs::read_dir(&status_path)? {
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
                if let Ok(spec) = get_spec(path.clone()) {
                    entries.push(SpecEntry {
                        file_name,
                        path: path.to_string_lossy().to_string(),
                        title: spec.metadata.title,
                        status: status.clone(),
                        updated: spec.metadata.updated.clone(),
                    });
                }
            }
        }
    }
    Ok(entries)
}

pub fn move_spec(project_dir: PathBuf, path: PathBuf, new_status: &str) -> Result<PathBuf> {
    crate::project::validate_status(new_status)?;
    if !path.exists() {
        return Err(anyhow!("Spec not found: {}", path.display()));
    }

    let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
    let specs_dir = crate::project::specs_dir(&project_dir);
    let target_dir = specs_dir.join(new_status);
    fs::create_dir_all(&target_dir)?;

    let base = file_name.trim_end_matches(".md");
    let target_path = unique_path(&target_dir, base);

    fs::rename(&path, &target_path).context("Failed to move spec file")?;

    let from_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();
    let to_name = target_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string();

    append_event(
        &project_dir,
        "logic",
        EventEntity::Spec,
        EventAction::Move,
        to_name,
        Some(format!("from={} to_status={}", from_name, new_status)),
    )?;

    Ok(target_path)
}

impl std::str::FromStr for SpecStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(SpecStatus::Draft),
            "active" => Ok(SpecStatus::Active),
            "archived" => Ok(SpecStatus::Archived),
            _ => Err(anyhow!("Invalid spec status: {}", s)),
        }
    }
}
