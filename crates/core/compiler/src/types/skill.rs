use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub license: Option<String>,
    #[serde(default)]
    pub compatibility: Option<String>,
    /// Parsed from the space-delimited `allowed-tools` frontmatter key.
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// Arbitrary key-value metadata from the `metadata` frontmatter block.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_skill_names() {
        assert!(is_valid_skill_name("my-skill"));
        assert!(is_valid_skill_name("review-pr"));
        assert!(is_valid_skill_name("deploy"));
        assert!(is_valid_skill_name("a1b2c3"));
        assert!(is_valid_skill_name("x"));
        // 64-char boundary
        assert!(is_valid_skill_name(&"a".repeat(64)));
    }

    #[test]
    fn invalid_skill_names() {
        assert!(!is_valid_skill_name(""));
        assert!(!is_valid_skill_name(&"a".repeat(65)));
        assert!(!is_valid_skill_name("-leading"));
        assert!(!is_valid_skill_name("trailing-"));
        assert!(!is_valid_skill_name("double--hyphen"));
        assert!(!is_valid_skill_name("UpperCase"));
        assert!(!is_valid_skill_name("has space"));
    }

    #[test]
    fn skill_new_fields_default() {
        let skill = Skill {
            id: "test".to_string(),
            name: "test".to_string(),
            description: None,
            license: None,
            compatibility: None,
            allowed_tools: vec![],
            metadata: HashMap::new(),
            content: String::new(),
            source: SkillSource::Custom,
        };
        assert!(skill.license.is_none());
        assert!(skill.compatibility.is_none());
        assert!(skill.allowed_tools.is_empty());
        assert!(skill.metadata.is_empty());
    }
}
