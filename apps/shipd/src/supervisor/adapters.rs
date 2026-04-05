//! Concrete adapters for job isolation and execution.
//!
//! `GitWorktreeIsolation` — creates git worktrees, writes phase specs, compiles agent config.
//! `TmuxExecutor` — spawns agents in tmux sessions, launches terminal tabs.

use anyhow::Result;
use async_trait::async_trait;
use runtime::events::job::PipelinePhase;
use runtime::services::dispatch_ports::{ExecutorHandle, IsolationStrategy, JobContext, JobExecutor};
use std::path::PathBuf;

use super::helpers::{
    compile_agent_config, create_worktree, ensure_tmux_session, worktrees_dir,
};
use super::terminal_launcher;

// ── GitWorktreeIsolation ────────────────────────────────────────────────────

/// Isolation strategy that creates git worktrees and compiles agent configs.
pub(crate) struct GitWorktreeIsolation;

#[async_trait]
impl IsolationStrategy for GitWorktreeIsolation {
    async fn prepare(&self, job: &JobContext) -> Result<PathBuf> {
        let worktree_path = worktrees_dir().join(&job.slug);

        // 1. Create git worktree
        create_worktree(&worktree_path, &job.branch, None)?;

        // 2. Create .ship-session and write spec with phase context
        let session_dir = worktree_path.join(".ship-session");
        std::fs::create_dir_all(&session_dir)?;
        write_phase_spec_from_context(&worktree_path, job);

        // 3. Run `ship use {agent}` in the worktree
        compile_agent_config(&worktree_path, &job.agent)?;

        Ok(worktree_path)
    }

    async fn cleanup(&self, job: &JobContext) -> Result<()> {
        let worktree_path = worktrees_dir().join(&job.slug);
        if worktree_path.exists() {
            super::helpers::remove_worktree(&worktree_path)?;
        }
        super::helpers::delete_branch(&job.branch)?;
        Ok(())
    }
}

/// Write the job spec into the worktree, reading from `spec_path` on the context.
fn write_phase_spec_from_context(worktree_path: &std::path::Path, job: &JobContext) {
    let dst_spec = worktree_path.join(".ship-session").join("job-spec.md");
    let src = std::path::Path::new(&job.spec_path);
    let content = if src.exists() {
        std::fs::read_to_string(src).unwrap_or_default()
    } else {
        let parent = worktree_path.parent().and_then(|p| p.parent());
        if let Some(root) = parent {
            let alt = root.join(&job.spec_path);
            std::fs::read_to_string(&alt).unwrap_or_default()
        } else {
            String::new()
        }
    };
    let _ = std::fs::write(&dst_spec, &content);
}

// ── TmuxExecutor ────────────────────────────────────────────────────────────

/// Job executor that spawns agents in tmux sessions.
pub(crate) struct TmuxExecutor;

#[async_trait]
impl JobExecutor for TmuxExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        let tmux_session = format!("job-{}", ctx.slug);
        ensure_tmux_session(&tmux_session, &ctx.work_dir)?;
        spawn_agent_with_mesh_id(&tmux_session, &ctx.slug);

        let (strategy, launched) = terminal_launcher::launch(&tmux_session);
        tracing::info!(
            slug = ctx.slug, strategy, launched,
            "job-dispatch: terminal launch attempted"
        );

        Ok(ExecutorHandle {
            pid: None,
            inner: Box::new(tmux_session),
        })
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        let session = match handle.inner.downcast_ref::<String>() {
            Some(s) => s,
            None => return false,
        };
        std::process::Command::new("tmux")
            .args(["has-session", "-t", session])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        let session = match handle.inner.downcast_ref::<String>() {
            Some(s) => s,
            None => return Ok(()),
        };
        super::helpers::kill_tmux_session(session)
    }

    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()> {
        let session = match handle.inner.downcast_ref::<String>() {
            Some(s) => s,
            None => return Ok(()),
        };
        let status = std::process::Command::new("tmux")
            .args(["send-keys", "-t", session, message, "Enter"])
            .status()
            .map_err(|e| anyhow::anyhow!("tmux send-keys failed: {e}"))?;
        if !status.success() {
            return Err(anyhow::anyhow!("tmux send-keys returned non-zero"));
        }
        Ok(())
    }
}

/// Send agent CLI command into the tmux session with SHIP_MESH_ID set.
fn spawn_agent_with_mesh_id(tmux_session: &str, slug: &str) {
    let cmd = format!(
        "SHIP_MESH_ID={slug} claude --dangerously-skip-permissions \
         --dangerously-load-development-channels server:ship"
    );
    let result = std::process::Command::new("tmux")
        .args(["send-keys", "-t", tmux_session, &cmd, "Enter"])
        .status();
    if let Err(e) = result {
        tracing::warn!(session = tmux_session, "tmux send-keys failed: {e}");
        return;
    }

    let session = tmux_session.to_string();
    std::thread::spawn(move || {
        for delay in [5, 3] {
            std::thread::sleep(std::time::Duration::from_secs(delay));
            let _ = std::process::Command::new("tmux")
                .args(["send-keys", "-t", &session, "1", "Enter"])
                .status();
        }
    });
}

/// Write phase spec into an existing worktree (used by pipeline advancement).
pub(crate) fn write_phase_spec(
    worktree_path: &std::path::Path,
    spec_path: &str,
    pipeline: &Option<Vec<PipelinePhase>>,
    phase_idx: Option<usize>,
) {
    let dst_spec = worktree_path.join(".ship-session").join("job-spec.md");
    let src = std::path::Path::new(spec_path);
    let mut content = if src.exists() {
        std::fs::read_to_string(src).unwrap_or_default()
    } else {
        let parent = worktree_path.parent().and_then(|p| p.parent());
        if let Some(root) = parent {
            let alt = root.join(spec_path);
            std::fs::read_to_string(&alt).unwrap_or_default()
        } else {
            String::new()
        }
    };

    if let (Some(pipeline), Some(idx)) = (pipeline, phase_idx) {
        let total = pipeline.len();
        let current = &pipeline[idx];
        let mut phase_header = format!(
            "## Current Phase\n\nPhase {} of {}: {}\nAgent: {}\nGoal: {}\n",
            idx + 1, total, current.goal, current.agent, current.goal,
        );
        if idx > 0 {
            phase_header.push_str("\nPrior phases completed:\n");
            for (i, p) in pipeline[..idx].iter().enumerate() {
                phase_header.push_str(&format!("- Phase {} ({}): {}\n", i + 1, p.agent, p.goal));
            }
        }
        phase_header.push('\n');
        if let Some(end) = content.find("---\n").and_then(|first| {
            content[first + 4..].find("---\n").map(|second| first + 4 + second + 4)
        }) {
            content.insert_str(end, &phase_header);
        } else {
            content = format!("{phase_header}\n{content}");
        }
    }

    if let Err(e) = std::fs::write(&dst_spec, &content) {
        tracing::error!("job-dispatch: failed to write job-spec.md: {e}");
    }
}
