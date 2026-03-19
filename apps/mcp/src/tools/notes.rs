use std::path::Path;

use ship_docs::{NoteScope, ops::note::update_note_content};

pub fn parse_note_scope(raw: Option<&str>) -> anyhow::Result<NoteScope> {
    raw.unwrap_or("project").parse::<NoteScope>()
}

pub fn create_note(
    project_dir: &Path,
    title: &str,
    content: Option<String>,
    branch: Option<&str>,
) -> String {
    let ship_dir = project_dir.join(".ship");
    let content = content.unwrap_or_default();
    match runtime::db::notes::create_note(&ship_dir, title, &content, vec![], branch) {
        Ok(note) => format!("Created note: {} (id: {})", note.title, note.id),
        Err(e) => format!("Error creating note: {}", e),
    }
}

pub fn update_note(
    scope: NoteScope,
    project_dir: Option<&Path>,
    file_name: &str,
    content: &str,
) -> String {
    match update_note_content(scope, project_dir, file_name, content) {
        Ok(note) => format!("Updated note: {}", note.title),
        Err(e) => format!("Error updating note: {}", e),
    }
}
