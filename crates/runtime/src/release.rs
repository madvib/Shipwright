use crate::fs_util::write_atomic;
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
pub enum ReleaseStatus {
    Planned,
    Active,
    Shipped,
    Archived,
}

impl Default for ReleaseStatus {
    fn default() -> Self {
        ReleaseStatus::Planned
    }
}

impl std::fmt::Display for ReleaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReleaseStatus::Planned => write!(f, "planned"),
            ReleaseStatus::Active => write!(f, "active"),
            ReleaseStatus::Shipped => write!(f, "shipped"),
            ReleaseStatus::Archived => write!(f, "archived"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ReleaseMetadata {
    #[serde(default)]
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub status: ReleaseStatus,
    pub created: String,
    pub updated: String,
    /// Whether this release line is still supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_date: Option<String>,
    #[serde(default)]
    pub feature_ids: Vec<String>,
    #[serde(default)]
    pub adr_ids: Vec<String>,
    #[serde(default)]
    pub breaking_changes: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
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
    pub status: ReleaseStatus,
    pub updated: String,
}

fn ship_dir_from_release_path(path: &Path) -> Option<PathBuf> {
    crate::project::ship_dir_from_path(path)
}

// ─── Validation ───────────────────────────────────────────────────────────────

fn validate_version(version: &str) -> Result<()> {
    let re = regex::Regex::new(r"^v\d+\.\d+\.\d+(-[a-z0-9]+(\.[0-9]+)?)?$")
        .expect("Invalid semver regex");
    if !re.is_match(version) {
        return Err(anyhow!(
            "Invalid release version: '{}'. Must follow vMAJOR.MINOR.PATCH[-PRE] pattern.",
            version
        ));
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
            let now = Utc::now().to_rfc3339();
            Ok(Release {
                metadata: ReleaseMetadata {
                    id: version.clone(),
                    version,
                    status: ReleaseStatus::default(),
                    created: now.clone(),
                    updated: now,
                    supported: None,
                    target_date: None,
                    feature_ids: Vec::new(),
                    adr_ids: Vec::new(),
                    breaking_changes: Vec::new(),
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

/// Create a new release file in `.ship/project/releases/`.
pub fn create_release(project_dir: PathBuf, version: &str, body: &str) -> Result<PathBuf> {
    validate_version(version)?;

    let releases_dir = crate::project::upcoming_releases_dir(&project_dir);
    fs::create_dir_all(&releases_dir)?;

    let template = crate::project::read_template(&project_dir, "release")?;
    let mut release = Release::from_markdown(&template)?;
    let now = Utc::now().to_rfc3339();

    release.metadata.id = version.to_string();
    release.metadata.version = version.to_string();
    release.metadata.created = now.clone();
    release.metadata.updated = now;

    if !body.is_empty() {
        release.body = body.to_string();
    }

    let base = version.to_string();
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

/// Resolve a release filename against known release locations.
/// Supports:
/// - `v0-1-0-alpha.md` (top-level historical or upcoming fallback)
/// - `upcoming/v0-1-0-alpha.md` (explicit upcoming path)
pub fn find_release_path(project_dir: &Path, file_name: &str) -> Result<PathBuf> {
    let direct = crate::project::releases_dir(project_dir).join(file_name);
    if direct.exists() {
        return Ok(direct);
    }

    let fallback_upcoming = crate::project::upcoming_releases_dir(project_dir).join(file_name);
    if fallback_upcoming.exists() {
        return Ok(fallback_upcoming);
    }

    Err(anyhow!("Release not found: {}", file_name))
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
    release.metadata.updated = Utc::now().to_rfc3339();
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

/// List all release files in `.ship/project/releases/`.
pub fn list_releases(project_dir: PathBuf) -> Result<Vec<ReleaseEntry>> {
    let releases_dir = crate::project::releases_dir(&project_dir);
    if !releases_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    let scan_dirs = vec![
        releases_dir.clone(),
        crate::project::upcoming_releases_dir(&project_dir),
    ];
    for scan_dir in scan_dirs {
        if !scan_dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&scan_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !(path.is_file() && path.extension().map_or(false, |e| e == "md")) {
                continue;
            }
            let file_name = path
                .strip_prefix(&releases_dir)
                .ok()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();
            if file_name.is_empty()
                || file_name.ends_with("TEMPLATE.md")
                || file_name.ends_with("README.md")
            {
                continue;
            }
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
