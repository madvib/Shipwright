//! Data loading from the runtime DB for the TUI.
//! All queries go through the runtime::db module -- never raw SQL here.

use anyhow::Result;
use std::path::Path;

pub use runtime::db::adrs::AdrRecord;
pub use runtime::db::jobs::Job;
pub use runtime::db::notes::Note;
pub use runtime::db::targets::{Capability, Target};
pub use runtime::events::EventRecord;

/// All valid ship config keys with human-readable labels.
pub const USER_PREF_KEYS: &[(&str, &str)] = &[
    ("identity.name", "Name"),
    ("identity.email", "Email"),
    ("defaults.provider", "Default Provider"),
    ("defaults.mode", "Default Mode"),
    ("worktrees.dir", "Worktrees Dir"),
    ("terminal.program", "Terminal"),
    ("dispatch.confirm", "Confirm Dispatch"),
    ("cloud.base_url", "Cloud URL"),
];

/// Simplified project config for display and editing.
#[derive(Debug, Default)]
pub struct ConfigSnapshot {
    pub name: String,
    pub version: String,
    pub description: String,
    pub id: String,
    /// User preferences from ~/.ship/config.toml (key, value) pairs.
    pub user_prefs: Vec<(String, String)>,
}

/// Snapshot of all data needed by the TUI. Loaded once per refresh cycle.
#[derive(Debug, Default)]
pub struct ViewData {
    pub targets: Vec<Target>,
    pub capabilities: Vec<Capability>,
    pub all_jobs: Vec<Job>,
    pub notes: Vec<Note>,
    pub adrs: Vec<AdrRecord>,
    pub events: Vec<EventRecord>,
    pub config: ConfigSnapshot,
}

/// Load all data from the platform DB. Returns Default on missing DB.
pub fn load_all(ship_dir: &Path) -> ViewData {
    ViewData {
        targets: load_targets(ship_dir),
        capabilities: load_capabilities(ship_dir),
        all_jobs: load_all_jobs(ship_dir),
        notes: load_notes(ship_dir),
        adrs: load_adrs(ship_dir),
        events: load_events(ship_dir),
        config: load_config(ship_dir),
    }
}

fn load_targets(_ship_dir: &Path) -> Vec<Target> {
    runtime::db::targets::list_targets(None).unwrap_or_default()
}

fn load_capabilities(_ship_dir: &Path) -> Vec<Capability> {
    runtime::db::targets::list_capabilities(None, None, None).unwrap_or_default()
}

fn load_all_jobs(_ship_dir: &Path) -> Vec<Job> {
    let mut jobs = runtime::db::jobs::list_jobs(None, None).unwrap_or_default();
    jobs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    jobs
}

fn load_notes(_ship_dir: &Path) -> Vec<Note> {
    runtime::db::notes::list_notes(None).unwrap_or_default()
}

fn load_adrs(_ship_dir: &Path) -> Vec<AdrRecord> {
    runtime::db::adrs::list_adrs().unwrap_or_default()
}

fn load_events(_ship_dir: &Path) -> Vec<EventRecord> {
    runtime::db::events::list_all_events().unwrap_or_default()
}

fn load_config(ship_dir: &Path) -> ConfigSnapshot {
    // Read module metadata from ship.jsonc (name, version, description).
    let manifest = std::fs::read_to_string(ship_dir.join("ship.jsonc"))
        .ok()
        .and_then(|s| compiler::manifest::ShipManifest::from_jsonc_str(&s).ok());

    // Read project ID from runtime config (it's stored in ship.jsonc but parsed via runtime).
    let project_id = runtime::get_config(Some(ship_dir.to_path_buf()))
        .map(|c| c.id)
        .unwrap_or_default();

    // Read user preferences from ~/.ship/config.toml.
    let ship_config = crate::config::ShipConfig::load();
    let user_prefs: Vec<(String, String)> = USER_PREF_KEYS
        .iter()
        .map(|(key, _label)| {
            let val = ship_config.get(key).unwrap_or_default();
            (key.to_string(), val)
        })
        .collect();

    ConfigSnapshot {
        name: manifest
            .as_ref()
            .map(|m| m.module.name.clone())
            .unwrap_or_else(|| "(unnamed)".to_string()),
        version: manifest
            .as_ref()
            .map(|m| m.module.version.clone())
            .unwrap_or_else(|| "0.0.0".to_string()),
        description: manifest
            .as_ref()
            .and_then(|m| m.module.description.clone())
            .unwrap_or_default(),
        id: if project_id.is_empty() {
            "-".to_string()
        } else {
            project_id
        },
        user_prefs,
    }
}

// -- Mutations ----------------------------------------------------------

pub fn create_note(_ship_dir: &Path, title: &str, content: &str) -> Result<Note> {
    runtime::db::notes::create_note(title, content, vec![], None)
}

pub fn update_note(
    _ship_dir: &Path,
    id: &str,
    title: Option<&str>,
    content: Option<&str>,
) -> Result<()> {
    runtime::db::notes::update_note(id, title, content, None)
}

pub fn delete_note(_ship_dir: &Path, id: &str) -> Result<()> {
    runtime::db::notes::delete_note(id)
}

pub fn create_adr(
    _ship_dir: &Path,
    title: &str,
    context: &str,
    decision: &str,
) -> Result<AdrRecord> {
    runtime::db::adrs::create_adr(title, context, decision, "proposed")
}

pub fn delete_adr(_ship_dir: &Path, id: &str) -> Result<()> {
    runtime::db::adrs::delete_adr(id)
}

pub fn update_job_status(_ship_dir: &Path, job_id: &str, status: &str) -> Result<()> {
    runtime::db::jobs::update_job_status(job_id, status)
}

pub fn save_user_prefs(values: &[(String, String)]) -> Result<()> {
    let mut config = crate::config::ShipConfig::load();
    for (key, val) in values {
        if val.is_empty() {
            continue;
        }
        config.set(key, val)?;
    }
    config.save()
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime::db::ensure_db;
    use runtime::project::init_project;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        (tmp, ship_dir)
    }

    #[test]
    fn load_all_on_empty_db_returns_defaults() {
        let (_tmp, ship_dir) = setup();
        let data = load_all(&ship_dir);
        assert!(data.targets.is_empty());
        assert!(data.capabilities.is_empty());
        assert!(data.all_jobs.is_empty());
        assert!(data.notes.is_empty());
        assert!(data.adrs.is_empty());
        assert!(data.events.is_empty());
    }

    #[test]
    fn all_jobs_includes_done() {
        let (_tmp, ship_dir) = setup();
        let _j1 =
            runtime::db::jobs::create_job("build", None, None, None, None, 0, None, vec![], vec![])
                .unwrap();
        let j2 =
            runtime::db::jobs::create_job("test", None, None, None, None, 0, None, vec![], vec![])
                .unwrap();
        runtime::db::jobs::update_job_status(&j2.id, "done").unwrap();

        let all = load_all_jobs(&ship_dir);
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn note_crud_roundtrip() {
        let (_tmp, ship_dir) = setup();
        let note = create_note(&ship_dir, "Test Note", "body text").unwrap();
        assert_eq!(note.title, "Test Note");

        update_note(&ship_dir, &note.id, Some("Updated"), None).unwrap();
        let notes = load_notes(&ship_dir);
        assert_eq!(notes[0].title, "Updated");

        delete_note(&ship_dir, &note.id).unwrap();
        assert!(load_notes(&ship_dir).is_empty());
    }

    #[test]
    fn adr_crud_roundtrip() {
        let (_tmp, ship_dir) = setup();
        let adr = create_adr(&ship_dir, "Test ADR", "context", "decision").unwrap();
        assert_eq!(adr.status, "proposed");

        delete_adr(&ship_dir, &adr.id).unwrap();
        assert!(load_adrs(&ship_dir).is_empty());
    }
}
