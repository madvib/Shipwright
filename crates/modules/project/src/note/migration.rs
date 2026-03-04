use super::db::{get_note_db, insert_note_db};
use super::types::{Note, NoteScope};
use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LegacyNoteMetadata {
    #[serde(default)]
    pub id: String,
    pub title: String,
    pub created: String,
    pub updated: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub fn import_notes_from_files(scope: NoteScope, ship_dir: Option<&Path>) -> Result<usize> {
    let notes_dir = match scope {
        NoteScope::Project => {
            let d = ship_dir.ok_or_else(|| anyhow!("Project scope requires ship_dir"))?;
            runtime::project::notes_dir(d)
        }
        NoteScope::User => runtime::project::get_global_dir()?.join("notes"),
    };
    if !notes_dir.exists() {
        return Ok(0);
    }

    let mut imported = 0usize;
    for entry in fs::read_dir(&notes_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
            let fname = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if fname == "TEMPLATE.md" || fname == "README.md" {
                continue;
            }

            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read note: {}", path.display()))?;

            let (metadata, body) = parse_legacy_note(&content)?;

            // Use the metadata ID if present, otherwise fallback to filename slug
            let id = if !metadata.id.is_empty() {
                metadata.id
            } else {
                fname.trim_end_matches(".md").to_string()
            };

            if get_note_db(scope, ship_dir, &id)?.is_some() {
                continue;
            }

            let note = Note {
                id,
                title: metadata.title,
                content: body,
                tags: metadata.tags,
                scope: NoteScope::Project,
                created_at: metadata.created,
                updated_at: metadata.updated,
            };
            insert_note_db(scope, ship_dir, &note)?;
            imported += 1;
        }
    }
    Ok(imported)
}

fn parse_legacy_note(content: &str) -> Result<(LegacyNoteMetadata, String)> {
    if content.starts_with("+++\n") {
        let rest = &content[4..];
        let end = rest
            .find("\n+++")
            .ok_or_else(|| anyhow!("Invalid note format: missing closing +++"))?;
        let toml_str = &rest[..end];
        let body = rest[end + 4..].trim_start_matches('\n').to_string();
        let metadata: LegacyNoteMetadata =
            toml::from_str(toml_str).context("Failed to parse note TOML frontmatter")?;
        Ok((metadata, body))
    } else {
        let now = Utc::now().to_rfc3339();
        let title = content
            .lines()
            .find(|line| line.starts_with("# "))
            .map(|line| line.trim_start_matches("# ").trim().to_string())
            .unwrap_or_else(|| "Untitled Note".to_string());
        Ok((
            LegacyNoteMetadata {
                id: "".to_string(), // Will fallback to filename
                title,
                created: now.clone(),
                updated: now,
                tags: Vec::new(),
            },
            content.to_string(),
        ))
    }
}
