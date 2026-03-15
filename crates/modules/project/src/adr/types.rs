use serde::{Deserialize, Serialize};
use specta::Type;

// ─── Status ───────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
#[serde(rename_all = "lowercase")]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Rejected,
    Superseded,
    Deprecated,
}

impl Default for AdrStatus {
    fn default() -> Self {
        AdrStatus::Proposed
    }
}

impl std::fmt::Display for AdrStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdrStatus::Proposed => write!(f, "proposed"),
            AdrStatus::Accepted => write!(f, "accepted"),
            AdrStatus::Rejected => write!(f, "rejected"),
            AdrStatus::Superseded => write!(f, "superseded"),
            AdrStatus::Deprecated => write!(f, "deprecated"),
        }
    }
}

impl std::str::FromStr for AdrStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "proposed" => Ok(AdrStatus::Proposed),
            "accepted" => Ok(AdrStatus::Accepted),
            "rejected" => Ok(AdrStatus::Rejected),
            "superseded" => Ok(AdrStatus::Superseded),
            "deprecated" => Ok(AdrStatus::Deprecated),
            _ => Err(()),
        }
    }
}

// ─── Core types ───────────────────────────────────────────────────────────────

/// The full ADR document returned to the UI / MCP.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ADR {
    pub metadata: AdrMetadata,
    /// Background, constraints, requirements — freeform text.
    pub context: String,
    /// The committed decision register — markdown narrative.
    pub decision: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AdrMetadata {
    pub id: String,
    pub title: String,
    pub date: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes_id: Option<String>,
}

/// List entry / summary — used by list commands and MCP resources.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AdrEntry {
    pub id: String,
    pub file_name: String,
    pub path: String,
    pub status: AdrStatus,
    pub adr: ADR,
}
