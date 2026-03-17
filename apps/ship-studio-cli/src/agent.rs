//! Agent-facing commands: `ship agent log`.
//! Called from skills and scripts; hidden from user help output.
//!
//! Job operations (create/update/list) are available internally via
//! `dispatch_job` but are not exposed through the CLI — they go through MCP.

use anyhow::{Context, Result};
use chrono::Utc;

use crate::cli::AgentCommands;
use crate::paths;

pub fn dispatch_agent(action: AgentCommands) -> Result<()> {
    match action {
        AgentCommands::Log { message } => agent_log(&message),
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

/// Return the `.ship/` directory for the current working directory, or error if absent.
fn ship_dir() -> Result<std::path::PathBuf> {
    let cwd = std::env::current_dir()?;
    let ship_dir = cwd.join(".ship");
    if !ship_dir.exists() {
        anyhow::bail!(".ship/ not found in {}. Run: ship init", cwd.display());
    }
    Ok(ship_dir)
}

/// Create a job and print its ID. Not wired to CLI — called by MCP or internally.
#[allow(dead_code)]
pub fn create_job(kind: &str, branch: Option<&str>) -> Result<()> {
    let dir = ship_dir()?;
    let job = runtime::db::jobs::create_job(&dir, kind, branch, None, None, None, 0, None, vec![])
        .with_context(|| format!("failed to create job (kind={kind})"))?;
    println!("{}", job.id);
    Ok(())
}

/// Update a job's status. Not wired to CLI — called by MCP or internally.
#[allow(dead_code)]
pub fn update_job(id: &str, status: &str) -> Result<()> {
    let dir = ship_dir()?;
    runtime::db::jobs::update_job_status(&dir, id, status)
        .with_context(|| format!("failed to update job {id} to status={status}"))?;
    println!("updated {id} -> {status}");
    Ok(())
}

/// List jobs, optionally filtered by branch and/or status. Not wired to CLI — called by MCP or internally.
#[allow(dead_code)]
pub fn list_jobs(branch: Option<&str>, status: Option<&str>) -> Result<()> {
    let dir = ship_dir()?;
    let jobs = runtime::db::jobs::list_jobs(&dir, branch, status)
        .context("failed to list jobs")?;
    if jobs.is_empty() {
        println!("no jobs found");
    } else {
        for job in &jobs {
            println!(
                "{}\t{}\t{}\t{}",
                job.id,
                job.kind,
                job.status,
                job.branch.as_deref().unwrap_or("-"),
            );
        }
    }
    Ok(())
}
