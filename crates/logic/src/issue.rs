use crate::config::get_config;
use crate::project::sanitize_file_name;
use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssueMetadata {
    pub title: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub links: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Issue {
    #[serde(flatten)]
    pub metadata: IssueMetadata,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssueEntry {
    pub file_name: String,
    pub status: String,
    pub path: String,
    pub issue: Issue,
}

impl Issue {
    pub fn to_markdown(&self) -> Result<String> {
        let yaml = serde_yaml::to_string(&self.metadata)?;
        Ok(format!("---\n{}---\n\n{}", yaml, self.description))
    }

    pub fn from_markdown(content: &str) -> Result<Self> {
        if !content.starts_with("---\n") {
            return Err(anyhow!("Invalid issue format: missing frontmatter start"));
        }

        let parts: Vec<&str> = content.splitn(3, "---\n").collect();
        if parts.len() < 3 {
            return Err(anyhow!("Invalid issue format: missing frontmatter end"));
        }

        let yaml = parts[1];
        let description = parts[2].trim().to_string();
        let metadata: IssueMetadata = serde_yaml::from_str(yaml)?;

        Ok(Issue {
            metadata,
            description,
        })
    }
}

pub fn create_issue(
    project_dir: PathBuf,
    title: &str,
    description: &str,
    status: &str,
) -> Result<PathBuf> {
    let issue = Issue {
        metadata: IssueMetadata {
            title: title.to_string(),
            status: status.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            links: Vec::new(),
        },
        description: description.to_string(),
    };

    let file_name = format!("{}.md", sanitize_file_name(title));
    let file_path = project_dir.join("Issues").join(status).join(&file_name);

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = issue.to_markdown()?;
    fs::write(&file_path, content).context("Failed to write issue file")?;

    Ok(file_path)
}

pub fn get_issue(path: PathBuf) -> Result<Issue> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read issue: {}", path.display()))?;

    if path.extension().map_or(false, |e| e == "json") {
        // Fallback for legacy JSON
        let issue: Issue = serde_json::from_str(&content)?;
        return Ok(issue);
    }

    Issue::from_markdown(&content)
}

pub fn update_issue(path: PathBuf, mut issue: Issue) -> Result<()> {
    issue.metadata.updated_at = Utc::now();
    let content = issue.to_markdown()?;
    fs::write(&path, content)
        .with_context(|| format!("Failed to write issue: {}", path.display()))?;
    Ok(())
}

pub fn list_issues(project_dir: PathBuf) -> Result<Vec<(String, String)>> {
    let mut issues = Vec::new();

    for status in ISSUE_STATUSES {
        let status_dir = project_dir.join("Issues").join(status);
        if status_dir.exists() {
            for entry in fs::read_dir(status_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file()
                    && (path.extension().map_or(false, |e| e == "md")
                        || path.extension().map_or(false, |e| e == "json"))
                {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        issues.push((file_name.to_string(), status.to_string()));
                    }
                }
            }
        }
    }

    Ok(issues)
}

pub fn list_issues_full(project_dir: PathBuf) -> Result<Vec<IssueEntry>> {
    let mut entries = Vec::new();

    for status in ISSUE_STATUSES {
        let status_dir = project_dir.join("Issues").join(status);
        if status_dir.exists() {
            for entry in fs::read_dir(&status_dir)? {
                let entry = entry?;
                let path = entry.path();
                let is_md = path.extension().map_or(false, |e| e == "md");
                let is_json = path.extension().map_or(false, |e| e == "json");

                if path.is_file() && (is_md || is_json) {
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    if let Ok(issue) = get_issue(path.clone()) {
                        entries.push(IssueEntry {
                            file_name,
                            status: status.to_string(),
                            path: path.to_string_lossy().to_string(),
                            issue,
                        });
                    }
                }
            }
        }
    }

    Ok(entries)
}

pub fn move_issue(
    _project_dir: PathBuf,
    path: PathBuf,
    _current_status: &str,
    new_status: &str,
) -> Result<PathBuf> {
    if !path.exists() {
        return Err(anyhow!("Issue not found: {}", path.display()));
    }

    // Determine new path
    // We assume path is [PROJECT]/.ship/Issues/[STATUS]/file.ext
    // We should probably use project_dir to be safer, but let's see where path comes from.
    // Existing move_issue used project_dir, file_name, current_status.
    // I'll update the signature slightly or keep it compatible.

    // For now, let's stick to a robust move logic that changes the extension to .md if it was .json
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let new_file_name = if file_name.ends_with(".json") {
        format!("{}.md", &file_name[..file_name.len() - 5])
    } else {
        file_name.to_string()
    };

    // Find project Issues dir by walking up from path
    let mut issues_dir = path.parent().unwrap().parent().unwrap().to_path_buf();
    let target_path = issues_dir.join(new_status).join(&new_file_name);

    if path.extension().map_or(false, |e| e == "json") {
        // Migrate on move
        let issue = get_issue(path.clone())?;
        let content = issue.to_markdown()?;
        fs::write(&target_path, content)?;
        fs::remove_file(path)?;
    } else {
        fs::rename(&path, &target_path).context("Failed to move issue file")?;
    }

    Ok(target_path)
}

pub fn delete_issue(path: PathBuf) -> Result<()> {
    fs::remove_file(&path)
        .with_context(|| format!("Failed to delete issue: {}", path.display()))?;
    Ok(())
}

pub fn add_link(file_path: PathBuf, target_path: &str) -> Result<()> {
    let mut issue = get_issue(file_path.clone())?;
    issue.metadata.links.push(target_path.to_string());
    update_issue(file_path, issue)?;
    Ok(())
}
