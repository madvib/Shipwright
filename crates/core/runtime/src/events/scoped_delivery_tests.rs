//! Tests for scoped delivery (DeliveryScope enforcement).
//!
//! Acceptance criteria covered:
//! - Global scope receives ALL events in subscribed namespace
//! - Workspace scope receives ONLY events matching its workspace_id
//! - Workspace scope does NOT receive events from other workspaces
//! - Directed scope receives ONLY events with matching target_actor_id
//! - Directed scope does NOT receive untargeted events in same namespace
//! - Elevated scope receives ONLY elevated events

#[cfg(feature = "unstable")]
mod scoped_delivery {
    use crate::events::envelope::EventEnvelope;
    use crate::events::identity::{ActorIdentity, ActorType};
    use crate::events::kernel_router::KernelRouter;
    use crate::events::permissions::{ActorPermissions, DeliveryScope, PermittedSubscription};
    use crate::events::validator::{CallerKind, EmitContext};
    use tempfile::tempdir;

    fn identity(label: &str, actor_type: ActorType) -> ActorIdentity {
        ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: label.into(),
            actor_type,
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

    // ── Global scope ─────────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn global_scope_receives_all_events_in_namespace() {
        let (_tmp, mut router) = setup();

        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Global,
            }],
        };
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(identity("sync", ActorType::Service), perms)
            .unwrap();

        // Event from workspace A.
        let ev_a = ev("studio.annotation")
            .with_context(Some("ws-a"), None);
        router.route(ev_a.clone(), &runtime_ctx()).await.unwrap();

        // Event from workspace B.
        let ev_b = ev("studio.feedback")
            .with_context(Some("ws-b"), None);
        router.route(ev_b.clone(), &runtime_ctx()).await.unwrap();

        // Event with no workspace.
        let ev_c = ev("studio.selection");
        router.route(ev_c.clone(), &runtime_ctx()).await.unwrap();

        let r1 = mb.try_recv().expect("must receive ev_a");
        let r2 = mb.try_recv().expect("must receive ev_b");
        let r3 = mb.try_recv().expect("must receive ev_c");
        assert_eq!(r1.id, ev_a.id);
        assert_eq!(r2.id, ev_b.id);
        assert_eq!(r3.id, ev_c.id);
    }

    // ── Workspace scope ──────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn workspace_scope_receives_only_matching_workspace() {
        let (_tmp, mut router) = setup();

        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Workspace,
            }],
        };
        // Agent is bound to workspace "feature/login".
        let id = identity("agent.login", ActorType::Agent);
        let (_store, mut mb) = router
            .spawn_actor_with_permissions_in_workspace(
                id,
                perms,
                "feature/login",
            )
            .unwrap();

        // Event from the agent's workspace — should be delivered.
        let ev_match = ev("studio.annotation")
            .with_context(Some("feature/login"), None);
        router.route(ev_match.clone(), &runtime_ctx()).await.unwrap();

        // Event from a different workspace — should NOT be delivered.
        let ev_other = ev("studio.annotation")
            .with_context(Some("feature/signup"), None);
        router.route(ev_other, &runtime_ctx()).await.unwrap();

        let received = mb.try_recv().expect("must receive matching workspace event");
        assert_eq!(received.id, ev_match.id);
        assert!(
            mb.try_recv().is_err(),
            "must NOT receive event from other workspace"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn workspace_scope_does_not_receive_other_workspace_events() {
        let (_tmp, mut router) = setup();

        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Workspace,
            }],
        };
        let id = identity("agent.isolated", ActorType::Agent);
        let (_store, mut mb) = router
            .spawn_actor_with_permissions_in_workspace(id, perms, "ws-mine")
            .unwrap();

        // Only other-workspace events.
        router
            .route(
                ev("studio.annotation").with_context(Some("ws-other"), None),
                &runtime_ctx(),
            )
            .await
            .unwrap();
        router
            .route(
                ev("studio.feedback").with_context(Some("ws-another"), None),
                &runtime_ctx(),
            )
            .await
            .unwrap();

        assert!(
            mb.try_recv().is_err(),
            "workspace-scoped actor must not receive any cross-workspace events"
        );
    }

    // ── Directed scope ───────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn directed_scope_receives_only_targeted_events() {
        let (_tmp, mut router) = setup();

        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Directed,
            }],
        };
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(
                identity("agent.reviewer", ActorType::Agent),
                perms,
            )
            .unwrap();

        // Directed event targeting this actor.
        let mut targeted = ev("studio.feedback");
        targeted.target_actor_id = Some("agent.reviewer".into());
        router.route(targeted.clone(), &runtime_ctx()).await.unwrap();

        // Untargeted event in the same namespace.
        let untargeted = ev("studio.annotation");
        router.route(untargeted, &runtime_ctx()).await.unwrap();

        let received = mb.try_recv().expect("must receive directed event");
        assert_eq!(received.id, targeted.id);
        assert!(
            mb.try_recv().is_err(),
            "directed-scope actor must not receive untargeted events"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn directed_scope_ignores_events_targeted_at_other_actor() {
        let (_tmp, mut router) = setup();

        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Directed,
            }],
        };
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(
                identity("agent.bystander", ActorType::Agent),
                perms,
            )
            .unwrap();

        let mut targeted = ev("studio.feedback");
        targeted.target_actor_id = Some("agent.reviewer".into());
        router.route(targeted, &runtime_ctx()).await.unwrap();

        assert!(
            mb.try_recv().is_err(),
            "directed event must not reach a different actor"
        );
    }

    // ── Elevated scope ───────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn elevated_scope_receives_only_elevated_events() {
        let (_tmp, mut router) = setup();

        let perms = ActorPermissions {
            emit: vec![],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Elevated,
            }],
        };
        let (_store, mut mb) = router
            .spawn_actor_with_permissions(
                identity("sync-service", ActorType::Service),
                perms,
            )
            .unwrap();

        // Non-elevated event — should not be delivered.
        router
            .route(ev("studio.annotation"), &runtime_ctx())
            .await
            .unwrap();

        // Elevated event — should be delivered.
        let elevated = ev("studio.config_changed").elevate();
        router.route(elevated.clone(), &runtime_ctx()).await.unwrap();

        let received = mb.try_recv().expect("must receive elevated event");
        assert_eq!(received.id, elevated.id);
        assert!(
            mb.try_recv().is_err(),
            "elevated-scope actor must not receive non-elevated events"
        );
    }

}
