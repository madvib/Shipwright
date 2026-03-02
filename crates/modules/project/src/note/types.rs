use serde::{Deserialize, Serialize};
use specta::Type;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NoteScope {
    Project,
    User,
}

impl FromStr for NoteScope {
    type Err = anyhow::Error;
    fn from_str(value: &str) -> anyhow::Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "project" => Ok(NoteScope::Project),
            "user" | "global" => Ok(NoteScope::User),
            other => Err(anyhow::anyhow!(
                "Unknown note scope '{}'. Use: project, user",
                other
            )),
        }
    }
}

/// Full note document — body is raw markdown, no frontmatter.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub scope: NoteScope,
    pub created_at: String,
    pub updated_at: String,
}

/// Lightweight list entry.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct NoteEntry {
    pub id: String,
    pub title: String,
    pub scope: NoteScope,
    pub updated: String,
}
