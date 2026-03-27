#[cfg(test)]
mod tests_supervision {
    use crate::actor::supervisor::{SupervisionAction, SupervisorPolicy, evaluate};
    use crate::actor::{create_actor, crash_actor, run_supervision};
    use crate::db::actor_events::ActorUpsert;
    use crate::db::{block_on, db_path, ensure_db, open_db_at};
    use crate::events::envelope::EventEnvelope;
    use crate::events::types::{ActorCrashed, ActorWoke, event_types};
    use anyhow::Result;
    use tempfile::tempdir;

    fn setup() -> tempfile::TempDir {
        let tmp = tempdir().unwrap();
        crate::project::init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        tmp
    }

    fn default_upsert<'a>(id: &'a str) -> ActorUpsert<'a> {
        ActorUpsert {
            id,
            kind: "test-worker",
            environment_type: "local",
            workspace_id: None,
            parent_actor_id: None,
            restart_count: 0,
        }
    }

    fn actor_status(id: &str) -> String {
        let db = db_path().unwrap();
        let mut conn = open_db_at(&db).unwrap();
        block_on(async {
            sqlx::query_scalar::<_, String>("SELECT status FROM actors WHERE id = ?")
                .bind(id)
                .fetch_one(&mut conn)
                .await
        })
        .unwrap()
    }

    fn make_crash_envelope(actor_id: &str, parent_id: &str, restart_count: u32) -> EventEnvelope {
        let payload = ActorCrashed {
            error: "test error".to_string(),
            restart_count,
        };
        EventEnvelope::new(event_types::ACTOR_CRASHED, actor_id, &payload)
            .unwrap()
            .with_actor_id(actor_id)
            .with_parent_actor_id(parent_id)
            .elevate()
    }

    // ── test 1: evaluate restarts a crashed actor within restart limit ─────────

    #[test]
    fn evaluate_restarts_crashed_actor_within_limit() -> Result<()> {
        let policy = SupervisorPolicy { max_restarts: 3 };
        let ev = make_crash_envelope("child-actor-1", "supervisor-1", 1);

        let actions = evaluate(&[ev], &policy);

        assert_eq!(actions.len(), 1);
        assert!(
            matches!(&actions[0], SupervisionAction::Restart { actor_id } if actor_id == "child-actor-1"),
            "expected Restart for restart_count=1 < max_restarts=3, got {:?}",
            actions[0]
        );
        Ok(())
    }

    // ── test 2: evaluate stops actor at max_restarts ───────────────────────────

    #[test]
    fn evaluate_stops_actor_at_max_restarts() -> Result<()> {
        let policy = SupervisorPolicy { max_restarts: 3 };
        let ev = make_crash_envelope("child-actor-2", "supervisor-1", 3);

        let actions = evaluate(&[ev], &policy);

        assert_eq!(actions.len(), 1);
        match &actions[0] {
            SupervisionAction::Stop { actor_id, reason } => {
                assert_eq!(actor_id, "child-actor-2");
                assert!(
                    reason.contains("max_restarts"),
                    "reason must mention max_restarts, got: {reason}"
                );
            }
            other => panic!("expected Stop, got {:?}", other),
        }
        Ok(())
    }

    // ── test 3: evaluate ignores non-crash events ─────────────────────────────

    #[test]
    fn evaluate_ignores_non_crash_events() -> Result<()> {
        let policy = SupervisorPolicy::default();

        let woke = EventEnvelope::new(event_types::ACTOR_WOKE, "child-1", &ActorWoke {})?
            .with_actor_id("child-1")
            .with_parent_actor_id("supervisor-1")
            .elevate();

        let slept = EventEnvelope::new(
            event_types::ACTOR_SLEPT,
            "child-1",
            &crate::events::types::ActorSlept { idle_secs: 10 },
        )?
        .with_actor_id("child-1")
        .with_parent_actor_id("supervisor-1")
        .elevate();

        let actions = evaluate(&[woke, slept], &policy);
        assert!(
            actions.is_empty(),
            "expected no actions for non-crash events, got {} actions",
            actions.len()
        );
        Ok(())
    }

    // ── test 4: run_supervision restarts a crashed actor ─────────────────────

    #[test]
    fn run_supervision_restarts_crashed_actor() -> Result<()> {
        let _tmp = setup();
        let supervisor_id = "supervisor-restart-t4";
        let child_id = "child-restart-t4";

        create_actor(default_upsert(supervisor_id))?;
        create_actor(ActorUpsert {
            id: child_id,
            kind: "test-worker",
            environment_type: "local",
            workspace_id: None,
            parent_actor_id: Some(supervisor_id),
            restart_count: 0,
        })?;

        // crash with restart_count=0 — below max_restarts=3, should trigger Restart
        crash_actor(child_id, "oom", 0, None, Some(supervisor_id))?;

        let policy = SupervisorPolicy { max_restarts: 3 };
        let actions = run_supervision(supervisor_id, None, None, &policy)?;

        let has_restart = actions.iter().any(|a| {
            matches!(a, SupervisionAction::Restart { actor_id } if actor_id == child_id)
        });
        assert!(has_restart, "expected Restart action for child actor, got {:?}", actions);

        // wake_actor sets status to 'active'
        assert_eq!(
            actor_status(child_id),
            "active",
            "child actor status must be 'active' after restart"
        );
        Ok(())
    }

    // ── test 5: run_supervision stops actor past max_restarts ─────────────────

    #[test]
    fn run_supervision_stops_actor_past_max_restarts() -> Result<()> {
        let _tmp = setup();
        let supervisor_id = "supervisor-stop-t5";
        let child_id = "child-stop-t5";

        create_actor(default_upsert(supervisor_id))?;
        create_actor(ActorUpsert {
            id: child_id,
            kind: "test-worker",
            environment_type: "local",
            workspace_id: None,
            parent_actor_id: Some(supervisor_id),
            restart_count: 0,
        })?;

        // crash with restart_count=3 — equal to max_restarts=3, should trigger Stop
        crash_actor(child_id, "fatal", 3, None, Some(supervisor_id))?;

        let policy = SupervisorPolicy { max_restarts: 3 };
        let actions = run_supervision(supervisor_id, None, None, &policy)?;

        let has_stop = actions.iter().any(|a| {
            matches!(a, SupervisionAction::Stop { actor_id, .. } if actor_id == child_id)
        });
        assert!(has_stop, "expected Stop action for child actor, got {:?}", actions);

        // stop_actor sets status to 'stopped'
        assert_eq!(
            actor_status(child_id),
            "stopped",
            "child actor status must be 'stopped' after max_restarts exceeded"
        );
        Ok(())
    }
}
