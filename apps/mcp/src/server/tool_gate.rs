use anyhow::Result;
use runtime::get_active_agent;
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

    const PLATFORM_TOOLS: &[&str] = &[
        "event",
        "open_project",
        "activate_workspace",
        "create_workspace",
        "complete_workspace",
        "list_stale_worktrees",
        "list_workspaces",
        "set_agent",
        "start_session",
        "end_session",
        "log_progress",
        "get_session",
        "list_sessions",
        "list_skills",
        "get_skill_vars",
        "set_skill_var",
        "list_skill_vars",
        "create_job",
        "update_job",
        "list_jobs",
        "get_job",
        "write_session_file",
        "read_session_file",
        "list_session_files",
        "mesh_send",
        "mesh_broadcast",
        "mesh_discover",
        "mesh_status",
    ];

    #[cfg(feature = "unstable")]
    const UNSTABLE_TOOLS: &[&str] = &[
        "create_adr",
    ];

    pub fn is_core_tool(tool_name: &str) -> bool {
        let normalized = Self::normalize_mode_tool_id(tool_name);
        if Self::PLATFORM_TOOLS.contains(&normalized.as_str()) {
            return true;
        }
        #[cfg(feature = "unstable")]
        if Self::UNSTABLE_TOOLS.contains(&normalized.as_str()) {
            return true;
        }
        false
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
            return Ok(());
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
