//! Projection system tests — prove that events are the source of truth.

#[cfg(test)]
mod workspace_tests {
    use crate::db::{block_on, db_path, ensure_db, open_db_at};
    use crate::events::types::event_types;
    use crate::events::EventEnvelope;
    use crate::projections::{EventBus, WorkspaceProjection};

    fn setup() -> sqlx::SqliteConnection {
        let ship_dir = crate::project::get_global_dir().unwrap();
        let base = ship_dir.parent().unwrap().to_path_buf();
        crate::project::init_project(base).unwrap();
        ensure_db().unwrap();
        open_db_at(&db_path().unwrap()).unwrap()
    }

    fn bus() -> EventBus {
        let mut bus = EventBus::new();
        bus.register(Box::new(WorkspaceProjection::new()));
        bus
    }

    fn workspace_created_event(branch: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::WORKSPACE_CREATED,
            branch,
            &serde_json::json!({
                "workspace_id": branch,
                "workspace_type": "feature",
                "status": "active",
                "active_agent": null,
                "providers": [],
                "mcp_servers": [],
                "skills": [],
                "is_worktree": false,
                "worktree_path": null
            }),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate()
    }

    fn workspace_compiled_event(branch: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::WORKSPACE_COMPILED,
            branch,
            &serde_json::json!({
                "config_generation": 1,
                "duration_ms": 42
            }),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate()
    }

    fn workspace_archived_event(branch: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::WORKSPACE_ARCHIVED,
            branch,
            &serde_json::json!({}),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate()
    }

    fn workspace_deleted_event(branch: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::WORKSPACE_DELETED,
            branch,
            &serde_json::json!({"branch": branch}),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate()
    }

    fn query_workspace_status(conn: &mut sqlx::SqliteConnection, branch: &str) -> Option<String> {
        let rows: Vec<(String,)> = block_on(async {
            sqlx::query_as(
                "SELECT status FROM workspace WHERE branch = ?",
            )
            .bind(branch)
            .fetch_all(&mut *conn)
            .await
        })
        .unwrap();
        rows.first().map(|r| r.0.clone())
    }

    fn query_workspace_exists(conn: &mut sqlx::SqliteConnection, branch: &str) -> bool {
        query_workspace_status(conn, branch).is_some()
    }

    // ── Tests ──────────────────────────────────────────────────────────

    #[test]
    fn dispatch_created_inserts_workspace_row() {
        let mut conn = setup();
        let bus = bus();
        let event = workspace_created_event("feature/proj-test-1");

        bus.dispatch(&event, &mut conn);

        let status = query_workspace_status(&mut conn, "feature/proj-test-1");
        assert_eq!(status.as_deref(), Some("active"), "created event must insert row");
    }

    #[test]
    fn dispatch_archived_updates_status() {
        let mut conn = setup();
        let bus = bus();

        bus.dispatch(&workspace_created_event("feature/proj-test-2"), &mut conn);
        bus.dispatch(&workspace_archived_event("feature/proj-test-2"), &mut conn);

        let status = query_workspace_status(&mut conn, "feature/proj-test-2");
        assert_eq!(status.as_deref(), Some("archived"));
    }

    #[test]
    fn dispatch_deleted_removes_row() {
        let mut conn = setup();
        let bus = bus();

        bus.dispatch(&workspace_created_event("feature/proj-test-3"), &mut conn);
        assert!(query_workspace_exists(&mut conn, "feature/proj-test-3"));

        bus.dispatch(&workspace_deleted_event("feature/proj-test-3"), &mut conn);
        assert!(!query_workspace_exists(&mut conn, "feature/proj-test-3"));
    }

    #[test]
    fn rebuild_from_events_produces_identical_state() {
        let mut conn = setup();
        let bus = bus();

        let events = vec![
            workspace_created_event("feature/rebuild-a"),
            workspace_compiled_event("feature/rebuild-a"),
            workspace_created_event("feature/rebuild-b"),
            workspace_archived_event("feature/rebuild-b"),
        ];

        for event in &events {
            bus.dispatch(event, &mut conn);
        }

        let status_a = query_workspace_status(&mut conn, "feature/rebuild-a");
        let status_b = query_workspace_status(&mut conn, "feature/rebuild-b");

        let report = bus.rebuild(&events, &mut conn).unwrap();
        assert_eq!(report.events_replayed, 4);

        let rebuilt_a = query_workspace_status(&mut conn, "feature/rebuild-a");
        let rebuilt_b = query_workspace_status(&mut conn, "feature/rebuild-b");
        assert_eq!(status_a, rebuilt_a, "rebuild must produce identical state for workspace A");
        assert_eq!(status_b, rebuilt_b, "rebuild must produce identical state for workspace B");
    }

    #[test]
    fn rebuild_report_lists_projections() {
        let mut conn = setup();
        let bus = bus();
        let report = bus.rebuild(&[], &mut conn).unwrap();
        assert!(
            report.projections_rebuilt.contains(&"workspace_state".to_string()),
            "report must list rebuilt projections"
        );
    }

    // ── Idempotency tests ─────────────────────────────────────────────

    #[test]
    fn compiled_projection_is_idempotent() {
        let mut conn = setup();
        let bus = bus();
        let branch = "feature/idem-compiled";

        bus.dispatch(&workspace_created_event(branch), &mut conn);

        let event = workspace_compiled_event(branch);
        bus.dispatch(&event, &mut conn);
        bus.dispatch(&event, &mut conn); // apply same event twice

        let config_gen: Option<i64> = block_on(async {
            sqlx::query_scalar("SELECT config_generation FROM workspace WHERE branch = ?")
                .bind(branch)
                .fetch_optional(&mut conn)
                .await
        })
        .unwrap();
        assert_eq!(config_gen, Some(1), "applying same compiled event twice must not double-increment");
    }

    #[test]
    fn tmux_assigned_projection_applies() {
        let mut conn = setup();
        let bus = bus();
        let branch = "feature/tmux-test";

        bus.dispatch(&workspace_created_event(branch), &mut conn);

        let event = EventEnvelope::new(
            event_types::WORKSPACE_TMUX_ASSIGNED,
            branch,
            &serde_json::json!({ "tmux_session_name": "ship-tmux" }),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate();
        bus.dispatch(&event, &mut conn);

        let name: Option<String> = block_on(async {
            sqlx::query_scalar("SELECT tmux_session_name FROM workspace WHERE branch = ?")
                .bind(branch)
                .fetch_optional(&mut conn)
                .await
        })
        .unwrap()
        .flatten();
        assert_eq!(name.as_deref(), Some("ship-tmux"));
    }

    #[test]
    fn started_projection_sets_worktree_fields() {
        let mut conn = setup();
        let bus = bus();
        let branch = "feature/started-test";

        bus.dispatch(&workspace_created_event(branch), &mut conn);

        let event = EventEnvelope::new(
            event_types::WORKSPACE_STARTED,
            branch,
            &serde_json::json!({
                "worktree_path": "/tmp/worktrees/started-test",
                "tmux_session_name": "ship-started"
            }),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate();
        bus.dispatch(&event, &mut conn);

        let row: Option<(i64, Option<String>, Option<String>)> = block_on(async {
            sqlx::query_as(
                "SELECT is_worktree, worktree_path, tmux_session_name \
                 FROM workspace WHERE branch = ?",
            )
            .bind(branch)
            .fetch_optional(&mut conn)
            .await
        })
        .unwrap();
        let (is_wt, wt_path, tmux) = row.unwrap();
        assert_eq!(is_wt, 1);
        assert_eq!(wt_path.as_deref(), Some("/tmp/worktrees/started-test"));
        assert_eq!(tmux.as_deref(), Some("ship-started"));
    }

    #[test]
    fn event_versioning_v1_compiled_without_new_fields() {
        // Scenario S7: v1 compiled event (with config_generation + duration_ms)
        // replays correctly even if future v2 adds fields with serde defaults.
        let mut conn = setup();
        let bus = bus();
        let branch = "feature/versioning-test";

        bus.dispatch(&workspace_created_event(branch), &mut conn);

        // Simulate a v1 event payload with only the original fields.
        let v1_event = EventEnvelope::new(
            event_types::WORKSPACE_COMPILED,
            branch,
            &serde_json::json!({
                "config_generation": 5,
                "duration_ms": 100
            }),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate();

        bus.dispatch(&v1_event, &mut conn);

        let config_gen: Option<i64> = block_on(async {
            sqlx::query_scalar("SELECT config_generation FROM workspace WHERE branch = ?")
                .bind(branch)
                .fetch_optional(&mut conn)
                .await
        })
        .unwrap();
        assert_eq!(config_gen, Some(5), "v1 event must set config_generation from payload");
    }

    #[test]
    fn rebuild_preserves_tmux_and_worktree_state() {
        let mut conn = setup();
        let bus = bus();
        let branch = "feature/rebuild-tmux";

        let events = vec![
            workspace_created_event(branch),
            EventEnvelope::new(
                event_types::WORKSPACE_STARTED,
                branch,
                &serde_json::json!({
                    "worktree_path": "/tmp/wt/rebuild",
                    "tmux_session_name": "ship-rebuild"
                }),
            )
            .unwrap()
            .with_context(Some(branch), None)
            .elevate(),
            workspace_compiled_event(branch),
        ];

        for e in &events {
            bus.dispatch(e, &mut conn);
        }

        // Capture state before rebuild.
        let before: Option<(Option<String>, Option<String>, i64)> = block_on(async {
            sqlx::query_as(
                "SELECT tmux_session_name, worktree_path, config_generation \
                 FROM workspace WHERE branch = ?",
            )
            .bind(branch)
            .fetch_optional(&mut conn)
            .await
        })
        .unwrap();

        bus.rebuild(&events, &mut conn).unwrap();

        let after: Option<(Option<String>, Option<String>, i64)> = block_on(async {
            sqlx::query_as(
                "SELECT tmux_session_name, worktree_path, config_generation \
                 FROM workspace WHERE branch = ?",
            )
            .bind(branch)
            .fetch_optional(&mut conn)
            .await
        })
        .unwrap();

        assert_eq!(before, after, "rebuild must reproduce identical state");
    }

    #[test]
    fn transactional_rollback_on_projection_failure() {
        // AC#4/S4: If projection fails, the event must also be rolled back.
        // We emit a workspace.activated event for a nonexistent branch via
        // the transactional path. The projection UPDATE affects 0 rows
        // (not an error), so instead we craft a malformed payload that
        // will fail deserialization in the projection handler.
        use crate::db::{ensure_db, open_db};
        use crate::events::store::append_event_with_conn;
        use crate::projections::{Projection, WorkspaceProjection};

        let ship_dir = crate::project::get_global_dir().unwrap();
        let base = ship_dir.parent().unwrap().to_path_buf();
        crate::project::init_project(base).unwrap();
        ensure_db().unwrap();

        // Create an event with a payload that will fail projection apply
        // (workspace.activated requires agent_id and providers fields).
        let bad_envelope = EventEnvelope::new(
            event_types::WORKSPACE_ACTIVATED,
            "feature/txn-rollback-test",
            &serde_json::json!("not-a-valid-payload"),
        )
        .unwrap()
        .elevate();

        let event_id = bad_envelope.id.clone();

        // Simulate the transactional path.
        let mut conn = open_db().unwrap();
        block_on(async { sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await }).unwrap();

        append_event_with_conn(&bad_envelope, &mut conn).unwrap();

        let proj = WorkspaceProjection::new();
        let apply_result = proj.apply(&bad_envelope, &mut conn);
        assert!(apply_result.is_err(), "projection must fail on bad payload");

        // Rollback since projection failed.
        block_on(async { sqlx::query("ROLLBACK").execute(&mut conn).await }).unwrap();
        drop(conn);

        // Verify the event was NOT persisted.
        let mut check_conn = open_db().unwrap();
        let count: i64 = block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE id = ?")
                .bind(&event_id)
                .fetch_one(&mut check_conn)
                .await
        })
        .unwrap();
        assert_eq!(count, 0, "event must be rolled back when projection fails");
    }
}
