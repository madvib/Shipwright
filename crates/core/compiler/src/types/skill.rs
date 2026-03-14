use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SkillSource {
    #[default]
    Custom,
    Builtin,
    AiGenerated,
    Community,
    Imported,
}

/// A skill / slash command. Stored as `agents/skills/<id>/SKILL.md` (agentskills.io spec).
/// The compiler receives pre-loaded `Skill` values — it does not read files.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Skill {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    pub content: String,
    #[serde(default)]
    pub source: SkillSource,
}

pub fn is_valid_skill_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 {
        return false;
    }
    if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        return false;
    }
    name.chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}
