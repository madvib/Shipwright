use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub actor: String,
    pub action: String,
    pub details: String,
}

pub fn log_action(project_dir: PathBuf, action: &str, details: &str) -> Result<()> {
    log_action_by(project_dir, "ship", action, details)
}

pub fn log_action_by(project_dir: PathBuf, actor: &str, action: &str, details: &str) -> Result<()> {
    let log_path = project_dir.join("log.md");
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let entry = format!("{} [{}] {}: {}\n", now, actor, action, details);

    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_path)?;

    use std::io::Write;
    file.write_all(entry.as_bytes())?;
    Ok(())
}

/// Read the raw log file contents
pub fn read_log(project_dir: PathBuf) -> Result<String> {
    let log_path = project_dir.join("log.md");
    if !log_path.exists() {
        return Ok(String::new());
    }
    Ok(fs::read_to_string(log_path)?)
}

/// Parse log entries from the log.md file into structured data.
/// Each line is expected to follow the format: `- [timestamp] **action**: details`
pub fn read_log_entries(project_dir: PathBuf) -> Result<Vec<LogEntry>> {
    let content = read_log(project_dir)?;
    let mut entries = Vec::new();

    for line in content.lines() {
        if line.starts_with("# ") || line.trim().is_empty() {
            continue;
        }
        // Format: "2026-02-22T14:30:00Z [actor] action: details"
        let mut parts = line.splitn(4, ' ');
        if let (Some(ts), Some(actor_bracket), Some(action_colon), Some(details)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        {
            let actor = actor_bracket.trim_start_matches('[').trim_end_matches(']').to_string();
            let action = action_colon.trim_end_matches(':').to_string();
            entries.push(LogEntry {
                timestamp: ts.to_string(),
                actor,
                action,
                details: details.to_string(),
            });
        }
    }

    // Return most recent first
    entries.reverse();
    Ok(entries)
}
