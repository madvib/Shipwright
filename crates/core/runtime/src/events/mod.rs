pub mod envelope;
pub mod filter;
pub mod store;
pub mod types;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_actor;

pub use envelope::EventEnvelope;
pub use filter::EventFilter;
pub use store::{EventStore, SqliteEventStore};

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::Path;

pub fn read_events(_ship_dir: &Path) -> Result<Vec<EventEnvelope>> {
    crate::db::events::list_all_events()
}

pub fn list_events_since(
    _ship_dir: &Path,
    since: &DateTime<Utc>,
    limit: Option<usize>,
) -> Result<Vec<EventEnvelope>> {
    crate::db::events::list_events_since_time(since, limit)
}

pub fn read_recent_events(_ship_dir: &Path, limit: usize) -> Result<Vec<EventEnvelope>> {
    crate::db::events::list_recent_events(limit)
}

/// Record a gate pass/fail outcome as a structured event.
pub fn record_gate_outcome(
    _ship_dir: &Path,
    job_id: &str,
    passed: bool,
    evidence: &str,
) -> Result<EventEnvelope> {
    crate::db::events::record_gate_outcome(job_id, passed, evidence)
}

/// List all gate outcomes (pass/fail events) for a given job.
pub fn list_gate_outcomes(_ship_dir: &Path, job_id: &str) -> Result<Vec<EventEnvelope>> {
    crate::db::events::list_gate_outcomes(job_id)
}

/// Query events with ID greater than the given cursor.
///
/// Used by the sync client to find events that haven't been pushed yet.
/// If `elevated_only` is true, only returns elevated events (platform-scope).
pub fn query_events_since(
    cursor: Option<&str>,
    elevated_only: bool,
) -> Result<Vec<EventEnvelope>> {
    crate::db::events::query_events_since(cursor, elevated_only)
}
