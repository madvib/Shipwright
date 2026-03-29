//! Actor projection tests — prove actor state is derived from events.

#[cfg(test)]
mod actor_tests {
    use crate::db::block_on;
    use crate::db::workspace_db::open_workspace_db;
    use crate::events::types::event_types;
    use crate::events::EventEnvelope;
    use crate::projections::{ActorProjection, EventBus};

    fn setup() -> (std::path::PathBuf, sqlx::SqliteConnection) {
        let ship_dir = crate::project::get_global_dir().unwrap();
        let base = ship_dir.parent().unwrap().to_path_buf();
        crate::project::init_project(base).unwrap();
        crate::db::ensure_db().unwrap();
        let conn = open_workspace_db(&ship_dir, "ws-proj-actor").unwrap();
        (ship_dir, conn)
    }

    fn bus() -> EventBus {
        let mut bus = EventBus::new();
        bus.register(Box::new(ActorProjection::new()));
        bus
    }

    fn actor_created_event(actor_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::ACTOR_CREATED,
            actor_id,
            &serde_json::json!({
                "kind": "test-worker",
                "environment_type": "local",
                "workspace_id": ws_id,
                "parent_actor_id": null,
                "restart_count": 0
            }),
        )
        .unwrap()
        .with_context(Some(ws_id), None)
        .with_actor_id(actor_id)
        .elevate()
    }

    fn actor_woke_event(actor_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(event_types::ACTOR_WOKE, actor_id, &serde_json::json!({}))
            .unwrap()
            .with_context(Some(ws_id), None)
            .with_actor_id(actor_id)
            .elevate()
    }

    fn actor_slept_event(actor_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::ACTOR_SLEPT,
            actor_id,
            &serde_json::json!({"idle_secs": 30}),
        )
        .unwrap()
        .with_context(Some(ws_id), None)
        .with_actor_id(actor_id)
        .elevate()
    }

    fn actor_stopped_event(actor_id: &str, ws_id: &str) -> EventEnvelope {
        EventEnvelope::new(
            event_types::ACTOR_STOPPED,
            actor_id,
            &serde_json::json!({"reason": "done"}),
        )
        .unwrap()
        .with_context(Some(ws_id), None)
        .with_actor_id(actor_id)
        .elevate()
    }

    fn actor_crashed_event(actor_id: &str, ws_id: &str, restart_count: u32) -> EventEnvelope {
        EventEnvelope::new(
            event_types::ACTOR_CRASHED,
            actor_id,
            &serde_json::json!({"error": "OOM", "restart_count": restart_count}),
        )
        .unwrap()
        .with_context(Some(ws_id), None)
        .with_actor_id(actor_id)
        .elevate()
    }

    fn query_actor_status(conn: &mut sqlx::SqliteConnection, actor_id: &str) -> Option<String> {
        let rows: Vec<(String,)> = block_on(async {
            sqlx::query_as("SELECT status FROM actors WHERE id = ?")
                .bind(actor_id)
                .fetch_all(&mut *conn)
                .await
        })
        .unwrap();
        rows.first().map(|r| r.0.clone())
    }

    fn query_actor_restart_count(conn: &mut sqlx::SqliteConnection, actor_id: &str) -> Option<i64> {
        let rows: Vec<(i64,)> = block_on(async {
            sqlx::query_as("SELECT restart_count FROM actors WHERE id = ?")
                .bind(actor_id)
                .fetch_all(&mut *conn)
                .await
        })
        .unwrap();
        rows.first().map(|r| r.0)
    }

    // ── Tests ──────────────────────────────────────────────────────────

    #[test]
    fn dispatch_created_inserts_actor_row() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();
        let event = actor_created_event("actor-proj-1", "ws-proj-actor");

        bus.dispatch(&event, &mut conn);

        let status = query_actor_status(&mut conn, "actor-proj-1");
        assert_eq!(status.as_deref(), Some("created"), "created event must insert row");
    }

    #[test]
    fn dispatch_woke_updates_status() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();

        bus.dispatch(&actor_created_event("actor-proj-2", "ws-proj-actor"), &mut conn);
        bus.dispatch(&actor_woke_event("actor-proj-2", "ws-proj-actor"), &mut conn);

        let status = query_actor_status(&mut conn, "actor-proj-2");
        assert_eq!(status.as_deref(), Some("active"));
    }

    #[test]
    fn dispatch_stopped_updates_status() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();

        bus.dispatch(&actor_created_event("actor-proj-3", "ws-proj-actor"), &mut conn);
        bus.dispatch(&actor_stopped_event("actor-proj-3", "ws-proj-actor"), &mut conn);

        let status = query_actor_status(&mut conn, "actor-proj-3");
        assert_eq!(status.as_deref(), Some("stopped"));
    }

    #[test]
    fn dispatch_crashed_updates_status_and_restart_count() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();

        bus.dispatch(&actor_created_event("actor-proj-4", "ws-proj-actor"), &mut conn);
        bus.dispatch(&actor_crashed_event("actor-proj-4", "ws-proj-actor", 3), &mut conn);

        let status = query_actor_status(&mut conn, "actor-proj-4");
        assert_eq!(status.as_deref(), Some("crashed"));
        let count = query_actor_restart_count(&mut conn, "actor-proj-4");
        assert_eq!(count, Some(3));
    }

    #[test]
    fn rebuild_from_actor_events_produces_identical_state() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();
        let ws = "ws-proj-actor";

        let events = vec![
            actor_created_event("actor-rebuild-a", ws),
            actor_woke_event("actor-rebuild-a", ws),
            actor_created_event("actor-rebuild-b", ws),
            actor_slept_event("actor-rebuild-b", ws),
            actor_crashed_event("actor-rebuild-a", ws, 2),
        ];

        for event in &events {
            bus.dispatch(event, &mut conn);
        }

        let status_a = query_actor_status(&mut conn, "actor-rebuild-a");
        let status_b = query_actor_status(&mut conn, "actor-rebuild-b");
        let count_a = query_actor_restart_count(&mut conn, "actor-rebuild-a");

        let report = bus.rebuild(&events, &mut conn).unwrap();
        assert_eq!(report.events_replayed, 5);

        let rebuilt_a = query_actor_status(&mut conn, "actor-rebuild-a");
        let rebuilt_b = query_actor_status(&mut conn, "actor-rebuild-b");
        let rebuilt_count = query_actor_restart_count(&mut conn, "actor-rebuild-a");
        assert_eq!(status_a, rebuilt_a, "rebuild must produce identical status for actor A");
        assert_eq!(status_b, rebuilt_b, "rebuild must produce identical status for actor B");
        assert_eq!(count_a, rebuilt_count, "rebuild must produce identical restart_count");
    }

    #[test]
    fn dispatch_slept_updates_status() {
        let (_ship_dir, mut conn) = setup();
        let bus = bus();

        bus.dispatch(&actor_created_event("actor-proj-5", "ws-proj-actor"), &mut conn);
        bus.dispatch(&actor_slept_event("actor-proj-5", "ws-proj-actor"), &mut conn);

        let status = query_actor_status(&mut conn, "actor-proj-5");
        assert_eq!(status.as_deref(), Some("sleeping"));
    }
}
