//! Codex app-server adapter.
//!
//! Spawns `codex app-server` as a child process and communicates via
//! JSON-RPC 2.0 over stdio. Implements the initialize → thread/start →
//! turn/start → (turn/steer)* flow.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex;

use super::{AgentHandle, AgentSpawnConfig, ProviderAdapter};

// ── JSON-RPC types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: impl Into<String>, params: serde_json::Value) -> Self {
        Self { jsonrpc: "2.0".to_string(), id, method: method.into(), params }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    /// Present on notifications (no id field).
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    pub fn is_notification(&self) -> bool {
        self.id.is_none() && self.method.is_some()
    }
}

// ── Process state ─────────────────────────────────────────────────────────────

struct CodexProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    thread_id: Option<String>,
}

impl CodexProcess {
    async fn send(&mut self, request: &JsonRpcRequest) -> Result<()> {
        let line = serde_json::to_string(request)? + "\n";
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn recv(&mut self) -> Result<JsonRpcResponse> {
        let mut line = String::new();
        self.stdout.read_line(&mut line).await?;
        serde_json::from_str(line.trim())
            .map_err(|e| anyhow!("codex JSON-RPC parse error: {e} (line: {line:?})"))
    }

    /// Send a request and wait for the matching response (skipping notifications).
    async fn call(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&JsonRpcRequest::new(id, method, params)).await?;
        loop {
            let resp = self.recv().await?;
            if resp.is_notification() {
                continue;
            }
            if resp.id == Some(id) {
                if let Some(err) = resp.error {
                    return Err(anyhow!("codex error in {method}: {err}"));
                }
                return Ok(resp.result.unwrap_or(serde_json::Value::Null));
            }
        }
    }
}

// ── CodexAdapter ──────────────────────────────────────────────────────────────

/// Adapter for the Codex app-server (JSON-RPC over stdio).
pub struct CodexAdapter {
    processes: Arc<Mutex<HashMap<String, CodexProcess>>>,
}

impl CodexAdapter {
    pub fn new() -> Self {
        Self { processes: Arc::new(Mutex::new(HashMap::new())) }
    }
}

#[async_trait]
impl ProviderAdapter for CodexAdapter {
    async fn spawn(&self, config: AgentSpawnConfig) -> Result<AgentHandle> {
        let mut child = tokio::process::Command::new("codex")
            .args(["app-server"])
            .current_dir(&config.worktree_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("failed to spawn codex app-server: {e}"))?;

        let pid = child.id().ok_or_else(|| anyhow!("codex process has no PID"))?;
        let stdin = child.stdin.take()
            .ok_or_else(|| anyhow!("codex stdin unavailable"))?;
        let stdout = BufReader::new(
            child.stdout.take().ok_or_else(|| anyhow!("codex stdout unavailable"))?,
        );

        let mut proc = CodexProcess {
            child,
            stdin,
            stdout,
            next_id: 1,
            thread_id: None,
        };

        // initialize handshake
        proc.call("initialize", serde_json::json!({})).await?;

        // thread/start — create a session with cwd and sandbox config
        let thread_result = proc.call("thread/start", serde_json::json!({
            "cwd": config.worktree_path.to_str().unwrap_or(""),
            "sandbox": { "type": "none" }
        })).await?;
        let thread_id = thread_result["thread_id"]
            .as_str()
            .map(String::from);
        proc.thread_id = thread_id.clone();

        // turn/start — inject the job spec as the first prompt
        let tid = thread_id.as_deref().unwrap_or("");
        proc.call("turn/start", serde_json::json!({
            "thread_id": tid,
            "prompt": config.job_spec,
        })).await?;

        let handle = AgentHandle {
            agent_id: config.agent_id.clone(),
            provider: "codex".to_string(),
            pid,
            thread_id: thread_id.clone(),
            started_at: Utc::now(),
        };

        self.processes.lock().await.insert(config.agent_id, proc);
        Ok(handle)
    }

    async fn steer(&self, handle: &AgentHandle, message: &str) -> Result<()> {
        let mut processes = self.processes.lock().await;
        let proc = processes.get_mut(&handle.agent_id)
            .ok_or_else(|| anyhow!("codex process not found: {}", handle.agent_id))?;
        let thread_id = proc.thread_id.clone()
            .unwrap_or_else(|| handle.thread_id.clone().unwrap_or_default());
        let id = proc.next_id;
        proc.next_id += 1;
        proc.send(&JsonRpcRequest::new(id, "turn/steer", serde_json::json!({
            "thread_id": thread_id,
            "message": message,
        }))).await
    }

    async fn is_alive(&self, handle: &AgentHandle) -> bool {
        let mut processes = self.processes.lock().await;
        let Some(proc) = processes.get_mut(&handle.agent_id) else {
            return false;
        };
        matches!(proc.child.try_wait(), Ok(None))
    }

    async fn stop(&self, handle: &AgentHandle) -> Result<()> {
        let mut processes = self.processes.lock().await;
        let proc = processes.get_mut(&handle.agent_id)
            .ok_or_else(|| anyhow!("codex process not found: {}", handle.agent_id))?;
        proc.child.kill().await
            .map_err(|e| anyhow!("failed to kill codex process {}: {e}", handle.pid))?;
        processes.remove(&handle.agent_id);
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonrpc_request_serializes_correctly() {
        let req = JsonRpcRequest::new(1, "initialize", serde_json::json!({}));
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""jsonrpc":"2.0""#));
        assert!(json.contains(r#""id":1"#));
        assert!(json.contains(r#""method":"initialize""#));
    }

    #[test]
    fn jsonrpc_response_deserializes_result() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"thread_id":"t-abc"}}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, Some(1));
        assert!(resp.result.is_some());
        assert_eq!(resp.result.as_ref().unwrap()["thread_id"], "t-abc");
        assert!(!resp.is_notification());
    }

    #[test]
    fn jsonrpc_notification_detected() {
        let json = r#"{"jsonrpc":"2.0","method":"turn/completed","params":{"thread_id":"t-abc"}}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(resp.is_notification());
        assert_eq!(resp.method.as_deref(), Some("turn/completed"));
    }

    #[test]
    fn jsonrpc_error_response_deserializes() {
        let json = r#"{"jsonrpc":"2.0","id":2,"error":{"code":-32600,"message":"Invalid request"}}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, Some(2));
        assert!(resp.error.is_some());
        assert!(resp.result.is_none());
    }
}
