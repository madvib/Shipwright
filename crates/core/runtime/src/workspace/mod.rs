pub(crate) mod compile;
mod context_hash;
mod crud;
pub(crate) mod event_upserts;
pub(crate) mod helpers;
mod lifecycle;
mod lifecycle_actors;
mod reconcile;
mod session;
mod session_lifecycle;
pub(crate) mod types;
pub(crate) mod types_session;

#[cfg(test)]
mod tests_compile;
#[cfg(test)]
mod tests_crud;
#[cfg(test)]
mod tests_workspace_db;
#[cfg(test)]
mod tests_workspace_db_routing;
#[cfg(test)]
mod tests_events;
#[cfg(test)]
mod tests_session;
#[cfg(test)]
mod tests_session_events;
#[cfg(test)]
mod tests_types;
#[cfg(test)]
mod tests_event_sourcing;
#[cfg(test)]
mod tests_actor_lifecycle;

// Re-export all public types so `use runtime::workspace::*` continues to work.
pub use types::{
    Environment, Process, ProcessStatus, Workspace, WorkspaceStatus,
};
pub use types_session::{
    CreateWorkspaceRequest, EndWorkspaceSessionRequest, WorkspaceProviderMatrix,
    WorkspaceRepairReport, WorkspaceSession, WorkspaceSessionRecord, WorkspaceSessionStatus,
};

// Re-export all public functions.
pub use crud::{
    delete_workspace, get_workspace, get_workspace_by_id, get_workspace_provider_matrix,
    list_workspaces, repair_workspace,
};
pub use event_upserts::{
    emit_workspace_archived_event, upsert_workspace_on_activate, upsert_workspace_on_archived,
    upsert_workspace_on_compile_failed, upsert_workspace_on_compiled, upsert_workspace_on_created,
    upsert_workspace_on_deleted, upsert_workspace_on_status_changed,
};
pub use helpers::validate_workspace_transition;
pub use lifecycle::{
    activate_workspace, create_workspace, seed_service_workspace,
    set_workspace_active_agent, set_workspace_started, set_workspace_tmux_session, sync_workspace,
    transition_workspace_status,
};
pub use session::{
    get_active_workspace_session, get_workspace_session_record, list_workspace_sessions,
    record_workspace_session_progress,
};
pub use reconcile::reconcile_workspace;
pub use session_lifecycle::{end_workspace_session, start_workspace_session};
pub use crate::db::workspace_db::{open_workspace_db, workspace_db_path};
