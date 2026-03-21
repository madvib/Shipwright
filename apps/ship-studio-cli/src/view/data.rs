//! DB load helpers for the view TUI — all infallible (return empty on error).

use runtime::EventRecord;
use runtime::db::{
    adrs::{AdrRecord, list_adrs},
    events::list_recent_events,
    jobs::{Job, JobLogEntry, list_jobs, list_logs},
    notes::{Note, list_notes},
    targets::{Capability, Target, list_capabilities, list_targets},
};
use std::path::Path;

pub fn load_targets(ship_dir: &Path) -> Vec<Target> {
    list_targets(ship_dir, None).unwrap_or_default()
}

pub fn load_caps(ship_dir: &Path, target_id: &str) -> Vec<Capability> {
    list_capabilities(ship_dir, Some(target_id), None, None).unwrap_or_default()
}

pub fn load_adrs(ship_dir: &Path) -> Vec<AdrRecord> {
    list_adrs(ship_dir).unwrap_or_default()
}

pub fn load_notes(ship_dir: &Path) -> Vec<Note> {
    list_notes(ship_dir, None).unwrap_or_default()
}

pub fn load_jobs_filtered(ship_dir: &Path, status: Option<&str>) -> Vec<Job> {
    list_jobs(ship_dir, None, status).unwrap_or_default()
}

pub fn load_logs(ship_dir: &Path, job_id: &str) -> Vec<JobLogEntry> {
    list_logs(ship_dir, None, Some(job_id), Some(20)).unwrap_or_default()
}

pub fn load_events(ship_dir: &Path, limit: usize) -> Vec<EventRecord> {
    list_recent_events(ship_dir, limit).unwrap_or_default()
}

/// Returns (actual, total) for all capabilities across all targets.
pub fn load_cap_progress(ship_dir: &Path, target_id: &str) -> (usize, usize) {
    let caps = list_capabilities(ship_dir, Some(target_id), None, None).unwrap_or_default();
    let actual = caps.iter().filter(|c| c.status == "actual").count();
    (actual, caps.len())
}
