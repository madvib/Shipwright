use serde::{Deserialize, Serialize};
use specta::Type;

// ─── Status ───────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
#[serde(rename_all = "kebab-case")]
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

impl std::str::FromStr for SpecStatus {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(SpecStatus::Draft),
            "active" => Ok(SpecStatus::Active),
            "archived" => Ok(SpecStatus::Archived),
            _ => Err(anyhow::anyhow!("Invalid spec status: {}", s)),
        }
    }
}

// ─── Core Types ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct SpecMetadata {
    pub id: String,
    pub title: String,
    pub created: String,
    pub updated: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Spec {
    pub metadata: SpecMetadata,
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct SpecEntry {
    pub id: String,
    pub file_name: String,
    pub path: String,
    pub status: SpecStatus,
    pub spec: Spec,
}
