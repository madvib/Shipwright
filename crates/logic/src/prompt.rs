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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PromptFrontmatter {
    id: String,
    name: String,
    description: Option<String>,
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
    };
    write_prompt(&path, &prompt)?;
    Ok(prompt)
}

pub fn update_prompt(project_dir: &Path, id: &str, name: Option<&str>, content: Option<&str>) -> Result<Prompt> {
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
