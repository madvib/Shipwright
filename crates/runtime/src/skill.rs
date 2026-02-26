use crate::fs_util::write_atomic;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};

/// A callable slash command / skill (→ `.claude/commands/<id>.md`).
/// Different from a Prompt (system instructions): skills are invoked
/// explicitly by the user with `/skill-name [args]` and can use `$ARGUMENTS`.
/// Stored as `.ship/agents/skills/<id>.md` with TOML frontmatter.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Skill {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// The command template. Use `$ARGUMENTS` as a placeholder for user input.
    pub content: String,
    /// Origin: "custom", "ai-generated", "community", "imported"
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "custom".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
struct SkillFrontmatter {
    id: String,
    name: String,
    description: Option<String>,
    #[serde(default = "default_source")]
    source: String,
}

fn skills_dir(project_dir: &Path) -> PathBuf {
    project_dir.join("agents").join("skills")
}

fn skill_path(project_dir: &Path, id: &str) -> PathBuf {
    skills_dir(project_dir).join(format!("{}.md", id))
}

fn parse_skill(path: &Path) -> Result<Skill> {
    let raw = fs::read_to_string(path)?;
    if let Some(body) = raw.strip_prefix("+++") {
        let parts: Vec<&str> = body.splitn(2, "+++").collect();
        if parts.len() == 2 {
            let fm: SkillFrontmatter = toml::from_str(parts[0].trim())?;
            return Ok(Skill {
                id: fm.id,
                name: fm.name,
                description: fm.description,
                content: parts[1].trim().to_string(),
                source: fm.source,
            });
        }
    }
    Err(anyhow!("Could not parse skill file: {}", path.display()))
}

fn write_skill(path: &Path, skill: &Skill) -> Result<()> {
    let fm = SkillFrontmatter {
        id: skill.id.clone(),
        name: skill.name.clone(),
        description: skill.description.clone(),
        source: skill.source.clone(),
    };
    let content = format!(
        "+++\n{}\n+++\n\n{}",
        toml::to_string(&fm)?.trim(),
        skill.content.trim()
    );
    write_atomic(path, content)
}

// ─── CRUD ─────────────────────────────────────────────────────────────────────

pub fn list_skills(project_dir: &Path) -> Result<Vec<Skill>> {
    let dir = skills_dir(project_dir);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut skills = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            match parse_skill(&path) {
                Ok(s) => skills.push(s),
                Err(e) => eprintln!("warn: skipping {}: {}", path.display(), e),
            }
        }
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

pub fn get_skill(project_dir: &Path, id: &str) -> Result<Skill> {
    let path = skill_path(project_dir, id);
    if !path.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    parse_skill(&path)
}

pub fn create_skill(project_dir: &Path, id: &str, name: &str, content: &str) -> Result<Skill> {
    fs::create_dir_all(skills_dir(project_dir))?;
    let path = skill_path(project_dir, id);
    if path.exists() {
        return Err(anyhow!("Skill '{}' already exists", id));
    }
    let skill = Skill {
        id: id.to_string(),
        name: name.to_string(),
        description: None,
        content: content.to_string(),
        source: "custom".to_string(),
    };
    write_skill(&path, &skill)?;
    Ok(skill)
}

pub fn update_skill(project_dir: &Path, id: &str, name: Option<&str>, content: Option<&str>) -> Result<Skill> {
    let path = skill_path(project_dir, id);
    let mut skill = parse_skill(&path)?;
    if let Some(n) = name { skill.name = n.to_string(); }
    if let Some(c) = content { skill.content = c.to_string(); }
    write_skill(&path, &skill)?;
    Ok(skill)
}

pub fn delete_skill(project_dir: &Path, id: &str) -> Result<()> {
    let path = skill_path(project_dir, id);
    if !path.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    fs::remove_file(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn create_and_get_round_trip() -> Result<()> {
        let tmp = tempdir()?;
        let s = create_skill(tmp.path(), "review", "Code Review", "Review this: $ARGUMENTS")?;
        assert_eq!(s.id, "review");
        assert_eq!(s.source, "custom");
        let got = get_skill(tmp.path(), "review")?;
        assert_eq!(got.content, "Review this: $ARGUMENTS");
        Ok(())
    }

    #[test]
    fn list_returns_all_skills() -> Result<()> {
        let tmp = tempdir()?;
        create_skill(tmp.path(), "a", "A", "content a")?;
        create_skill(tmp.path(), "b", "B", "content b")?;
        assert_eq!(list_skills(tmp.path())?.len(), 2);
        Ok(())
    }

    #[test]
    fn list_empty_dir_returns_empty() -> Result<()> {
        let tmp = tempdir()?;
        assert!(list_skills(tmp.path())?.is_empty());
        Ok(())
    }

    #[test]
    fn update_skill_persists() -> Result<()> {
        let tmp = tempdir()?;
        create_skill(tmp.path(), "s", "Old", "old")?;
        update_skill(tmp.path(), "s", Some("New"), Some("new $ARGUMENTS"))?;
        let reloaded = get_skill(tmp.path(), "s")?;
        assert_eq!(reloaded.name, "New");
        assert_eq!(reloaded.content, "new $ARGUMENTS");
        Ok(())
    }

    #[test]
    fn delete_removes_skill() -> Result<()> {
        let tmp = tempdir()?;
        create_skill(tmp.path(), "gone", "Gone", "x")?;
        delete_skill(tmp.path(), "gone")?;
        assert!(get_skill(tmp.path(), "gone").is_err());
        Ok(())
    }

    #[test]
    fn duplicate_rejected() -> Result<()> {
        let tmp = tempdir()?;
        create_skill(tmp.path(), "dup", "Dup", "x")?;
        assert!(create_skill(tmp.path(), "dup", "Dup2", "y").is_err());
        Ok(())
    }
}
