//! Integration tests for EventRouter wiring.
//!
//! These verify the full emit → persist → broadcast pipeline.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;

use crate::events::envelope::EventEnvelope;
use crate::events::filter::EventFilter;
use crate::events::router::EventRouter;
use crate::events::store::EventStore;
use crate::events::validator::{CallerKind, EmitContext};

// ── Mock EventStore ─────────────────────────────────────────────────────────

struct MockEventStore {
    events: Mutex<Vec<EventEnvelope>>,
}

impl MockEventStore {
    fn new() -> Self {
        Self { events: Mutex::new(Vec::new()) }
    }
    fn len(&self) -> usize {
        self.events.lock().unwrap().len()
    }
}

impl EventStore for MockEventStore {
    fn append(&self, event: &EventEnvelope) -> Result<()> {
        self.events.lock().unwrap().push(event.clone());
        Ok(())
    }
    fn get(&self, id: &str) -> Result<Option<EventEnvelope>> {
        Ok(self.events.lock().unwrap().iter().find(|e| e.id == id).cloned())
    }
    fn query(&self, _filter: &EventFilter) -> Result<Vec<EventEnvelope>> {
        Ok(self.events.lock().unwrap().clone())
    }
    fn query_aggregate(&self, entity_id: &str) -> Result<Vec<EventEnvelope>> {
        Ok(self.events.lock().unwrap().iter()
            .filter(|e| e.entity_id == entity_id).cloned().collect())
    }
    fn query_correlation(&self, cid: &str) -> Result<Vec<EventEnvelope>> {
        Ok(self.events.lock().unwrap().iter()
            .filter(|e| e.correlation_id.as_deref() == Some(cid)).cloned().collect())
    }
}

type SinkLog = Arc<tokio::sync::Mutex<Vec<EventEnvelope>>>;

// ── Helpers ─────────────────────────────────────────────────────────────────

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    }
}

fn new_router(store: Arc<MockEventStore>) -> EventRouter {
    EventRouter::new(store, 64)
}

fn workspace_event(branch: &str) -> EventEnvelope {
    EventEnvelope::new("workspace.created", branch, &serde_json::json!({
        "workspace_id": branch, "workspace_type": "feature", "status": "active",
    }))
    .unwrap()
    .with_context(Some(branch), None)
    .with_actor_id(branch)
    .elevate()
}

fn session_started_event(session_id: &str, ws_id: &str) -> EventEnvelope {
    EventEnvelope::new("session.started", session_id, &serde_json::json!({
        "goal": "test", "workspace_id": ws_id, "workspace_branch": ws_id,
    }))
    .unwrap()
    .with_context(Some(ws_id), Some(session_id))
    .with_actor_id(session_id)
    .with_parent_actor_id(ws_id)
    .elevate()
}

fn session_progress_event(session_id: &str, ws_id: &str) -> EventEnvelope {
    EventEnvelope::new("session.progress", session_id, &serde_json::json!({
        "message": "doing stuff",
    }))
    .unwrap()
    .with_context(Some(ws_id), Some(session_id))
    .with_actor_id(session_id)
    .with_parent_actor_id(ws_id)
}

fn actor_created_event(actor_id: &str, ws_id: &str) -> EventEnvelope {
    EventEnvelope::new("actor.created", actor_id, &serde_json::json!({
        "kind": "agent", "environment_type": "mcp",
    }))
    .unwrap()
    .with_context(Some(ws_id), None)
    .with_actor_id(actor_id)
    .elevate()
}

// ── Router broadcast tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_emit_workspace_event_reaches_platform_subscriber() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store);
    let mut rx = router.subscribe_platform();

    router.emit(workspace_event("feature/auth"), &runtime_ctx()).await.unwrap();

    let received = rx.try_recv().unwrap();
    assert_eq!(received.event_type, "workspace.created");
    assert_eq!(received.entity_id, "feature/auth");
}

#[tokio::test]
async fn test_emit_workspace_event_persisted_in_store() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store.clone());
    let event = workspace_event("feature/auth");
    let id = event.id.clone();

    router.emit(event, &runtime_ctx()).await.unwrap();

    assert_eq!(store.len(), 1);
    assert_eq!(store.get(&id).unwrap().unwrap().event_type, "workspace.created");
}

#[tokio::test]
async fn test_emit_session_started_reaches_both_buses() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store);
    let mut platform_rx = router.subscribe_platform();
    let mut ws_rx = router.subscribe_workspace("feature/auth").await;

    router.emit(session_started_event("sess-1", "feature/auth"), &runtime_ctx()).await.unwrap();

    assert_eq!(platform_rx.try_recv().unwrap().event_type, "session.started");
    assert_eq!(ws_rx.try_recv().unwrap().event_type, "session.started");
}

#[tokio::test]
async fn test_emit_session_progress_workspace_only() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store);
    let mut platform_rx = router.subscribe_platform();
    let mut ws_rx = router.subscribe_workspace("feature/auth").await;

    router.emit(session_progress_event("sess-1", "feature/auth"), &runtime_ctx()).await.unwrap();

    assert!(platform_rx.try_recv().is_err(), "progress must NOT reach platform");
    assert_eq!(ws_rx.try_recv().unwrap().event_type, "session.progress");
}

#[tokio::test]
async fn test_emit_actor_event_reaches_workspace_subscriber() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store);
    let mut ws_rx = router.subscribe_workspace("feature/auth").await;

    router.emit(actor_created_event("agent-1", "feature/auth"), &runtime_ctx()).await.unwrap();

    let received = ws_rx.try_recv().unwrap();
    assert_eq!(received.event_type, "actor.created");
    assert_eq!(received.entity_id, "agent-1");
}

#[tokio::test]
async fn test_elevated_workspace_event_dual_broadcast() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store);
    let mut platform_rx = router.subscribe_platform();
    let mut ws_rx = router.subscribe_workspace("feature/auth").await;

    router.emit(workspace_event("feature/auth"), &runtime_ctx()).await.unwrap();

    assert!(platform_rx.try_recv().is_ok());
    assert!(ws_rx.try_recv().is_ok());
}

#[tokio::test]
async fn test_router_initialization_idempotent() {
    let store = Arc::new(MockEventStore::new());
    let r1 = new_router(store.clone());
    let r2 = new_router(store.clone());

    r1.emit(workspace_event("br-1"), &runtime_ctx()).await.unwrap();
    r2.emit(workspace_event("br-2"), &runtime_ctx()).await.unwrap();

    assert_eq!(store.len(), 2);
}

// ── End-to-end: emit → broadcast → relay-like sink ──────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_full_loop_emit_to_sink() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store.clone());
    let rx = router.subscribe_workspace("feature/auth").await;
    let log: SinkLog = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let log_clone = log.clone();

    let handle = tokio::spawn(async move {
        let mut rx = rx;
        while let Ok(env) = rx.recv().await {
            log_clone.lock().await.push(env);
        }
    });

    let event = workspace_event("feature/auth");
    let event_id = event.id.clone();
    router.emit(event, &runtime_ctx()).await.unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(store.len(), 1);
    let received = log.lock().await;
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].id, event_id);

    drop(router);
    let _ = handle.await;
}
