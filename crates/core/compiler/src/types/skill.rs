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
    /// Canonical identifier from `stable-id:` frontmatter field.
    ///
    /// When set, this is used as the key for state file lookups instead of the
    /// directory name. Allows renaming the skill directory without orphaning state.
    /// Must be a valid skill id (`[a-z0-9][a-z0-9-]*`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stable_id: Option<String>,
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
    /// Resolved variable values for template substitution.
    /// Merged from: user state (~/.ship/state/skills/{id}.json) +
    /// project state (.ship/state/skills/{id}.json) + vars.json defaults.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub vars: HashMap<String, serde_json::Value>,
    /// Artifact types this skill produces (e.g. `["html", "adr"]`).
    /// Used to infer platform event subscriptions at actor spawn time.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<String>,
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
            stable_id: None,
            description: None,
            license: None,
            compatibility: None,
            allowed_tools: vec![],
            metadata: HashMap::new(),
            content: String::new(),
            source: SkillSource::Custom,
            vars: HashMap::new(),
            artifacts: vec![],
        };
        assert!(skill.stable_id.is_none());
        assert!(skill.license.is_none());
        assert!(skill.compatibility.is_none());
        assert!(skill.allowed_tools.is_empty());
        assert!(skill.metadata.is_empty());
        assert!(skill.vars.is_empty());
        assert!(skill.artifacts.is_empty());
    }
}
