//! Dispatch tool helpers — business logic for agent lifecycle MCP tools.

use std::path::Path;

use runtime::services::dispatch::{AgentSpawnConfig, init_dispatch_service};

use crate::requests::{DispatchAgentRequest, SteerAgentRequest, StopAgentRequest};
use crate::util::configured_worktree_dir;

/// Spawn an agent: create worktree, compile provider config, launch process.
pub async fn dispatch_agent(project_dir: &Path, req: DispatchAgentRequest) -> String {
    let provider = req.provider.as_deref().unwrap_or("claude").to_string();
    let worktree_base = configured_worktree_dir(project_dir);
    let worktree_path = worktree_base.join(&req.slug);
    let agent_id = format!("agent.{}.{}", req.agent_profile, req.slug);

    let config = AgentSpawnConfig {
        agent_id: agent_id.clone(),
        agent_profile: req.agent_profile,
        worktree_path,
        job_spec: req.job_spec,
        provider,
    };

    let svc = init_dispatch_service();
    match svc.lock().await.spawn_agent(config).await {
        Ok(id) => format!("dispatched: {id}"),
        Err(e) => format!("Error: {e}"),
    }
}

/// List all running agents with their status.
pub async fn list_agents() -> String {
    let Some(svc) = runtime::services::dispatch::dispatch_service() else {
        return "[]".to_string();
    };
    let guard = svc.lock().await;
    let agents = guard.list_agents();
    if agents.is_empty() {
        return "no agents running".to_string();
    }
    match serde_json::to_string_pretty(&agents) {
        Ok(json) => json,
        Err(e) => format!("Error serializing agents: {e}"),
    }
}

/// Kill a running agent and deregister it from the mesh.
pub async fn stop_agent(req: StopAgentRequest) -> String {
    let Some(svc) = runtime::services::dispatch::dispatch_service() else {
        return "Error: dispatch service not initialized".to_string();
    };
    match svc.lock().await.stop_agent(&req.agent_id).await {
        Ok(()) => format!("stopped: {}", req.agent_id),
        Err(e) => format!("Error: {e}"),
    }
}

/// Inject a message into a running agent.
pub async fn steer_agent(req: SteerAgentRequest) -> String {
    let Some(svc) = runtime::services::dispatch::dispatch_service() else {
        return "Error: dispatch service not initialized".to_string();
    };
    match svc.lock().await.steer_agent(&req.agent_id, &req.message).await {
        Ok(()) => format!("steered: {}", req.agent_id),
        Err(e) => format!("Error: {e}"),
    }
}
