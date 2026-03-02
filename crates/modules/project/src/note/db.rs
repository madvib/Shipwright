use super::types::{Note, NoteScope};
use anyhow::{Result, anyhow};
use chrono::Utc;
use runtime::state_db::{block_on, open_global_connection, open_project_connection};
use std::path::Path;

pub fn open_for_scope(scope: NoteScope, ship_dir: Option<&Path>) -> Result<sqlx::SqliteConnection> {
    match scope {
        NoteScope::Project => {
            let dir = ship_dir.ok_or_else(|| anyhow!("Project notes require an active project"))?;
            open_project_connection(dir)
        }
        NoteScope::User => open_global_connection(),
    }
}

pub fn insert_note_db(scope: NoteScope, ship_dir: Option<&Path>, note: &Note) -> Result<()> {
    let mut conn = open_for_scope(scope, ship_dir)?;
    let tags_json = serde_json::to_string(&note.tags).unwrap_or_else(|_| "[]".to_string());
    let scope_str = match scope {
        NoteScope::Project => "project",
        NoteScope::User => "user",
    };
    block_on(async {
        sqlx::query(
            "INSERT INTO note (id, title, content, tags_json, scope, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               title      = excluded.title,
               content    = excluded.content,
               tags_json  = excluded.tags_json,
               updated_at = excluded.updated_at",
        )
        .bind(&note.id)
        .bind(&note.title)
        .bind(&note.content)
        .bind(&tags_json)
        .bind(scope_str)
        .bind(&note.created_at)
        .bind(&note.updated_at)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

pub fn get_note_db(scope: NoteScope, ship_dir: Option<&Path>, id: &str) -> Result<Option<Note>> {
    let mut conn = open_for_scope(scope, ship_dir)?;
    use sqlx::Row;
    let row = block_on(async {
        sqlx::query(
            "SELECT id, title, content, tags_json, created_at, updated_at
             FROM note WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.map(|r| {
        let tags_json: String = r.get(3);
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        Note {
            id: r.get(0),
            title: r.get(1),
            content: r.get(2),
            tags,
            scope,
            created_at: r.get(4),
            updated_at: r.get(5),
        }
    }))
}

pub fn list_notes_db(scope: NoteScope, ship_dir: Option<&Path>) -> Result<Vec<Note>> {
    let mut conn = open_for_scope(scope, ship_dir)?;
    use sqlx::Row;
    let rows = block_on(async {
        sqlx::query(
            "SELECT id, title, content, tags_json, created_at, updated_at
             FROM note ORDER BY updated_at DESC",
        )
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let tags_json: String = r.get(3);
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            Note {
                id: r.get(0),
                title: r.get(1),
                content: r.get(2),
                tags,
                scope,
                created_at: r.get(4),
                updated_at: r.get(5),
            }
        })
        .collect())
}

pub fn delete_note_db(scope: NoteScope, ship_dir: Option<&Path>, id: &str) -> Result<()> {
    let mut conn = open_for_scope(scope, ship_dir)?;
    block_on(async {
        sqlx::query("DELETE FROM note WHERE id = ?")
            .bind(id)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}

pub fn update_note_db(
    scope: NoteScope,
    ship_dir: Option<&Path>,
    id: &str,
    title: &str,
    content: &str,
) -> Result<()> {
    let mut conn = open_for_scope(scope, ship_dir)?;
    let now = Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query("UPDATE note SET title = ?, content = ?, updated_at = ? WHERE id = ?")
            .bind(title)
            .bind(content)
            .bind(&now)
            .bind(id)
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}
