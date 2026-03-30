//! Async projection integration tests.

#[cfg(test)]
mod tests {
    use crate::db::{block_on, db_path, ensure_db, open_db_at};
    use crate::events::envelope::EventEnvelope;
    use crate::events::store::{EventStore, SqliteEventStore};
    use crate::events::types::event_types;
    use crate::projections::{EventBus, WorkspaceProjection};

    fn setup() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        crate::project::init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        tmp
    }

    fn workspace_created(branch: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::WORKSPACE_CREATED,
            branch,
            &serde_json::json!({
                "workspace_id": branch,
                "workspace_type": "feature",
                "status": "active",
                "providers": [],
                "mcp_servers": [],
                "skills": [],
                "is_worktree": false
            }),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .with_actor_id(branch)
        .elevate()
    }

    fn workspace_archived(branch: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::WORKSPACE_ARCHIVED,
            branch,
            &serde_json::json!({}),
        )
        .unwrap()
        .with_context(Some(branch), None)
        .elevate()
    }

    // ── Test: projection rebuild from events table produces correct state ──────

    #[test]
    fn projection_rebuild_from_events_table_produces_correct_state() {
        let _tmp = setup();

        let store = SqliteEventStore::new().unwrap();
        let events = vec![
            workspace_created("feature/rebuild-src-a"),
            workspace_archived("feature/rebuild-src-a"),
            workspace_created("feature/rebuild-src-b"),
        ];
        for ev in &events {
            store.append(ev).unwrap();
        }

        let mut conn = open_db_at(&db_path().unwrap()).unwrap();
        let mut bus = EventBus::new();
        bus.register(Box::new(WorkspaceProjection::new()));
        let report = bus.rebuild(&events, &mut conn).unwrap();

        assert_eq!(report.events_replayed, 3);
        assert!(
            report.projections_rebuilt.contains(&"workspace_state".to_string()),
            "rebuild report must list workspace_state"
        );

        let status_a: Option<String> = block_on(async {
            sqlx::query_scalar("SELECT status FROM workspace WHERE branch = ?")
                .bind("feature/rebuild-src-a")
                .fetch_optional(&mut conn)
                .await
        })
        .unwrap();
        let status_b: Option<String> = block_on(async {
            sqlx::query_scalar("SELECT status FROM workspace WHERE branch = ?")
                .bind("feature/rebuild-src-b")
                .fetch_optional(&mut conn)
                .await
        })
        .unwrap();

        assert_eq!(status_a.as_deref(), Some("archived"), "archived status must be set");
        assert_eq!(status_b.as_deref(), Some("active"), "workspace-b must be active");
    }
}
