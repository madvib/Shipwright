use super::*;

#[test]
fn studio_server_registers_exactly_studio_tools() {
    let server = StudioServer::new();
    let names = server.registered_tool_names();
    let expected: &[&str] = &[
        "open_project",
        "pull_agents",
        "list_local_agents",
        "push_bundle",
        "list_project_skills",
        "list_skills",
        "write_skill_file",
        "delete_skill_file",
        "get_skill_vars",
        "set_skill_var",
        "list_skill_vars",
        "list_session_files",
        "read_session_file",
        "write_session_file",
        "delete_session_file",
    ];
    for tool in expected {
        assert!(
            names.iter().any(|n| n == tool),
            "{tool} missing from studio router"
        );
    }
    assert_eq!(
        names.len(),
        expected.len(),
        "studio build should register exactly {} tools, got: {:?}",
        expected.len(),
        names
    );
}

#[test]
fn studio_server_does_not_include_workspace_tools() {
    let server = StudioServer::new();
    let names = server.registered_tool_names();
    let excluded = [
        "activate_workspace",
        "create_workspace",
        "complete_workspace",
        "list_workspaces",
        "list_stale_worktrees",
        "start_session",
        "end_session",
        "log_progress",
        "set_agent",
        "list_events",
    ];
    for tool in excluded {
        assert!(
            !names.iter().any(|n| n == tool),
            "{tool} should NOT be in studio router"
        );
    }
}
