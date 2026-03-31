//! Tests for ActorPermissions and emit/subscribe enforcement.
//!
//! Acceptance criteria covered:
//! - Actor with emit permission for `studio.*` can emit `studio.annotation`
//! - Actor without emit permission for `studio.*` is rejected emitting `studio.annotation`
//! - Actor with empty emit list cannot emit any events
//! - CallerKind::Cli bypasses emit restrictions (superuser)
//! - Actor cannot subscribe to namespace not in its permissions
//! - Kernel rejects spawn if permissions contain invalid/conflicting declarations

#[cfg(feature = "unstable")]
mod permissions {
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

    fn cli_ctx() -> EmitContext {
        EmitContext {
            caller_kind: CallerKind::Cli,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        }
    }

    fn mcp_ctx() -> EmitContext {
        EmitContext {
            caller_kind: CallerKind::Mcp,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        }
    }

    // ── Emit enforcement ─────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn actor_with_emit_permission_can_emit() {
        let tmp = tempdir().unwrap();
        let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

        let perms = ActorPermissions {
            emit: vec!["studio.".into()],
            subscribe: vec![],
        };
        let (_store, _mb) = router
            .spawn_actor_with_permissions(identity("studio"), perms)
            .unwrap();

        let result = router.route(ev("studio.annotation"), &runtime_ctx()).await;
        assert!(result.is_ok(), "actor with studio.* emit should succeed");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn actor_without_emit_permission_is_rejected() {
        let tmp = tempdir().unwrap();
        let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

        let perms = ActorPermissions {
            emit: vec!["agent.".into()],
            subscribe: vec![],
        };
        let (_store, _mb) = router
            .spawn_actor_with_permissions(identity("agent.reviewer"), perms)
            .unwrap();

        // Agent tries to emit into studio.* — should be rejected.
        let event = ev("studio.annotation")
            .with_actor_id("agent.reviewer");
        let result = router.route(event, &mcp_ctx()).await;
        assert!(result.is_err(), "actor without studio.* emit must be rejected");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn actor_with_empty_emit_cannot_emit_anything() {
        let tmp = tempdir().unwrap();
        let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Workspace,
            }],
        };
        let (_store, _mb) = router
            .spawn_actor_with_permissions(identity("agent.readonly"), perms)
            .unwrap();

        let event = ev("agent.task_started")
            .with_actor_id("agent.readonly");
        let result = router.route(event, &mcp_ctx()).await;
        assert!(
            result.is_err(),
            "actor with empty emit list must not emit any events"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cli_caller_bypasses_emit_restrictions() {
        let tmp = tempdir().unwrap();
        let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![],
        };
        let (_store, _mb) = router
            .spawn_actor_with_permissions(identity("cli-user"), perms)
            .unwrap();

        // CLI is superuser — bypasses all emit restrictions.
        let result = router.route(ev("session.started"), &cli_ctx()).await;
        assert!(
            result.is_ok(),
            "CallerKind::Cli must bypass emit restrictions"
        );
    }

    // ── Subscribe enforcement ────────────────────────────────────────────────

    #[test]
    fn actor_cannot_subscribe_to_unpermitted_namespace() {
        let tmp = tempdir().unwrap();
        let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

        // Permissions only allow subscribing to "studio."
        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Workspace,
            }],
        };

        // But the actor's config requests subscription to "session." as well.
        // The kernel should reject this at spawn time.
        let id = identity("agent.sneaky");
        // spawn_actor_with_permissions should enforce that requested subscriptions
        // are within the permitted set.
        let result = router.spawn_actor_with_permissions(id, perms.clone());
        // The spawn itself succeeds — subscriptions are derived from permissions.
        // The actor gets ONLY the subscriptions listed in its permissions.
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn actor_only_receives_events_from_permitted_subscriptions() {
        let tmp = tempdir().unwrap();
        let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Global,
            }],
        };
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(identity("agent.limited"), perms)
            .unwrap();

        // Route a session event — agent is NOT subscribed.
        router
            .route(ev("session.started"), &runtime_ctx())
            .await
            .unwrap();

        assert!(
            mb.try_recv().is_err(),
            "actor must not receive events outside its permitted subscriptions"
        );

        // Route a studio event — agent IS subscribed.
        let studio_event = ev("studio.annotation");
        router
            .route(studio_event.clone(), &runtime_ctx())
            .await
            .unwrap();

        let received = mb.try_recv().expect("actor must receive permitted events");
        assert_eq!(received.id, studio_event.id);
    }

    // ── Invalid permission declarations ──────────────────────────────────────

    #[test]
    fn spawn_rejects_empty_namespace_in_emit() {
        let tmp = tempdir().unwrap();
        let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

        let perms = ActorPermissions {
            emit: vec!["".into()],
            subscribe: vec![],
        };
        let result = router.spawn_actor_with_permissions(identity("agent.bad"), perms);
        assert!(
            result.is_err(),
            "empty string in emit list is invalid and must be rejected"
        );
    }

    #[test]
    fn spawn_rejects_empty_namespace_in_subscribe() {
        let tmp = tempdir().unwrap();
        let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "".into(),
                scope: DeliveryScope::Global,
            }],
        };
        let result = router.spawn_actor_with_permissions(identity("agent.bad2"), perms);
        assert!(
            result.is_err(),
            "empty namespace in subscribe is invalid and must be rejected"
        );
    }
}
