use crate::events::envelope::EventEnvelope;
use crate::events::filter::EventFilter;
use crate::events::store::{EventStore, SqliteEventStore};
use crate::events::types::event_types;
use crate::events::types::ProjectLog;
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
    _project_dir: &std::path::Path,
    actor: &str,
    action: &str,
    details: &str,
) -> Result<()> {
    let store = SqliteEventStore::new()?;
    let payload = ProjectLog {
        action: action.to_string(),
        details: details.to_string(),
    };
    let mut envelope = EventEnvelope::new(event_types::PROJECT_LOG, "project", &payload)?;
    envelope.actor = actor.to_string();
    store.append(&envelope)?;
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
pub fn read_log_entries(_project_dir: &std::path::Path) -> Result<Vec<LogEntry>> {
    let store = SqliteEventStore::new()?;
    let events = store.query(&EventFilter {
        event_type: Some(event_types::PROJECT_LOG.to_string()),
        ..Default::default()
    })?;

    let mut entries: Vec<LogEntry> = events
        .into_iter()
        .filter_map(|event| {
            let payload: Option<ProjectLog> = serde_json::from_str(&event.payload_json).ok();
            payload.map(|p| LogEntry {
                timestamp: event.created_at.to_rfc3339(),
                actor: event.actor,
                action: p.action,
                details: p.details,
            })
        })
        .collect();

    // Return most recent first
    entries.reverse();
    if entries.len() > MAX_LOG_ENTRIES {
        entries.truncate(MAX_LOG_ENTRIES);
    }
    Ok(entries)
}
