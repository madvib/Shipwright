//! Tests for actor auto-creation on workspace activation.

#[cfg(test)]
mod tests {
    use crate::db::{block_on, ensure_db};
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

    fn query_actors(
        ship_dir: &std::path::Path,
        ws_id: &str,
    ) -> Vec<(String, String, String)> {
        let mut conn = open_workspace_db(ship_dir, ws_id).unwrap();
        block_on(async {
            sqlx::query_as::<_, (String, String, String)>(
                "SELECT id, kind, status FROM actors ORDER BY created_at",
            )
            .fetch_all(&mut conn)
            .await
        })
        .unwrap()
    }

    // ── test 1: activate workspace creates actor ─────────────────────────────

    #[test]
    fn activate_workspace_creates_actor_in_workspace_db() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch = "feature/actor-auto-1";

        activate_workspace(&ship_dir, branch)?;

        let ws_id = crate::workspace::helpers::workspace_id_from_branch(branch);
        let actors = query_actors(&ship_dir, &ws_id);

        assert_eq!(actors.len(), 1, "expected exactly 1 actor, got {}", actors.len());
        let (id, kind, status) = &actors[0];
        assert!(
            id.ends_with("/default"),
            "actor id must end with /default when no agent set, got: {id}"
        );
        assert_eq!(kind, "workspace");
        assert_eq!(status, "created");
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
        let actors = query_actors(&ship_dir, &ws_id);

        assert_eq!(
            actors.len(),
            1,
            "second activation must not create duplicate actor, got {} actors",
            actors.len()
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
        let actors_before = query_actors(&ship_dir, &ws_id);
        assert_eq!(actors_before.len(), 1);

        // Change agent and reactivate
        set_workspace_active_agent(&ship_dir, branch, Some("test-agent"))?;
        activate_workspace(&ship_dir, branch)?;

        let actors_after = query_actors(&ship_dir, &ws_id);

        // Old actor should be stopped, new actor should be created
        let stopped_count = actors_after.iter().filter(|(_, _, s)| s == "stopped").count();
        let created_count = actors_after.iter().filter(|(_, _, s)| s == "created").count();

        assert_eq!(stopped_count, 1, "old actor must be stopped");
        assert_eq!(created_count, 1, "new actor must be created");
        assert_eq!(actors_after.len(), 2, "expected 2 actors total (old stopped + new created)");

        // Verify the new actor has the correct agent in its ID
        let new_actor = actors_after.iter().find(|(_, _, s)| s == "created").unwrap();
        assert!(
            new_actor.0.ends_with("/test-agent"),
            "new actor id must end with /test-agent, got: {}",
            new_actor.0
        );

        Ok(())
    }
}
