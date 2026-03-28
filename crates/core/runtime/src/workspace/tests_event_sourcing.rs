//! Failing tests that define the complete event-sourced contract for workspace writes.
//!
//! Every workspace mutation must emit a typed event. These tests are RED until
//! the implementation agent wires the missing event emissions.

#[cfg(test)]
mod tests {
    use crate::db::{block_on, db_path, ensure_db, open_db_at};
    use crate::workspace::*;
    use crate::db::workspace_state::demote_other_active_workspaces_db;
    use anyhow::Result;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = crate::project::get_global_dir().unwrap();
        let base = ship_dir.parent().unwrap().to_path_buf();
        crate::project::init_project(base).unwrap();
        ensure_db().unwrap();
        (tmp, ship_dir)
    }

    fn count_events_by_type_and_entity(event_type: &str, entity_id: &str) -> i64 {
        let mut conn = open_db_at(&db_path().unwrap()).unwrap();
        block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE event_type = ? AND entity_id = ?",
            )
            .bind(event_type)
            .bind(entity_id)
            .fetch_one(&mut conn)
            .await
        })
        .unwrap()
    }

    // ── test 1: create_workspace emits workspace.created ─────────────────────

    #[test]
    fn create_workspace_emits_created_event() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch = "feature/es-create-test";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;

        let count = count_events_by_type_and_entity("workspace.created", branch);
        assert_eq!(
            count, 1,
            "create_workspace must emit a workspace.created event, got {count} rows"
        );
        Ok(())
    }

    // ── test 2: delete_workspace emits workspace.deleted ─────────────────────

    #[test]
    fn delete_workspace_emits_deleted_event() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch = "feature/es-delete-test";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;

        delete_workspace(&ship_dir, branch)?;

        let count = count_events_by_type_and_entity("workspace.deleted", branch);
        assert_eq!(
            count, 1,
            "delete_workspace must emit a workspace.deleted event, got {count} rows"
        );
        Ok(())
    }

    // ── test 3: transition to Archived -> Active emits workspace.status_changed

    #[test]
    fn status_transition_to_idle_emits_status_changed_event() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch = "feature/es-status-idle-test";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;

        // Transition to Archived first so we can transition back to Active
        // (the non-archive return path that currently calls plain upsert_workspace).
        transition_workspace_status(&ship_dir, branch, WorkspaceStatus::Archived)?;
        transition_workspace_status(&ship_dir, branch, WorkspaceStatus::Active)?;

        let count = count_events_by_type_and_entity("workspace.status_changed", branch);
        assert_eq!(
            count, 1,
            "transition to Active must emit a workspace.status_changed event, got {count} rows"
        );
        Ok(())
    }

    // ── test 4: second non-archive transition also emits workspace.status_changed

    #[test]
    fn status_transition_to_frozen_emits_status_changed_event() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch = "feature/es-status-frozen-test";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;

        // Archive then reactivate: the Active transition is the gap under test.
        transition_workspace_status(&ship_dir, branch, WorkspaceStatus::Archived)?;
        transition_workspace_status(&ship_dir, branch, WorkspaceStatus::Active)?;

        // Two non-archive status transitions must each produce an event.
        transition_workspace_status(&ship_dir, branch, WorkspaceStatus::Archived)?;
        transition_workspace_status(&ship_dir, branch, WorkspaceStatus::Active)?;

        let count = count_events_by_type_and_entity("workspace.status_changed", branch);
        assert_eq!(
            count, 2,
            "each non-archive status transition must emit workspace.status_changed, got {count} rows"
        );
        Ok(())
    }

    // ── test 5: demote_other_active_workspaces_db emits archived event per workspace

    #[test]
    fn bulk_demotion_emits_archived_event_per_workspace() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch_a = "feature/es-demote-alpha";
        let branch_b = "feature/es-demote-beta";
        let branch_active = "feature/es-demote-keeper";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch_a.to_string(),
                ..Default::default()
            },
        )?;
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch_b.to_string(),
                ..Default::default()
            },
        )?;
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch_active.to_string(),
                ..Default::default()
            },
        )?;

        demote_other_active_workspaces_db(branch_active)?;

        let count_a = count_events_by_type_and_entity("workspace.archived", branch_a);
        let count_b = count_events_by_type_and_entity("workspace.archived", branch_b);

        assert_eq!(
            count_a, 1,
            "demoted workspace '{branch_a}' must have a workspace.archived event, got {count_a}"
        );
        assert_eq!(
            count_b, 1,
            "demoted workspace '{branch_b}' must have a workspace.archived event, got {count_b}"
        );
        Ok(())
    }

    // ── test 6: seed_service_workspace emits workspace.created ───────────────

    #[test]
    fn seed_service_workspace_emits_created_event() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        // Ensure no service workspace exists yet by using a fresh isolated DB
        // (guaranteed by the per-test get_global_dir isolation).
        seed_service_workspace(&ship_dir)?;

        let count = count_events_by_type_and_entity("workspace.created", "ship");
        assert_eq!(
            count, 1,
            "seed_service_workspace must emit a workspace.created event for branch 'ship', got {count}"
        );
        Ok(())
    }

    #[test]
    fn set_active_agent_on_inactive_workspace_emits_agent_changed_event() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        let req = CreateWorkspaceRequest {
            branch: "feature/agent-change-test".to_string(),
            workspace_type: Some(ShipWorkspaceKind::Feature),
            status: None,
            active_agent: None,
            providers: None,
            mcp_servers: None,
            skills: None,
            is_worktree: Some(false),
            worktree_path: None,
            context_hash: None,
        };
        create_workspace(&ship_dir, req)?;

        set_workspace_active_agent(&ship_dir, "feature/agent-change-test", None)?;

        let count = count_events_by_type_and_entity(
            "workspace.agent_changed",
            "feature/agent-change-test",
        );
        assert_eq!(
            count, 1,
            "set_workspace_active_agent on inactive workspace must emit workspace.agent_changed, got {count}"
        );
        Ok(())
    }
}
