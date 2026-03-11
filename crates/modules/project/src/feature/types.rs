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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
#[serde(rename_all = "kebab-case")]
pub enum FeatureDocStatus {
    NotStarted,
    Draft,
    Reviewed,
    Published,
}

impl Default for FeatureDocStatus {
    fn default() -> Self {
        FeatureDocStatus::NotStarted
    }
}

impl std::fmt::Display for FeatureDocStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureDocStatus::NotStarted => write!(f, "not-started"),
            FeatureDocStatus::Draft => write!(f, "draft"),
            FeatureDocStatus::Reviewed => write!(f, "reviewed"),
            FeatureDocStatus::Published => write!(f, "published"),
        }
    }
}

impl std::str::FromStr for FeatureDocStatus {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "not-started" => Ok(FeatureDocStatus::NotStarted),
            "draft" => Ok(FeatureDocStatus::Draft),
            "reviewed" => Ok(FeatureDocStatus::Reviewed),
            "published" => Ok(FeatureDocStatus::Published),
            _ => Err(anyhow::anyhow!("Invalid feature doc status: {}", s)),
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
    pub active_target_id: Option<String>,
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
pub struct FeatureDeclarationCriterion {
    pub text: String,
    pub has_pass_fail_condition: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureDeclaration {
    pub narrative: String,
    #[serde(default)]
    pub acceptance_criteria: Vec<FeatureDeclarationCriterion>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureStatusCheck {
    pub text: String,
    pub passing: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureObservedStatus {
    pub narrative: String,
    #[serde(default)]
    pub checks: Vec<FeatureStatusCheck>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureDelta {
    pub declaration_missing: bool,
    pub status_missing: bool,
    #[serde(default)]
    pub unmet_acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub failing_checks: Vec<String>,
    #[serde(default)]
    pub missing_pass_fail_criteria: Vec<String>,
    pub drift_score: u32,
    #[serde(default)]
    pub actionable_items: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureModel {
    pub declaration: FeatureDeclaration,
    pub status: FeatureObservedStatus,
    pub delta: FeatureDelta,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureEntry {
    pub id: String,
    pub file_name: String,
    pub path: String,
    pub status: FeatureStatus,
    pub feature: Feature,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct FeatureDocumentation {
    pub feature_id: String,
    pub status: FeatureDocStatus,
    pub content: String,
    pub revision: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_verified_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
