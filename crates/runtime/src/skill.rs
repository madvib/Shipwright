use crate::fs_util::write_atomic;
use crate::project::get_global_dir;
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Origin of a skill document.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type)]
#[serde(rename_all = "kebab-case")]
pub enum SkillSource {
    Custom,
    Builtin,
    AiGenerated,
    Community,
    Imported,
}

impl Default for SkillSource {
    fn default() -> Self {
        SkillSource::Custom
    }
}

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
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    /// The command template. Use `$ARGUMENTS` as a placeholder for user input.
    pub content: String,
    #[serde(default)]
    pub source: SkillSource,
}

#[derive(Serialize, Deserialize, Debug)]
struct SkillFrontmatter {
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    source: SkillSource,
}

fn skills_dir(project_dir: &Path) -> PathBuf {
    project_dir.join("agents").join("skills")
}

fn user_skills_dir() -> Result<PathBuf> {
    Ok(get_global_dir()?.join("agents").join("skills"))
}

fn skill_dir(project_dir: &Path, id: &str) -> PathBuf {
    skills_dir(project_dir).join(id)
}

fn user_skill_dir(id: &str) -> Result<PathBuf> {
    Ok(user_skills_dir()?.join(id))
}

fn parse_skill(dir: &Path) -> Result<Skill> {
    let config_path = dir.join("skill.toml");
    let content_path = dir.join("index.md");

    if !config_path.exists() {
        return Err(anyhow!("Missing skill.toml in {}", dir.display()));
    }

    let config_raw = fs::read_to_string(&config_path)?;
    let fm: SkillFrontmatter = toml::from_str(&config_raw)
        .with_context(|| format!("Failed to parse skill.toml in {}", dir.display()))?;

    let content = if content_path.exists() {
        fs::read_to_string(&content_path)?
    } else {
        String::new()
    };

    Ok(Skill {
        id: fm.id,
        name: fm.name,
        description: fm.description,
        version: fm.version,
        author: fm.author,
        content,
        source: fm.source,
    })
}

fn write_skill(dir: &Path, skill: &Skill) -> Result<()> {
    fs::create_dir_all(dir)?;
    let fm = SkillFrontmatter {
        id: skill.id.clone(),
        name: skill.name.clone(),
        description: skill.description.clone(),
        version: skill.version.clone(),
        author: skill.author.clone(),
        source: skill.source.clone(),
    };
    let config_path = dir.join("skill.toml");
    let config_content = toml::to_string(&fm)?;
    write_atomic(&config_path, config_content)?;

    let content_path = dir.join("index.md");
    write_atomic(&content_path, skill.content.clone())?;
    Ok(())
}

// ─── CRUD ─────────────────────────────────────────────────────────────────────

fn list_skills_from_dir(dir: &Path) -> Result<Vec<Skill>> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut skills = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            match parse_skill(&path) {
                Ok(s) => skills.push(s),
                Err(e) => eprintln!("warn: skipping {}: {}", path.display(), e),
            }
        }
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

pub fn list_skills(project_dir: &Path) -> Result<Vec<Skill>> {
    list_skills_from_dir(&skills_dir(project_dir))
}

pub fn list_user_skills() -> Result<Vec<Skill>> {
    let dir = user_skills_dir()?;
    list_skills_from_dir(&dir)
}

/// Returns merged user + project skills keyed by id.
/// Project-scoped skills override user-scoped skills with the same id.
pub fn list_effective_skills(project_dir: &Path) -> Result<Vec<Skill>> {
    let mut by_id: HashMap<String, Skill> = HashMap::new();
    for skill in list_user_skills()? {
        by_id.insert(skill.id.clone(), skill);
    }
    for skill in list_skills(project_dir)? {
        by_id.insert(skill.id.clone(), skill);
    }
    let mut merged = by_id.into_values().collect::<Vec<_>>();
    merged.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(merged)
}

pub fn get_skill(project_dir: &Path, id: &str) -> Result<Skill> {
    let dir = skill_dir(project_dir, id);
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    parse_skill(&dir)
}

pub fn get_user_skill(id: &str) -> Result<Skill> {
    let dir = user_skill_dir(id)?;
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    parse_skill(&dir)
}

/// Resolve a skill by checking project scope first, then user scope.
pub fn get_effective_skill(project_dir: &Path, id: &str) -> Result<Skill> {
    let local_dir = skill_dir(project_dir, id);
    if local_dir.exists() {
        return parse_skill(&local_dir);
    }

    let global_dir = user_skill_dir(id)?;
    if global_dir.exists() {
        return parse_skill(&global_dir);
    }

    Err(anyhow!("Skill '{}' not found in project or user scope", id))
}

pub fn create_skill(project_dir: &Path, id: &str, name: &str, content: &str) -> Result<Skill> {
    let dir = skill_dir(project_dir, id);
    if dir.exists() {
        return Err(anyhow!("Skill '{}' already exists", id));
    }
    let skill = Skill {
        id: id.to_string(),
        name: name.to_string(),
        description: None,
        version: None,
        author: None,
        content: content.to_string(),
        source: SkillSource::Custom,
    };
    write_skill(&dir, &skill)?;
    // Register in project config so checkout hook includes this skill automatically.
    let mut config = crate::config::get_config(Some(project_dir.to_path_buf()))?;
    if !config.agent.skills.contains(&id.to_string()) {
        config.agent.skills.push(id.to_string());
        crate::config::save_config(&config, Some(project_dir.to_path_buf()))?;
    }
    Ok(skill)
}

pub fn create_user_skill(id: &str, name: &str, content: &str) -> Result<Skill> {
    let dir = user_skill_dir(id)?;
    if dir.exists() {
        return Err(anyhow!("Skill '{}' already exists", id));
    }
    let skill = Skill {
        id: id.to_string(),
        name: name.to_string(),
        description: None,
        version: None,
        author: None,
        content: content.to_string(),
        source: SkillSource::Custom,
    };
    write_skill(&dir, &skill)?;
    Ok(skill)
}

pub fn update_skill(
    project_dir: &Path,
    id: &str,
    name: Option<&str>,
    content: Option<&str>,
) -> Result<Skill> {
    let dir = skill_dir(project_dir, id);
    let mut skill = parse_skill(&dir)?;
    if let Some(n) = name {
        skill.name = n.to_string();
    }
    if let Some(c) = content {
        skill.content = c.to_string();
    }
    write_skill(&dir, &skill)?;
    Ok(skill)
}

pub fn update_user_skill(id: &str, name: Option<&str>, content: Option<&str>) -> Result<Skill> {
    let dir = user_skill_dir(id)?;
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    let mut skill = parse_skill(&dir)?;
    if let Some(n) = name {
        skill.name = n.to_string();
    }
    if let Some(c) = content {
        skill.content = c.to_string();
    }
    write_skill(&dir, &skill)?;
    Ok(skill)
}

pub fn delete_skill(project_dir: &Path, id: &str) -> Result<()> {
    let dir = skill_dir(project_dir, id);
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    fs::remove_dir_all(dir)?;
    Ok(())
}

pub fn delete_user_skill(id: &str) -> Result<()> {
    let dir = user_skill_dir(id)?;
    if !dir.exists() {
        return Err(anyhow!("Skill '{}' not found", id));
    }
    fs::remove_dir_all(dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn create_and_get_round_trip() -> Result<()> {
        let tmp = tempdir()?;
        let project_dir = tmp.path();
        let s = create_skill(
            project_dir,
            "review",
            "Code Review",
            "Review this: $ARGUMENTS",
        )?;
        assert_eq!(s.id, "review");
        assert_eq!(s.source, SkillSource::Custom);
        let got = get_skill(project_dir, "review")?;
        assert_eq!(got.content, "Review this: $ARGUMENTS");
        assert!(skill_dir(project_dir, "review").is_dir());
        assert!(
            skill_dir(project_dir, "review")
                .join("skill.toml")
                .is_file()
        );
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
