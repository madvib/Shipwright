use crate::db::{block_on, open_db_at};
use crate::events::envelope::EventEnvelope;
use crate::events::filter::EventFilter;
use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::validator::{CallerKind, EmitContext};
use tempfile::tempdir;

// ── Fixtures ──────────────────────────────────────────────────────────────────

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    }
}

fn ev(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
}

fn studio_config() -> ActorConfig {
    ActorConfig {
        namespace: "studio".into(),
        write_namespaces: vec!["studio.".into()],
        read_namespaces: vec!["studio.".into(), "session.".into()],
        subscribe_namespaces: vec!["studio.".into()],
    }
}

fn setup() -> (tempfile::TempDir, KernelRouter) {
    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();
    (tmp, router)
}

// ── ActorStore namespace enforcement ─────────────────────────────────────────

#[test]
fn actor_store_rejects_write_outside_namespace() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("studio", studio_config()).unwrap();
    let err = store.append(&ev("session.started")).unwrap_err();
    assert!(
        err.to_string().contains("write namespace violation"),
        "unexpected error: {err}"
    );
}

#[test]
fn actor_store_allows_write_inside_namespace() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("studio", studio_config()).unwrap();
    store.append(&ev("studio.canvas_updated")).unwrap();
}

#[test]
fn actor_store_rejects_read_outside_namespace() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("studio", studio_config()).unwrap();
    let filter = EventFilter {
        event_type: Some("workspace.created".into()),
        ..Default::default()
    };
    let err = store.query(&filter).unwrap_err();
    assert!(
        err.to_string().contains("read namespace violation"),
        "unexpected error: {err}"
    );
}

#[test]
fn actor_store_allows_read_in_permitted_namespace() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("studio", studio_config()).unwrap();
    // studio. is in read_namespaces
    let filter = EventFilter {
        event_type: Some("studio.canvas_updated".into()),
        ..Default::default()
    };
    assert!(store.query(&filter).is_ok());
    // session. is also in read_namespaces
    let filter = EventFilter {
        event_type: Some("session.started".into()),
        ..Default::default()
    };
    assert!(store.query(&filter).is_ok());
}

#[test]
fn actor_store_unfiltered_query_allowed() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("studio", studio_config()).unwrap();
    store.append(&ev("studio.ready")).unwrap();
    let results = store.query(&EventFilter::default()).unwrap();
    assert_eq!(results.len(), 1);
}

// ── Multi-actor isolation ─────────────────────────────────────────────────────

#[test]
fn actors_cannot_read_each_others_stores() {
    let (_tmp, mut router) = setup();

    let agent_cfg = |id: &str| ActorConfig {
        namespace: id.into(),
        write_namespaces: vec!["agent.".into()],
        read_namespaces: vec!["agent.".into()],
        subscribe_namespaces: vec![],
    };

    let (store_a, _mb_a) = router.spawn_actor("agent-a", agent_cfg("agent-a")).unwrap();
    let (store_b, _mb_b) = router.spawn_actor("agent-b", agent_cfg("agent-b")).unwrap();

    store_a.append(&ev("agent.task_started")).unwrap();

    // Actor B's store is empty — it never saw A's write.
    let results = store_b.query(&EventFilter::default()).unwrap();
    assert!(results.is_empty(), "actor B must not see actor A's events");

    // Actor A can see its own event.
    let results = store_a.query(&EventFilter::default()).unwrap();
    assert_eq!(results.len(), 1);
}

// ── KernelRouter routing ──────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn route_delivers_to_subscribed_mailbox() {
    let (_tmp, mut router) = setup();
    let (_store, mut mb) = router.spawn_actor("studio", studio_config()).unwrap();

    let event = ev("studio.canvas_updated");
    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    let received = mb.try_recv().expect("mailbox should have an event");
    assert_eq!(received.id, event.id);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn route_skips_unsubscribed_mailbox() {
    let (_tmp, mut router) = setup();
    let agent_cfg = ActorConfig {
        namespace: "agent".into(),
        write_namespaces: vec!["agent.".into()],
        read_namespaces: vec!["agent.".into()],
        subscribe_namespaces: vec!["agent.".into()],
    };
    let (_store, mut mb) = router.spawn_actor("agent-1", agent_cfg).unwrap();

    // studio event — agent-1 is not subscribed to "studio."
    router
        .route(ev("studio.canvas_updated"), &runtime_ctx())
        .await
        .unwrap();

    assert!(
        mb.try_recv().is_err(),
        "unsubscribed actor must not receive the event"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn route_delivers_to_multiple_subscribers() {
    let (_tmp, mut router) = setup();

    let sub_cfg = |id: &str| ActorConfig {
        namespace: id.into(),
        write_namespaces: vec![format!("{id}.")],
        read_namespaces: vec![format!("{id}."), "studio.".into()],
        subscribe_namespaces: vec!["studio.".into()],
    };

    let (_s1, mut mb1) = router.spawn_actor("watcher-1", sub_cfg("watcher-1")).unwrap();
    let (_s2, mut mb2) = router.spawn_actor("watcher-2", sub_cfg("watcher-2")).unwrap();

    let event = ev("studio.canvas_updated");
    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    assert_eq!(mb1.try_recv().unwrap().id, event.id);
    assert_eq!(mb2.try_recv().unwrap().id, event.id);
}

// ── Kernel store ──────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kernel_store_receives_system_events() {
    let (_tmp, router) = setup();
    let event = ev("session.started");
    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    // Verify via raw SQL — kernel store has the event.
    let mut conn = open_db_at(router.kernel_store_path()).unwrap();
    let count: i64 = block_on(async {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE id = ?",
        )
        .bind(&event.id)
        .fetch_one(&mut conn)
        .await
    })
    .unwrap();
    assert_eq!(count, 1, "system event must be in kernel store");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kernel_store_does_not_receive_actor_events() {
    let (_tmp, mut router) = setup();
    let (_store, _mb) = router.spawn_actor("studio", studio_config()).unwrap();

    router
        .route(ev("studio.canvas_updated"), &runtime_ctx())
        .await
        .unwrap();

    let mut conn = open_db_at(router.kernel_store_path()).unwrap();
    let count: i64 = block_on(async {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE event_type = 'studio.canvas_updated'",
        )
        .fetch_one(&mut conn)
        .await
    })
    .unwrap();
    assert_eq!(count, 0, "actor events must not appear in the kernel store");
}

// ── Actor lifecycle ───────────────────────────────────────────────────────────

#[test]
fn stop_actor_removes_mailbox_and_subscriptions() {
    let (_tmp, mut router) = setup();
    router.spawn_actor("studio", studio_config()).unwrap();
    assert_eq!(router.actor_count(), 1);

    router.stop_actor("studio").unwrap();
    assert_eq!(router.actor_count(), 0);
}

#[test]
fn stop_actor_errors_on_unknown_actor() {
    let (_tmp, mut router) = setup();
    assert!(router.stop_actor("ghost").is_err());
}

#[test]
fn spawn_actor_rejects_duplicate_id() {
    let (_tmp, mut router) = setup();
    router.spawn_actor("studio", studio_config()).unwrap();
    assert!(router.spawn_actor("studio", studio_config()).is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stopped_actor_mailbox_is_closed() {
    let (_tmp, mut router) = setup();
    let (_store, mut mb) = router.spawn_actor("studio", studio_config()).unwrap();

    router.stop_actor("studio").unwrap();

    // Sender was dropped — recv returns None.
    assert!(mb.recv().await.is_none());
}
