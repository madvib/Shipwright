//! Projection system tests — prove that events are the source of truth.

#[cfg(test)]
mod tests {
    use crate::db::{block_on, db_path, ensure_db, open_db_at};
    use crate::events::types::event_types;
    use crate::events::EventEnvelope;
    use crate::projections::{EventBus, WorkspaceProjection};

    fn setup() -> sqlx::SqliteConnection {
        // init_project sets up the global dir + runs migrations
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

        // 1) Apply events live
        let events = vec![
            workspace_created_event("feature/rebuild-a"),
            workspace_compiled_event("feature/rebuild-a"),
            workspace_created_event("feature/rebuild-b"),
            workspace_archived_event("feature/rebuild-b"),
        ];

        for event in &events {
            bus.dispatch(event, &mut conn);
        }

        // capture live state
        let status_a = query_workspace_status(&mut conn, "feature/rebuild-a");
        let status_b = query_workspace_status(&mut conn, "feature/rebuild-b");

        // 2) Truncate and rebuild
        let report = bus.rebuild(&events, &mut conn).unwrap();
        assert_eq!(report.events_replayed, 4);

        // 3) Rebuilt state must match live state
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
}
