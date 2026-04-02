use anyhow::{Result, anyhow};
use cli_framework::hooks::{
    handle_after_tool, handle_before_tool, handle_session_end, handle_session_start,
};
use std::io::Read;

fn actor_id() -> Result<String> {
    std::env::var("SHIP_ACTOR_ID")
        .map_err(|_| anyhow!("SHIP_ACTOR_ID env var is not set"))
}

fn workspace_id() -> Result<String> {
    std::env::var("SHIP_WORKSPACE_ID")
        .map_err(|_| anyhow!("SHIP_WORKSPACE_ID env var is not set"))
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

pub fn run_session_start() -> Result<()> {
    let actor = actor_id()?;
    let workspace = workspace_id()?;
    let event = handle_session_start(&actor, &workspace)?;
    println!("{}", serde_json::to_string(&event)?);
    Ok(())
}

pub fn run_before_tool() -> Result<()> {
    let stdin = read_stdin()?;
    let actor = actor_id()?;
    let workspace = workspace_id()?;
    if let Some(event) = handle_before_tool(&stdin, &actor, &workspace)? {
        println!("{}", serde_json::to_string(&event)?);
    }
    Ok(())
}

pub fn run_after_tool() -> Result<()> {
    let stdin = read_stdin()?;
    let actor = actor_id()?;
    let workspace = workspace_id()?;
    if let Some(event) = handle_after_tool(&stdin, &actor, &workspace)? {
        println!("{}", serde_json::to_string(&event)?);
    }
    Ok(())
}

pub fn run_session_end() -> Result<()> {
    let actor = actor_id()?;
    let workspace = workspace_id()?;
    let event = handle_session_end(&actor, &workspace)?;
    println!("{}", serde_json::to_string(&event)?);
    // Best-effort: broadcast session.ended via ship CLI (MCP bridge path).
    broadcast_session_ended(&actor, &workspace);
    Ok(())
}

/// Broadcast agent.session.ended via `ship mesh broadcast` CLI.
/// Non-fatal — if ship is not available or mesh is down this is a no-op.
fn broadcast_session_ended(actor_id: &str, workspace_id: &str) {
    let payload = serde_json::json!({
        "event_type": "agent.session.ended",
        "from_agent_id": actor_id,
        "payload": { "workspace_id": workspace_id }
    });
    let _ = std::process::Command::new("ship")
        .args(["mesh", "broadcast", &payload.to_string()])
        .status();
}
