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
pub struct ReleaseMetadata {
    #[serde(default)]
    pub id: String,
    pub version: String,
    #[serde(default = "default_status")]
    pub status: String, // planned | active | shipped | archived
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_date: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub adrs: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_status() -> String {
    "planned".to_string()
}

#[derive(Debug, Clone, Type)]
pub struct Release {
    pub metadata: ReleaseMetadata,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct ReleaseEntry {
    pub file_name: String,
    pub path: String,
    pub version: String,
    pub status: String,
    pub updated: DateTime<Utc>,
}

fn ship_dir_from_release_path(path: &Path) -> Option<PathBuf> {
    // .ship/releases/<file>.md -> go up two levels
    path.parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
}

// ─── Validation ───────────────────────────────────────────────────────────────

fn validate_version(version: &str) -> Result<()> {
    if version.trim().is_empty() {
        return Err(anyhow!("Release version cannot be empty"));
    }
    Ok(())
}

// ─── Serialisation ────────────────────────────────────────────────────────────

impl Release {
    pub fn to_markdown(&self) -> Result<String> {
        let toml_str = toml::to_string(&self.metadata)
            .context("Failed to serialise release metadata as TOML")?;
        Ok(format!("+++\n{}+++\n\n{}", toml_str, self.body))
    }

    pub fn from_markdown(content: &str) -> Result<Self> {
        if content.starts_with("+++\n") {
            Self::from_toml_markdown(content)
        } else {
            let version = content
                .lines()
                .find(|l| l.starts_with("# "))
                .map(|l| l.trim_start_matches("# ").trim().to_string())
                .unwrap_or_default();
            let now = Utc::now();
            Ok(Release {
                metadata: ReleaseMetadata {
                    id: String::new(),
                    version,
                    status: default_status(),
                    created: now,
                    updated: now,
                    target_date: None,
                    features: Vec::new(),
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
            .ok_or_else(|| anyhow!("Invalid release format: missing closing +++"))?;
        let toml_str = &rest[..end];
        let body = rest[end + 4..].trim_start_matches('\n').to_string();
        let metadata: ReleaseMetadata =
            toml::from_str(toml_str).context("Failed to parse release TOML frontmatter")?;
        Ok(Release { metadata, body })
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

/// Create a new release file in `.ship/releases/`.
pub fn create_release(project_dir: PathBuf, version: &str, body: &str) -> Result<PathBuf> {
    validate_version(version)?;

    let releases_dir = project_dir.join("releases");
    fs::create_dir_all(&releases_dir)?;

    let now = Utc::now();
    let release = Release {
        metadata: ReleaseMetadata {
            id: Uuid::new_v4().to_string(),
            version: version.to_string(),
            status: default_status(),
            created: now,
            updated: now,
            target_date: None,
            features: Vec::new(),
            adrs: Vec::new(),
            tags: Vec::new(),
        },
        body: if body.is_empty() {
            "## Goal\n\n\n## Scope\n\n- [ ]\n\n## Included Features\n\n- [ ]\n\n## Notes\n\n"
                .to_string()
        } else {
            body.to_string()
        },
    };

    let base = sanitize_file_name(version);
    let file_path = unique_path(&releases_dir, &base);
    write_atomic(&file_path, release.to_markdown()?)?;
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    append_event(
        &project_dir,
        "logic",
        EventEntity::Release,
        EventAction::Create,
        file_name,
        Some(format!("version={}", version)),
    )?;
    Ok(file_path)
}

/// Read and parse a release file.
pub fn get_release(path: PathBuf) -> Result<Release> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read release: {}", path.display()))?;
    Release::from_markdown(&content)
}

/// Read the raw markdown content of a release.
pub fn get_release_raw(path: PathBuf) -> Result<String> {
    fs::read_to_string(&path).with_context(|| format!("Failed to read release: {}", path.display()))
}

/// Overwrite a release's body content, updating the `updated` timestamp.
pub fn update_release(path: PathBuf, body: &str) -> Result<()> {
    let mut release = get_release(path.clone())?;
    release.metadata.updated = Utc::now();
    let version = release.metadata.version.clone();
    release.body = body.to_string();
    write_atomic(&path, release.to_markdown()?)
        .with_context(|| format!("Failed to write release: {}", path.display()))?;
    if let Some(project_dir) = ship_dir_from_release_path(&path) {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        append_event(
            &project_dir,
            "logic",
            EventEntity::Release,
            EventAction::Update,
            file_name,
            Some(format!("version={}", version)),
        )?;
    }
    Ok(())
}

/// List all release files in `.ship/releases/`.
pub fn list_releases(project_dir: PathBuf) -> Result<Vec<ReleaseEntry>> {
    let releases_dir = project_dir.join("releases");
    if !releases_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&releases_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "md") {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if let Ok(release) = get_release(path.clone()) {
                entries.push(ReleaseEntry {
                    file_name,
                    path: path.to_string_lossy().to_string(),
                    version: release.metadata.version,
                    status: release.metadata.status,
                    updated: release.metadata.updated,
                });
            }
        }
    }
    Ok(entries)
}
