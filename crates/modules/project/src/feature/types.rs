use serde::{Deserialize, Serialize};
use specta::Type;

// ─── Status ───────────────────────────────────────────────────────────────────

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

// ─── Core types ───────────────────────────────────────────────────────────────

pub use runtime::agents::config::FeatureAgentConfig;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureMetadata {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created: String,
    pub updated: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<FeatureAgentConfig>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureTodo {
    pub id: String,
    pub text: String,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureCriterion {
    pub id: String,
    pub text: String,
    pub met: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Feature {
    pub metadata: FeatureMetadata,
    pub body: String,
    #[serde(default)]
    pub todos: Vec<FeatureTodo>,
    #[serde(default)]
    pub criteria: Vec<FeatureCriterion>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureEntry {
    pub id: String,
    pub file_name: String,
    pub path: String,
    pub status: FeatureStatus,
    pub feature: Feature,
}
