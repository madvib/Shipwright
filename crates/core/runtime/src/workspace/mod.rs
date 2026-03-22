mod types;
mod types_session;
pub(crate) mod helpers;
mod context_hash;
pub(crate) mod compile;
mod crud;
mod lifecycle;
mod session;
mod session_lifecycle;

#[cfg(test)]
mod tests_types;
#[cfg(test)]
mod tests_crud;
#[cfg(test)]
mod tests_compile;
#[cfg(test)]
mod tests_session;

// Re-export all public types so `use runtime::workspace::*` continues to work.
pub use types::{
    Environment, Process, ProcessStatus, ShipWorkspaceKind, Workspace, WorkspaceStatus,
};
pub use types_session::{
    CreateWorkspaceRequest, EndWorkspaceSessionRequest, WorkspaceProviderMatrix,
    WorkspaceRepairReport, WorkspaceSession, WorkspaceSessionRecord, WorkspaceSessionStatus,
};

// Re-export all public functions.
pub use crud::{
    delete_workspace, get_workspace, get_workspace_provider_matrix, list_workspaces,
    repair_workspace, upsert_workspace,
};
pub use helpers::validate_workspace_transition;
pub use lifecycle::{
    activate_workspace, create_workspace, get_active_workspace_type, seed_service_workspace,
    set_workspace_active_agent, sync_workspace, transition_workspace_status,
};
pub use session::{
    get_active_workspace_session, get_workspace_session_record, list_workspace_sessions,
    record_workspace_session_progress,
};
pub use session_lifecycle::{end_workspace_session, start_workspace_session};
