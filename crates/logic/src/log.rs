use crate::{EventAction, EventEntity, append_event};
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct LogEntry {
    pub timestamp: String,
    pub actor: String,
    pub action: String,
    pub details: String,
}

pub const MAX_LOG_ENTRIES: usize = 200;

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
    prune_log_file(&log_path, MAX_LOG_ENTRIES)?;
    let _ = append_event(
        &project_dir,
        actor,
        EventEntity::Project,
        EventAction::Log,
        action.to_string(),
        Some(details.to_string()),
    );
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

fn parse_log_line(line: &str) -> Option<LogEntry> {
    if line.starts_with("# ") || line.trim().is_empty() {
        return None;
    }
    // Format: "2026-02-22T14:30:00Z [actor] action: details"
    let mut parts = line.splitn(4, ' ');
    let (Some(ts), Some(actor_bracket), Some(action_colon), Some(details)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };

    Some(LogEntry {
        timestamp: ts.to_string(),
        actor: actor_bracket
            .trim_start_matches('[')
            .trim_end_matches(']')
            .to_string(),
        action: action_colon.trim_end_matches(':').to_string(),
        details: details.to_string(),
    })
}

fn prune_log_file(log_path: &PathBuf, max_entries: usize) -> Result<()> {
    let content = fs::read_to_string(log_path).unwrap_or_default();
    let mut lines: Vec<&str> = content
        .lines()
        .filter(|line| parse_log_line(line).is_some())
        .collect();

    if lines.len() <= max_entries {
        return Ok(());
    }

    let keep_from = lines.len().saturating_sub(max_entries);
    let trimmed = lines.split_off(keep_from);
    let mut rewritten = trimmed.join("\n");
    if !rewritten.is_empty() {
        rewritten.push('\n');
    }
    fs::write(log_path, rewritten)?;
    Ok(())
}

/// Parse log entries from the log.md file into structured data.
pub fn read_log_entries(project_dir: PathBuf) -> Result<Vec<LogEntry>> {
    let content = read_log(project_dir)?;
    let mut entries: Vec<LogEntry> = content.lines().filter_map(parse_log_line).collect();

    // Return most recent first
    entries.reverse();
    if entries.len() > MAX_LOG_ENTRIES {
        entries.truncate(MAX_LOG_ENTRIES);
    }
    Ok(entries)
}
