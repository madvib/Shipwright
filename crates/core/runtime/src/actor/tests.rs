#[cfg(test)]
mod tests {
    use crate::actor::{
        create_actor, crash_actor, sleep_actor, stop_actor, wake_actor,
    };
    use crate::db::{block_on, ensure_db, open_db_at};
    use crate::events::filter::EventFilter;
    use crate::events::store::{EventStore, SqliteEventStore};
    use crate::events::types::event_types;
    use anyhow::Result;
    use tempfile::tempdir;

    fn setup() -> tempfile::TempDir {
        let tmp = tempdir().unwrap();
        crate::project::init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        tmp
    }

    fn event_store() -> SqliteEventStore {
        SqliteEventStore::new().unwrap()
    }

    // ── test 1: create_actor emits actor.created ──────────────────────────────

    #[test]
    fn create_actor_emits_created_event() -> Result<()> {
        let _tmp = setup();
        let id = "actor-create-t1";

        create_actor(id, "test-worker", "local", None, None)?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(id.to_string()),
            event_type: Some(event_types::ACTOR_CREATED.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected actor.created event");
        let payload: crate::events::types::ActorCreated =
            serde_json::from_str(&events[0].payload_json)?;
        assert_eq!(payload.kind, "test-worker");
        Ok(())
    }

    // ── test 2: wake_actor emits actor.woke ───────────────────────────────────

    #[test]
    fn wake_actor_emits_woke_event() -> Result<()> {
        let _tmp = setup();
        let id = "actor-wake-t2";

        create_actor(id, "test-worker", "local", None, None)?;
        wake_actor(id, None, None)?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(id.to_string()),
            event_type: Some(event_types::ACTOR_WOKE.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected actor.woke event");
        Ok(())
    }

    // ── test 3: sleep_actor emits actor.slept with correct idle_secs ─────────

    #[test]
    fn sleep_actor_emits_slept_event() -> Result<()> {
        let _tmp = setup();
        let id = "actor-sleep-t3";

        create_actor(id, "test-worker", "local", None, None)?;
        sleep_actor(id, 30, None, None)?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(id.to_string()),
            event_type: Some(event_types::ACTOR_SLEPT.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected actor.slept event");
        let payload: crate::events::types::ActorSlept =
            serde_json::from_str(&events[0].payload_json)?;
        assert_eq!(payload.idle_secs, 30);
        Ok(())
    }

    // ── test 4: stop_actor emits actor.stopped with reason ───────────────────

    #[test]
    fn stop_actor_emits_stopped_event() -> Result<()> {
        let _tmp = setup();
        let id = "actor-stop-t4";

        create_actor(id, "test-worker", "local", None, None)?;
        stop_actor(id, "graceful shutdown", None, None)?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(id.to_string()),
            event_type: Some(event_types::ACTOR_STOPPED.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected actor.stopped event");
        let payload: crate::events::types::ActorStopped =
            serde_json::from_str(&events[0].payload_json)?;
        assert_eq!(payload.reason, "graceful shutdown");
        Ok(())
    }

    // ── test 5: crash_actor emits actor.crashed + persists restart_count ─────

    #[test]
    fn crash_actor_emits_crashed_event() -> Result<()> {
        let _tmp = setup();
        let id = "actor-crash-t5";
        let db = crate::db::db_path()?;

        create_actor(id, "test-worker", "local", None, None)?;
        crash_actor(id, "OOM", 1, None, None)?;

        // Event check
        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(id.to_string()),
            event_type: Some(event_types::ACTOR_CRASHED.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected actor.crashed event");
        let payload: crate::events::types::ActorCrashed =
            serde_json::from_str(&events[0].payload_json)?;
        assert_eq!(payload.error, "OOM");
        assert_eq!(payload.restart_count, 1);

        // DB check: restart_count must equal what we passed (set by projection)
        let mut conn = open_db_at(&db)?;
        let count: i64 = block_on(async {
            sqlx::query_scalar("SELECT restart_count FROM actors WHERE id = ?")
                .bind(id)
                .fetch_one(&mut conn)
                .await
        })?;
        assert_eq!(count, 1, "restart_count must be persisted in actors table");
        Ok(())
    }

    // ── test 6: all actor events carry correct metadata ───────────────────────

    #[test]
    fn all_actor_events_have_correct_metadata() -> Result<()> {
        let _tmp = setup();
        let id = "actor-meta-t6";

        create_actor(id, "meta-worker", "local", Some("ws-meta"), Some("parent-meta"))?;
        wake_actor(id, Some("ws-meta"), Some("parent-meta"))?;
        sleep_actor(id, 10, Some("ws-meta"), Some("parent-meta"))?;
        stop_actor(id, "done", Some("ws-meta"), Some("parent-meta"))?;

        // Actor events go to workspace DB, query there
        let ship_dir = crate::project::get_global_dir()?;
        let mut ws_conn = crate::db::workspace_db::open_workspace_db(&ship_dir, "ws-meta")?;
        let all_events: Vec<(String, String, Option<String>, bool)> = block_on(async {
            sqlx::query_as(
                "SELECT event_type, entity_id, actor_id, elevated FROM events \
                 WHERE entity_id = ? ORDER BY created_at",
            )
            .bind(id)
            .fetch_all(&mut ws_conn)
            .await
        })?;

        assert!(
            !all_events.is_empty(),
            "expected actor lifecycle events for id '{id}'"
        );

        for (event_type, entity_id, actor_id, elevated) in &all_events {
            assert_eq!(entity_id, id, "entity_id must equal actor id");
            assert_eq!(
                actor_id.as_deref(),
                Some(id),
                "actor_id must equal actor id for event {event_type}"
            );
            assert!(elevated, "actor lifecycle events must have elevated=true");
        }
        Ok(())
    }

    // ── test 7: event insert failure rolls back ──────────────────────────────

    #[test]
    fn actor_event_failure_rolls_back() -> Result<()> {
        let _tmp = setup();
        let db = crate::db::db_path()?;

        // Install BEFORE INSERT trigger that blocks events for sentinel actor_id.
        let mut conn = open_db_at(&db)?;
        block_on(async {
            sqlx::query(
                "CREATE TRIGGER IF NOT EXISTS actors_test_block \
                 BEFORE INSERT ON events \
                 WHEN NEW.actor_id = 'atomicity-actor-sentinel' \
                 BEGIN \
                   SELECT RAISE(FAIL, 'atomicity test: forced actor event insert failure'); \
                 END",
            )
            .execute(&mut conn)
            .await
        })?;
        drop(conn);

        let sentinel_id = "atomicity-actor-sentinel";
        let result = create_actor(sentinel_id, "sentinel", "local", None, None);

        assert!(
            result.is_err(),
            "transaction must fail when event INSERT is blocked by trigger"
        );

        // actors row must NOT have been committed (projection never ran).
        let mut conn2 = open_db_at(&db)?;
        let count: i64 = block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM actors WHERE id = ?")
                .bind(sentinel_id)
                .fetch_one(&mut conn2)
                .await
        })?;
        assert_eq!(
            count, 0,
            "actors row must not exist when event INSERT fails; got {count} rows"
        );

        Ok(())
    }
}
