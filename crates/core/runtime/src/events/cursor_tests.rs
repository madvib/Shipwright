//! Tests for ActorCursor and guaranteed delivery (cursor-based replay).
//!
//! Acceptance criteria covered:
//! - On actor startup with existing cursor: events since cursor are replayed
//! - On actor startup with no cursor (first spawn): no replay, cursor starts at HEAD
//! - After actor processes event: cursor advances to that event's ID
//! - After actor crash + restart: receives events emitted while offline
//! - Replay respects subscription scope (only replays matching events)
//! - Replay delivers events in order (ULID sorted)

#[cfg(feature = "unstable")]
mod cursor {
    use crate::events::cursor::ActorCursor;
    use crate::events::envelope::EventEnvelope;
    use crate::events::identity::{ActorIdentity, ActorType};
    use crate::events::kernel_router::KernelRouter;
    use crate::events::permissions::{ActorPermissions, DeliveryScope, PermittedSubscription};
    use crate::events::validator::{CallerKind, EmitContext};
    use tempfile::tempdir;

    fn identity(label: &str) -> ActorIdentity {
        ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: label.into(),
            actor_type: ActorType::Agent,
        }
    }

    fn ev(event_type: &str) -> EventEnvelope {
        EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
    }

    fn runtime_ctx() -> EmitContext {
        EmitContext {
            caller_kind: CallerKind::Runtime,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        }
    }

    fn global_studio_perms() -> ActorPermissions {
        ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Global,
            }],
        }
    }

    fn setup() -> (tempfile::TempDir, KernelRouter) {
        let tmp = tempdir().unwrap();
        let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();
        (tmp, router)
    }

    // ── Cursor struct ────────────────────────────────────────────────────────

    #[test]
    fn actor_cursor_stores_label_and_event_id() {
        let cursor = ActorCursor {
            actor_label: "agent.reviewer".into(),
            last_event_id: Some("01HWXYZ".into()),
        };
        assert_eq!(cursor.actor_label, "agent.reviewer");
        assert_eq!(cursor.last_event_id.as_deref(), Some("01HWXYZ"));
    }

    #[test]
    fn actor_cursor_none_means_first_spawn() {
        let cursor = ActorCursor {
            actor_label: "agent.new".into(),
            last_event_id: None,
        };
        assert!(cursor.last_event_id.is_none());
    }

    // ── Replay on startup ────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn first_spawn_no_replay_cursor_starts_at_head() {
        let (_tmp, mut router) = setup();

        // Emit some events before spawning the actor.
        router.route(ev("studio.annotation"), &runtime_ctx()).await.unwrap();
        router.route(ev("studio.feedback"), &runtime_ctx()).await.unwrap();

        // First spawn — no cursor exists. Actor should NOT receive old events.
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(identity("agent.new"), global_studio_perms())
            .unwrap();

        assert!(
            mb.try_recv().is_err(),
            "first spawn must not replay historical events"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn restart_replays_events_since_cursor() {
        let (_tmp, mut router) = setup();

        // Spawn agent, receive one event, then stop.
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(identity("agent.replayer"), global_studio_perms())
            .unwrap();

        let ev1 = ev("studio.annotation");
        router.route(ev1.clone(), &runtime_ctx()).await.unwrap();

        let received = mb.try_recv().expect("must receive ev1");
        assert_eq!(received.id, ev1.id);

        // Advance cursor (kernel tracks this on successful delivery).
        router.stop_actor("agent.replayer").unwrap();

        // Emit events while actor is offline.
        let ev2 = ev("studio.feedback");
        let ev3 = ev("studio.selection");
        router.route(ev2.clone(), &runtime_ctx()).await.unwrap();
        router.route(ev3.clone(), &runtime_ctx()).await.unwrap();

        // Restart — should replay ev2 and ev3.
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(identity("agent.replayer"), global_studio_perms())
            .unwrap();

        let r2 = mb.try_recv().expect("must replay ev2");
        let r3 = mb.try_recv().expect("must replay ev3");
        assert_eq!(r2.id, ev2.id);
        assert_eq!(r3.id, ev3.id);
        assert!(mb.try_recv().is_err(), "no more events to replay");
    }

    // ── Cursor advancement ───────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cursor_advances_after_event_processed() {
        let (_tmp, mut router) = setup();

        let (_store, mut mb) = router
            .spawn_actor_with_permissions(identity("agent.cursor"), global_studio_perms())
            .unwrap();

        let ev1 = ev("studio.annotation");
        let ev2 = ev("studio.feedback");
        router.route(ev1.clone(), &runtime_ctx()).await.unwrap();
        router.route(ev2.clone(), &runtime_ctx()).await.unwrap();

        // Process both events.
        let _ = mb.try_recv().unwrap();
        let _ = mb.try_recv().unwrap();

        // Stop and restart — cursor should be after ev2.
        router.stop_actor("agent.cursor").unwrap();

        // No new events emitted.
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(identity("agent.cursor"), global_studio_perms())
            .unwrap();

        assert!(
            mb.try_recv().is_err(),
            "cursor should have advanced past all processed events"
        );
    }

    // ── Crash + restart ──────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn crash_restart_receives_events_emitted_while_offline() {
        let (_tmp, mut router) = setup();

        let (_store, _mb) = router
            .spawn_actor_with_permissions(identity("agent.crasher"), global_studio_perms())
            .unwrap();

        // Simulate crash: stop without processing any events.
        router.stop_actor("agent.crasher").unwrap();

        // Events emitted while actor is offline.
        let ev1 = ev("studio.urgent");
        let ev2 = ev("studio.critical");
        router.route(ev1.clone(), &runtime_ctx()).await.unwrap();
        router.route(ev2.clone(), &runtime_ctx()).await.unwrap();

        // Restart — cursor was at start, so both events should replay.
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(identity("agent.crasher"), global_studio_perms())
            .unwrap();

        let r1 = mb.try_recv().expect("must replay ev1 after crash");
        let r2 = mb.try_recv().expect("must replay ev2 after crash");
        assert_eq!(r1.id, ev1.id);
        assert_eq!(r2.id, ev2.id);
    }

    // ── Replay respects scope ────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn replay_respects_workspace_scope() {
        let (_tmp, mut router) = setup();

        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Workspace,
            }],
        };

        // First spawn in workspace "ws-a".
        let (_store, _mb) = router
            .spawn_actor_with_permissions_in_workspace(
                identity("agent.scoped"),
                perms.clone(),
                "ws-a",
            )
            .unwrap();
        router.stop_actor("agent.scoped").unwrap();

        // Events in both workspaces while offline.
        let ev_a = ev("studio.annotation")
            .with_context(Some("ws-a"), None);
        let ev_b = ev("studio.annotation")
            .with_context(Some("ws-b"), None);
        router.route(ev_a.clone(), &runtime_ctx()).await.unwrap();
        router.route(ev_b, &runtime_ctx()).await.unwrap();

        // Restart — only ws-a events should replay.
        let (_store, mut mb) = router
            .spawn_actor_with_permissions_in_workspace(
                identity("agent.scoped"),
                perms,
                "ws-a",
            )
            .unwrap();

        let received = mb.try_recv().expect("must replay ws-a event");
        assert_eq!(received.id, ev_a.id);
        assert!(
            mb.try_recv().is_err(),
            "must NOT replay events from ws-b"
        );
    }

    // ── Replay ordering ──────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn replay_delivers_events_in_ulid_order() {
        let (_tmp, mut router) = setup();

        let (_store, _mb) = router
            .spawn_actor_with_permissions(identity("agent.ordered"), global_studio_perms())
            .unwrap();
        router.stop_actor("agent.ordered").unwrap();

        // Emit events in order while offline.
        let ev1 = ev("studio.first");
        let ev2 = ev("studio.second");
        let ev3 = ev("studio.third");
        router.route(ev1.clone(), &runtime_ctx()).await.unwrap();
        router.route(ev2.clone(), &runtime_ctx()).await.unwrap();
        router.route(ev3.clone(), &runtime_ctx()).await.unwrap();

        // Restart — events must replay in ULID order.
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(identity("agent.ordered"), global_studio_perms())
            .unwrap();

        let r1 = mb.try_recv().expect("must replay first");
        let r2 = mb.try_recv().expect("must replay second");
        let r3 = mb.try_recv().expect("must replay third");

        // ULID is monotonically increasing — IDs should be in order.
        assert!(r1.id < r2.id, "events must be in ULID order: {} < {}", r1.id, r2.id);
        assert!(r2.id < r3.id, "events must be in ULID order: {} < {}", r2.id, r3.id);

        assert_eq!(r1.id, ev1.id);
        assert_eq!(r2.id, ev2.id);
        assert_eq!(r3.id, ev3.id);
    }
}
