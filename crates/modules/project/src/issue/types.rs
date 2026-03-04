use serde::{Deserialize, Serialize};
use specta::Type;

// ─── Priority ─────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
#[serde(rename_all = "lowercase")]
pub enum IssuePriority {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for IssuePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssuePriority::Critical => write!(f, "critical"),
            IssuePriority::High => write!(f, "high"),
            IssuePriority::Medium => write!(f, "medium"),
            IssuePriority::Low => write!(f, "low"),
        }
    }
}

// ─── Status ───────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
#[serde(rename_all = "kebab-case")]
pub enum IssueStatus {
    Backlog,
    InProgress,
    Blocked,
    Done,
}

impl Default for IssueStatus {
    fn default() -> Self {
        IssueStatus::Backlog
    }
}

impl std::fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueStatus::Backlog => write!(f, "backlog"),
            IssueStatus::InProgress => write!(f, "in-progress"),
            IssueStatus::Blocked => write!(f, "blocked"),
            IssueStatus::Done => write!(f, "done"),
        }
    }
}

impl std::str::FromStr for IssueStatus {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "backlog" => Ok(IssueStatus::Backlog),
            "in-progress" => Ok(IssueStatus::InProgress),
            "blocked" => Ok(IssueStatus::Blocked),
            "done" => Ok(IssueStatus::Done),
            _ => Err(anyhow::anyhow!("Invalid issue status: {}", s)),
        }
    }
}

// ─── Core Types ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Type)]
pub struct IssueLink {
    #[serde(rename = "type")]
    pub type_: String,
    pub target: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct IssueMetadata {
    pub id: String,
    pub title: String,
    pub created: String,
    pub updated: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<IssuePriority>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
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
    pub id: String,
    pub file_name: String,
    pub path: String,
    pub status: IssueStatus,
    pub issue: Issue,
}
