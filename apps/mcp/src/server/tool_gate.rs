use anyhow::Result;
use runtime::{get_active_agent, workspace::get_active_workspace_type};
use std::path::Path;
use std::process::Command as ProcessCommand;

use super::ShipServer;

impl ShipServer {
    pub fn normalize_mode_tool_id(raw: &str) -> String {
        let mut normalized = raw.trim().to_ascii_lowercase().replace('-', "_");
        if let Some(stripped) = normalized.strip_prefix("ship_") {
            normalized = stripped.to_string();
        }
        if let Some(stripped) = normalized.strip_suffix("_tool") {
            normalized = stripped.to_string();
        }
        normalized
    }

    pub fn core_tools() -> &'static [&'static str] {
        &[
            "open_project",
            "create_note",
            "update_note",
            "create_adr",
            "activate_workspace",
            "create_workspace",
            "complete_workspace",
            "list_stale_worktrees",
            "set_agent",
            "list_workspaces",
            "start_session",
            "end_session",
            "log_progress",
            "list_skills",
            "create_job",
            "update_job",
            "list_jobs",
            "append_job_log",
            "claim_file",
            "get_file_owner",
            "list_events",
            "provider_matrix",
            "create_target",
            "update_target",
            "list_targets",
            "get_target",
            "create_capability",
            "update_capability",
            "delete_capability",
            "mark_capability_actual",
            "list_capabilities",
        ]
    }

    pub fn is_core_tool(tool_name: &str) -> bool {
        let normalized = Self::normalize_mode_tool_id(tool_name);
        Self::core_tools().contains(&normalized.as_str())
    }

    pub fn is_project_workspace_tool(_tool_name: &str) -> bool {
        false
    }

    pub fn mode_allows_tool(tool_name: &str, active_tools: &[String]) -> bool {
        if active_tools.is_empty() {
            return true;
        }
        let normalized_tool = Self::normalize_mode_tool_id(tool_name);
        active_tools
            .iter()
            .map(|t| Self::normalize_mode_tool_id(t))
            .any(|allowed| allowed == normalized_tool)
    }

    pub fn enforce_mode_tool_gate(project_dir: &Path, tool_name: &str) -> Result<(), String> {
        if Self::is_core_tool(tool_name) {
            return Ok(());
        }
        if Self::is_project_workspace_tool(tool_name) {
            let active_type = get_active_workspace_type(project_dir).unwrap_or(None);
            if matches!(active_type, Some(runtime::ShipWorkspaceKind::Service)) {
                return Ok(());
            }
        }
        let active_agent =
            get_active_agent(Some(project_dir.to_path_buf())).map_err(|e| e.to_string())?;
        if let Some(ref mode) = active_agent {
            if Self::mode_allows_tool(tool_name, &mode.active_tools) {
                return Ok(());
            }
            let allowed = if mode.active_tools.is_empty() {
                "all tools".to_string()
            } else {
                mode.active_tools.join(", ")
            };
            return Err(format!(
                "Tool '{}' blocked by active mode '{}' (allowed: {}).",
                tool_name, mode.id, allowed
            ));
        }
        Err(format!(
            "Tool '{}' is not in the core workflow surface. \
             Activate the service workspace ('ship') or a mode with this tool in its \
             active_tools list to use it.",
            tool_name
        ))
    }

    pub(crate) fn resolve_workspace_branch_for_project(
        project_dir: &Path,
        branch: Option<&str>,
    ) -> Result<String, String> {
        if let Some(b) = branch {
            let trimmed = b.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
        let Some(root) = project_dir.parent() else {
            return Err("Error: Could not resolve project root".to_string());
        };
        current_branch(root).map_err(|e| e.to_string())
    }
}

pub(crate) fn current_branch(project_root: &Path) -> Result<String> {
    let output = ProcessCommand::new("git")
        .args(["branch", "--show-current"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("Failed to determine current git branch");
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        anyhow::bail!("Current HEAD is detached; cannot map to a feature branch");
    }
    Ok(branch)
}
