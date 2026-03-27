use std::path::Path;
use std::process::Command as ProcessCommand;

use crate::requests::{CompleteWorkspaceRequest, ListStaleWorktreesRequest};
use crate::util::configured_worktree_dir;

pub fn complete_workspace(project_dir: &Path, req: CompleteWorkspaceRequest) -> String {
    // project_dir is the project root (parent of .ship), resolved by get_effective_project_dir.
    let project_root = project_dir;

    let workspace_id = req.workspace_id.trim();
    let worktree_path = configured_worktree_dir(project_root).join(workspace_id);

    let config_path = worktree_path.join("workspace.jsonc");
    let (ws_name, ws_kind, ws_preset) = if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                match serde_json::from_str::<super::workspace::WorkspaceConfig>(&content) {
                    Ok(cfg) => (cfg.name, cfg.kind, cfg.preset_id),
                    Err(_) => (workspace_id.to_string(), "unknown".to_string(), None),
                }
            }
            Err(_) => (workspace_id.to_string(), "unknown".to_string(), None),
        }
    } else {
        (workspace_id.to_string(), "unknown".to_string(), None)
    };

    let should_prune = req.prune_worktree.unwrap_or(ws_kind == "imperative");

    let ship_dir = project_root.join(".ship");
    let slug = runtime::project::project_slug_from_ship_dir(&ship_dir);
    let global_dir = runtime::project::get_global_dir().unwrap_or_else(|_| {
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".ship")
    });
    let sessions_dir = global_dir.join("sessions").join(&slug).join(workspace_id);
    if let Err(e) = std::fs::create_dir_all(&sessions_dir) {
        return format!("Error creating sessions dir: {}", e);
    }
    let handoff_path = sessions_dir.join("handoff.md");
    let timestamp = chrono::Utc::now().to_rfc3339();
    let preset_line = ws_preset
        .as_deref()
        .map(|p| format!("**Preset:** {}", p))
        .unwrap_or_else(|| "**Preset:** (none)".to_string());
    let handoff_content = format!(
        "# Handoff: {ws_name}\n\n\
        **Completed:** {timestamp}\n\
        **Workspace:** {workspace_id}\n\
        **Kind:** {ws_kind}\n\
        {preset_line}\n\n\
        ## Summary\n\n\
        {summary}\n\n\
        ## Context for next session\n\n\
        _Fill this in before ending the session._\n",
        ws_name = ws_name,
        timestamp = timestamp,
        workspace_id = workspace_id,
        ws_kind = ws_kind,
        preset_line = preset_line,
        summary = req.summary,
    );
    if let Err(e) = std::fs::write(&handoff_path, &handoff_content) {
        return format!("Error writing handoff.md: {}", e);
    }

    if should_prune && worktree_path.exists() {
        let status = ProcessCommand::new("git")
            .args([
                "worktree",
                "remove",
                worktree_path.to_str().unwrap_or_default(),
                "--force",
            ])
            .current_dir(project_root)
            .status();
        return match status {
            Ok(s) if s.success() => format!(
                "Workspace '{}' completed. Worktree pruned.\nHandoff: {}",
                workspace_id,
                handoff_path.display()
            ),
            Ok(_) => format!(
                "Workspace '{}' completed. Warning: worktree removal failed (may already be gone).\nHandoff: {}",
                workspace_id,
                handoff_path.display()
            ),
            Err(e) => format!(
                "Workspace '{}' completed. Warning: failed to run git worktree remove: {}\nHandoff: {}",
                workspace_id,
                e,
                handoff_path.display()
            ),
        };
    }

    format!(
        "Workspace '{}' completed.\nHandoff: {}",
        workspace_id,
        handoff_path.display()
    )
}

pub fn list_stale_worktrees(project_dir: &Path, req: ListStaleWorktreesRequest) -> String {
    // project_dir is the project root (parent of .ship), resolved by get_effective_project_dir.
    let project_root = project_dir;

    let idle_hours = req.idle_hours.unwrap_or(24);
    let idle_secs = idle_hours as u64 * 3600;

    let output = match ProcessCommand::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output()
    {
        Ok(o) => o,
        Err(e) => return format!("Error running git worktree list: {}", e),
    };

    if !output.status.success() {
        return format!(
            "Error: git worktree list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let worktrees_prefix = configured_worktree_dir(project_root)
        .to_string_lossy()
        .to_string();

    let mut current_path: Option<String> = None;
    let mut current_wt_branch: Option<String> = None;
    let mut entries: Vec<(String, Option<String>)> = Vec::new();

    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(p) = current_path.take() {
                entries.push((p, current_wt_branch.take()));
            }
            current_path = Some(path.to_string());
            current_wt_branch = None;
        } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            current_wt_branch = Some(branch.to_string());
        } else if line.is_empty()
            && let Some(p) = current_path.take()
        {
            entries.push((p, current_wt_branch.take()));
        }
    }
    if let Some(p) = current_path.take() {
        entries.push((p, current_wt_branch.take()));
    }

    let now = std::time::SystemTime::now();
    let mut stale: Vec<(String, Option<String>, u64)> = Vec::new();

    for (path, branch) in &entries {
        if !path.starts_with(&worktrees_prefix) {
            continue;
        }
        let meta = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let modified = match meta.modified() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let elapsed_secs = now
            .duration_since(modified)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if elapsed_secs >= idle_secs {
            stale.push((path.clone(), branch.clone(), elapsed_secs / 3600));
        }
    }

    if stale.is_empty() {
        return format!(
            "No stale worktrees found (threshold: {} hours).",
            idle_hours
        );
    }

    let mut out = format!("Stale worktrees (idle > {} hours):\n", idle_hours);
    for (path, branch, hours) in &stale {
        let branch_str = branch.as_deref().unwrap_or("(detached)");
        out.push_str(&format!(
            "- {} [branch: {}] idle {}h\n",
            path, branch_str, hours
        ));
    }
    out
}
