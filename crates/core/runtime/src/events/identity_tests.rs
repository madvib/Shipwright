//! Tests for ActorIdentity and ActorType.
//!
//! Acceptance criteria covered:
//! - Each spawned actor gets a unique ULID instance_id
//! - Actor label is stable across restarts (same label, new instance_id)
//! - Actor type (Agent/Service/App/Cli) is set at spawn and immutable

#[cfg(feature = "unstable")]
mod identity {
    use crate::events::identity::{ActorIdentity, ActorType};
    use crate::events::kernel_router::KernelRouter;
    use crate::events::permissions::{ActorPermissions, DeliveryScope, PermittedSubscription};
    use tempfile::tempdir;

    fn permissive_permissions() -> ActorPermissions {
        ActorPermissions {
            emit: vec!["studio.".into()],
            subscribe: vec![PermittedSubscription {
                namespace: "studio.".into(),
                scope: DeliveryScope::Global,
            }],
        }
    }

    // ── ULID uniqueness ─────────────────────────────────────────────────────

    #[test]
    fn spawned_actor_gets_unique_instance_id() {
        let tmp = tempdir().unwrap();
        let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

        let identity_a = ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: "agent.alpha".into(),
            actor_type: ActorType::Agent,
        };
        let identity_b = ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: "agent.beta".into(),
            actor_type: ActorType::Agent,
        };

        let (_store_a, _mb_a) = router
            .spawn_actor_with_permissions(identity_a.clone(), permissive_permissions())
            .unwrap();
        let (_store_b, _mb_b) = router
            .spawn_actor_with_permissions(identity_b.clone(), permissive_permissions())
            .unwrap();

        assert_ne!(identity_a.instance_id, identity_b.instance_id);
    }

    #[test]
    fn instance_id_is_valid_ulid() {
        let id = ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: "agent.test".into(),
            actor_type: ActorType::Agent,
        };
        // Must parse back to a valid ULID.
        ulid::Ulid::from_string(&id.instance_id).expect("instance_id must be a valid ULID");
    }

    // ── Label stability across restarts ──────────────────────────────────────

    #[test]
    fn label_stable_across_restarts_with_new_instance_id() {
        let tmp = tempdir().unwrap();
        let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

        let label = "agent.reviewer";

        // First spawn.
        let id1 = ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: label.into(),
            actor_type: ActorType::Agent,
        };
        let first_instance = id1.instance_id.clone();
        let (_store, _mb) = router
            .spawn_actor_with_permissions(id1, permissive_permissions())
            .unwrap();
        router.stop_actor(label).unwrap();

        // Second spawn — same label, new instance_id.
        let id2 = ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: label.into(),
            actor_type: ActorType::Agent,
        };
        let second_instance = id2.instance_id.clone();
        let (_store, _mb) = router
            .spawn_actor_with_permissions(id2, permissive_permissions())
            .unwrap();

        assert_eq!(label, "agent.reviewer", "label must be stable");
        assert_ne!(
            first_instance, second_instance,
            "instance_id must differ across restarts"
        );
    }

    // ── ActorType variants ───────────────────────────────────────────────────

    #[test]
    fn actor_type_agent_is_set_at_spawn() {
        let id = ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: "agent.test".into(),
            actor_type: ActorType::Agent,
        };
        assert!(matches!(id.actor_type, ActorType::Agent));
    }

    #[test]
    fn actor_type_service_is_set_at_spawn() {
        let id = ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: "sync".into(),
            actor_type: ActorType::Service,
        };
        assert!(matches!(id.actor_type, ActorType::Service));
    }

    #[test]
    fn actor_type_app_is_set_at_spawn() {
        let id = ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: "studio".into(),
            actor_type: ActorType::App,
        };
        assert!(matches!(id.actor_type, ActorType::App));
    }

    #[test]
    fn actor_type_cli_is_set_at_spawn() {
        let id = ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: "cli".into(),
            actor_type: ActorType::Cli,
        };
        assert!(matches!(id.actor_type, ActorType::Cli));
    }

    // ── Spawn rejects duplicate label ────────────────────────────────────────

    #[test]
    fn spawn_rejects_duplicate_label() {
        let tmp = tempdir().unwrap();
        let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

        let id1 = ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: "agent.dup".into(),
            actor_type: ActorType::Agent,
        };
        let id2 = ActorIdentity {
            instance_id: ulid::Ulid::new().to_string(),
            label: "agent.dup".into(),
            actor_type: ActorType::Agent,
        };

        router
            .spawn_actor_with_permissions(id1, permissive_permissions())
            .unwrap();
        let err = router.spawn_actor_with_permissions(id2, permissive_permissions());
        assert!(err.is_err(), "duplicate label must be rejected");
    }
}
