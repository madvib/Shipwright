mod agents;
mod branch;
mod capability;
mod compat;
mod compat_workspace;
mod feature_capability;
mod init;
mod kv;
mod migrations;
mod schema;
mod schema_ext;
mod session;
mod types;
mod util;
mod workspace;

#[cfg(test)]
mod tests;

// ─── Types ────────────────────────────────────────────────────────────────────
pub use types::{
    AgentArtifactRegistryDb, AgentModeDb, AgentRuntimeSettingsDb, CapabilityDb, CapabilityMapDb,
    DatabaseMigrationReport, FeatureBranchLinks, WorkspaceDbListRow, WorkspaceDbRow,
    WorkspaceSessionDb, WorkspaceSessionRecordDb, WorkspaceUpsert,
};

// ─── Init / migrations ────────────────────────────────────────────────────────
pub use init::{
    clear_global_migration_meta, clear_project_migration_meta, ensure_global_database,
    ensure_project_database, mark_migration_meta_complete_global,
    mark_migration_meta_complete_project, migration_meta_complete_global,
    migration_meta_complete_project, open_global_connection, open_project_connection,
    project_db_path,
};

// ─── KV / managed state ───────────────────────────────────────────────────────
pub use kv::{get_managed_state_db, set_managed_state_db};

// ─── Agents ───────────────────────────────────────────────────────────────────
pub use agents::{
    delete_agent_mode_db, get_agent_artifact_registry_by_external_id_db,
    get_agent_artifact_registry_by_uuid_db, get_agent_runtime_settings_db, list_agent_modes_db,
    set_agent_runtime_settings_db, upsert_agent_artifact_registry_db, upsert_agent_mode_db,
};

// ─── Branch / entity links ────────────────────────────────────────────────────
pub use branch::{
    clear_branch_doc, clear_branch_link, get_branch_doc, get_branch_link,
    get_feature_agent_config, get_feature_agent_providers, get_feature_by_branch_links,
    get_feature_links, list_target_features_db, replace_target_features_db, set_branch_doc,
    set_branch_link,
};

// ─── Feature-capability links ─────────────────────────────────────────────────
pub use feature_capability::{get_feature_primary_capability_db, set_feature_primary_capability_db};

// ─── Workspace CRUD ───────────────────────────────────────────────────────────
pub use workspace::{
    delete_workspace_db, demote_other_active_workspaces_db, get_workspace_db, list_workspaces_db,
    upsert_workspace_db,
};

// ─── Capability CRUD ──────────────────────────────────────────────────────────
pub use capability::{
    list_capabilities_db, list_capability_maps_db, upsert_capability_db, upsert_capability_map_db,
};

// ─── Sessions ─────────────────────────────────────────────────────────────────
pub use session::{
    get_active_workspace_session_db, get_workspace_session_db, get_workspace_session_record_db,
    insert_workspace_session_db, insert_workspace_session_record_db,
    list_workspace_sessions_db, update_workspace_session_db,
};

// ─── Utilities (pub for use by module crates and tests) ───────────────────────
pub use util::block_on;
