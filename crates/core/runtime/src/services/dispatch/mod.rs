//! Dispatch service — spawns, tracks, and steers agent processes.
//!
//! `DispatchService` is a runtime-managed registry for running agents. It
//! creates git worktrees, compiles provider config via `ship use`, spawns the
//! agent process through the appropriate `ProviderAdapter`, and registers the
//! agent on the mesh. Callers interact through the global singleton.
//!
//! # Example
//!
//! ```rust,ignore
//! let svc = init_dispatch_service();
//! let agent_id = svc.lock().await.spawn_agent(config).await?;
//! ```

pub mod claude;
pub mod codex;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

// ── Public types ──────────────────────────────────────────────────────────────

/// Configuration for spawning a new agent.
#[derive(Clone)]
pub struct AgentSpawnConfig {
    /// Stable identifier, e.g. `"agent.rust-runtime.job/foo"`.
    pub agent_id: String,
    /// Agent profile name, e.g. `"rust-runtime"`.
    pub agent_profile: String,
    /// Absolute path to the git worktree for this agent.
    pub worktree_path: PathBuf,
    /// Contents of job-spec.md to write into `.ship-session/`.
    pub job_spec: String,
    /// Provider name: `"claude"` or `"codex"`.
    pub provider: String,
}

/// Serializable descriptor for a running agent process.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentHandle {
    pub agent_id: String,
    pub provider: String,
    /// OS process ID of the agent process.
    pub pid: u32,
    /// Codex thread ID, if applicable.
    pub thread_id: Option<String>,
    pub started_at: DateTime<Utc>,
}

/// Live status of an agent.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Running,
    Stopped,
}

// ── Ports: Isolation + Execution ─────────────────────────────────────────────

/// Context for a job being dispatched. Carries everything an executor needs.
#[derive(Clone, Debug)]
pub struct JobContext {
    pub job_id: String,
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
    pub work_dir: PathBuf,
    pub env: HashMap<String, String>,
    pub model: Option<String>,
    pub provider: Option<String>,
}

/// Handle returned by a `JobExecutor` after spawning an agent.
pub struct ExecutorHandle {
    pub pid: Option<u32>,
    /// Opaque adapter-specific state.
    pub inner: Box<dyn std::any::Any + Send>,
}

/// Port: prepares and cleans up the isolated working directory for a job.
#[async_trait]
pub trait IsolationStrategy: Send + Sync {
    /// Prepare an isolated working directory. Returns the path.
    async fn prepare(&self, job: &JobContext) -> Result<PathBuf>;
    /// Clean up after job completion or failure.
    async fn cleanup(&self, job: &JobContext) -> Result<()>;
}

/// Port: spawns and manages the agent process lifecycle.
#[async_trait]
pub trait JobExecutor: Send + Sync {
    /// Spawn the agent process in the prepared work directory.
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle>;
    /// Check if the agent process is still running.
    async fn is_alive(&self, handle: &ExecutorHandle) -> bool;
    /// Terminate the agent process.
    async fn stop(&self, handle: &ExecutorHandle) -> Result<()>;
    /// Inject a message into the running agent.
    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()>;
}

// ── ProviderAdapter trait (deprecated — migrate to JobExecutor) ──────────────

/// Implemented by each provider (Claude, Codex) to manage agent processes.
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    /// Spawn an agent process and return its handle.
    async fn spawn(&self, config: AgentSpawnConfig) -> Result<AgentHandle>;
    /// Inject a message into a running agent.
    async fn steer(&self, handle: &AgentHandle, message: &str) -> Result<()>;
    /// Return true if the agent process is still running.
    async fn is_alive(&self, handle: &AgentHandle) -> bool;
    /// Terminate the agent process.
    async fn stop(&self, handle: &AgentHandle) -> Result<()>;
}

// ── DispatchService ───────────────────────────────────────────────────────────

/// Runtime service that manages the lifecycle of spawned agent processes.
pub struct DispatchService {
    agents: HashMap<String, AgentHandle>,
    adapters: HashMap<String, Arc<dyn ProviderAdapter>>,
}

impl DispatchService {
    /// Create with the built-in Claude and Codex adapters registered.
    pub fn new() -> Self {
        let mut adapters: HashMap<String, Arc<dyn ProviderAdapter>> = HashMap::new();
        adapters.insert("claude".to_string(), Arc::new(claude::ClaudeAdapter::new()));
        adapters.insert("codex".to_string(), Arc::new(codex::CodexAdapter::new()));
        Self { agents: HashMap::new(), adapters }
    }

    /// Create worktree, write job spec, compile agent config, spawn the process,
    /// register on mesh, and return the agent_id.
    pub async fn spawn_agent(&mut self, config: AgentSpawnConfig) -> Result<String> {
        create_worktree(&config)?;
        write_job_spec(&config)?;
        compile_agent(&config)?;

        let adapter = self.adapters.get(&config.provider)
            .map(Arc::clone)
            .ok_or_else(|| anyhow!("unknown provider: {}", config.provider))?;

        let handle = adapter.spawn(config).await?;
        let agent_id = handle.agent_id.clone();
        register_on_mesh(&agent_id).await;
        self.agents.insert(agent_id.clone(), handle);
        Ok(agent_id)
    }

    /// Inject a message into a running agent via its provider adapter.
    pub async fn steer_agent(&self, agent_id: &str, message: &str) -> Result<()> {
        let handle = self.agents.get(agent_id)
            .ok_or_else(|| anyhow!("agent not found: {agent_id}"))?;
        let adapter = self.adapters.get(&handle.provider)
            .map(Arc::clone)
            .ok_or_else(|| anyhow!("no adapter for provider: {}", handle.provider))?;
        adapter.steer(handle, message).await
    }

    /// Kill the agent process, deregister from mesh, and remove from registry.
    pub async fn stop_agent(&mut self, agent_id: &str) -> Result<()> {
        let handle = self.agents.get(agent_id)
            .ok_or_else(|| anyhow!("agent not found: {agent_id}"))?
            .clone();
        let adapter = self.adapters.get(&handle.provider)
            .map(Arc::clone)
            .ok_or_else(|| anyhow!("no adapter for provider: {}", handle.provider))?;
        adapter.stop(&handle).await?;
        deregister_from_mesh(agent_id).await;
        self.agents.remove(agent_id);
        Ok(())
    }

    /// Return all registered agent handles.
    pub fn list_agents(&self) -> Vec<AgentHandle> {
        self.agents.values().cloned().collect()
    }

    /// Return the live status of an agent, or `None` if not registered.
    pub async fn agent_status(&self, agent_id: &str) -> Option<AgentStatus> {
        let handle = self.agents.get(agent_id)?;
        let adapter = self.adapters.get(&handle.provider)?.clone();
        Some(if adapter.is_alive(handle).await {
            AgentStatus::Running
        } else {
            AgentStatus::Stopped
        })
    }
}

// ── Global singleton ──────────────────────────────────────────────────────────

static DISPATCH: OnceLock<Arc<Mutex<DispatchService>>> = OnceLock::new();

/// Initialize the global DispatchService. Idempotent.
pub fn init_dispatch_service() -> Arc<Mutex<DispatchService>> {
    if let Some(svc) = DISPATCH.get() {
        return svc.clone();
    }
    let svc = Arc::new(Mutex::new(DispatchService::new()));
    let _ = DISPATCH.set(svc);
    DISPATCH.get().expect("just set").clone()
}

/// Get the global DispatchService, or `None` if not initialized.
pub fn dispatch_service() -> Option<Arc<Mutex<DispatchService>>> {
    DISPATCH.get().cloned()
}

// ── Worktree + agent bootstrap ────────────────────────────────────────────────

fn create_worktree(config: &AgentSpawnConfig) -> Result<()> {
    if config.worktree_path.exists() {
        return Ok(());
    }
    let parent = config.worktree_path.parent()
        .ok_or_else(|| anyhow!("invalid worktree path: no parent"))?;
    std::fs::create_dir_all(parent)?;
    let branch = format!("job/{}", config.worktree_path
        .file_name().and_then(|n| n.to_str()).unwrap_or("agent"));
    let status = std::process::Command::new("git")
        .args(["worktree", "add",
               config.worktree_path.to_str().unwrap_or_default(),
               "-b", &branch])
        .status()
        .map_err(|e| anyhow!("git worktree add: {e}"))?;
    if !status.success() {
        return Err(anyhow!("git worktree add failed for {:?}", config.worktree_path));
    }
    Ok(())
}

fn write_job_spec(config: &AgentSpawnConfig) -> Result<()> {
    let session_dir = config.worktree_path.join(".ship-session");
    std::fs::create_dir_all(&session_dir)?;
    std::fs::write(session_dir.join("job-spec.md"), &config.job_spec)?;
    Ok(())
}

fn compile_agent(config: &AgentSpawnConfig) -> Result<()> {
    let status = std::process::Command::new("ship")
        .args(["use", &config.agent_profile])
        .current_dir(&config.worktree_path)
        .status()
        .map_err(|e| anyhow!("ship use: {e}"))?;
    if !status.success() {
        return Err(anyhow!("ship use {} failed", config.agent_profile));
    }
    Ok(())
}

// ── Mesh helpers ──────────────────────────────────────────────────────────────

async fn register_on_mesh(agent_id: &str) {
    let Some(kr) = crate::events::kernel_router() else { return };
    if let Ok(event) = crate::events::EventEnvelope::new(
        "mesh.register",
        agent_id,
        &serde_json::json!({ "agent_id": agent_id, "capabilities": ["dispatch"] }),
    ) {
        let ctx = crate::events::EmitContext {
            caller_kind: crate::events::CallerKind::Mcp,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        };
        let _ = kr.lock().await.route(event.with_actor_id(agent_id), &ctx).await;
    }
}

async fn deregister_from_mesh(agent_id: &str) {
    let Some(kr) = crate::events::kernel_router() else { return };
    if let Ok(event) = crate::events::EventEnvelope::new(
        "mesh.deregister",
        agent_id,
        &serde_json::json!({ "agent_id": agent_id }),
    ) {
        let ctx = crate::events::EmitContext {
            caller_kind: crate::events::CallerKind::Mcp,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        };
        let _ = kr.lock().await.route(event.with_actor_id(agent_id), &ctx).await;
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_service_has_no_agents() {
        let svc = DispatchService::new();
        assert!(svc.list_agents().is_empty());
    }

    #[tokio::test]
    async fn steer_unknown_agent_returns_error() {
        let svc = DispatchService::new();
        let err = svc.steer_agent("agent.missing.foo", "hello").await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("agent not found"));
    }

    #[tokio::test]
    async fn stop_unknown_agent_returns_error() {
        let mut svc = DispatchService::new();
        let err = svc.stop_agent("agent.missing.foo").await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("agent not found"));
    }

    #[tokio::test]
    async fn agent_status_unknown_returns_none() {
        let svc = DispatchService::new();
        let status = svc.agent_status("agent.missing.foo").await;
        assert!(status.is_none());
    }

    #[test]
    fn init_dispatch_service_idempotent() {
        let a = init_dispatch_service();
        let b = init_dispatch_service();
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[test]
    fn dispatch_service_some_after_init() {
        let _ = init_dispatch_service();
        assert!(dispatch_service().is_some());
    }
}
