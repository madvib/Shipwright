//! Claude provider adapter.
//!
//! Spawns `claude --dangerously-skip-permissions --dangerously-load-development-channels`
//! as a child process in the agent's worktree. Steers the agent by writing
//! messages to its stdin.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::Utc;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStdin};
use tokio::sync::Mutex;

use super::{AgentHandle, AgentSpawnConfig, ProviderAdapter};

struct ClaudeProcess {
    child: Child,
    stdin: Option<ChildStdin>,
}

/// Adapter for the Claude CLI agent.
pub struct ClaudeAdapter {
    processes: Arc<Mutex<HashMap<String, ClaudeProcess>>>,
}

impl ClaudeAdapter {
    pub fn new() -> Self {
        Self { processes: Arc::new(Mutex::new(HashMap::new())) }
    }
}

#[async_trait]
impl ProviderAdapter for ClaudeAdapter {
    async fn spawn(&self, config: AgentSpawnConfig) -> Result<AgentHandle> {
        let mut child = tokio::process::Command::new("claude")
            .args([
                "--dangerously-skip-permissions",
                "--dangerously-load-development-channels",
            ])
            .current_dir(&config.worktree_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("failed to spawn claude: {e}"))?;

        let pid = child.id().ok_or_else(|| anyhow!("claude process has no PID"))?;
        let stdin = child.stdin.take();

        let handle = AgentHandle {
            agent_id: config.agent_id.clone(),
            provider: "claude".to_string(),
            pid,
            thread_id: None,
            started_at: Utc::now(),
        };

        self.processes.lock().await.insert(
            config.agent_id,
            ClaudeProcess { child, stdin },
        );

        Ok(handle)
    }

    async fn steer(&self, handle: &AgentHandle, message: &str) -> Result<()> {
        let mut processes = self.processes.lock().await;
        let entry = processes.get_mut(&handle.agent_id)
            .ok_or_else(|| anyhow!("claude process not found: {}", handle.agent_id))?;
        let stdin = entry.stdin.as_mut()
            .ok_or_else(|| anyhow!("stdin not available for {}", handle.agent_id))?;
        stdin.write_all(message.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        Ok(())
    }

    async fn is_alive(&self, handle: &AgentHandle) -> bool {
        let mut processes = self.processes.lock().await;
        let Some(entry) = processes.get_mut(&handle.agent_id) else {
            return false;
        };
        // try_wait: Ok(None) means still running
        matches!(entry.child.try_wait(), Ok(None))
    }

    async fn stop(&self, handle: &AgentHandle) -> Result<()> {
        let mut processes = self.processes.lock().await;
        let entry = processes.get_mut(&handle.agent_id)
            .ok_or_else(|| anyhow!("claude process not found: {}", handle.agent_id))?;
        entry.child.kill().await
            .map_err(|e| anyhow!("failed to kill claude process {}: {e}", handle.pid))?;
        processes.remove(&handle.agent_id);
        Ok(())
    }
}
