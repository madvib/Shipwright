use super::*;
use runtime::project::init_project;
use runtime::workspace::{
    CreateWorkspaceRequest as RuntimeCreateWorkspaceRequest, ShipWorkspaceKind,
    create_workspace as runtime_create_workspace,
};
use runtime::{AgentProfile, add_agent};
use tempfile::tempdir;

#[test]
fn mode_gate_normalizes_and_blocks_disallowed_tools() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    add_agent(
        Some(project_dir.clone()),
        AgentProfile {
            id: "mode-gate-test".to_string(),
            name: "Mode Gate Test".to_string(),
            active_tools: vec!["ship_list_notes".to_string()],
            ..Default::default()
        },
    )
    .expect("add agent");
    runtime::set_active_agent(Some(project_dir.clone()), Some("mode-gate-test"))
        .expect("set active agent");

    ShipServer::enforce_mode_tool_gate(&project_dir, "list_notes").expect("list_notes allowed");
    ShipServer::enforce_mode_tool_gate(&project_dir, "ship_list_notes_tool")
        .expect("prefixed note tool allowed");
    ShipServer::enforce_mode_tool_gate(&project_dir, "create_workspace")
        .expect("create_workspace must be core");
    ShipServer::enforce_mode_tool_gate(&project_dir, "complete_workspace")
        .expect("complete_workspace must be core");

    let blocked = ShipServer::enforce_mode_tool_gate(&project_dir, "create_spec")
        .expect_err("create_spec should be blocked");
    assert!(
        blocked.contains("blocked by active mode"),
        "unexpected mode gate message: {}",
        blocked
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_workspace_control_plane_round_trip() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    let server = ShipServer::new();
    *server.active_project.lock().await = Some(project_dir.clone());

    runtime_create_workspace(
        &project_dir,
        RuntimeCreateWorkspaceRequest {
            branch: "feature/mode-control-plane".to_string(),
            workspace_type: Some(ShipWorkspaceKind::Feature),
            status: None,
            active_agent: None,
            providers: None,
            mcp_servers: None,
            skills: None,
            is_worktree: Some(false),
            worktree_path: None,
            context_hash: None,
        },
    )
    .expect("create workspace for test");

    let fetched = server
        .resolve_resource_uri("ship://workspaces/feature/mode-control-plane", &project_dir)
        .await
        .expect("workspace resource");
    assert!(
        fetched.contains("\"id\": \"feature-mode-control-plane\""),
        "unexpected get workspace response: {}",
        fetched
    );

    let sessions_before = server
        .resolve_resource_uri("ship://sessions/feature/mode-control-plane", &project_dir)
        .await
        .expect("sessions resource");
    assert_eq!(
        sessions_before.trim(),
        "[]",
        "expected no sessions before start, got {}",
        sessions_before
    );
}

// ── notification peer lifecycle ─────────────────────────────────────

#[tokio::test]
async fn notification_peer_starts_as_none() {
    let server = ShipServer::new();
    assert!(
        server.notification_peer.lock().await.is_none(),
        "notification_peer should be None after construction"
    );
}

#[tokio::test]
async fn notify_resources_changed_is_noop_without_peer() {
    let server = ShipServer::new();
    // Should not panic when no peer is stored
    server.notify_resources_changed().await;
}

// ── normalize_mode_tool_id ──────────────────────────────────────────

#[test]
fn strips_ship_prefix() {
    assert_eq!(
        ShipServer::normalize_mode_tool_id("ship_create_note"),
        "create_note"
    );
}

#[test]
fn strips_tool_suffix() {
    assert_eq!(
        ShipServer::normalize_mode_tool_id("list_notes_tool"),
        "list_notes"
    );
}

#[test]
fn strips_both_prefix_and_suffix() {
    assert_eq!(
        ShipServer::normalize_mode_tool_id("ship_create_workspace_tool"),
        "create_workspace"
    );
}

#[test]
fn lowercases_and_replaces_hyphens() {
    assert_eq!(
        ShipServer::normalize_mode_tool_id("Ship-Create-Note"),
        "create_note"
    );
}

#[test]
fn already_normalized_unchanged() {
    assert_eq!(
        ShipServer::normalize_mode_tool_id("create_note"),
        "create_note"
    );
}

#[test]
fn trims_whitespace() {
    assert_eq!(
        ShipServer::normalize_mode_tool_id("  ship_foo_tool  "),
        "foo"
    );
}

// ── is_core_tool ────────────────────────────────────────────────────

#[test]
fn core_tools_are_recognized() {
    let platform_tools = [
        "open_project",
        "activate_workspace",
        "create_workspace",
        "complete_workspace",
        "set_agent",
        "start_session",
        "end_session",
        "log_progress",
        "get_session",
        "list_sessions",
        "list_skills",
    ];
    for tool in platform_tools {
        assert!(
            ShipServer::is_core_tool(tool),
            "{tool} should be a core tool"
        );
    }
}

#[test]
fn core_tool_with_prefix_and_suffix() {
    assert!(ShipServer::is_core_tool("ship_create_workspace_tool"));
    assert!(ShipServer::is_core_tool("ship_log_progress_tool"));
}

#[test]
#[cfg(feature = "unstable")]
fn unstable_tools_are_core_when_feature_enabled() {
    assert!(
        ShipServer::is_core_tool("create_adr"),
        "create_adr should be a core tool with unstable feature"
    );
}

#[test]
fn non_core_tool_is_not_core() {
    assert!(!ShipServer::is_core_tool("create_spec"));
    assert!(!ShipServer::is_core_tool("random_tool"));
}

// ── mode_allows_tool ────────────────────────────────────────────────

#[test]
fn empty_active_tools_allows_everything() {
    assert!(ShipServer::mode_allows_tool("anything", &[]));
    assert!(ShipServer::mode_allows_tool("create_spec", &[]));
}

#[test]
fn tool_in_active_tools_allowed() {
    let active = vec!["create_spec".to_string(), "list_notes".to_string()];
    assert!(ShipServer::mode_allows_tool("create_spec", &active));
    assert!(ShipServer::mode_allows_tool("list_notes", &active));
}

#[test]
fn tool_not_in_active_tools_blocked() {
    let active = vec!["create_spec".to_string()];
    assert!(!ShipServer::mode_allows_tool("delete_everything", &active));
}

#[test]
fn normalization_applied_to_both_sides() {
    let active = vec!["ship_create_spec_tool".to_string()];
    assert!(ShipServer::mode_allows_tool("create_spec", &active));
    assert!(ShipServer::mode_allows_tool(
        "ship_create_spec_tool",
        &active
    ));
}

// ── tool registration surface ──────────────────────────────────────

#[test]
fn stable_build_registers_only_platform_tools() {
    let server = ShipServer::new();
    let names = server.registered_tool_names();
    let expected: &[&str] = &[
        "event",
        "open_project",
        "set_agent",
        "activate_workspace",
        "list_workspaces",
        "create_workspace",
        "complete_workspace",
        "list_stale_worktrees",
        "start_session",
        "end_session",
        "log_progress",
        "get_session",
        "list_sessions",
        "list_session_files",
        "read_session_file",
        "write_session_file",
        "list_skills",
        "get_skill_vars",
        "set_skill_var",
        "list_skill_vars",
        "mesh_send",
        "mesh_broadcast",
        "mesh_discover",
        "mesh_status",
        // Studio-only tools NOT here: pull_agents, list_local_agents,
        // push_bundle, write/delete_skill_file, list_project_skills, emit_studio_event
        // Removed: list_events — agents do not have read access to the event store
    ];
    for tool in expected {
        assert!(
            names.iter().any(|n| n == tool),
            "{tool} missing from router"
        );
    }
    #[cfg(not(feature = "unstable"))]
    assert_eq!(
        names.len(),
        expected.len(),
        "stable build should register exactly {} tools, got: {:?}",
        expected.len(),
        names
    );
}

#[test]
#[cfg(feature = "unstable")]
fn unstable_build_registers_all_tools() {
    let server = ShipServer::new();
    let names = server.registered_tool_names();
    assert!(
        names.iter().any(|n| n == "create_adr"),
        "create_adr missing from unstable router"
    );
    // stable (24) + unstable (1)
    assert_eq!(
        names.len(),
        25,
        "unstable build should register 25 tools, got: {:?}",
        names
    );
}

