use super::db::{delete_note_db, get_note_db, insert_note_db, list_notes_db, update_note_db};
use super::types::{Note, NoteEntry, NoteScope};
use anyhow::Result;
use chrono::Utc;
use std::path::Path;

pub fn create_note(
    scope: NoteScope,
    ship_dir: Option<&Path>,
    title: &str,
    content: &str,
) -> Result<Note> {
    let id = runtime::gen_nanoid();
    let now = Utc::now().to_rfc3339();
    let note = Note {
        id,
        title: title.to_string(),
        content: content.to_string(),
        tags: vec![],
        scope,
        created_at: now.clone(),
        updated_at: now,
    };
    insert_note_db(scope, ship_dir, &note)?;
    Ok(note)
}

pub fn get_note_by_id(scope: NoteScope, ship_dir: Option<&Path>, id: &str) -> Result<Note> {
    get_note_db(scope, ship_dir, id)?.ok_or_else(|| anyhow::anyhow!("Note not found: {}", id))
}

pub fn list_notes(scope: NoteScope, ship_dir: Option<&Path>) -> Result<Vec<NoteEntry>> {
    let notes = list_notes_db(scope, ship_dir)?;
    Ok(notes
        .into_iter()
        .map(|n| NoteEntry {
            id: n.id,
            title: n.title,
            scope: n.scope,
            updated: n.updated_at,
        })
        .collect())
}

pub fn update_note(
    scope: NoteScope,
    ship_dir: Option<&Path>,
    id: &str,
    title: &str,
    content: &str,
) -> Result<Note> {
    update_note_db(scope, ship_dir, id, title, content)?;
    get_note_by_id(scope, ship_dir, id)
}

pub fn update_note_content(
    scope: NoteScope,
    ship_dir: Option<&Path>,
    id: &str,
    content: &str,
) -> Result<Note> {
    let note = get_note_by_id(scope, ship_dir, id)?;
    update_note_db(scope, ship_dir, id, &note.title, content)?;
    get_note_by_id(scope, ship_dir, id)
}

pub fn delete_note(scope: NoteScope, ship_dir: Option<&Path>, id: &str) -> Result<()> {
    delete_note_db(scope, ship_dir, id)
}
