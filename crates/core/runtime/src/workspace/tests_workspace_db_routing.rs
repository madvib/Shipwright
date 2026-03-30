//! Routing tests for event persistence architecture.
//!
//! Verifies that all events (actor, session, workspace.*) go to platform.db
//! only, with workspace isolation enforced by the workspace_id field.

#[cfg(test)]
mod tests {
    use crate::actor::create_actor;
    use crate::db::session_events::insert_session_with_started_event;
    use crate::db::{block_on, db_path, ensure_db, open_db_at};
    use crate::events::types::event_types;
    use crate::events::types::SessionStarted;
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

    // ── test 6: actor events route to platform.db only ────────────────────────

    #[test]
    fn actor_event_writes_to_platform_db_only() -> Result<()> {
        let (_tmp, _ship_dir) = setup();
        let ws_id = "ws-test-actor";
        let actor_id = "actor-1";

        create_actor(actor_id, "test-worker", "local", Some(ws_id), None)?;

        let mut platform_conn = open_db_at(&db_path()?)?;
        let platform_count: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE event_type = ? AND entity_id = ?",
            )
            .bind(event_types::ACTOR_CREATED)
            .bind(actor_id)
            .fetch_one(&mut platform_conn)
            .await
        })?;
        assert_eq!(platform_count, 1, "actor.created must be in platform.db");

        Ok(())
    }

    // ── test 7: session events route to platform.db only ─────────────────────

    #[test]
    fn session_event_writes_to_platform_db_only() -> Result<()> {
        let (_tmp, _ship_dir) = setup();
        let ws_id = "ws-test-session";
        let session_id = "sess-ws-routing-test";

        let payload = SessionStarted {
            goal: None,
            workspace_id: ws_id.to_string(),
            workspace_branch: "feature/ws-routing".to_string(),
            ..Default::default()
        };
        insert_session_with_started_event(session_id, ws_id, &payload)?;

        let mut platform_conn = open_db_at(&db_path()?)?;
        let platform_count: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE event_type = ? AND entity_id = ?",
            )
            .bind(event_types::SESSION_STARTED)
            .bind(session_id)
            .fetch_one(&mut platform_conn)
            .await
        })?;
        assert_eq!(platform_count, 1, "session.started must be in platform.db");

        Ok(())
    }

    // ── test 8: workspace isolation enforced by workspace_id field ────────────

    #[test]
    fn two_workspaces_isolated_by_workspace_id_in_platform_db() -> Result<()> {
        let (_tmp, _ship_dir) = setup();
        let ws_alpha = "ws-alpha";
        let ws_beta = "ws-beta";

        create_actor("actor-alpha", "worker", "local", Some(ws_alpha), None)?;
        create_actor("actor-beta", "worker", "local", Some(ws_beta), None)?;

        let mut conn = open_db_at(&db_path()?)?;

        // ws-alpha: own event present when filtered by workspace_id
        let alpha_own: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE entity_id = 'actor-alpha' AND workspace_id = ?",
            )
            .bind(ws_alpha)
            .fetch_one(&mut conn)
            .await
        })?;
        assert_eq!(alpha_own, 1, "actor-alpha event must have workspace_id = ws-alpha");

        // ws-alpha workspace_id must not contain ws-beta's actors
        let alpha_leak: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE entity_id = 'actor-beta' AND workspace_id = ?",
            )
            .bind(ws_alpha)
            .fetch_one(&mut conn)
            .await
        })?;
        assert_eq!(alpha_leak, 0, "actor-beta must not appear under workspace ws-alpha");

        // ws-beta: own event present when filtered by workspace_id
        let beta_own: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE entity_id = 'actor-beta' AND workspace_id = ?",
            )
            .bind(ws_beta)
            .fetch_one(&mut conn)
            .await
        })?;
        assert_eq!(beta_own, 1, "actor-beta event must have workspace_id = ws-beta");

        // ws-beta workspace_id must not contain ws-alpha's actors
        let beta_leak: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE entity_id = 'actor-alpha' AND workspace_id = ?",
            )
            .bind(ws_beta)
            .fetch_one(&mut conn)
            .await
        })?;
        assert_eq!(beta_leak, 0, "actor-alpha must not appear under workspace ws-beta");

        Ok(())
    }

    // ── test 9: workspace.* events stay in platform.db ────────────────────────

    #[test]
    fn workspace_event_writes_to_platform_db() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch = "feature/ws-platform-routing";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                ..Default::default()
            },
        )?;
        activate_workspace(&ship_dir, branch)?;

        let mut platform_conn = open_db_at(&db_path()?)?;
        let count: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE event_type = ? AND entity_id = ?",
            )
            .bind(event_types::WORKSPACE_ACTIVATED)
            .bind(branch)
            .fetch_one(&mut platform_conn)
            .await
        })?;
        assert_eq!(count, 1, "workspace.activated must be in platform.db, got {count} rows");

        Ok(())
    }
}
