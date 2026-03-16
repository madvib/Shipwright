//! Agent-facing commands: `ship agent log`, `ship agent job`.
//! Called from skills and scripts; hidden from user help output.

use anyhow::Result;
use chrono::Utc;

use crate::cli::{AgentCommands, JobCommands};
use crate::paths;

pub fn dispatch_agent(action: AgentCommands) -> Result<()> {
    match action {
        AgentCommands::Log { message } => agent_log(&message),
        AgentCommands::Job { action } => dispatch_job(action),
    }
}

/// Append a timestamped entry to .ship/agent.log.
fn agent_log(message: &str) -> Result<()> {
    let log_path = paths::project_dir().join("agent.log");
    let entry = format!("[{}] {}\n", Utc::now().to_rfc3339(), message);
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&log_path)?;
    file.write_all(entry.as_bytes())?;
    Ok(())
}

fn dispatch_job(action: JobCommands) -> Result<()> {
    match action {
        JobCommands::Create { kind, branch } => {
            println!("[job create] kind={} branch={}", kind, branch.as_deref().unwrap_or("-"));
            Ok(())
        }
        JobCommands::Update { id, status } => {
            println!("[job update] id={} status={}", id, status);
            Ok(())
        }
        JobCommands::List { branch, status } => {
            println!("[job list] branch={} status={}", branch.as_deref().unwrap_or("*"), status.as_deref().unwrap_or("*"));
            Ok(())
        }
    }
}
