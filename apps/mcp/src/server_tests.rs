use super::*;
use runtime::{AgentProfile, add_agent};
use ship_docs::init_project;
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
    ShipServer::enforce_mode_tool_gate(&project_dir, "create_workspace_tool")
        .expect("create workspace must remain control-plane allowed");
    ShipServer::enforce_mode_tool_gate(&project_dir, "repair_workspace")
        .expect("workspace repair must remain control-plane allowed");

    let blocked = ShipServer::enforce_mode_tool_gate(&project_dir, "update_note")
        .expect_err("update_note should be blocked");
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

    let created = server
        .create_workspace_tool(Parameters(CreateWorkspaceToolRequest {
            branch: "feature/mode-control-plane".to_string(),
            workspace_type: Some("feature".to_string()),
            environment_id: None,
            feature_id: None,
            target_id: None,
            agent_id: None,
            is_worktree: Some(false),
            worktree_path: None,
            activate: Some(true),
        }))
        .await;
    assert!(
        created.contains("\"branch\": \"feature/mode-control-plane\""),
        "unexpected create workspace response: {}",
        created
    );

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

// ── normalize_mode_tool_id ──────────────────────────────────────────

#[test]
fn strips_ship_prefix() {
    assert_eq!(ShipServer::normalize_mode_tool_id("ship_create_note"), "create_note");
}

#[test]
fn strips_tool_suffix() {
    assert_eq!(ShipServer::normalize_mode_tool_id("list_notes_tool"), "list_notes");
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
    assert_eq!(ShipServer::normalize_mode_tool_id("Ship-Create-Note"), "create_note");
}

#[test]
fn already_normalized_unchanged() {
    assert_eq!(ShipServer::normalize_mode_tool_id("create_note"), "create_note");
}

#[test]
fn trims_whitespace() {
    assert_eq!(ShipServer::normalize_mode_tool_id("  ship_foo_tool  "), "foo");
}

// ── is_core_tool ────────────────────────────────────────────────────

#[test]
fn core_tools_are_recognized() {
    let core_tools = [
        "open_project",
        "create_note",
        "create_adr",
        "activate_workspace",
        "create_workspace",
        "complete_workspace",
        "set_agent",
        "start_session",
        "end_session",
        "log_progress",
        "list_skills",
        "create_job",
        "update_job",
        "list_jobs",
    ];
    for tool in core_tools {
        assert!(ShipServer::is_core_tool(tool), "{tool} should be a core tool");
    }
}

#[test]
fn core_tool_with_prefix_and_suffix() {
    assert!(ShipServer::is_core_tool("ship_create_workspace_tool"));
    assert!(ShipServer::is_core_tool("ship_log_progress_tool"));
}

#[test]
fn non_core_tool_is_not_core() {
    assert!(!ShipServer::is_core_tool("create_spec"));
    assert!(!ShipServer::is_core_tool("update_note"));
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
    assert!(ShipServer::mode_allows_tool("ship_create_spec_tool", &active));
}
