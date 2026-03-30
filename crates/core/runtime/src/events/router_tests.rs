use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::events::EventEnvelope;
use crate::events::filter::EventFilter;
use crate::events::router::EventRouter;
use crate::events::store::EventStore;
use crate::events::validator::{
    CallerKind, EmitContext, EventValidator, ValidationError,
};

// ── Mock store ───────────────────────────────────────────────────────────────

struct MockEventStore {
    events: Mutex<Vec<EventEnvelope>>,
}

impl MockEventStore {
    fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
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
        Ok(self
            .events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.entity_id == entity_id)
            .cloned()
            .collect())
    }

    fn query_correlation(&self, cid: &str) -> Result<Vec<EventEnvelope>> {
        Ok(self
            .events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.correlation_id.as_deref() == Some(cid))
            .cloned()
            .collect())
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    }
}

fn make_event(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
}

fn make_ws_event(event_type: &str, ws_id: &str) -> EventEnvelope {
    make_event(event_type).with_context(Some(ws_id), None)
}

fn make_elevated_event(event_type: &str) -> EventEnvelope {
    make_event(event_type).elevate()
}

fn new_router(store: Arc<MockEventStore>) -> EventRouter {
    EventRouter::new(store, 64)
}

// ── Rejecting validator (for test_validator_rejects) ─────────────────────────

struct RejectAllValidator;

impl EventValidator for RejectAllValidator {
    fn validate(&self, _: &EventEnvelope, _: &EmitContext) -> Result<(), ValidationError> {
        Err(ValidationError::RateLimited {
            producer: "test".into(),
        })
    }
}

struct PassAllValidator;

impl EventValidator for PassAllValidator {
    fn validate(&self, _: &EventEnvelope, _: &EmitContext) -> Result<(), ValidationError> {
        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn emit_persists_to_store() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store.clone());
    let event = make_event("workspace.created");
    let id = event.id.clone();

    router.emit(event, &runtime_ctx()).await.unwrap();

    assert_eq!(store.len(), 1);
    assert!(store.get(&id).unwrap().is_some());
}

#[tokio::test]
async fn emit_broadcasts_elevated_to_platform() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store);
    let mut rx = router.subscribe_platform();

    let event = make_elevated_event("workspace.created");
    router.emit(event, &runtime_ctx()).await.unwrap();

    let received = rx.try_recv().unwrap();
    assert_eq!(received.event_type, "workspace.created");
}

#[tokio::test]
async fn emit_does_not_broadcast_non_elevated_to_platform() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store);
    let mut rx = router.subscribe_platform();

    let event = make_event("session.started");
    router.emit(event, &runtime_ctx()).await.unwrap();

    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn emit_broadcasts_to_workspace() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store);
    let mut rx = router.subscribe_workspace("ws-1").await;

    let event = make_ws_event("session.started", "ws-1");
    router.emit(event, &runtime_ctx()).await.unwrap();

    let received = rx.try_recv().unwrap();
    assert_eq!(received.event_type, "session.started");
}

#[tokio::test]
async fn emit_does_not_leak_between_workspaces() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store);
    let _rx_a = router.subscribe_workspace("ws-a").await;
    let mut rx_b = router.subscribe_workspace("ws-b").await;

    let event = make_ws_event("session.started", "ws-a");
    router.emit(event, &runtime_ctx()).await.unwrap();

    assert!(rx_b.try_recv().is_err());
}

#[tokio::test]
async fn lazy_channel_creation() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store);

    assert_eq!(router.workspace_channel_count().await, 0);
    let _rx = router.subscribe_workspace("ws-1").await;
    assert_eq!(router.workspace_channel_count().await, 1);
}

#[tokio::test]
async fn validator_rejects_invalid_event() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store.clone())
        .with_validator(Box::new(RejectAllValidator));

    let event = make_event("workspace.created");
    let result = router.emit(event, &runtime_ctx()).await;

    assert!(result.is_err());
    assert_eq!(store.len(), 0);
}

#[tokio::test]
async fn multiple_validators_all_must_pass() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store.clone())
        .with_validator(Box::new(PassAllValidator))
        .with_validator(Box::new(RejectAllValidator));

    let event = make_event("workspace.created");
    let result = router.emit(event, &runtime_ctx()).await;

    assert!(result.is_err());
    assert_eq!(store.len(), 0);
}

#[tokio::test]
async fn workspace_isolation() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store);

    let mut rx_a = router.subscribe_workspace("ws-a").await;
    let mut rx_b = router.subscribe_workspace("ws-b").await;
    let mut rx_c = router.subscribe_workspace("ws-c").await;

    router
        .emit(make_ws_event("e1", "ws-a"), &runtime_ctx())
        .await
        .unwrap();
    router
        .emit(make_ws_event("e2", "ws-b"), &runtime_ctx())
        .await
        .unwrap();
    router
        .emit(make_ws_event("e3", "ws-c"), &runtime_ctx())
        .await
        .unwrap();

    assert_eq!(rx_a.try_recv().unwrap().event_type, "e1");
    assert!(rx_a.try_recv().is_err());

    assert_eq!(rx_b.try_recv().unwrap().event_type, "e2");
    assert!(rx_b.try_recv().is_err());

    assert_eq!(rx_c.try_recv().unwrap().event_type, "e3");
    assert!(rx_c.try_recv().is_err());
}

#[tokio::test]
async fn platform_and_workspace_dual_broadcast() {
    let store = Arc::new(MockEventStore::new());
    let router = new_router(store);

    let mut platform_rx = router.subscribe_platform();
    let mut ws_rx = router.subscribe_workspace("ws-1").await;

    let event = make_elevated_event("workspace.created").with_context(Some("ws-1"), None);
    router.emit(event, &runtime_ctx()).await.unwrap();

    assert_eq!(platform_rx.try_recv().unwrap().event_type, "workspace.created");
    assert_eq!(ws_rx.try_recv().unwrap().event_type, "workspace.created");
}
