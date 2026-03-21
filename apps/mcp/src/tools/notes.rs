use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteScope {
    Project,
    User,
}

impl std::str::FromStr for NoteScope {
    type Err = anyhow::Error;
    fn from_str(value: &str) -> anyhow::Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "project" => Ok(NoteScope::Project),
            "user" | "global" => Ok(NoteScope::User),
            other => Err(anyhow::anyhow!(
                "Unknown note scope '{}'. Use: project, user",
                other
            )),
        }
    }
}

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
    _scope: NoteScope,
    project_dir: Option<&Path>,
    id: &str,
    content: &str,
) -> String {
    let Some(dir) = project_dir else {
        return "Error: project directory required for note update".to_string();
    };
    let ship_dir = dir.join(".ship");
    match runtime::db::notes::update_note(&ship_dir, id, None, Some(content), None) {
        Ok(()) => format!("Updated note: {}", id),
        Err(e) => format!("Error updating note: {}", e),
    }
}
