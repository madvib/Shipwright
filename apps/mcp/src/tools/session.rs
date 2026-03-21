use std::path::Path;

use runtime::workspace::{
    EndWorkspaceSessionRequest as RuntimeEndWorkspaceSessionRequest,
    get_active_workspace_session as runtime_get_active_workspace_session,
    record_workspace_session_progress as runtime_record_workspace_session_progress,
    start_workspace_session as runtime_start_workspace_session,
    end_workspace_session as runtime_end_workspace_session,
};

use crate::requests::{EndSessionRequest, LogProgressRequest, StartSessionRequest};

pub fn start_session(project_dir: &Path, req: StartSessionRequest, branch: &str) -> String {
    match runtime_start_workspace_session(
        project_dir,
        branch,
        req.goal,
        req.agent_id,
        req.provider_id,
    ) {
        Ok(session) => serde_json::to_string_pretty(&session)
            .unwrap_or_else(|e| format!("Error serializing workspace session: {}", e)),
        Err(err) => format!("Error: {}", err),
    }
}

pub fn end_session(project_dir: &Path, req: EndSessionRequest, branch: &str) -> String {
    let updated_workspace_ids = req.updated_workspace_ids.unwrap_or_default();
    let end_req = RuntimeEndWorkspaceSessionRequest {
        summary: req.summary,
        updated_workspace_ids,
        model: req.model,
        files_changed: req.files_changed,
        gate_result: req.gate_result,
    };
    match runtime_end_workspace_session(project_dir, branch, end_req) {
        Ok(session) => serde_json::to_string_pretty(&session)
            .unwrap_or_else(|e| format!("Error serializing workspace session: {}", e)),
        Err(err) => format!("Error: {}", err),
    }
}

pub fn log_progress(project_dir: &Path, req: LogProgressRequest, branch: &str) -> String {
    match runtime_get_active_workspace_session(project_dir, branch) {
        Ok(None) => {
            return format!(
                "No active session for '{}'. Call start_session first.",
                branch
            );
        }
        Err(err) => return format!("Error checking session: {}", err),
        Ok(Some(_)) => {}
    }
    match runtime_record_workspace_session_progress(project_dir, branch, &req.note) {
        Ok(()) => format!("Progress logged for session on '{}'.", branch),
        Err(e) => format!("Error logging progress: {}", e),
    }
}
