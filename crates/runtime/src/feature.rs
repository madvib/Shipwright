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
pub struct FeatureMetadata {
    #[serde(default)]
    pub id: String,
    pub title: String,
    #[serde(default = "default_status")]
    pub status: String, // active | paused | complete | archived
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<String>,
    #[serde(default)]
    pub adrs: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_status() -> String {
    "active".to_string()
}

#[derive(Debug, Clone, Type)]
pub struct Feature {
    pub metadata: FeatureMetadata,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct FeatureEntry {
    pub file_name: String,
    pub path: String,
    pub title: String,
    pub status: String,
    pub release: Option<String>,
    pub updated: DateTime<Utc>,
}

fn ship_dir_from_feature_path(path: &Path) -> Option<PathBuf> {
    // .ship/features/<file>.md -> go up two levels
    path.parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
}

// ─── Validation ───────────────────────────────────────────────────────────────

fn validate_title(title: &str) -> Result<()> {
    if title.trim().is_empty() {
        return Err(anyhow!("Feature title cannot be empty"));
    }
    Ok(())
}

// ─── Serialisation ────────────────────────────────────────────────────────────

impl Feature {
    pub fn to_markdown(&self) -> Result<String> {
        let toml_str = toml::to_string(&self.metadata)
            .context("Failed to serialise feature metadata as TOML")?;
        Ok(format!("+++\n{}+++\n\n{}", toml_str, self.body))
    }

    pub fn from_markdown(content: &str) -> Result<Self> {
        if content.starts_with("+++\n") {
            Self::from_toml_markdown(content)
        } else {
            let title = content
                .lines()
                .find(|l| l.starts_with("# "))
                .map(|l| l.trim_start_matches("# ").trim().to_string())
                .unwrap_or_default();
            let now = Utc::now();
            Ok(Feature {
                metadata: FeatureMetadata {
                    id: String::new(),
                    title,
                    status: default_status(),
                    created: now,
                    updated: now,
                    owner: None,
                    release: None,
                    spec: None,
                    adrs: Vec::new(),
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
            .ok_or_else(|| anyhow!("Invalid feature format: missing closing +++"))?;
        let toml_str = &rest[..end];
        let body = rest[end + 4..].trim_start_matches('\n').to_string();
        let metadata: FeatureMetadata =
            toml::from_str(toml_str).context("Failed to parse feature TOML frontmatter")?;
        Ok(Feature { metadata, body })
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

/// Create a new feature file in `.ship/features/`.
pub fn create_feature(
    project_dir: PathBuf,
    title: &str,
    body: &str,
    release: Option<&str>,
    spec: Option<&str>,
) -> Result<PathBuf> {
    validate_title(title)?;

    let features_dir = project_dir.join("features");
    fs::create_dir_all(&features_dir)?;

    let now = Utc::now();
    let feature = Feature {
        metadata: FeatureMetadata {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            status: default_status(),
            created: now,
            updated: now,
            owner: None,
            release: release.filter(|s| !s.trim().is_empty()).map(str::to_string),
            spec: spec.filter(|s| !s.trim().is_empty()).map(str::to_string),
            adrs: Vec::new(),
            tags: Vec::new(),
        },
        body: if body.is_empty() {
            "## Why\n\n\n## Acceptance Criteria\n\n- [ ]\n\n## Delivery Todos\n\n- [ ]\n\n## Notes\n\n"
                .to_string()
        } else {
            body.to_string()
        },
    };

    let base = sanitize_file_name(title);
    let file_path = unique_path(&features_dir, &base);
    write_atomic(&file_path, feature.to_markdown()?)?;
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    append_event(
        &project_dir,
        "logic",
        EventEntity::Feature,
        EventAction::Create,
        file_name,
        Some(format!("title={}", title)),
    )?;
    Ok(file_path)
}

/// Read and parse a feature file.
pub fn get_feature(path: PathBuf) -> Result<Feature> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read feature: {}", path.display()))?;
    Feature::from_markdown(&content)
}

/// Read the raw markdown content of a feature.
pub fn get_feature_raw(path: PathBuf) -> Result<String> {
    fs::read_to_string(&path).with_context(|| format!("Failed to read feature: {}", path.display()))
}

/// Overwrite a feature's body content, updating the `updated` timestamp.
pub fn update_feature(path: PathBuf, body: &str) -> Result<()> {
    let mut feature = get_feature(path.clone())?;
    feature.metadata.updated = Utc::now();
    let title = feature.metadata.title.clone();
    feature.body = body.to_string();
    write_atomic(&path, feature.to_markdown()?)
        .with_context(|| format!("Failed to write feature: {}", path.display()))?;
    if let Some(project_dir) = ship_dir_from_feature_path(&path) {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        append_event(
            &project_dir,
            "logic",
            EventEntity::Feature,
            EventAction::Update,
            file_name,
            Some(format!("title={}", title)),
        )?;
    }
    Ok(())
}

/// List all feature files in `.ship/features/`.
pub fn list_features(project_dir: PathBuf) -> Result<Vec<FeatureEntry>> {
    let features_dir = project_dir.join("features");
    if !features_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&features_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "md") {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if let Ok(feature) = get_feature(path.clone()) {
                entries.push(FeatureEntry {
                    file_name,
                    path: path.to_string_lossy().to_string(),
                    title: feature.metadata.title,
                    status: feature.metadata.status,
                    release: feature.metadata.release,
                    updated: feature.metadata.updated,
                });
            }
        }
    }
    Ok(entries)
}
