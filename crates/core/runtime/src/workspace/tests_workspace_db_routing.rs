//! Routing tests for per-workspace SQLite databases.
//!
//! Verifies that actor/session events route to workspace DBs, workspace.* events
//! stay in platform.db, and two workspaces have isolated DBs.

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

    // ── test 6: actor events route to workspace DB, not platform.db ───────────

    #[test]
    fn actor_event_writes_to_workspace_db_not_platform() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let ws_id = "ws-test-actor";
        let actor_id = "actor-1";

        create_actor(actor_id, "test-worker", "local", Some(ws_id), None)?;

        // Must appear in workspace DB
        let mut ws_conn = open_workspace_db(&ship_dir, ws_id)?;
        let ws_count: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE event_type = ? AND entity_id = ?",
            )
            .bind(event_types::ACTOR_CREATED)
            .bind(actor_id)
            .fetch_one(&mut ws_conn)
            .await
        })?;
        assert_eq!(ws_count, 1, "actor.created must be in workspace DB");

        // Must NOT appear in platform.db
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
        assert_eq!(platform_count, 0, "actor.created must NOT be in platform.db");

        Ok(())
    }

    // ── test 7: session events route to workspace DB, not platform.db ─────────

    #[test]
    fn session_event_writes_to_both_platform_and_workspace_db() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let ws_id = "ws-test-session";
        let session_id = "sess-ws-routing-test";

        let payload = SessionStarted {
            goal: None,
            workspace_id: ws_id.to_string(),
            workspace_branch: "feature/ws-routing".to_string(),
            ..Default::default()
        };
        insert_session_with_started_event(session_id, ws_id, &payload)?;

        // Must appear in workspace DB
        let mut ws_conn = open_workspace_db(&ship_dir, ws_id)?;
        let ws_count: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE event_type = ? AND entity_id = ?",
            )
            .bind(event_types::SESSION_STARTED)
            .bind(session_id)
            .fetch_one(&mut ws_conn)
            .await
        })?;
        assert_eq!(ws_count, 1, "session.started must be in workspace DB");

        // Must ALSO appear in platform.db (elevated for SessionProjection)
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

    // ── test 8: two workspaces have isolated DBs ───────────────────────────────

    #[test]
    fn two_workspaces_have_isolated_dbs() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let ws_alpha = "ws-alpha";
        let ws_beta = "ws-beta";

        create_actor("actor-alpha", "worker", "local", Some(ws_alpha), None)?;
        create_actor("actor-beta", "worker", "local", Some(ws_beta), None)?;

        // ws-alpha DB: own event present, ws-beta event absent
        let mut alpha_conn = open_workspace_db(&ship_dir, ws_alpha)?;
        let alpha_own: i64 = block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE entity_id = 'actor-alpha'")
                .fetch_one(&mut alpha_conn)
                .await
        })?;
        assert_eq!(alpha_own, 1, "ws-alpha db must contain actor-alpha event");

        let alpha_leak: i64 = block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE entity_id = 'actor-beta'")
                .fetch_one(&mut alpha_conn)
                .await
        })?;
        assert_eq!(alpha_leak, 0, "ws-alpha db must not contain actor-beta event");

        // ws-beta DB: own event present, ws-alpha event absent
        let mut beta_conn = open_workspace_db(&ship_dir, ws_beta)?;
        let beta_own: i64 = block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE entity_id = 'actor-beta'")
                .fetch_one(&mut beta_conn)
                .await
        })?;
        assert_eq!(beta_own, 1, "ws-beta db must contain actor-beta event");

        let beta_leak: i64 = block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE entity_id = 'actor-alpha'")
                .fetch_one(&mut beta_conn)
                .await
        })?;
        assert_eq!(beta_leak, 0, "ws-beta db must not contain actor-alpha event");

        // The two DB paths must be different files
        let alpha_path = workspace_db_path(&ship_dir, ws_alpha);
        let beta_path = workspace_db_path(&ship_dir, ws_beta);
        assert_ne!(alpha_path, beta_path, "ws-alpha and ws-beta must have different DB paths");

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
