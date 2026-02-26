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

/// A typed link between issues or to external resources.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Type)]
pub struct IssueLink {
    #[serde(rename = "type")]
    pub type_: String,
    pub target: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct IssueMetadata {
    #[serde(default)]
    pub id: String,
    pub title: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<String>,
    #[serde(default)]
    pub links: Vec<IssueLink>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Issue {
    #[serde(flatten)]
    pub metadata: IssueMetadata,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct IssueEntry {
    pub file_name: String,
    pub status: String,
    pub path: String,
    pub issue: Issue,
}

fn ship_dir_from_issue_path(path: &Path) -> Option<PathBuf> {
    // .ship/issues/<status>/<file>.md -> go up three levels
    path.parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
}

fn persist_issue(path: &Path, issue: &Issue) -> Result<()> {
    let content = issue.to_markdown()?;
    write_atomic(path, content)
        .with_context(|| format!("Failed to write issue: {}", path.display()))
}

// ─── Validation ───────────────────────────────────────────────────────────────

fn validate_title(title: &str) -> Result<()> {
    if title.trim().is_empty() {
        return Err(anyhow!("Issue title cannot be empty"));
    }
    Ok(())
}

fn validate_status(status: &str) -> Result<()> {
    if status.trim().is_empty() {
        return Err(anyhow!("Status cannot be empty"));
    }
    if status.contains('/') || status.contains('\\') || status.contains("..") {
        return Err(anyhow!(
            "Invalid status '{}': must not contain path separators",
            status
        ));
    }
    Ok(())
}

// ─── Serialisation ────────────────────────────────────────────────────────────

impl Issue {
    pub fn to_markdown(&self) -> Result<String> {
        let toml_str = toml::to_string(&self.metadata)
            .context("Failed to serialise issue metadata as TOML")?;
        Ok(format!("+++\n{}+++\n\n{}", toml_str, self.description))
    }

    /// Parse both new TOML (`+++`) and legacy YAML (`---`) frontmatter.
    pub fn from_markdown(content: &str) -> Result<Self> {
        if content.starts_with("+++\n") {
            Self::from_toml_markdown(content)
        } else if content.starts_with("---\n") {
            Self::from_yaml_markdown_legacy(content)
        } else {
            Err(anyhow!("Invalid issue format: missing frontmatter start"))
        }
    }

    fn from_toml_markdown(content: &str) -> Result<Self> {
        let rest = &content[4..]; // skip leading "+++\n"
        let end = rest
            .find("\n+++")
            .ok_or_else(|| anyhow!("Invalid issue format: missing closing +++"))?;
        let toml_str = &rest[..end];
        let description = rest[end + 4..].trim_start_matches('\n').to_string();
        let metadata: IssueMetadata =
            toml::from_str(toml_str).context("Failed to parse issue TOML frontmatter")?;
        Ok(Issue {
            metadata,
            description,
        })
    }

    /// Minimal YAML reader for the old `---` format — avoids keeping serde_yaml.
    fn from_yaml_markdown_legacy(content: &str) -> Result<Self> {
        let parts: Vec<&str> = content.splitn(3, "---\n").collect();
        if parts.len() < 3 {
            return Err(anyhow!(
                "Invalid legacy issue format: incomplete frontmatter"
            ));
        }
        let yaml = parts[1];
        let description = parts[2].trim_start_matches('\n').to_string();

        let mut title = String::new();
        let mut created = Utc::now();
        let mut updated = Utc::now();

        for line in yaml.lines() {
            if let Some(v) = line.strip_prefix("title: ") {
                title = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("created_at: ") {
                if let Ok(dt) = v.trim().parse::<DateTime<Utc>>() {
                    created = dt;
                }
            } else if let Some(v) = line.strip_prefix("updated_at: ") {
                if let Ok(dt) = v.trim().parse::<DateTime<Utc>>() {
                    updated = dt;
                }
            }
        }

        Ok(Issue {
            metadata: IssueMetadata {
                title,
                created,
                updated,
                ..Default::default()
            },
            description,
        })
    }
}

// ─── File helpers ─────────────────────────────────────────────────────────────

/// Return a path that doesn't collide with existing files in `dir`.
/// If `dir/base.md` exists, tries `dir/base-2.md`, `dir/base-3.md`, etc.
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

pub fn create_issue(
    project_dir: PathBuf,
    title: &str,
    description: &str,
    status: &str,
) -> Result<PathBuf> {
    validate_title(title)?;
    validate_status(status)?;

    let issue = Issue {
        metadata: IssueMetadata {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            created: Utc::now(),
            updated: Utc::now(),
            ..Default::default()
        },
        description: description.to_string(),
    };

    let base = sanitize_file_name(title);
    let status_dir = project_dir.join("issues").join(status);
    fs::create_dir_all(&status_dir)?;

    let file_path = unique_path(&status_dir, &base);
    write_atomic(&file_path, issue.to_markdown()?)?;
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    append_event(
        &project_dir,
        "logic",
        EventEntity::Issue,
        EventAction::Create,
        file_name,
        Some(format!("title={} status={}", title, status)),
    )?;
    Ok(file_path)
}

pub fn get_issue(path: PathBuf) -> Result<Issue> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read issue: {}", path.display()))?;
    Issue::from_markdown(&content)
}

pub fn update_issue(path: PathBuf, mut issue: Issue) -> Result<()> {
    issue.metadata.updated = Utc::now();
    persist_issue(&path, &issue)?;
    if let Some(project_dir) = ship_dir_from_issue_path(&path) {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        append_event(
            &project_dir,
            "logic",
            EventEntity::Issue,
            EventAction::Update,
            file_name,
            Some(format!("title={}", issue.metadata.title)),
        )?;
    }
    Ok(())
}

pub fn list_issues(project_dir: PathBuf) -> Result<Vec<(String, String)>> {
    let mut issues = Vec::new();
    let issues_dir = project_dir.join("issues");
    if !issues_dir.exists() {
        return Ok(issues);
    }
    for status_entry in fs::read_dir(&issues_dir)? {
        let status_entry = status_entry?;
        let status_path = status_entry.path();
        if !status_path.is_dir() {
            continue;
        }
        let status = status_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        for entry in fs::read_dir(&status_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "md") {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    issues.push((file_name.to_string(), status.clone()));
                }
            }
        }
    }
    Ok(issues)
}

pub fn list_issues_full(project_dir: PathBuf) -> Result<Vec<IssueEntry>> {
    let mut entries = Vec::new();
    let issues_dir = project_dir.join("issues");
    if !issues_dir.exists() {
        return Ok(entries);
    }
    for status_entry in fs::read_dir(&issues_dir)? {
        let status_entry = status_entry?;
        let status_path = status_entry.path();
        if !status_path.is_dir() {
            continue;
        }
        let status = status_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        for entry in fs::read_dir(&status_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "md") {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                if let Ok(issue) = get_issue(path.clone()) {
                    entries.push(IssueEntry {
                        file_name,
                        status: status.clone(),
                        path: path.to_string_lossy().to_string(),
                        issue,
                    });
                }
            }
        }
    }
    Ok(entries)
}

pub fn move_issue(
    project_dir: PathBuf,
    path: PathBuf,
    _current_status: &str,
    new_status: &str,
) -> Result<PathBuf> {
    validate_status(new_status)?;
    if !path.exists() {
        return Err(anyhow!("Issue not found: {}", path.display()));
    }

    let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
    let issues_dir = path.parent().unwrap().parent().unwrap().to_path_buf();
    let target_dir = issues_dir.join(new_status);
    fs::create_dir_all(&target_dir)?;

    // Resolve collisions in target directory
    let base = file_name.trim_end_matches(".md");
    let target_path = unique_path(&target_dir, base);

    fs::rename(&path, &target_path).context("Failed to move issue file")?;
    let from_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let to_name = target_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    append_event(
        &project_dir,
        "logic",
        EventEntity::Issue,
        EventAction::Move,
        to_name,
        Some(format!("from={} to_status={}", from_name, new_status)),
    )?;
    Ok(target_path)
}

pub fn delete_issue(path: PathBuf) -> Result<()> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    fs::remove_file(&path)
        .with_context(|| format!("Failed to delete issue: {}", path.display()))?;
    if let Some(project_dir) = ship_dir_from_issue_path(&path) {
        append_event(
            &project_dir,
            "logic",
            EventEntity::Issue,
            EventAction::Delete,
            file_name,
            None,
        )?;
    }
    Ok(())
}

pub fn append_note(path: PathBuf, note: &str) -> Result<()> {
    let mut issue = get_issue(path.clone())?;
    let title = issue.metadata.title.clone();
    issue.metadata.updated = Utc::now();
    if !issue.description.is_empty() && !issue.description.ends_with('\n') {
        issue.description.push('\n');
    }
    issue.description.push('\n');
    issue.description.push_str(note.trim_end());
    issue.description.push('\n');
    persist_issue(&path, &issue)?;
    if let Some(project_dir) = ship_dir_from_issue_path(&path) {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        append_event(
            &project_dir,
            "logic",
            EventEntity::Issue,
            EventAction::Note,
            file_name,
            Some(format!("title={}", title)),
        )?;
    }
    Ok(())
}

pub fn add_link(file_path: PathBuf, link_type: &str, target: &str) -> Result<()> {
    let mut issue = get_issue(file_path.clone())?;
    let title = issue.metadata.title.clone();
    issue.metadata.updated = Utc::now();
    issue.metadata.links.push(IssueLink {
        type_: link_type.to_string(),
        target: target.to_string(),
    });
    persist_issue(&file_path, &issue)?;
    if let Some(project_dir) = ship_dir_from_issue_path(&file_path) {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        append_event(
            &project_dir,
            "logic",
            EventEntity::Issue,
            EventAction::Link,
            file_name,
            Some(format!(
                "type={} target={} title={}",
                link_type, target, title
            )),
        )?;
    }
    Ok(())
}

// ─── Migration ────────────────────────────────────────────────────────────────

/// Assign UUIDs to any TOML issues that have an empty `id` field.
pub fn backfill_issue_ids(project_dir: &PathBuf) -> Result<usize> {
    let issues_dir = project_dir.join("issues");
    if !issues_dir.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for status_entry in fs::read_dir(&issues_dir)? {
        let status_path = status_entry?.path();
        if !status_path.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&status_path)? {
            let path = entry?.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "md") {
                if let Ok(mut issue) = get_issue(path.clone()) {
                    if issue.metadata.id.is_empty() {
                        issue.metadata.id = Uuid::new_v4().to_string();
                        let content = issue.to_markdown()?;
                        write_atomic(&path, content)?;
                        count += 1;
                    }
                }
            }
        }
    }
    Ok(count)
}

/// Convert all legacy YAML-frontmatter issues in a project to TOML in-place.
pub fn migrate_yaml_issues(project_dir: &PathBuf) -> Result<usize> {
    let issues_dir = project_dir.join("issues");
    if !issues_dir.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for status_entry in fs::read_dir(&issues_dir)? {
        let status_path = status_entry?.path();
        if !status_path.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&status_path)? {
            let path = entry?.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "md") {
                let content = fs::read_to_string(&path)?;
                if content.starts_with("---\n") {
                    if let Ok(issue) = Issue::from_yaml_markdown_legacy(&content) {
                        let new_content = issue.to_markdown()?;
                        write_atomic(&path, new_content)?;
                        count += 1;
                    }
                }
            }
        }
    }
    Ok(count)
}
