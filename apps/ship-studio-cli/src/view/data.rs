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

// ── CRUD tab loaders ─────────────────────────────────────────────────────────

use crate::mcp::McpEntry;
use crate::paths::{list_agent_ids, agents_skills_dir, global_skills_dir, agents_mcp_path};
use crate::profile::WorkspaceState;
use crate::config::ShipConfig;

pub fn load_agents() -> Vec<(String, String)> {
    list_agent_ids(false, false)
        .into_iter()
        .map(|(id, scope)| (id, scope.to_string()))
        .collect()
}

pub fn load_workspace_state(ship_dir: &Path) -> (Option<String>, Option<String>) {
    let ws = WorkspaceState::load(ship_dir);
    (ws.active_agent, ws.compiled_at)
}

pub fn load_agent_detail(agent_id: &str) -> String {
    use crate::profile::find_agent_file;
    let project_root = std::path::Path::new(".");
    match find_agent_file(agent_id, project_root) {
        Some(path) => std::fs::read_to_string(&path)
            .unwrap_or_else(|e| format!("Error reading {}: {e}", path.display())),
        None => format!("Agent file not found for '{agent_id}'"),
    }
}

pub fn load_skills() -> Vec<(String, String)> {
    let mut skills = Vec::new();
    let project_dir = agents_skills_dir();
    if project_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&project_dir) {
            for e in entries.flatten() {
                if e.path().is_dir() && e.path().join("SKILL.md").exists() {
                    skills.push((e.file_name().to_string_lossy().to_string(), "local".to_string()));
                }
            }
        }
    }
    let global_dir = global_skills_dir();
    if global_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&global_dir) {
            for e in entries.flatten() {
                if e.path().is_dir() && e.path().join("SKILL.md").exists() {
                    skills.push((e.file_name().to_string_lossy().to_string(), "global".to_string()));
                }
            }
        }
    }
    skills.sort_by(|a, b| a.0.cmp(&b.0));
    skills
}

pub fn load_mcp_servers() -> Vec<McpEntry> {
    let path = agents_mcp_path();
    crate::mcp::McpFile::load(&path).map(|f| f.servers).unwrap_or_default()
}

pub fn load_settings() -> Vec<(String, String)> {
    ShipConfig::load().list()
}
