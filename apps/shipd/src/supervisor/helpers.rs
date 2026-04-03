//! Subprocess helpers for the supervisor — worktree, tmux, and agent-config utilities.

use anyhow::Result;
use std::path::PathBuf;

pub(crate) fn worktrees_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SHIP_WORKTREE_DIR") {
        return PathBuf::from(dir);
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join("dev").join("ship-worktrees")
}

pub(crate) fn repo_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SHIP_REPO_DIR") {
        return PathBuf::from(dir);
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub(crate) fn create_worktree(
    worktree_path: &std::path::Path,
    branch: &str,
    base_branch: Option<&str>,
) -> Result<()> {
    if worktree_path.exists() {
        return Ok(());
    }
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let path_str = worktree_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("worktree path is not valid UTF-8"))?;

    let status = std::process::Command::new("git")
        .args(["worktree", "add", path_str, branch])
        .current_dir(repo_dir())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;

    if !status.success() {
        // Branch may not exist yet — create it from base_branch.
        let base = base_branch.unwrap_or("HEAD");
        let args = vec!["worktree", "add", path_str, "-b", branch, base];
        let status2 = std::process::Command::new("git")
            .args(&args)
            .current_dir(repo_dir())
            .status()
            .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;
        if !status2.success() {
            return Err(anyhow::anyhow!(
                "git worktree add failed for branch '{}' at '{}'",
                branch,
                worktree_path.display()
            ));
        }
    }
    Ok(())
}

pub(crate) fn compile_agent_config(
    worktree_path: &std::path::Path,
    agent_id: &str,
) -> Result<()> {
    let output = std::process::Command::new("ship")
        .args(["use", agent_id])
        .current_dir(worktree_path)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run ship: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "exit {}: {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        ));
    }
    Ok(())
}

pub(crate) fn ensure_tmux_session(
    session_name: &str,
    worktree_path: &std::path::Path,
) -> Result<()> {
    // Check if session already exists.
    let check = std::process::Command::new("tmux")
        .args(["has-session", "-t", session_name])
        .status();

    if let Ok(s) = check {
        if s.success() {
            return Ok(());
        }
    }

    let path_str = worktree_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("worktree path is not valid UTF-8"))?;

    let status = std::process::Command::new("tmux")
        .args(["new-session", "-d", "-s", session_name, "-c", path_str])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run tmux: {e}"))?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "tmux new-session failed for session '{session_name}'"
        ));
    }
    Ok(())
}

pub(crate) fn send_agent_command(session_name: &str, providers: &[String]) {
    let Some(cmd) = provider_cli(providers) else {
        tracing::info!(session = session_name, "no known provider CLI; skipping agent spawn");
        return;
    };

    let result = std::process::Command::new("tmux")
        .args(["send-keys", "-t", session_name, &cmd, "Enter"])
        .status();

    if let Err(e) = result {
        tracing::warn!(session = session_name, "tmux send-keys failed: {e}");
    }
}

pub(crate) fn provider_cli(providers: &[String]) -> Option<String> {
    for p in providers {
        match p.as_str() {
            "claude" | "claude-code" => {
                return Some(
                    "claude --dangerously-skip-permissions --dangerously-load-development-channels"
                        .to_string(),
                )
            }
            "codex" => return Some("codex".to_string()),
            _ => {}
        }
    }
    None
}
