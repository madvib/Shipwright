use std::collections::HashMap;

use crate::types::Skill;

use super::provider::ProviderDescriptor;

pub(super) fn build_skill_files(
    desc: &ProviderDescriptor,
    skills: &[Skill],
) -> HashMap<String, String> {
    let Some(base) = desc.skills_dir.base_path() else {
        return HashMap::new();
    };
    skills
        .iter()
        .filter(|skill| !skill.content.trim().is_empty())
        .map(|skill| {
            let path = format!("{}/{}/SKILL.md", base, skill.id);
            let content = format_skill_file(skill);
            (path, content)
        })
        .collect()
}

pub(super) fn format_skill_file(skill: &Skill) -> String {
    let description = skill
        .description
        .as_deref()
        .unwrap_or("No description provided.");
    format!(
        "---\nname: {}\ndescription: {}\n---\n\n{}",
        skill.id, description, skill.content
    )
}
