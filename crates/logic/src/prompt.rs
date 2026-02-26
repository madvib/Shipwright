use crate::fs_util::write_atomic;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};

/// A named agent instruction set (system prompt / CLAUDE.md equivalent).
/// Stored as `<project_dir>/agents/prompts/<id>.md` with TOML frontmatter.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Markdown content — becomes CLAUDE.md, GEMINI.md, or the codex instructions field
    pub content: String,
    /// Origin: "custom", "ai-generated", "community", "imported"
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "custom".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PromptFrontmatter {
    id: String,
    name: String,
    description: Option<String>,
    #[serde(default = "default_source")]
    source: String,
}

fn prompts_dir(project_dir: &Path) -> PathBuf {
    project_dir.join("agents").join("prompts")
}

fn prompt_path(project_dir: &Path, id: &str) -> PathBuf {
    prompts_dir(project_dir).join(format!("{}.md", id))
}

fn parse_prompt(path: &Path) -> Result<Prompt> {
    let raw = fs::read_to_string(path)?;
    if let Some(body) = raw.strip_prefix("+++") {
        let parts: Vec<&str> = body.splitn(2, "+++").collect();
        if parts.len() == 2 {
            let fm: PromptFrontmatter = toml::from_str(parts[0].trim())?;
            return Ok(Prompt {
                id: fm.id,
                name: fm.name,
                description: fm.description,
                content: parts[1].trim().to_string(),
                source: fm.source,
            });
        }
    }
    Err(anyhow!("Could not parse prompt file: {}", path.display()))
}

fn write_prompt(path: &Path, prompt: &Prompt) -> Result<()> {
    let fm = PromptFrontmatter {
        id: prompt.id.clone(),
        name: prompt.name.clone(),
        description: prompt.description.clone(),
        source: prompt.source.clone(),
    };
    let content = format!(
        "+++\n{}\n+++\n\n{}",
        toml::to_string(&fm)?.trim(),
        prompt.content.trim()
    );
    write_atomic(path, content)
}

// ─── CRUD ─────────────────────────────────────────────────────────────────────

pub fn list_prompts(project_dir: &Path) -> Result<Vec<Prompt>> {
    let dir = prompts_dir(project_dir);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut prompts = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            match parse_prompt(&path) {
                Ok(p) => prompts.push(p),
                Err(e) => eprintln!("warn: skipping {}: {}", path.display(), e),
            }
        }
    }
    prompts.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(prompts)
}

pub fn get_prompt(project_dir: &Path, id: &str) -> Result<Prompt> {
    let path = prompt_path(project_dir, id);
    if !path.exists() {
        return Err(anyhow!("Prompt '{}' not found", id));
    }
    parse_prompt(&path)
}

pub fn create_prompt(project_dir: &Path, id: &str, name: &str, content: &str) -> Result<Prompt> {
    let dir = prompts_dir(project_dir);
    fs::create_dir_all(&dir)?;
    let path = prompt_path(project_dir, id);
    if path.exists() {
        return Err(anyhow!("Prompt '{}' already exists", id));
    }
    let prompt = Prompt {
        id: id.to_string(),
        name: name.to_string(),
        description: None,
        content: content.to_string(),
        source: "custom".to_string(),
    };
    write_prompt(&path, &prompt)?;
    Ok(prompt)
}

pub fn update_prompt(
    project_dir: &Path,
    id: &str,
    name: Option<&str>,
    content: Option<&str>,
) -> Result<Prompt> {
    let path = prompt_path(project_dir, id);
    let mut prompt = parse_prompt(&path)?;
    if let Some(n) = name {
        prompt.name = n.to_string();
    }
    if let Some(c) = content {
        prompt.content = c.to_string();
    }
    write_prompt(&path, &prompt)?;
    Ok(prompt)
}

pub fn delete_prompt(project_dir: &Path, id: &str) -> Result<()> {
    let path = prompt_path(project_dir, id);
    if !path.exists() {
        return Err(anyhow!("Prompt '{}' not found", id));
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
        let p = create_prompt(tmp.path(), "expert-dev", "Expert Dev", "You are an expert.")?;
        assert_eq!(p.id, "expert-dev");
        assert_eq!(p.name, "Expert Dev");
        assert_eq!(p.content, "You are an expert.");

        let got = get_prompt(tmp.path(), "expert-dev")?;
        assert_eq!(got.id, p.id);
        assert_eq!(got.name, p.name);
        assert_eq!(got.content, p.content);
        Ok(())
    }

    #[test]
    fn list_prompts_returns_all() -> Result<()> {
        let tmp = tempdir()?;
        create_prompt(tmp.path(), "alpha", "Alpha", "content a")?;
        create_prompt(tmp.path(), "beta", "Beta", "content b")?;
        let prompts = list_prompts(tmp.path())?;
        assert_eq!(prompts.len(), 2);
        let ids: Vec<&str> = prompts.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"alpha"));
        assert!(ids.contains(&"beta"));
        Ok(())
    }

    #[test]
    fn list_prompts_empty_dir_returns_empty() -> Result<()> {
        let tmp = tempdir()?;
        let prompts = list_prompts(tmp.path())?;
        assert!(prompts.is_empty());
        Ok(())
    }

    #[test]
    fn update_prompt_name_and_content() -> Result<()> {
        let tmp = tempdir()?;
        create_prompt(tmp.path(), "p1", "Old Name", "old content")?;
        let updated = update_prompt(tmp.path(), "p1", Some("New Name"), Some("new content"))?;
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.content, "new content");
        // Verify persisted
        let reloaded = get_prompt(tmp.path(), "p1")?;
        assert_eq!(reloaded.name, "New Name");
        assert_eq!(reloaded.content, "new content");
        Ok(())
    }

    #[test]
    fn delete_prompt_removes_file() -> Result<()> {
        let tmp = tempdir()?;
        create_prompt(tmp.path(), "bye", "Bye", "content")?;
        assert!(get_prompt(tmp.path(), "bye").is_ok());
        delete_prompt(tmp.path(), "bye")?;
        assert!(get_prompt(tmp.path(), "bye").is_err());
        Ok(())
    }

    #[test]
    fn duplicate_create_rejected() -> Result<()> {
        let tmp = tempdir()?;
        create_prompt(tmp.path(), "dup", "Dup", "c")?;
        let result = create_prompt(tmp.path(), "dup", "Dup2", "c2");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
        Ok(())
    }

    #[test]
    fn get_nonexistent_returns_error() -> Result<()> {
        let tmp = tempdir()?;
        let result = get_prompt(tmp.path(), "nope");
        assert!(result.is_err());
        Ok(())
    }
}
