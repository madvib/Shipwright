use chrono::Utc;
use tempfile::tempdir;

use crate::db::{block_on, open_db_at};
use crate::events::envelope::EventEnvelope;
use crate::events::filter::EventFilter;
use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::snapshot::ActorSnapshot;

// ── Fixtures ──────────────────────────────────────────────────────────────────

fn setup() -> (tempfile::TempDir, KernelRouter) {
    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();
    (tmp, router)
}

fn agent_config(id: &str) -> ActorConfig {
    ActorConfig {
        namespace: id.into(),
        write_namespaces: vec![format!("{id}.")],
        read_namespaces: vec![format!("{id}.")],
        subscribe_namespaces: vec![format!("{id}.")],
    }
}

fn ev(id: &str, event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, id, &serde_json::json!({})).unwrap()
}

fn kernel_event_count(router: &KernelRouter, event_type: &str) -> i64 {
    let mut conn = open_db_at(router.kernel_store_path()).unwrap();
    block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE event_type = ?")
            .bind(event_type)
            .fetch_one(&mut conn)
            .await
    })
    .unwrap()
}

// ── Serialization roundtrip ───────────────────────────────────────────────────

#[test]
fn roundtrip_serialization() {
    let snap = ActorSnapshot {
        actor_id: "agent-1".into(),
        namespace: "agent".into(),
        config: agent_config("agent"),
        db_bytes: b"SQLite format 3".to_vec(),
        created_at: Utc::now(),
        event_count: 5,
        last_event_id: Some("01JABCDEF".into()),
    };

    let bytes = snap.to_bytes().unwrap();
    let restored = ActorSnapshot::from_bytes(&bytes).unwrap();

    assert_eq!(restored.actor_id, snap.actor_id);
    assert_eq!(restored.namespace, snap.namespace);
    assert_eq!(restored.db_bytes, snap.db_bytes);
    assert_eq!(restored.event_count, snap.event_count);
    assert_eq!(restored.last_event_id, snap.last_event_id);
    assert_eq!(restored.config.namespace, snap.config.namespace);
    assert_eq!(restored.config.write_namespaces, snap.config.write_namespaces);
}

// ── snapshot() ───────────────────────────────────────────────────────────────

#[test]
fn snapshot_does_not_stop_actor() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("agent-1", agent_config("agent-1")).unwrap();
    store.append(&ev("entity-1", "agent-1.task_started")).unwrap();

    let snap = router.snapshot("agent-1").unwrap();

    assert_eq!(snap.actor_id, "agent-1");
    assert_eq!(snap.event_count, 1);
    assert!(snap.last_event_id.is_some());
    assert_eq!(router.actor_count(), 1, "actor must still be live after snapshot");
}

#[test]
fn snapshot_emits_kernel_lifecycle_event() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("agent-1", agent_config("agent-1")).unwrap();
    store.append(&ev("entity-1", "agent-1.task_started")).unwrap();

    router.snapshot("agent-1").unwrap();

    assert_eq!(
        kernel_event_count(&router, "kernel.actor.snapshot"),
        1,
        "kernel.actor.snapshot must be written to kernel store"
    );
}

#[test]
fn snapshot_of_running_actor_does_not_corrupt_state() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("agent-1", agent_config("agent-1")).unwrap();
    store.append(&ev("entity-1", "agent-1.event_one")).unwrap();

    router.snapshot("agent-1").unwrap();

    // Original actor can still write and read
    store.append(&ev("entity-1", "agent-1.event_two")).unwrap();
    let events = store.query(&EventFilter::default()).unwrap();
    assert_eq!(events.len(), 2, "original actor must see both events after snapshot");
}

#[test]
fn snapshot_errors_on_unknown_actor() {
    let (_tmp, router) = setup();
    assert!(router.snapshot("ghost").is_err());
}

// ── suspend() ────────────────────────────────────────────────────────────────

#[test]
fn suspend_produces_snapshot_and_stops_actor() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("agent-1", agent_config("agent-1")).unwrap();
    store.append(&ev("entity-1", "agent-1.task_started")).unwrap();

    let snap = router.suspend("agent-1").unwrap();

    assert_eq!(snap.event_count, 1);
    assert_eq!(router.actor_count(), 0, "actor must be stopped after suspend");
    assert_eq!(kernel_event_count(&router, "kernel.actor.suspended"), 1);
}

// ── restore() ────────────────────────────────────────────────────────────────

#[test]
fn restore_errors_if_actor_id_already_exists() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("agent-1", agent_config("agent-1")).unwrap();
    store.append(&ev("entity-1", "agent-1.task_started")).unwrap();

    let snap = router.snapshot("agent-1").unwrap();
    match router.restore(snap) {
        Err(e) => assert!(
            e.to_string().contains("already exists"),
            "unexpected error: {e}"
        ),
        Ok(_) => panic!("expected error but restore succeeded"),
    }
}

#[test]
fn restore_events_intact_after_stop() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("agent-1", agent_config("agent-1")).unwrap();
    store.append(&ev("entity-1", "agent-1.event_one")).unwrap();
    store.append(&ev("entity-1", "agent-1.event_two")).unwrap();

    let snap = router.suspend("agent-1").unwrap();
    assert_eq!(snap.event_count, 2);

    let (restored_store, _mb) = router.restore(snap).unwrap();

    let events = restored_store.query(&EventFilter::default()).unwrap();
    assert_eq!(events.len(), 2, "restored actor must have full event history");
    assert_eq!(events[0].event_type, "agent-1.event_one");
    assert_eq!(events[1].event_type, "agent-1.event_two");
    assert_eq!(kernel_event_count(&router, "kernel.actor.restored"), 1);
}

#[test]
fn restored_actor_can_append_new_events() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("agent-1", agent_config("agent-1")).unwrap();
    store.append(&ev("entity-1", "agent-1.event_one")).unwrap();

    let snap = router.suspend("agent-1").unwrap();
    let (restored_store, _mb) = router.restore(snap).unwrap();

    restored_store
        .append(&ev("entity-1", "agent-1.event_two"))
        .unwrap();

    let events = restored_store.query(&EventFilter::default()).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].event_type, "agent-1.event_two");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restored_actor_receives_new_messages() {
    let (_tmp, mut router) = setup();
    let (store, _mb) = router.spawn_actor("agent-1", agent_config("agent-1")).unwrap();
    store.append(&ev("entity-1", "agent-1.event_one")).unwrap();

    let snap = router.suspend("agent-1").unwrap();
    let (_restored_store, mut mb) = router.restore(snap).unwrap();

    let inbox_event =
        EventEnvelope::new("agent-1.incoming", "entity-1", &serde_json::json!({})).unwrap();
    let ctx = crate::events::validator::EmitContext {
        caller_kind: crate::events::validator::CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };
    router.route(inbox_event.clone(), &ctx).await.unwrap();

    let received = mb.try_recv().expect("restored actor mailbox must receive event");
    assert_eq!(received.id, inbox_event.id);
}
