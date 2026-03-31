//! Agent-facing commands: `ship agent log`.
//! Called from skills and scripts; hidden from user help output.

use anyhow::Result;
use chrono::Utc;

use crate::paths;

/// Append a timestamped entry to .ship/agent.log.
pub fn agent_log(message: &str) -> Result<()> {
    let log_path = paths::project_dir().join("agent.log");
    let entry = format!("[{}] {}\n", Utc::now().to_rfc3339(), message);
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    file.write_all(entry.as_bytes())?;
    Ok(())
}
