use super::{OpsError, OpsResult, append_project_log, default_hooks};
use crate::{Note, NoteEntry, NoteScope};
use runtime::{EventAction, EventEntity, RuntimeHooks};
use std::path::Path;

fn project_dir_for_scope<'a>(
    scope: NoteScope,
    ship_dir: Option<&'a Path>,
) -> OpsResult<Option<&'a Path>> {
    match scope {
        NoteScope::Project => ship_dir.map(Some).ok_or_else(|| {
            OpsError::Validation("Project notes require an active project".to_string())
        }),
        NoteScope::User => Ok(None),
    }
}

fn append_note_event(
    scope: NoteScope,
    ship_dir: Option<&Path>,
    action: EventAction,
    subject: &str,
    details: Option<String>,
) -> OpsResult<()> {
    if let Some(project_dir) = project_dir_for_scope(scope, ship_dir)? {
        default_hooks()
            .append_entity_event(
                project_dir,
                "logic",
                EventEntity::Note,
                action,
                subject,
                details,
            )
            .map_err(OpsError::from)?;
    }
    Ok(())
}

pub fn create_note(
    scope: NoteScope,
    ship_dir: Option<&Path>,
    title: &str,
    content: &str,
) -> OpsResult<Note> {
    let _ = project_dir_for_scope(scope, ship_dir)?;
    if title.trim().is_empty() {
        return Err(OpsError::Validation(
            "Note title cannot be empty".to_string(),
        ));
    }
    let note = crate::note::create_note(scope, ship_dir, title, content).map_err(OpsError::from)?;
    append_note_event(
        scope,
        ship_dir,
        EventAction::Create,
        &note.id,
        Some(format!("title={}", note.title)),
    )?;
    if let Some(project_dir) = project_dir_for_scope(scope, ship_dir)? {
        append_project_log(
            project_dir,
            "note create",
            &format!("Created note: {}", note.title),
        )?;
    }
    Ok(note)
}

pub fn get_note_by_id(scope: NoteScope, ship_dir: Option<&Path>, id: &str) -> OpsResult<Note> {
    let _ = project_dir_for_scope(scope, ship_dir)?;
    crate::note::get_note_by_id(scope, ship_dir, id).map_err(OpsError::from)
}

pub fn list_notes(scope: NoteScope, ship_dir: Option<&Path>) -> OpsResult<Vec<NoteEntry>> {
    let _ = project_dir_for_scope(scope, ship_dir)?;
    crate::note::list_notes(scope, ship_dir).map_err(OpsError::from)
}

pub fn update_note(
    scope: NoteScope,
    ship_dir: Option<&Path>,
    id: &str,
    title: &str,
    content: &str,
) -> OpsResult<Note> {
    let _ = project_dir_for_scope(scope, ship_dir)?;
    if title.trim().is_empty() {
        return Err(OpsError::Validation(
            "Note title cannot be empty".to_string(),
        ));
    }
    let note =
        crate::note::update_note(scope, ship_dir, id, title, content).map_err(OpsError::from)?;
    append_note_event(
        scope,
        ship_dir,
        EventAction::Update,
        &note.id,
        Some(format!("title={}", note.title)),
    )?;
    if let Some(project_dir) = project_dir_for_scope(scope, ship_dir)? {
        append_project_log(
            project_dir,
            "note update",
            &format!("Updated note: {}", note.title),
        )?;
    }
    Ok(note)
}

pub fn update_note_content(
    scope: NoteScope,
    ship_dir: Option<&Path>,
    id: &str,
    content: &str,
) -> OpsResult<Note> {
    let _ = project_dir_for_scope(scope, ship_dir)?;
    let note =
        crate::note::update_note_content(scope, ship_dir, id, content).map_err(OpsError::from)?;
    append_note_event(
        scope,
        ship_dir,
        EventAction::Update,
        &note.id,
        Some(format!("title={}", note.title)),
    )?;
    if let Some(project_dir) = project_dir_for_scope(scope, ship_dir)? {
        append_project_log(
            project_dir,
            "note update",
            &format!("Updated note: {}", note.title),
        )?;
    }
    Ok(note)
}

pub fn delete_note(scope: NoteScope, ship_dir: Option<&Path>, id: &str) -> OpsResult<()> {
    let _ = project_dir_for_scope(scope, ship_dir)?;
    let note = crate::note::get_note_by_id(scope, ship_dir, id).map_err(OpsError::from)?;
    crate::note::delete_note(scope, ship_dir, id).map_err(OpsError::from)?;
    append_note_event(scope, ship_dir, EventAction::Delete, &note.id, None)?;
    if let Some(project_dir) = project_dir_for_scope(scope, ship_dir)? {
        append_project_log(
            project_dir,
            "note delete",
            &format!("Deleted note: {}", note.title),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_project;
    use tempfile::tempdir;

    #[test]
    fn project_note_ops_emit_note_events() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let project_dir = init_project(tmp.path().to_path_buf())?;

        let note = create_note(NoteScope::Project, Some(&project_dir), "event-test", "body")?;
        let events = runtime::list_events_since(&project_dir, 0, Some(100))?;
        assert!(events.iter().any(|event| {
            event.entity == EventEntity::Note
                && event.action == EventAction::Create
                && event.subject == note.id
        }));

        Ok(())
    }

    #[test]
    fn project_note_scope_requires_project_dir() {
        let err = create_note(NoteScope::Project, None, "missing-project", "body")
            .expect_err("expected validation error when project dir missing");
        assert!(matches!(err, OpsError::Validation(_)));
    }

    #[test]
    fn user_note_ops_work_without_project_dir() -> anyhow::Result<()> {
        let note = create_note(NoteScope::User, None, "user-note", "body")?;
        assert!(!note.id.trim().is_empty());
        let loaded = get_note_by_id(NoteScope::User, None, &note.id)?;
        assert_eq!(loaded.title, "user-note");
        Ok(())
    }
}
