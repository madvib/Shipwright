//! Note CRUD — local sqlite store, cloud sync via synced_at.

use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use std::path::Path;

use crate::db::{block_on, open_db};
use crate::gen_nanoid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub branch: Option<String>,
    pub synced_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

const COLS: &str = "id, title, content, tags_json, branch, synced_at, created_at, updated_at";

pub fn create_note(
    _ship_dir: &Path,
    title: &str,
    content: &str,
    tags: Vec<String>,
    branch: Option<&str>,
) -> Result<Note> {
    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    let id = gen_nanoid();
    let tags_json = serde_json::to_string(&tags)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO note (id, title, content, tags_json, branch, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(title)
        .bind(content)
        .bind(&tags_json)
        .bind(branch)
        .bind(&now)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(Note {
        id,
        title: title.to_string(),
        content: content.to_string(),
        tags,
        branch: branch.map(str::to_string),
        synced_at: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_note(
    _ship_dir: &Path,
    id: &str,
    title: Option<&str>,
    content: Option<&str>,
    tags: Option<Vec<String>>,
) -> Result<()> {
    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    // Fetch current, apply partial update.
    let current =
        get_note_impl(&mut conn, id)?.ok_or_else(|| anyhow::anyhow!("note {id} not found"))?;
    let new_title = title.unwrap_or(&current.title);
    let new_content = content.unwrap_or(&current.content);
    let new_tags = tags.unwrap_or(current.tags);
    let tags_json = serde_json::to_string(&new_tags)?;
    block_on(async {
        sqlx::query(
            "UPDATE note SET title = ?, content = ?, tags_json = ?, updated_at = ? WHERE id = ?",
        )
        .bind(new_title)
        .bind(new_content)
        .bind(&tags_json)
        .bind(&now)
        .bind(id)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn get_note(_ship_dir: &Path, id: &str) -> Result<Option<Note>> {
    let mut conn = open_db()?;
    get_note_impl(&mut conn, id)
}

fn get_note_impl(conn: &mut sqlx::SqliteConnection, id: &str) -> Result<Option<Note>> {
    let row = block_on(async {
        sqlx::query(&format!("SELECT {COLS} FROM note WHERE id = ?"))
            .bind(id)
            .fetch_optional(conn)
            .await
    })?;
    Ok(row.map(|r| row_to_note(&r)))
}

pub fn list_notes(_ship_dir: &Path, branch: Option<&str>) -> Result<Vec<Note>> {
    let mut conn = open_db()?;
    let rows = match branch {
        Some(b) => block_on(async {
            sqlx::query(&format!(
                "SELECT {COLS} FROM note WHERE branch = ? ORDER BY updated_at DESC"
            ))
            .bind(b)
            .fetch_all(&mut conn)
            .await
        })?,
        None => block_on(async {
            sqlx::query(&format!("SELECT {COLS} FROM note ORDER BY updated_at DESC"))
                .fetch_all(&mut conn)
                .await
        })?,
    };
    Ok(rows.iter().map(row_to_note).collect())
}

pub fn delete_note(_ship_dir: &Path, id: &str) -> Result<()> {
    let mut conn = open_db()?;
    block_on(async {
        sqlx::query("DELETE FROM note WHERE id = ?")
            .bind(id)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

/// Mark a note as synced to cloud. Called by sync layer after successful push.
pub fn mark_synced(_ship_dir: &Path, id: &str) -> Result<()> {
    let mut conn = open_db()?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query("UPDATE note SET synced_at = ? WHERE id = ?")
            .bind(&now)
            .bind(id)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

fn row_to_note(row: &sqlx::sqlite::SqliteRow) -> Note {
    Note {
        id: row.get(0),
        title: row.get(1),
        content: row.get(2),
        tags: serde_json::from_str::<Vec<String>>(&row.get::<String, _>(3)).unwrap_or_default(),
        branch: row.get(4),
        synced_at: row.get(5),
        created_at: row.get(6),
        updated_at: row.get(7),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ensure_db;
    use crate::project::init_project;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        (tmp, ship_dir)
    }

    #[test]
    fn test_create_and_get_note() {
        let (_tmp, ship_dir) = setup();
        let note = create_note(
            &ship_dir,
            "My Note",
            "hello world",
            vec!["tag1".to_string()],
            None,
        )
        .unwrap();
        let got = get_note(&ship_dir, &note.id).unwrap().unwrap();
        assert_eq!(got.title, "My Note");
        assert_eq!(got.tags, vec!["tag1".to_string()]);
        assert!(got.synced_at.is_none());
    }

    #[test]
    fn test_update_note() {
        let (_tmp, ship_dir) = setup();
        let note = create_note(&ship_dir, "Draft", "initial", vec![], None).unwrap();
        update_note(&ship_dir, &note.id, Some("Final"), None, None).unwrap();
        let got = get_note(&ship_dir, &note.id).unwrap().unwrap();
        assert_eq!(got.title, "Final");
        assert_eq!(got.content, "initial");
    }

    #[test]
    fn test_list_notes_filtered_by_branch() {
        let (_tmp, ship_dir) = setup();
        create_note(&ship_dir, "Note A", "", vec![], Some("feat/x")).unwrap();
        create_note(&ship_dir, "Note B", "", vec![], Some("main")).unwrap();
        create_note(&ship_dir, "Note C", "", vec![], None).unwrap();
        let feat_notes = list_notes(&ship_dir, Some("feat/x")).unwrap();
        assert_eq!(feat_notes.len(), 1);
        let all = list_notes(&ship_dir, None).unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_delete_note() {
        let (_tmp, ship_dir) = setup();
        let note = create_note(&ship_dir, "Temp", "", vec![], None).unwrap();
        delete_note(&ship_dir, &note.id).unwrap();
        assert!(get_note(&ship_dir, &note.id).unwrap().is_none());
    }

    #[test]
    fn test_mark_synced() {
        let (_tmp, ship_dir) = setup();
        let note = create_note(&ship_dir, "Syncable", "body", vec![], None).unwrap();
        assert!(note.synced_at.is_none());
        mark_synced(&ship_dir, &note.id).unwrap();
        let got = get_note(&ship_dir, &note.id).unwrap().unwrap();
        assert!(got.synced_at.is_some());
    }

    // ── Priority 4 gap tests ──────────────────────────────────────────────────

    /// Create a note and an ADR, list both, delete the note, verify the ADR remains intact.
    #[test]
    fn test_note_and_adr_independence_on_delete() {
        let (_tmp, ship_dir) = setup();

        let note = create_note(&ship_dir, "A note", "note body", vec![], None).unwrap();
        let adr = crate::db::adrs::create_adr(
            &ship_dir,
            "An ADR",
            "some context",
            "some decision",
            "proposed",
        )
        .unwrap();

        // Both exist.
        assert_eq!(list_notes(&ship_dir, None).unwrap().len(), 1);
        assert_eq!(crate::db::adrs::list_adrs(&ship_dir).unwrap().len(), 1);

        // Delete the note.
        delete_note(&ship_dir, &note.id).unwrap();
        assert!(get_note(&ship_dir, &note.id).unwrap().is_none());

        // ADR is untouched.
        let remaining_adrs = crate::db::adrs::list_adrs(&ship_dir).unwrap();
        assert_eq!(remaining_adrs.len(), 1);
        assert_eq!(remaining_adrs[0].id, adr.id);

        // Create a second note; ADR count is still 1.
        create_note(&ship_dir, "New note", "", vec![], None).unwrap();
        assert_eq!(crate::db::adrs::list_adrs(&ship_dir).unwrap().len(), 1);
        assert_eq!(list_notes(&ship_dir, None).unwrap().len(), 1);
    }

    /// Delete an ADR and verify the note is not affected.
    #[test]
    fn test_adr_delete_does_not_affect_notes() {
        let (_tmp, ship_dir) = setup();

        create_note(&ship_dir, "Persistent note", "body", vec![], None).unwrap();
        let adr = crate::db::adrs::create_adr(&ship_dir, "Transient ADR", "ctx", "dec", "proposed")
            .unwrap();

        crate::db::adrs::delete_adr(&ship_dir, &adr.id).unwrap();

        assert!(
            crate::db::adrs::get_adr(&ship_dir, &adr.id)
                .unwrap()
                .is_none()
        );
        assert_eq!(list_notes(&ship_dir, None).unwrap().len(), 1);
    }
}
