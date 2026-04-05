//! HTTP client for forwarding mesh operations to ship-network daemon.
//! Calls the REST API at /api/mesh/* (not the MCP endpoint).

use anyhow::{Result, anyhow};
use serde_json::Value;

/// Get the ship-network port from ~/.ship/network.port
fn get_network_port() -> Result<u16> {
    let global_dir = runtime::project::get_global_dir()?;
    let port_file = global_dir.join("network.port");
    let port_str = std::fs::read_to_string(&port_file)
        .map_err(|_| anyhow!("ship network is not running — start it with: ship network serve"))?;
    port_str
        .trim()
        .parse::<u16>()
        .map_err(|e| anyhow!("failed to parse network port: {e}"))
}

fn base_url() -> Result<String> {
    Ok(format!("http://127.0.0.1:{}/api", get_network_port()?))
}

/// POST JSON to a REST endpoint, return the response body.
async fn post(path: &str, body: Value) -> Result<String> {
    let url = format!("{}{}", base_url()?, path);
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow!("ship-network unavailable: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| anyhow!("failed to read response: {e}"))?;
    if !status.is_success() {
        return Err(anyhow!("ship-network error ({}): {}", status, text));
    }
    Ok(text)
}

/// GET from a REST endpoint.
async fn get(path: &str) -> Result<String> {
    let url = format!("{}{}", base_url()?, path);
    let resp = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("ship-network unavailable: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| anyhow!("failed to read response: {e}"))?;
    if !status.is_success() {
        return Err(anyhow!("ship-network error ({}): {}", status, text));
    }
    Ok(text)
}

pub async fn mesh_register(agent_id: &str, capabilities: Vec<String>) -> Result<String> {
    post("/mesh/register", serde_json::json!({
        "agent_id": agent_id,
        "capabilities": capabilities,
    })).await
}

pub async fn mesh_send(from: &str, to: &str, body: Value) -> Result<String> {
    post("/mesh/send", serde_json::json!({
        "from": from,
        "to": to,
        "body": body,
    })).await
}

pub async fn mesh_broadcast(from: &str, body: Value, capability_filter: Option<String>) -> Result<String> {
    post("/mesh/broadcast", serde_json::json!({
        "from": from,
        "body": body,
        "capability_filter": capability_filter,
    })).await
}

pub async fn mesh_discover() -> Result<String> {
    get("/mesh/discover").await
}

pub async fn mesh_status(agent_id: &str, status: &str) -> Result<String> {
    post("/mesh/status", serde_json::json!({
        "agent_id": agent_id,
        "status": status,
    })).await
}

/// Spawn an actor in the daemon's KernelRouter with the given config.
/// The daemon stashes the mailbox for SSE delivery via /mesh/events/{actor_id}.
pub async fn actor_spawn(
    actor_id: &str,
    config: &runtime::events::ActorConfig,
    capabilities: Option<Vec<String>>,
) -> Result<String> {
    post("/actor/spawn", serde_json::json!({
        "actor_id": actor_id,
        "config": config,
        "capabilities": capabilities,
    })).await
}

/// Route a pre-built EventEnvelope through the daemon's KernelRouter.
/// This ensures events reach daemon subscribers (job-dispatch, workspace-sync, etc).
pub async fn event_route(
    envelope: &runtime::events::EventEnvelope,
    workspace_id: Option<&str>,
    session_id: Option<&str>,
) -> Result<String> {
    post("/events/route", serde_json::json!({
        "envelope": envelope,
        "workspace_id": workspace_id,
        "session_id": session_id,
    })).await
}

/// Open an SSE stream for mesh events targeted at this agent.
/// Returns a receiver that yields EventEnvelopes as they arrive.
/// The background task runs until the connection drops or the receiver is dropped.
pub fn mesh_subscribe(
    agent_id: &str,
) -> Result<tokio::sync::mpsc::UnboundedReceiver<runtime::events::EventEnvelope>> {
    let url = format!("{}/mesh/events/{}", base_url()?, agent_id);
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        let resp = match reqwest::Client::new()
            .get(&url)
            .header("Accept", "text/event-stream")
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("mesh SSE connect failed: {e}");
                return;
            }
        };

        let status = resp.status();
        tracing::info!("mesh SSE connected, status={status}");
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!("mesh SSE rejected: {status} {body}");
            return;
        }

        let mut stream = resp.bytes_stream();
        use futures_util::StreamExt;
        let mut buf = String::new();

        tracing::info!("mesh SSE entering read loop");
        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("mesh SSE read error: {e}");
                    break;
                }
            };
            let chunk_str = String::from_utf8_lossy(&chunk);
            tracing::info!(len = chunk.len(), "mesh SSE chunk received");
            buf.push_str(&chunk_str);

            // Parse SSE frames: lines ending with \n\n
            while let Some(pos) = buf.find("\n\n") {
                let frame = buf[..pos].to_string();
                buf = buf[pos + 2..].to_string();

                // Extract data lines from the SSE frame
                let data: String = frame
                    .lines()
                    .filter_map(|line| line.strip_prefix("data:").or_else(|| line.strip_prefix("data: ")))
                    .collect::<Vec<_>>()
                    .join("\n");

                if data.is_empty() {
                    continue;
                }

                match serde_json::from_str::<runtime::events::EventEnvelope>(&data) {
                    Ok(event) => {
                        tracing::info!(event_type = %event.event_type, "mesh SSE event parsed");
                        if tx.send(event).is_err() {
                            tracing::warn!("mesh SSE receiver dropped");
                            return;
                        }
                    }
                    Err(e) => {
                        tracing::debug!(data = %data, "mesh SSE parse skip: {e}");
                    }
                }
            }
        }
        tracing::info!("mesh SSE stream ended");
    });

    Ok(rx)
}
