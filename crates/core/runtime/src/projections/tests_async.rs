//! Async projection integration tests.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use anyhow::Result;
    use sqlx::SqliteConnection;

    use crate::db::{block_on, db_path, ensure_db, open_db_at};
    use crate::events::EventEnvelope;
    use crate::events::filter::EventFilter;
    use crate::events::router::EventRouter;
    use crate::events::store::{EventStore, SqliteEventStore};
    use crate::events::types::event_types;
    use crate::events::validator::{CallerKind, EmitContext};
    use crate::projections::async_projection::spawn_projection;
    use crate::projections::spawn::spawn_with_failure_counter;
    use crate::projections::{AsyncProjection, EventBus, WorkspaceProjection};

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

    fn runtime_ctx(branch: &str) -> EmitContext {
        EmitContext {
            caller_kind: CallerKind::Runtime,
            skill_id: None,
            workspace_id: Some(branch.to_string()),
            session_id: None,
        }
    }

    // ── Test 1: emit event → async projection applies it (not inline) ─────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn async_projection_applies_broadcast_event() {
        let _tmp = setup();
        let db = db_path().unwrap();

        let store: Arc<dyn EventStore> = Arc::new(SqliteEventStore::new().unwrap());
        let router = Arc::new(EventRouter::new(store, 64));

        let db_clone = db.clone();
        let _handle = spawn_projection(
            Arc::new(WorkspaceProjection::new()),
            router.subscribe_platform(),
            move || open_db_at(&db_clone),
        );

        let event = workspace_created("feature/async-apply-1");
        router.emit(event, &runtime_ctx("feature/async-apply-1")).await.unwrap();

        // Poll up to 2 seconds for the async projection to apply.
        let mut count: i64 = 0;
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let mut conn = open_db_at(&db).unwrap();
            count = block_on(async {
                sqlx::query_scalar("SELECT COUNT(*) FROM workspace WHERE branch = ?")
                    .bind("feature/async-apply-1")
                    .fetch_one(&mut conn)
                    .await
            })
            .unwrap();
            if count == 1 {
                break;
            }
        }

        assert_eq!(count, 1, "async projection must apply workspace.created event");
    }

    // ── Test 2: projection failure does not prevent event persistence ─────────

    struct FailingProjection;

    impl AsyncProjection for FailingProjection {
        fn name(&self) -> &str {
            "always-fail"
        }
        fn event_types(&self) -> &[&str] {
            &[event_types::WORKSPACE_CREATED]
        }
        fn apply(&self, _: &EventEnvelope, _: &mut SqliteConnection) -> Result<()> {
            Err(anyhow::anyhow!("forced failure"))
        }
        fn truncate(&self, _: &mut SqliteConnection) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn projection_failure_does_not_prevent_event_persistence() {
        let _tmp = setup();
        let db = db_path().unwrap();

        let store: Arc<dyn EventStore> = Arc::new(SqliteEventStore::new().unwrap());
        let router = Arc::new(EventRouter::new(store.clone(), 64));

        let db_clone = db.clone();
        let ph = spawn_with_failure_counter(
            Arc::new(FailingProjection),
            router.subscribe_platform(),
            move || open_db_at(&db_clone),
        );

        let event = workspace_created("feature/proj-fail-1");
        router.emit(event.clone(), &runtime_ctx("feature/proj-fail-1")).await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Event must be in the store (persisted before broadcast)
        let persisted = store
            .query(&EventFilter {
                entity_id: Some("feature/proj-fail-1".to_string()),
                event_type: Some(event_types::WORKSPACE_CREATED.to_string()),
                ..Default::default()
            })
            .unwrap();
        assert!(!persisted.is_empty(), "event must be persisted even when projection fails");

        // Failure counter must have incremented
        let failures = ph.failure_count.load(Ordering::Relaxed);
        assert!(failures > 0, "failure counter must increment on projection error; got {failures}");
    }

    // ── Test 3: projection rebuild from events table produces correct state ───

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
