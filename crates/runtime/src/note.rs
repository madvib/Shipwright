use crate::fs_util::write_atomic;
use crate::project::sanitize_file_name;
use crate::{EventAction, EventEntity, append_event};
use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NoteScope {
    Project,
    User,
}

impl FromStr for NoteScope {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "project" => Ok(NoteScope::Project),
            "user" | "global" => Ok(NoteScope::User),
            other => Err(anyhow!(
                "Unknown note scope '{}'. Use: project, user",
                other
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct NoteMetadata {
    #[serde(default)]
    pub id: String,
    pub title: String,
    pub created: String,
    pub updated: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Type)]
pub struct Note {
    pub metadata: NoteMetadata,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct NoteEntry {
    pub file_name: String,
    pub path: String,
    pub title: String,
    pub updated: String,
}

fn unique_path(dir: &Path, base: &str) -> PathBuf {
    let candidate = dir.join(format!("{}.md", base));
    if !candidate.exists() {
        return candidate;
    }
    let mut n = 2u32;
    loop {
        let candidate = dir.join(format!("{}-{}.md", base, n));
        if !candidate.exists() {
            return candidate;
        }
        n += 1;
    }
}

fn ship_dir_from_note_path(path: &Path) -> Option<PathBuf> {
    crate::project::ship_dir_from_path(path)
}

fn note_dir(scope: NoteScope, project_dir: Option<&Path>) -> Result<PathBuf> {
    match scope {
        NoteScope::Project => {
            let ship_dir = project_dir
                .ok_or_else(|| anyhow!("Project note scope requires an active project"))?;
            Ok(crate::project::notes_dir(ship_dir))
        }
        NoteScope::User => Ok(crate::project::get_global_dir()?.join("notes")),
    }
}

fn validate_title(title: &str) -> Result<()> {
    if title.trim().is_empty() {
        return Err(anyhow!("Note title cannot be empty"));
    }
    Ok(())
}

impl Note {
    pub fn to_markdown(&self) -> Result<String> {
        let toml_str =
            toml::to_string(&self.metadata).context("Failed to serialise note metadata as TOML")?;
        Ok(format!("+++\n{}+++\n\n{}", toml_str, self.body))
    }

    pub fn from_markdown(content: &str) -> Result<Self> {
        if content.starts_with("+++\n") {
            let rest = &content[4..]; // skip "+++\n"
            let end = rest
                .find("\n+++")
                .ok_or_else(|| anyhow!("Invalid note format: missing closing +++"))?;
            let toml_str = &rest[..end];
            let body = rest[end + 4..].trim_start_matches('\n').to_string();
            let metadata: NoteMetadata =
                toml::from_str(toml_str).context("Failed to parse note TOML frontmatter")?;
            Ok(Note { metadata, body })
        } else {
            let now = Utc::now().to_rfc3339();
            let title = content
                .lines()
                .find(|line| line.starts_with("# "))
                .map(|line| line.trim_start_matches("# ").trim().to_string())
                .unwrap_or_default();
            Ok(Note {
                metadata: NoteMetadata {
                    id: crate::gen_nanoid(),
                    title,
                    created: now.clone(),
                    updated: now,
                    tags: Vec::new(),
                },
                body: content.to_string(),
            })
        }
    }
}

pub fn create_note(
    scope: NoteScope,
    project_dir: Option<PathBuf>,
    title: &str,
    body: &str,
) -> Result<PathBuf> {
    validate_title(title)?;
    let dir = note_dir(scope, project_dir.as_deref())?;
    fs::create_dir_all(&dir)?;

    let ship_path = project_dir
        .clone()
        .unwrap_or_else(|| crate::project::get_global_dir().unwrap_or_default());
    let template = crate::project::read_template(&ship_path, "note")?;
    let mut note = Note::from_markdown(&template)?;
    let now = Utc::now().to_rfc3339();

    note.metadata.id = crate::gen_nanoid();
    note.metadata.title = title.to_string();
    note.metadata.created = now.clone();
    note.metadata.updated = now;

    if !body.is_empty() {
        note.body = body.to_string();
    }

    let base = sanitize_file_name(title);
    let file_path = unique_path(&dir, &base);
    write_atomic(&file_path, note.to_markdown()?)?;

    if let (NoteScope::Project, Some(ship_dir)) = (scope, project_dir) {
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();
        append_event(
            &ship_dir,
            "logic",
            EventEntity::Note,
            EventAction::Create,
            file_name,
            Some(format!("title={}", title)),
        )?;
    }

    Ok(file_path)
}

pub fn get_note(path: PathBuf) -> Result<Note> {
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read note: {}", path.display()))?;
    Note::from_markdown(&content)
}

pub fn get_note_raw(path: PathBuf) -> Result<String> {
    fs::read_to_string(&path).with_context(|| format!("Failed to read note: {}", path.display()))
}

pub fn update_note(path: PathBuf, body: &str) -> Result<()> {
    let mut note = get_note(path.clone())?;
    note.metadata.updated = Utc::now().to_rfc3339();
    note.body = body.to_string();
    write_atomic(&path, note.to_markdown()?)
        .with_context(|| format!("Failed to write note: {}", path.display()))?;

    if let Some(ship_dir) = ship_dir_from_note_path(&path) {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();
        append_event(
            &ship_dir,
            "logic",
            EventEntity::Note,
            EventAction::Update,
            file_name,
            Some(format!("title={}", note.metadata.title)),
        )?;
    }
    Ok(())
}

pub fn list_notes(scope: NoteScope, project_dir: Option<PathBuf>) -> Result<Vec<NoteEntry>> {
    let dir = note_dir(scope, project_dir.as_deref())?;
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("")
                .to_string();
            if file_name == "TEMPLATE.md" || file_name == "README.md" {
                continue;
            }
            if let Ok(note) = get_note(path.clone()) {
                entries.push(NoteEntry {
                    file_name,
                    path: path.to_string_lossy().to_string(),
                    title: note.metadata.title,
                    updated: note.metadata.updated.clone(),
                });
            }
        }
    }
    Ok(entries)
}

pub fn note_path_for_scope(
    scope: NoteScope,
    project_dir: Option<PathBuf>,
    file_name: &str,
) -> Result<PathBuf> {
    Ok(note_dir(scope, project_dir.as_deref())?.join(file_name))
}
