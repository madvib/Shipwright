#[cfg(test)]
mod tests {
    use crate::db::{ensure_db, open_db_at};
    use crate::db::block_on;
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

    // ── test 1: activate emits workspace.activated ────────────────────────────

    #[test]
    fn activate_workspace_emits_activated_event() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/evt-activate".to_string(),
                ..Default::default()
            },
        )?;
        activate_workspace(&ship_dir, "feature/evt-activate")?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some("feature/evt-activate".to_string()),
            event_type: Some(event_types::WORKSPACE_ACTIVATED.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected at least one workspace.activated event");
        let ev = &events[0];
        assert_eq!(ev.event_type, event_types::WORKSPACE_ACTIVATED);
        Ok(())
    }

    // ── test 2: compile emits workspace.compiled with correct payload ─────────

    #[test]
    fn activate_workspace_emits_compiled_event_with_payload() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/evt-compiled".to_string(),
                ..Default::default()
            },
        )?;
        let ws = activate_workspace(&ship_dir, "feature/evt-compiled")?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some("feature/evt-compiled".to_string()),
            event_type: Some(event_types::WORKSPACE_COMPILED.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected workspace.compiled event after activate");
        let ev = &events[0];
        let payload: crate::events::types::WorkspaceCompiled =
            serde_json::from_str(&ev.payload_json)?;
        let _ = ws; // workspace is lean now; config_generation is in the event
        assert!(payload.duration_ms < 60_000, "duration_ms should be reasonable");
        Ok(())
    }

    // ── test 3: compile failure emits workspace.compile_failed ────────────────

    #[test]
    #[ignore = "provider resolution always falls back to claude; need a different compile failure trigger"]
    fn compile_failure_emits_compile_failed_event() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        // A workspace that references a non-existent agent will fail to compile.
        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/evt-fail".to_string(),
                ..Default::default()
            },
        )?;
        // Point at a bogus agent so compile fails during agent resolution.
        let mut config = crate::config::get_config(Some(ship_dir.clone()))?;
        config.active_agent = Some("nonexistent-agent-id".to_string());
        crate::config::save_config(&config, Some(ship_dir.clone()))?;
        // activate_workspace will call compile_workspace_context which errors.
        let result = activate_workspace(&ship_dir, "feature/evt-fail");
        assert!(result.is_err(), "expected compile failure");

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some("feature/evt-fail".to_string()),
            event_type: Some(event_types::WORKSPACE_COMPILE_FAILED.to_string()),
            ..Default::default()
        })?;

        assert!(
            !events.is_empty(),
            "expected workspace.compile_failed event after compile error"
        );
        let ev = &events[0];
        let payload: crate::events::types::WorkspaceCompileFailed =
            serde_json::from_str(&ev.payload_json)?;
        assert!(!payload.error.is_empty(), "error string must be non-empty");
        Ok(())
    }

    // ── test 4: archive emits workspace.archived ──────────────────────────────

    #[test]
    fn archive_workspace_emits_archived_event() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: "feature/evt-archive".to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;
        transition_workspace_status(
            &ship_dir,
            "feature/evt-archive",
            WorkspaceStatus::Archived,
        )?;

        let store = event_store();
        let events = store.query(&EventFilter {
            entity_id: Some("feature/evt-archive".to_string()),
            event_type: Some(event_types::WORKSPACE_ARCHIVED.to_string()),
            ..Default::default()
        })?;

        assert!(!events.is_empty(), "expected workspace.archived event");
        Ok(())
    }

    // ── test 5: all events carry correct entity_id, actor_id, workspace_id, elevated ──

    #[test]
    fn workspace_events_have_correct_metadata() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let branch = "feature/evt-metadata";

        create_workspace(
            &ship_dir,
            CreateWorkspaceRequest {
                branch: branch.to_string(),
                status: Some(WorkspaceStatus::Active),
                ..Default::default()
            },
        )?;
        activate_workspace(&ship_dir, branch)?;
        transition_workspace_status(&ship_dir, branch, WorkspaceStatus::Archived)?;

        let store = event_store();
        let all_ws_events = store.query(&EventFilter {
            entity_id: Some(branch.to_string()),
            ..Default::default()
        })?;

        // Filter to just the typed lifecycle events
        let typed: Vec<_> = all_ws_events
            .iter()
            .filter(|e| {
                matches!(
                    e.event_type.as_str(),
                    event_types::WORKSPACE_ACTIVATED
                        | event_types::WORKSPACE_COMPILED
                        | event_types::WORKSPACE_ARCHIVED
                )
            })
            .collect();

        assert!(
            !typed.is_empty(),
            "expected typed workspace events for branch '{}'",
            branch
        );

        for ev in &typed {
            assert_eq!(
                ev.entity_id, branch,
                "entity_id must equal workspace branch"
            );
            assert_eq!(
                ev.actor_id.as_deref(),
                Some(branch),
                "actor_id must equal workspace branch, got {:?}",
                ev.actor_id
            );
            assert_eq!(
                ev.workspace_id.as_deref(),
                Some(branch),
                "workspace_id must equal workspace branch, got {:?}",
                ev.workspace_id
            );
            assert!(ev.elevated, "workspace lifecycle events must have elevated=true");
        }
        Ok(())
    }

    // ── test 6: atomicity — event insert failure prevents projection write ───
    //
    // Add a BEFORE INSERT trigger that raises for any event with actor_id equal
    // to "atomicity-sentinel". Pass that string as the branch name so run_tx
    // sets actor_id to the same value, forcing a failure. Since the event is
    // never committed, the projection never runs and no workspace row appears.

    #[test]
    fn event_insert_failure_prevents_workspace_write() -> Result<()> {
        use crate::db::workspace_events::emit_workspace_activated;
        use crate::events::types::WorkspaceActivated;

        let (_tmp, _ship_dir) = setup();
        let db = crate::db::db_path()?;

        let mut conn = open_db_at(&db)?;
        block_on(async {
            sqlx::query(
                "CREATE TRIGGER IF NOT EXISTS events_test_block \
                 BEFORE INSERT ON events \
                 WHEN NEW.actor_id = 'atomicity-sentinel' \
                 BEGIN \
                   SELECT RAISE(FAIL, 'atomicity test: forced event insert failure'); \
                 END",
            )
            .execute(&mut conn)
            .await
        })?;
        drop(conn);

        let sentinel_branch = "atomicity-sentinel";
        let payload = WorkspaceActivated {
            agent_id: None,
            providers: vec![],
        };
        let result = emit_workspace_activated(sentinel_branch, &payload);

        assert!(
            result.is_err(),
            "transaction must fail when event INSERT is blocked by trigger"
        );

        // No workspace row should exist — the event was never committed,
        // so the projection never ran.
        let mut conn2 = open_db_at(&db)?;
        let count: i64 = block_on(async {
            sqlx::query_scalar("SELECT COUNT(*) FROM workspace WHERE branch = ?")
                .bind(sentinel_branch)
                .fetch_one(&mut conn2)
                .await
        })?;
        assert_eq!(
            count, 0,
            "workspace row must not exist when event INSERT fails; got {count} rows"
        );

        Ok(())
    }
}
