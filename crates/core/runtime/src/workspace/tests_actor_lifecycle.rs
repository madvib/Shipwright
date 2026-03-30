//! Tests for actor auto-creation on workspace activation.

#[cfg(test)]
mod tests {
    use crate::db::{block_on, ensure_db, open_db};
    use crate::workspace::*;
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

    /// Query actor lifecycle events from platform.db for a workspace.
    /// Returns (actor_id, event_type) pairs ordered by event id.
    fn query_actor_events(ws_id: &str) -> Vec<(String, String)> {
        let mut conn = open_db().unwrap();
        block_on(async {
            sqlx::query_as::<_, (String, String)>(
                "SELECT actor_id, event_type FROM events \
                 WHERE workspace_id = ? AND actor_id IS NOT NULL \
                 AND event_type IN ('actor.created', 'actor.stopped') \
                 ORDER BY id",
            )
            .bind(ws_id)
            .fetch_all(&mut conn)
            .await
        })
        .unwrap()
    }

    // ── test 1: activate workspace creates actor ─────────────────────────────

    #[test]
    fn activate_workspace_creates_actor_event_in_platform_db() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch = "feature/actor-auto-1";

        activate_workspace(&ship_dir, branch)?;

        let ws_id = crate::workspace::helpers::workspace_id_from_branch(branch);
        let events = query_actor_events(&ws_id);

        assert_eq!(events.len(), 1, "expected exactly 1 actor event, got {}", events.len());
        let (actor_id, event_type) = &events[0];
        assert!(
            actor_id.ends_with("/default"),
            "actor id must end with /default when no agent set, got: {actor_id}"
        );
        assert_eq!(event_type, "actor.created");
        Ok(())
    }

    // ── test 2: idempotent — second activation does not duplicate actor ──────

    #[test]
    fn activate_workspace_twice_does_not_duplicate_actor() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch = "feature/actor-auto-2";

        activate_workspace(&ship_dir, branch)?;
        activate_workspace(&ship_dir, branch)?;

        let ws_id = crate::workspace::helpers::workspace_id_from_branch(branch);
        let events = query_actor_events(&ws_id);
        let created_count = events.iter().filter(|(_, et)| et == "actor.created").count();

        assert_eq!(
            created_count,
            1,
            "second activation must not create duplicate actor.created event, got {}",
            created_count
        );
        Ok(())
    }

    // ── test 3: agent change stops old actor, creates new one ────────────────

    #[test]
    fn agent_change_stops_old_actor_and_creates_new() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch = "feature/actor-auto-3";

        // Register an agent in the project config so validation passes
        let config = crate::config::ProjectConfig {
            modes: vec![crate::config::AgentProfile {
                id: "test-agent".to_string(),
                name: "Test Agent".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        crate::config::save_config(&config, Some(ship_dir.clone()))?;

        // Create workspace first (needed for set_workspace_active_agent lookup)
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;

        // First activation with default agent
        activate_workspace(&ship_dir, branch)?;

        let ws_id = crate::workspace::helpers::workspace_id_from_branch(branch);
        let events_before = query_actor_events(&ws_id);
        assert_eq!(events_before.len(), 1);

        // Change agent and reactivate
        set_workspace_active_agent(&ship_dir, branch, Some("test-agent"))?;
        activate_workspace(&ship_dir, branch)?;

        let events_after = query_actor_events(&ws_id);

        let stopped_count = events_after.iter().filter(|(_, et)| et == "actor.stopped").count();
        let created_count = events_after.iter().filter(|(_, et)| et == "actor.created").count();

        assert_eq!(stopped_count, 1, "old actor must be stopped");
        assert_eq!(created_count, 2, "expected 2 actor.created events (initial + new agent)");

        // Verify the latest created actor has the correct agent in its ID
        let new_actor_id = events_after
            .iter()
            .filter(|(_, et)| et == "actor.created")
            .last()
            .map(|(id, _)| id)
            .unwrap();
        assert!(
            new_actor_id.ends_with("/test-agent"),
            "new actor id must end with /test-agent, got: {new_actor_id}"
        );

        Ok(())
    }
}
