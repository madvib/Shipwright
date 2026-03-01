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

#[derive(Serialize, Debug, Clone, Default, Type)]
pub struct FeatureMcpRef {
    pub id: String,
}

impl<'de> serde::Deserialize<'de> for FeatureMcpRef {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = FeatureMcpRef;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("an mcp server id string or {id = \"...\"} table")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(FeatureMcpRef { id: v.to_string() })
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                let mut id = None;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "id" {
                        id = Some(map.next_value()?);
                    } else {
                        map.next_value::<serde::de::IgnoredAny>()?;
                    }
                }
                Ok(FeatureMcpRef {
                    id: id.ok_or_else(|| serde::de::Error::missing_field("id"))?,
                })
            }
        }
        d.deserialize_any(Visitor)
    }
}

#[derive(Serialize, Debug, Clone, Default, Type)]
pub struct FeatureSkillRef {
    pub id: String,
}

impl<'de> serde::Deserialize<'de> for FeatureSkillRef {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = FeatureSkillRef;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a skill id string or {id = \"...\"}  table")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(FeatureSkillRef { id: v.to_string() })
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                let mut id = None;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "id" {
                        id = Some(map.next_value()?);
                    } else {
                        map.next_value::<serde::de::IgnoredAny>()?;
                    }
                }
                Ok(FeatureSkillRef {
                    id: id.ok_or_else(|| serde::de::Error::missing_field("id"))?,
                })
            }
        }
        d.deserialize_any(Visitor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct FeatureAgentConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cost_per_session: Option<f64>,
    #[serde(default)]
    pub mcp_servers: Vec<FeatureMcpRef>,
    #[serde(default)]
    pub skills: Vec<FeatureSkillRef>,
    /// Providers to generate config for. Empty = inherit from project-level providers.
    /// Alpha supports: "claude", "gemini", "codex"
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
#[serde(rename_all = "kebab-case")]
pub enum FeatureStatus {
    Planned,
    InProgress,
    Implemented,
    Deprecated,
}

impl Default for FeatureStatus {
    fn default() -> Self {
        FeatureStatus::Planned
    }
}

impl std::fmt::Display for FeatureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureStatus::Planned => write!(f, "planned"),
            FeatureStatus::InProgress => write!(f, "in-progress"),
            FeatureStatus::Implemented => write!(f, "implemented"),
            FeatureStatus::Deprecated => write!(f, "deprecated"),
        }
    }
}

impl std::str::FromStr for FeatureStatus {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "planned" => Ok(FeatureStatus::Planned),
            "in-progress" => Ok(FeatureStatus::InProgress),
            "implemented" => Ok(FeatureStatus::Implemented),
            "deprecated" => Ok(FeatureStatus::Deprecated),
            _ => Err(anyhow::anyhow!("Invalid feature status: {}", s)),
        }
    }
}

impl FeatureStatus {
    pub fn from_path(path: &std::path::Path) -> Self {
        path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .and_then(|s| s.parse::<Self>().ok())
            .unwrap_or(FeatureStatus::Planned)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureMetadata {
    #[serde(default)]
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created: String,
    pub updated: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<FeatureAgentConfig>,
    #[serde(default)]
    pub adr_ids: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
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
    pub status: FeatureStatus,
    pub release_id: Option<String>,
    pub spec_id: Option<String>,
    pub branch: Option<String>,
    pub updated: String,
    pub description: Option<String>,
}

fn ship_dir_from_feature_path(path: &Path) -> Option<PathBuf> {
    crate::project::ship_dir_from_path(path)
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
            let now = Utc::now().to_rfc3339();
            Ok(Feature {
                metadata: FeatureMetadata {
                    id: String::new(),
                    title,
                    created: now.clone(),
                    updated: now,
                    owner: None,
                    release_id: None,
                    spec_id: None,
                    branch: None,
                    description: None,
                    agent: None,
                    adr_ids: Vec::new(),
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

/// Strip TOML frontmatter (`+++ ... +++`) if present and return only the body.
/// Guards against callers (e.g. MCP tools) passing fully-formed markdown that
/// would otherwise produce double frontmatter when wrapped by `to_markdown()`.
pub fn extract_body(content: &str) -> String {
    if content.starts_with("+++\n") {
        let rest = &content[4..];
        if let Some(end) = rest.find("\n+++") {
            return rest[end + 4..].trim_start_matches('\n').to_string();
        }
    }
    content.to_string()
}
/// Create a new feature file in `.ship/project/features/`.
pub fn create_feature(
    project_dir: PathBuf,
    title: &str,
    body: &str,
    release_id: Option<&str>,
    spec_id: Option<&str>,
    branch: Option<&str>,
) -> Result<PathBuf> {
    validate_title(title)?;

    let ship_path = crate::project::get_project_dir(Some(project_dir.clone()))?;
    let template_str = crate::project::read_template(&ship_path, "feature")?;
    let mut feature =
        Feature::from_markdown(&template_str).context("Failed to parse feature template")?;

    let now = Utc::now().to_rfc3339();
    feature.metadata.id = crate::gen_nanoid();
    feature.metadata.title = title.to_string();
    feature.metadata.created = now.clone();
    feature.metadata.updated = now;

    feature.metadata.release_id = release_id
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string());
    feature.metadata.spec_id = spec_id
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string());
    feature.metadata.branch = branch
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string());

    feature.metadata.description = None;

    if !body.trim().is_empty() {
        feature.body = body.to_string();
    }

    let features_dir = crate::project::features_dir(&project_dir);
    fs::create_dir_all(&features_dir)?;

    let base = sanitize_file_name(title);
    let status_dir = features_dir.join(FeatureStatus::Planned.to_string());
    fs::create_dir_all(&status_dir)?;

    let file_path = unique_path(&status_dir, &base);
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
    feature.metadata.updated = Utc::now().to_rfc3339();
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

/// List all feature files in `.ship/project/features/`.
/// Pass `status_filter` to return only features with that status.
pub fn list_features(
    project_dir: PathBuf,
    status_filter: Option<FeatureStatus>,
) -> Result<Vec<FeatureEntry>> {
    let features_dir = crate::project::features_dir(&project_dir);
    if !features_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();

    // Check flat directory (legacy)
    search_feature_dir(&features_dir, &status_filter, &mut entries)?;

    // Check subdirectories
    for status in &["planned", "in-progress", "implemented", "deprecated"] {
        let status_dir = features_dir.join(status);
        if status_dir.exists() {
            search_feature_dir(&status_dir, &status_filter, &mut entries)?;
        }
    }

    Ok(entries)
}

fn search_feature_dir(
    dir: &Path,
    status_filter: &Option<FeatureStatus>,
    entries: &mut Vec<FeatureEntry>,
) -> Result<()> {
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
            if let Ok(feature) = get_feature(path.clone()) {
                let status_str = dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("planned");
                let status = status_str.parse::<FeatureStatus>().unwrap_or_default();

                if let Some(f) = status_filter {
                    if &status != f {
                        continue;
                    }
                }
                entries.push(FeatureEntry {
                    file_name,
                    path: path.to_string_lossy().to_string(),
                    title: feature.metadata.title,
                    status,
                    release_id: feature.metadata.release_id,
                    spec_id: feature.metadata.spec_id,
                    branch: feature.metadata.branch,
                    updated: feature.metadata.updated.clone(),
                    description: feature.metadata.description.clone(),
                });
            }
        }
    }
    Ok(())
}

pub fn find_feature_path(project_dir: &Path, file_name: &str) -> Result<PathBuf> {
    let features_dir = crate::project::features_dir(project_dir);
    for status in &["planned", "in-progress", "implemented", "deprecated"] {
        let candidate = features_dir.join(status).join(file_name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    let candidate = features_dir.join(file_name);
    if candidate.exists() {
        return Ok(candidate);
    }
    Err(anyhow!("Feature not found: {}", file_name))
}

/// Set a feature's status to `InProgress` and record its branch.
pub fn feature_start(project_dir: PathBuf, file_name: &str, branch: &str) -> Result<()> {
    let path = find_feature_path(&project_dir, file_name)?;
    let mut feature = get_feature(path.clone())?;
    feature.metadata.branch = Some(branch.to_string());
    feature.metadata.updated = Utc::now().to_rfc3339();
    let target_dir =
        crate::project::features_dir(&project_dir).join(FeatureStatus::InProgress.to_string());
    fs::create_dir_all(&target_dir)?;
    let target_path = target_dir.join(file_name);
    if path != target_path {
        fs::rename(&path, &target_path).context("Failed to move feature to in-progress")?;
    }
    write_atomic(&target_path, feature.to_markdown()?)?;
    // Index branch → feature UUID in DB for fast checkout lookup (non-fatal if DB unavailable)
    let _ = crate::state_db::set_branch_doc(&project_dir, branch, "feature", &feature.metadata.id);
    append_event(
        &project_dir,
        "logic",
        EventEntity::Feature,
        EventAction::Update,
        file_name,
        Some(format!("started branch={}", branch)),
    )?;
    Ok(())
}

/// Set a feature's status to `Implemented`.
pub fn feature_done(project_dir: PathBuf, file_name: &str) -> Result<()> {
    let path = find_feature_path(&project_dir, file_name)?;
    let mut feature = get_feature(path.clone())?;
    feature.metadata.updated = Utc::now().to_rfc3339();
    let target_dir =
        crate::project::features_dir(&project_dir).join(FeatureStatus::Implemented.to_string());
    fs::create_dir_all(&target_dir)?;
    let target_path = target_dir.join(file_name);
    if path != target_path {
        fs::rename(&path, &target_path).context("Failed to move feature to implemented")?;
    }
    write_atomic(&target_path, feature.to_markdown()?)?;
    append_event(
        &project_dir,
        "logic",
        EventEntity::Feature,
        EventAction::Update,
        file_name,
        Some("done — marked implemented".to_string()),
    )?;
    Ok(())
}
