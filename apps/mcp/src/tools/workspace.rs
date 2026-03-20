use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use runtime::workspace::{
    CreateWorkspaceRequest as RuntimeCreateWorkspaceRequest, ShipWorkspaceKind,
    activate_workspace as runtime_activate_workspace,
    create_workspace as runtime_create_workspace,
    list_workspaces as runtime_list_workspaces,
    repair_workspace as runtime_repair_workspace,
    set_workspace_active_agent,
    sync_workspace as runtime_sync_workspace,
};

use crate::requests::{
    ActivateWorkspaceRequest, CreateWorkspaceRequest, RegisterWorkspaceRequest,
    ListWorkspacesRequest, RepairWorkspaceRequest, SyncWorkspaceRequest,
};
use crate::util::configured_worktree_dir;

pub fn register_workspace(
    project_dir: &Path,
    req: RegisterWorkspaceRequest,
) -> String {
    let parsed_workspace_type = match req.workspace_type {
        Some(workspace_type) => match workspace_type.parse::<ShipWorkspaceKind>() {
            Ok(parsed) => Some(parsed),
            Err(err) => return format!("Error: {}", err),
        },
        None => None,
    };

    let workspace_request = RuntimeCreateWorkspaceRequest {
        branch: req.branch.clone(),
        workspace_type: parsed_workspace_type,
        status: None,
        environment_id: req.environment_id,
        feature_id: req.feature_id,
        target_id: req.target_id,
        active_agent: req.agent_id,
        providers: None,
        mcp_servers: None,
        skills: None,
        is_worktree: req.is_worktree,
        worktree_path: req.worktree_path,
        context_hash: None,
    };

    let workspace = match runtime_create_workspace(project_dir, workspace_request) {
        Ok(workspace) => workspace,
        Err(err) => return format!("Error: {}", err),
    };

    let workspace = if req.activate.unwrap_or_default() {
        match runtime_activate_workspace(project_dir, &workspace.branch) {
            Ok(active) => active,
            Err(err) => return format!("Error: {}", err),
        }
    } else {
        workspace
    };

    serde_json::to_string_pretty(&workspace)
        .unwrap_or_else(|e| format!("Error serializing workspace: {}", e))
}

pub fn activate_workspace(project_dir: &Path, req: ActivateWorkspaceRequest) -> String {
    let mut workspace = match runtime_activate_workspace(project_dir, &req.branch) {
        Ok(workspace) => workspace,
        Err(err) => return format!("Error: {}", err),
    };
    if let Some(agent_id) = req.agent_id.as_deref() {
        workspace = match set_workspace_active_agent(project_dir, &req.branch, Some(agent_id)) {
            Ok(workspace) => workspace,
            Err(err) => return format!("Error: {}", err),
        };
    }
    serde_json::to_string_pretty(&workspace)
        .unwrap_or_else(|e| format!("Error serializing workspace: {}", e))
}

pub fn sync_workspace(project_dir: &Path, _req: SyncWorkspaceRequest, branch: &str) -> String {
    match runtime_sync_workspace(project_dir, branch) {
        Ok(workspace) => serde_json::to_string_pretty(&workspace)
            .unwrap_or_else(|e| format!("Error serializing workspace: {}", e)),
        Err(err) => format!("Error: {}", err),
    }
}

pub fn repair_workspace(project_dir: &Path, req: RepairWorkspaceRequest, branch: &str) -> String {
    let dry_run = req.dry_run.unwrap_or(true);
    match runtime_repair_workspace(project_dir, branch, dry_run) {
        Ok(report) => serde_json::to_string_pretty(&report)
            .unwrap_or_else(|e| format!("Error serializing workspace repair report: {}", e)),
        Err(err) => format!("Error: {}", err),
    }
}

pub fn list_workspaces(project_dir: &Path, req: ListWorkspacesRequest) -> String {
    let workspaces = match runtime_list_workspaces(project_dir) {
        Ok(ws) => ws,
        Err(e) => return format!("Error listing workspaces: {}", e),
    };
    let filtered: Vec<_> = if let Some(ref status_filter) = req.status {
        let lower = status_filter.to_ascii_lowercase();
        workspaces
            .into_iter()
            .filter(|w| format!("{:?}", w.status).to_ascii_lowercase() == lower)
            .collect()
    } else {
        workspaces
    };
    if filtered.is_empty() {
        return "No workspaces found.".to_string();
    }
    let mut out = String::from("Workspaces:\n");
    for w in &filtered {
        out.push_str(&format!(
            "- {} [{:?}] status={:?}",
            w.branch, w.workspace_type, w.status
        ));
        if let Some(ref mode) = w.active_agent {
            out.push_str(&format!(" mode={}", mode));
        }
        out.push('\n');
    }
    out
}

pub fn create_workspace(project_dir: &Path, req: CreateWorkspaceRequest) -> String {
    let Some(project_root) = project_dir.parent() else {
        return "Error: could not resolve project root from ship dir".to_string();
    };

    let branch = req.branch.as_deref().map(|b| b.to_string()).unwrap_or_else(|| {
        req.name
            .to_ascii_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    });

    let worktrees_dir = configured_worktree_dir(project_root);
    let worktree_path = worktrees_dir.join(&branch);
    let base_branch = req.base_branch.as_deref().unwrap_or("main");
    let kind = req.kind.to_ascii_lowercase();
    let is_service = kind == "service";

    if !is_service {
        if let Err(e) = std::fs::create_dir_all(&worktrees_dir) {
            return format!("Error: could not create worktrees dir: {}", e);
        }
        if let Some(msg) = create_git_worktree(project_root, &worktree_path, &branch, base_branch) {
            return msg;
        }
        if let Err(warn) = write_workspace_toml(&worktree_path, &req.name, &kind, &req.preset_id, &req.file_scope) {
            return warn;
        }
        format!(
            "Created workspace '{}' (branch: {}, kind: {})\nWorktree: {}",
            req.name, branch, kind, worktree_path.display()
        )
    } else {
        format!(
            "Created service workspace '{}' (branch: {}, kind: service)\n\
            No worktree created for service workspaces.",
            req.name, branch
        )
    }
}

fn create_git_worktree(
    project_root: &Path,
    worktree_path: &PathBuf,
    branch: &str,
    base_branch: &str,
) -> Option<String> {
    let status = ProcessCommand::new("git")
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap_or_default(),
            "-b",
            branch,
            base_branch,
        ])
        .current_dir(project_root)
        .status();

    let ok = match status {
        Ok(s) => s.success(),
        Err(e) => return Some(format!("Error running git worktree add: {}", e)),
    };

    if !ok {
        let status2 = ProcessCommand::new("git")
            .args([
                "worktree",
                "add",
                worktree_path.to_str().unwrap_or_default(),
                branch,
            ])
            .current_dir(project_root)
            .status();
        match status2 {
            Ok(s) if s.success() => {}
            Ok(_) => {
                return Some(format!(
                    "Error: git worktree add failed for branch '{}'. \
                    The branch may not exist or the worktree path is already in use.",
                    branch
                ))
            }
            Err(e) => return Some(format!("Error running git worktree add: {}", e)),
        }
    }
    None
}

fn write_workspace_toml(
    worktree_path: &PathBuf,
    name: &str,
    kind: &str,
    preset_id: &Option<String>,
    file_scope: &Option<String>,
) -> Result<(), String> {
    let workspace_toml_path = worktree_path.join("workspace.toml");
    let created_at = chrono::Utc::now().to_rfc3339();
    let mut toml_content = format!(
        "name = \"{}\"\nkind = \"{}\"\ncreated_at = \"{}\"\n",
        name, kind, created_at
    );
    if let Some(pid) = preset_id {
        toml_content.push_str(&format!("preset_id = \"{}\"\n", pid));
    }
    if let Some(scope) = file_scope {
        toml_content.push_str(&format!("file_scope = \"{}\"\n", scope));
    }
    std::fs::write(&workspace_toml_path, &toml_content).map_err(|e| {
        format!(
            "Warning: worktree created at '{}' but failed to write workspace.toml: {}",
            worktree_path.display(),
            e
        )
    })
}
