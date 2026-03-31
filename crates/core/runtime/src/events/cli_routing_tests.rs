//! Tests: CLI-originated events route through KernelRouter.
//!
//! Uses local KernelRouter instances (not the global OnceLock singleton)
//! to keep tests isolated. Each test spins up its own router and verifies
//! that the routing path works end-to-end.

use tempfile::tempdir;

use crate::db::{block_on, open_db_at};
use crate::events::envelope::EventEnvelope;
use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::types::event_types;
use crate::events::validator::{CallerKind, EmitContext};

fn cli_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Cli,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    }
}

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    }
}

fn agent_config() -> ActorConfig {
    ActorConfig {
        namespace: "agent.mcp".into(),
        write_namespaces: vec!["".into()],
        read_namespaces: vec!["agent.".into()],
        subscribe_namespaces: vec![
            "workspace.".into(),
            "session.".into(),
            "gate.".into(),
        ],
    }
}

fn ev(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-cli", &serde_json::json!({})).unwrap()
}

// ── workspace event reaches kernel store ──────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn workspace_event_reaches_kernel_store() {
    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let event = ev(event_types::WORKSPACE_CREATED);
    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    let mut conn = open_db_at(router.kernel_store_path()).unwrap();
    let count: i64 = block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE id = ?")
            .bind(&event.id)
            .fetch_one(&mut conn)
            .await
    })
    .unwrap();
    assert_eq!(count, 1, "workspace event must reach kernel store");
}

// ── session event delivered to subscribed actor mailbox ───────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn session_event_delivered_to_subscribed_mailbox() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let (_store, mut mailbox) = router.spawn_actor("agent.mcp", agent_config()).unwrap();

    let event = ev(event_types::SESSION_STARTED);
    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    let received = mailbox.try_recv().expect("session event must reach subscribed actor mailbox");
    assert_eq!(received.id, event.id);
    assert_eq!(received.event_type, event_types::SESSION_STARTED);
}

// ── gate outcome with CallerKind::Cli reaches kernel store ───────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn gate_outcome_with_cli_context_reaches_kernel_store() {
    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let event = ev(event_types::GATE_PASSED);
    // CallerKind::Cli is trusted — must not be rejected by validators.
    router.route(event.clone(), &cli_ctx()).await.unwrap();

    let mut conn = open_db_at(router.kernel_store_path()).unwrap();
    let count: i64 = block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE id = ?")
            .bind(&event.id)
            .fetch_one(&mut conn)
            .await
    })
    .unwrap();
    assert_eq!(count, 1, "gate outcome with Cli context must reach kernel store");
}

// ── gate outcome delivered to subscribed actor mailbox ────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn gate_outcome_delivered_to_subscribed_mailbox() {
    let tmp = tempdir().unwrap();
    let mut router = KernelRouter::new(tmp.path().join(".ship")).unwrap();

    let (_store, mut mailbox) = router.spawn_actor("agent.mcp", agent_config()).unwrap();

    let event = ev(event_types::GATE_PASSED);
    router.route(event.clone(), &cli_ctx()).await.unwrap();

    let received = mailbox.try_recv().expect("gate event must reach subscribed actor mailbox");
    assert_eq!(received.id, event.id);
}

// ── fallback: no router → events go to platform.db only ──────────────────────
//
// This test verifies the fallback path by directly calling SqliteEventStore,
// which is what workspace_events/session_events/actor_events do when
// kernel_router() returns None.

#[test]
fn event_persisted_to_platform_db_without_router() {
    use crate::db::ensure_db;
    use crate::events::store::{EventStore, SqliteEventStore};
    use crate::project::init_project;

    let tmp = tempdir().unwrap();
    let _ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    ensure_db().unwrap();

    let store = SqliteEventStore::new().unwrap();
    let event = ev(event_types::WORKSPACE_CREATED);
    store.append(&event).unwrap();

    let got = store.get(&event.id).unwrap().expect("event must be in platform.db");
    assert_eq!(got.id, event.id);
    assert_eq!(got.event_type, event_types::WORKSPACE_CREATED);
}
