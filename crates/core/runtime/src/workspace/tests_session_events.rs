#[cfg(test)]
mod tests {
    use crate::db::{block_on, ensure_db, open_db_at};
    use crate::events::filter::EventFilter;
    use crate::events::store::{EventStore, SqliteEventStore};
    use crate::events::types::event_types;
    use crate::workspace::*;
    use anyhow::Result;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = crate::project::init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        (tmp, ship_dir)
    }

    fn event_store() -> SqliteEventStore {
        SqliteEventStore::new().unwrap()
    }

    // ── test 1: start session → session.started event with correct goal ───────

    #[test]
    fn start_session_emits_started_event_with_goal() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/se-started".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/se-started",
            Some("implement events".to_string()),
            None,
            None,
        )?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            event_type: Some(event_types::SESSION_STARTED.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected session.started event");
        let ev = &events[0];
        let payload: crate::events::types::SessionStarted =
            serde_json::from_str(&ev.payload_json)?;
        assert_eq!(
            payload.goal.as_deref(),
            Some("implement events"),
            "goal must match"
        );
        Ok(())
    }

    // ── test 2: log progress → session.progress event, elevated=0 ─────────────

    #[test]
    fn log_progress_emits_progress_event_not_elevated() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/se-progress".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/se-progress",
            None,
            None,
            None,
        )?;

        record_workspace_session_progress(
            &ship_dir,
            "feature/se-progress",
            "halfway done",
        )?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            event_type: Some(event_types::SESSION_PROGRESS.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected session.progress event");
        let ev = &events[0];
        let payload: crate::events::types::SessionProgress =
            serde_json::from_str(&ev.payload_json)?;
        assert_eq!(payload.message, "halfway done", "message must match");
        assert!(!ev.elevated, "session.progress must not be elevated");
        Ok(())
    }

    // ── test 3: end session → session.ended event with summary + duration ──────

    #[test]
    fn end_session_emits_ended_event_with_summary() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/se-ended".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/se-ended",
            None,
            None,
            None,
        )?;

        end_workspace_session(
            &ship_dir,
            "feature/se-ended",
            EndWorkspaceSessionRequest {
                summary: Some("all done".to_string()),
                gate_result: Some("pass".to_string()),
                ..Default::default()
            },
        )?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            event_type: Some(event_types::SESSION_ENDED.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected session.ended event");
        let ev = &events[0];
        let payload: crate::events::types::SessionEnded =
            serde_json::from_str(&ev.payload_json)?;
        assert_eq!(payload.summary.as_deref(), Some("all done"), "summary must match");
        assert!(payload.duration_secs.is_some(), "duration_secs must be set");
        assert_eq!(payload.gate_result.as_deref(), Some("pass"), "gate_result must match");
        Ok(())
    }

    // ── test 4: all events carry correct entity_id, actor_id, session_id, workspace_id, parent_actor_id ──

    #[test]
    fn session_events_have_correct_metadata() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch = "feature/se-metadata";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let workspace = get_workspace(&ship_dir, branch)?
            .ok_or_else(|| anyhow::anyhow!("workspace not found"))?;

        let session = start_workspace_session(&ship_dir, branch, Some("meta test".to_string()), None, None)?;

        record_workspace_session_progress(&ship_dir, branch, "checkpoint")?;

        end_workspace_session(
            &ship_dir,
            branch,
            EndWorkspaceSessionRequest {
                summary: Some("done".to_string()),
                ..Default::default()
            },
        )?;

        let store = event_store();
        let all = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            ..Default::default()
        })?;

        let typed: Vec<_> = all
            .iter()
            .filter(|e| {
                matches!(
                    e.event_type.as_str(),
                    event_types::SESSION_STARTED
                        | event_types::SESSION_PROGRESS
                        | event_types::SESSION_ENDED
                )
            })
            .collect();

        assert_eq!(typed.len(), 3, "expected exactly 3 session lifecycle events");

        for ev in &typed {
            assert_eq!(ev.entity_id, session.id, "entity_id must equal session id");
            assert_eq!(
                ev.actor_id.as_deref(),
                Some(session.id.as_str()),
                "actor_id must equal session id, got {:?}",
                ev.actor_id
            );
            assert_eq!(
                ev.session_id.as_deref(),
                Some(session.id.as_str()),
                "session_id must equal session id, got {:?}",
                ev.session_id
            );
            assert_eq!(
                ev.workspace_id.as_deref(),
                Some(workspace.id.as_str()),
                "workspace_id must equal workspace id, got {:?}",
                ev.workspace_id
            );
            assert_eq!(
                ev.parent_actor_id.as_deref(),
                Some(workspace.id.as_str()),
                "parent_actor_id must equal workspace id, got {:?}",
                ev.parent_actor_id
            );
        }
        Ok(())
    }

    // ── test 5: started/ended elevated=1, progress elevated=0 ─────────────────

    #[test]
    fn session_started_and_ended_are_elevated_progress_is_not() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/se-elevated".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;

        let session = start_workspace_session(
            &ship_dir,
            "feature/se-elevated",
            None,
            None,
            None,
        )?;

        record_workspace_session_progress(&ship_dir, "feature/se-elevated", "midpoint")?;

        end_workspace_session(
            &ship_dir,
            "feature/se-elevated",
            EndWorkspaceSessionRequest::default(),
        )?;

        let store = event_store();
        let all = store.query(&EventFilter {
            entity_id: Some(session.id.clone()),
            ..Default::default()
        })?;

        let get = |et: &str| {
            all.iter()
                .find(|e| e.event_type == et)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("event '{}' not found", et))
        };

        let started = get(event_types::SESSION_STARTED)?;
        let progress = get(event_types::SESSION_PROGRESS)?;
        let ended = get(event_types::SESSION_ENDED)?;

        assert!(started.elevated, "session.started must be elevated");
        assert!(!progress.elevated, "session.progress must NOT be elevated");
        assert!(ended.elevated, "session.ended must be elevated");

        Ok(())
    }

    // ── test 6: atomicity — if event insert fails, session row rolls back ──────

    #[test]
    fn session_event_insert_failure_rolls_back_session_write() -> Result<()> {
        use crate::db::session_events_testutil::insert_session_started_event;
        use crate::events::types::SessionStarted;

        let (_tmp, _ship_dir) = setup();
        let db = crate::db::db_path()?;

        let mut conn = open_db_at(&db)?;
        block_on(async {
            sqlx::query(
                "CREATE TRIGGER IF NOT EXISTS session_events_test_block \
                 BEFORE INSERT ON events \
                 WHEN NEW.actor_id = 'sess-atomicity-sentinel' \
                 BEGIN \
                   SELECT RAISE(FAIL, 'atomicity test: forced session event insert failure'); \
                 END",
            )
            .execute(&mut conn)
            .await
        })?;
        drop(conn);

        let sentinel_session_id = "sess-atomicity-sentinel";
        let sentinel_workspace_id = "ws-atomicity-sentinel";
        let payload = SessionStarted { goal: None, ..Default::default() };

        let result = insert_session_started_event(
            sentinel_session_id,
            sentinel_workspace_id,
            &payload,
        );

        assert!(
            result.is_err(),
            "transaction must fail when event INSERT is blocked by trigger"
        );

        // The session row must NOT have been committed — verify via direct query.
        let mut conn2 = open_db_at(&db)?;
        let count: i64 = block_on(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE actor_id = ?",
            )
            .bind(sentinel_session_id)
            .fetch_one(&mut conn2)
            .await
        })?;
        assert_eq!(
            count, 0,
            "no events must be present when transaction rolls back; got {count}"
        );

        Ok(())
    }
}
