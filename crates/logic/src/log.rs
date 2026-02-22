use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub action: String,
    pub details: String,
}

pub fn log_action(project_dir: PathBuf, action: &str, details: &str) -> Result<()> {
    let log_path = project_dir.join("log.md");
    let now = Utc::now().to_rfc3339();
    let entry = format!("- [{}] **{}**: {}\n", now, action, details);

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
        // Skip header lines and empty lines
        if line.starts_with("# ") || line.trim().is_empty() {
            continue;
        }
        // Parse: - [timestamp] **action**: details
        if let Some(rest) = line.strip_prefix("- [") {
            if let Some(bracket_end) = rest.find("] **") {
                let timestamp = rest[..bracket_end].to_string();
                let after_ts = &rest[bracket_end + 4..]; // skip "] **"
                if let Some(stars_end) = after_ts.find("**: ") {
                    let action = after_ts[..stars_end].to_string();
                    let details = after_ts[stars_end + 4..].to_string();
                    entries.push(LogEntry {
                        timestamp,
                        action,
                        details,
                    });
                }
            }
        }
    }

    // Return most recent first
    entries.reverse();
    Ok(entries)
}
