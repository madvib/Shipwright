//! Integration tests — cross-component event flow without the mesh layer.
//!
//! Tests prove: directed delivery exclusion, workspace-scoped routing,
//! actor store persistence, multi-service coexistence, and event ordering.

#[cfg(feature = "unstable")]
mod cross_component {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tempfile::tempdir;

    use anyhow::Result;
    use crate::events::filter::EventFilter;
    use crate::events::identity::{ActorIdentity, ActorType};
    use crate::events::kernel_router::{ActorConfig, KernelRouter};
    use crate::events::permissions::{ActorPermissions, DeliveryScope, PermittedSubscription};
    use crate::events::validator::{CallerKind, EmitContext};
    use crate::events::{ActorStore, EventEnvelope};
    use crate::services::{ServiceHandler, spawn_service};

    // ── Fixtures ──────────────────────────────────────────────────────────────

    fn setup() -> (tempfile::TempDir, KernelRouter) {
        let tmp = tempdir().unwrap();
        let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();
        (tmp, router)
    }

    fn ctx() -> EmitContext {
        EmitContext { caller_kind: CallerKind::Runtime, skill_id: None, workspace_id: None, session_id: None }
    }

    fn identity(label: &str) -> ActorIdentity {
        ActorIdentity { instance_id: ulid::Ulid::new().to_string(), label: label.into(), actor_type: ActorType::Agent }
    }

    fn global_perms(ns: &str) -> ActorPermissions {
        ActorPermissions {
            emit: vec![format!("{ns}.")],
            subscribe: vec![PermittedSubscription { namespace: format!("{ns}."), scope: DeliveryScope::Global }],
        }
    }

    fn ws_perms(ns: &str) -> ActorPermissions {
        ActorPermissions {
            emit: vec![format!("{ns}.")],
            subscribe: vec![PermittedSubscription { namespace: format!("{ns}."), scope: DeliveryScope::Workspace }],
        }
    }

    fn ev(t: &str) -> EventEnvelope {
        EventEnvelope::new(t, "e1", &serde_json::json!({})).unwrap()
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn directed_delivery_only_reaches_target() {
        let (_tmp, mut router) = setup();
        let (_sa, mut mb_a) = router.spawn_actor_with_permissions(identity("agent.a"), global_perms("chat")).unwrap();
        let (_sb, mut mb_b) = router.spawn_actor_with_permissions(identity("agent.b"), global_perms("chat")).unwrap();
        let (_sc, mut mb_c) = router.spawn_actor_with_permissions(identity("agent.c"), global_perms("chat")).unwrap();

        let mut directed = ev("chat.message");
        directed.target_actor_id = Some("agent.b".into());
        router.route(directed.clone(), &ctx()).await.unwrap();

        let received = mb_b.try_recv().expect("agent.b must receive the directed event");
        assert_eq!(received.id, directed.id);
        assert!(mb_a.try_recv().is_err(), "agent.a must NOT receive a directed event targeting agent.b");
        assert!(mb_c.try_recv().is_err(), "agent.c must NOT receive a directed event targeting agent.b");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn workspace_scoped_delivery() {
        let (_tmp, mut router) = setup();
        let (_sa, mut mb_a) = router.spawn_actor_with_permissions_in_workspace(
            identity("agent.ws1"), ws_perms("build"), "ws-1",
        ).unwrap();
        let (_sb, mut mb_b) = router.spawn_actor_with_permissions_in_workspace(
            identity("agent.ws2"), ws_perms("build"), "ws-2",
        ).unwrap();
        let ctx = ctx();

        let for_ws1 = ev("build.started").with_context(Some("ws-1"), None);
        router.route(for_ws1.clone(), &ctx).await.unwrap();

        let received = mb_a.try_recv().expect("ws-1 actor must receive build event for ws-1");
        assert_eq!(received.id, for_ws1.id);
        assert!(mb_b.try_recv().is_err(), "ws-2 actor must NOT receive build event for ws-1");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn actor_store_persists_events() {
        let (_tmp, mut router) = setup();
        let perms = ActorPermissions {
            emit: vec!["log.".into()],
            subscribe: vec![PermittedSubscription { namespace: "log.".into(), scope: DeliveryScope::Global }],
        };
        let (store, mut mb) = router.spawn_actor_with_permissions(identity("agent.logger"), perms).unwrap();
        let ctx = ctx();

        // Route 5 events — they arrive in the mailbox
        for i in 0..5u32 {
            let e = EventEnvelope::new("log.entry", "entity", &serde_json::json!({ "seq": i })).unwrap();
            router.route(e, &ctx).await.unwrap();
        }

        // Drain mailbox and persist to store (simulating what the actor would do)
        let mut persisted = Vec::new();
        while let Ok(e) = mb.try_recv() {
            store.append(&e).unwrap();
            persisted.push(e);
        }
        assert_eq!(persisted.len(), 5, "all 5 events must arrive in mailbox");

        let stored = store.query(&EventFilter::default()).unwrap();
        assert_eq!(stored.len(), 5, "all 5 events must be persisted in actor store");
        // Events stored in ULID order (monotonically increasing)
        for w in stored.windows(2) {
            assert!(w[0].id < w[1].id, "events must be in emission order");
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn multiple_services_coexist() {
        let (_tmp, mut router) = setup();

        let mesh_calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let health_calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let mesh_config = ActorConfig {
            namespace: "mesh".into(),
            write_namespaces: vec!["mesh.".into()],
            read_namespaces: vec!["mesh.".into()],
            subscribe_namespaces: vec!["mesh.".into()],
        };
        let health_config = ActorConfig {
            namespace: "health".into(),
            write_namespaces: vec!["health.".into()],
            read_namespaces: vec!["health.".into()],
            subscribe_namespaces: vec!["health.".into()],
        };

        spawn_service(&mut router, "mesh-svc", mesh_config,
            Box::new(RecordingService { name: "mesh-svc".into(), calls: mesh_calls.clone() })).unwrap();
        spawn_service(&mut router, "health-svc", health_config,
            Box::new(RecordingService { name: "health-svc".into(), calls: health_calls.clone() })).unwrap();

        let ctx = ctx();
        router.route(EventEnvelope::new("mesh.register", "x", &serde_json::json!({})).unwrap(), &ctx).await.unwrap();
        router.route(EventEnvelope::new("health.check", "x", &serde_json::json!({})).unwrap(), &ctx).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let mc = mesh_calls.lock().unwrap();
        let hc = health_calls.lock().unwrap();
        assert!(mc.iter().any(|e| e == "mesh.register"), "mesh service must handle mesh.register");
        assert!(!mc.iter().any(|e| e == "health.check"), "mesh service must NOT handle health.check");
        assert!(hc.iter().any(|e| e == "health.check"), "health service must handle health.check");
        assert!(!hc.iter().any(|e| e == "mesh.register"), "health service must NOT handle mesh.register");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn event_ordering_preserved_under_load() {
        const N: u32 = 1000;
        let (_tmp, mut router) = setup();
        // Mailbox capacity is 256; drain concurrently to avoid backpressure deadlock.
        let (_s, mb) = router.spawn_actor_with_permissions(identity("counter.actor"), global_perms("counter")).unwrap();
        let ctx = ctx();

        let drain = tokio::spawn(async move {
            let mut mb = mb;
            let mut seqs = Vec::new();
            loop {
                match tokio::time::timeout(Duration::from_secs(5), mb.recv()).await {
                    Ok(Some(e)) => {
                        let p: serde_json::Value = serde_json::from_str(&e.payload_json).unwrap();
                        seqs.push(p["seq"].as_u64().unwrap());
                        if seqs.len() as u32 == N { break; }
                    }
                    _ => break,
                }
            }
            seqs
        });

        for i in 0..N {
            let e = EventEnvelope::new("counter.tick", "entity", &serde_json::json!({ "seq": i })).unwrap();
            router.route(e, &ctx).await.unwrap();
        }

        let received = drain.await.unwrap();
        assert_eq!(received.len() as u32, N, "all {N} events must be delivered");
        for w in received.windows(2) {
            assert!(w[0] < w[1], "events must arrive in emission order: {} then {}", w[0], w[1]);
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn duplicate_actor_id_spawn_fails() {
        let (_tmp, mut router) = setup();
        router.spawn_actor_with_permissions(identity("agent.dup"), global_perms("data")).unwrap();
        let result = router.spawn_actor_with_permissions(identity("agent.dup"), global_perms("data"));
        assert!(result.is_err(), "spawning a duplicate actor id must return an error");
        let msg = result.err().unwrap().to_string();
        assert!(msg.contains("agent.dup"), "error must name the conflicting actor id");
    }

    // ── Recording service ─────────────────────────────────────────────────────

    struct RecordingService {
        name: String,
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl ServiceHandler for RecordingService {
        fn name(&self) -> &str { &self.name }
        fn handle(&mut self, event: &EventEnvelope, _store: &ActorStore) -> Result<()> {
            self.calls.lock().unwrap().push(event.event_type.clone());
            Ok(())
        }
    }
}
