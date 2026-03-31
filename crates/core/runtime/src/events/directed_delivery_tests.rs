//! Tests for directed delivery via target_actor_id on EventEnvelope.
//!
//! Acceptance criteria covered:
//! - Directed event delivered ONLY to target actor
//! - Directed event NOT delivered to other subscribers of same namespace
//! - target_actor_id = None broadcasts to all matching subscribers
//! - Directed event to nonexistent actor is persisted but not delivered

#[cfg(feature = "unstable")]
mod directed_delivery {
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

    fn setup() -> (tempfile::TempDir, KernelRouter) {
        let tmp = tempdir().unwrap();
        let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();
        (tmp, router)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn directed_event_delivered_only_to_target_actor() {
        let (_tmp, mut router) = setup();

        let global_perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Global,
            }],
        };
        let directed_perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Directed,
            }],
        };

        let (_s1, mut mb_target) = router
            .spawn_actor_with_permissions(identity("agent.reviewer"), directed_perms)
            .unwrap();
        let (_s2, mut mb_other) = router
            .spawn_actor_with_permissions(identity("agent.watcher"), global_perms)
            .unwrap();

        let mut directed = ev("studio.feedback");
        directed.target_actor_id = Some("agent.reviewer".into());
        router.route(directed.clone(), &runtime_ctx()).await.unwrap();

        // Target receives the event.
        let received = mb_target.try_recv().expect("target must receive directed event");
        assert_eq!(received.id, directed.id);

        // Other subscriber does NOT receive directed events.
        assert!(
            mb_other.try_recv().is_err(),
            "directed event must NOT be delivered to non-target subscribers"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn broadcast_event_reaches_all_subscribers() {
        let (_tmp, mut router) = setup();

        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Global,
            }],
        };

        let (_s1, mut mb1) = router
            .spawn_actor_with_permissions(identity("agent.a"), perms.clone())
            .unwrap();
        let (_s2, mut mb2) = router
            .spawn_actor_with_permissions(identity("agent.b"), perms)
            .unwrap();

        // No target_actor_id — broadcast.
        let event = ev("studio.annotation");
        assert!(event.target_actor_id.is_none());
        router.route(event.clone(), &runtime_ctx()).await.unwrap();

        assert_eq!(mb1.try_recv().unwrap().id, event.id);
        assert_eq!(mb2.try_recv().unwrap().id, event.id);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn directed_event_to_nonexistent_actor_does_not_crash() {
        let (_tmp, mut router) = setup();

        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Global,
            }],
        };
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(identity("agent.alive"), perms)
            .unwrap();

        let mut directed = ev("studio.feedback");
        directed.target_actor_id = Some("agent.ghost".into());

        // Must not crash — event is persisted but not delivered to anyone.
        let result = router.route(directed, &runtime_ctx()).await;
        assert!(result.is_ok(), "directed to nonexistent actor must not crash");

        // The alive agent also does NOT receive it (it's directed to ghost).
        assert!(
            mb.try_recv().is_err(),
            "directed event must not be broadcast to other actors"
        );
    }
}
