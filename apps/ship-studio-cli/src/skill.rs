//! Skill management: list + create local project skills.

use anyhow::Result;
use std::path::Path;

use crate::paths::{agents_skills_dir, global_skills_dir};

pub fn list() -> Result<()> {
    let mut found = false;

    let project_dir = agents_skills_dir();
    if project_dir.exists() {
        let skills = collect_skill_ids(&project_dir);
        if !skills.is_empty() {
            println!("Project skills (.ship/agents/skills/):");
            for id in &skills { println!("  - {}", id); }
            found = true;
        }
    }

    let global_dir = global_skills_dir();
    if global_dir.exists() {
        let skills = collect_skill_ids(&global_dir);
        if !skills.is_empty() {
            println!("Global skills (~/.ship/skills/):");
            for id in &skills { println!("  - {}", id); }
            found = true;
        }
    }

    if !found {
        println!("No skills installed.");
        println!("Create one with: ship skill create <id>");
    }
    Ok(())
}

pub fn create(id: &str, name: Option<&str>, description: Option<&str>) -> Result<()> {
    if !is_valid_id(id) {
        anyhow::bail!("Invalid skill ID '{}'. Use lowercase letters, digits, and hyphens.", id);
    }
    let skill_dir = agents_skills_dir().join(id);
    if skill_dir.exists() {
        anyhow::bail!("Skill '{}' already exists at {}", id, skill_dir.display());
    }
    std::fs::create_dir_all(&skill_dir)?;

    let name = name.unwrap_or(id);
    let description = description.unwrap_or("Describe when this skill should activate.");
    let content = format!(
"---
name: {name}
description: {description}
---

## Instructions

<!-- Add instructions for the agent here -->
");
    std::fs::write(skill_dir.join("SKILL.md"), content)?;
    println!("✓ created skill '{}' at {}", id, skill_dir.display());
    println!("  Edit: {}/SKILL.md", skill_dir.display());
    Ok(())
}

pub fn remove(id: &str, global: bool) -> Result<()> {
    let dir = if global { global_skills_dir().join(id) } else { agents_skills_dir().join(id) };
    if !dir.exists() {
        anyhow::bail!("Skill '{}' not found at {}", id, dir.display());
    }
    std::fs::remove_dir_all(&dir)?;
    println!("✓ removed skill '{}'", id);
    Ok(())
}

fn collect_skill_ids(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else { return vec![]; };
    let mut ids: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir() && e.path().join("SKILL.md").exists())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    ids.sort();
    ids
}

fn is_valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_skill_ids() {
        assert!(is_valid_id("rust-expert"));
        assert!(is_valid_id("test-driven"));
        assert!(is_valid_id("skill123"));
    }

    #[test]
    fn invalid_skill_ids() {
        assert!(!is_valid_id(""));
        assert!(!is_valid_id("Rust Expert"));
        assert!(!is_valid_id("rust_expert"));
        assert!(!is_valid_id("rust/expert"));
    }

    #[test]
    fn collect_skill_ids_from_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        for id in &["beta-skill", "alpha-skill"] {
            let skill_dir = dir.join(id);
            std::fs::create_dir_all(&skill_dir).unwrap();
            std::fs::write(skill_dir.join("SKILL.md"), "content").unwrap();
        }
        // Dir without SKILL.md should be ignored
        std::fs::create_dir_all(dir.join("no-skill-md")).unwrap();

        let ids = collect_skill_ids(dir);
        assert_eq!(ids, vec!["alpha-skill", "beta-skill"]);
    }
}
