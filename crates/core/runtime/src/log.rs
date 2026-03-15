use crate::{EventAction, EventEntity, read_events};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct LogEntry {
    pub timestamp: String,
    pub actor: String,
    pub action: String,
    pub details: String,
}

pub const MAX_LOG_ENTRIES: usize = 200;

pub fn log_action(project_dir: &std::path::Path, action: &str, details: &str) -> Result<()> {
    log_action_by(project_dir, "ship", action, details)
}

pub fn log_action_by(
    project_dir: &std::path::Path,
    actor: &str,
    action: &str,
    details: &str,
) -> Result<()> {
    crate::append_event(
        project_dir,
        actor,
        EventEntity::Project,
        EventAction::Log,
        action.to_string(),
        Some(details.to_string()),
    )?;
    Ok(())
}

/// Read legacy-compatible log output synthesized from event entries.
pub fn read_log(project_dir: &std::path::Path) -> Result<String> {
    let mut out = String::new();
    for entry in read_log_entries(project_dir)? {
        out.push_str(&format!(
            "{} [{}] {}: {}\n",
            entry.timestamp, entry.actor, entry.action, entry.details
        ));
    }
    Ok(out)
}

/// Parse log entries from the event stream into structured log rows.
pub fn read_log_entries(project_dir: &std::path::Path) -> Result<Vec<LogEntry>> {
    let mut entries: Vec<LogEntry> = read_events(project_dir)?
        .into_iter()
        .filter(|event| event.action == EventAction::Log)
        .map(|event| LogEntry {
            timestamp: event.timestamp.to_rfc3339(),
            actor: event.actor,
            action: event.subject,
            details: event.details.unwrap_or_default(),
        })
        .collect();

    // Return most recent first
    entries.reverse();
    if entries.len() > MAX_LOG_ENTRIES {
        entries.truncate(MAX_LOG_ENTRIES);
    }
    Ok(entries)
}
