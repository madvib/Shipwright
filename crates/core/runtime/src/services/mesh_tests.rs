//! Tests for the agent mesh service.

use tempfile::tempdir;
use tokio::sync::mpsc;

use crate::events::actor_store::init_actor_db;
use crate::events::{ActorStore, EventEnvelope};
use crate::services::mesh::{AgentStatus, MeshService};
use crate::services::ServiceHandler;

fn setup() -> (MeshService, mpsc::UnboundedReceiver<EventEnvelope>, ActorStore) {
    let (tx, rx) = mpsc::unbounded_channel();
    let svc = MeshService::new(tx);
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("events.db");
    init_actor_db(&db_path).unwrap();
    let store = ActorStore::new("mesh", db_path, vec!["mesh.".into()], vec!["mesh.".into()]);
    (svc, rx, store)
}

fn ev(event_type: &str, entity: &str, payload: serde_json::Value) -> EventEnvelope {
    EventEnvelope::new(event_type, entity, &payload).unwrap()
}

fn register_event(id: &str, caps: &[&str]) -> EventEnvelope {
    let caps: Vec<String> = caps.iter().map(|s| s.to_string()).collect();
    ev("mesh.register", id, serde_json::json!({ "agent_id": id, "capabilities": caps }))
}

fn deregister_event(id: &str) -> EventEnvelope {
    ev("mesh.deregister", id, serde_json::json!({ "agent_id": id }))
}

fn send_event(from: &str, to: &str, body: serde_json::Value) -> EventEnvelope {
    ev("mesh.send", from, serde_json::json!({ "from": from, "to": to, "body": body }))
}

fn broadcast_event(from: &str, body: serde_json::Value, cap_filter: Option<&str>) -> EventEnvelope {
    let mut p = serde_json::json!({ "from": from, "body": body });
    if let Some(cap) = cap_filter { p["capability_filter"] = serde_json::json!(cap); }
    ev("mesh.broadcast", from, p)
}

fn discover_event(from: &str, cap: Option<&str>, status: Option<&str>) -> EventEnvelope {
    let mut p = serde_json::json!({ "from": from });
    if let Some(c) = cap { p["capability"] = serde_json::json!(c); }
    if let Some(s) = status { p["status"] = serde_json::json!(s); }
    ev("mesh.discover.request", from, p)
}

fn status_event(id: &str, status: &str) -> EventEnvelope {
    ev("mesh.status", id, serde_json::json!({ "agent_id": id, "status": status }))
}

#[test]
fn register_adds_agent() {
    let (mut svc, _rx, store) = setup();
    svc.handle(&register_event("agent.rust", &["rust", "testing"]), &store)
        .unwrap();
    assert_eq!(svc.agent_count(), 1);
    let entry = svc.get_agent("agent.rust").unwrap();
    assert_eq!(entry.capabilities, vec!["rust", "testing"]);
    assert_eq!(entry.status, AgentStatus::Active);
}

#[test]
fn deregister_removes_agent() {
    let (mut svc, _rx, store) = setup();
    svc.handle(&register_event("agent.rust", &["rust"]), &store)
        .unwrap();
    svc.handle(&deregister_event("agent.rust"), &store).unwrap();
    assert_eq!(svc.agent_count(), 0);
}

#[test]
fn duplicate_register_updates_entry() {
    let (mut svc, _rx, store) = setup();
    svc.handle(&register_event("agent.rust", &["rust"]), &store)
        .unwrap();
    svc.handle(&register_event("agent.rust", &["rust", "wasm"]), &store)
        .unwrap();
    assert_eq!(svc.agent_count(), 1);
    let entry = svc.get_agent("agent.rust").unwrap();
    assert_eq!(entry.capabilities, vec!["rust", "wasm"]);
}

#[test]
fn status_updates_agent() {
    let (mut svc, _rx, store) = setup();
    svc.handle(&register_event("agent.rust", &["rust"]), &store)
        .unwrap();
    svc.handle(&status_event("agent.rust", "busy"), &store)
        .unwrap();
    assert_eq!(svc.get_agent("agent.rust").unwrap().status, AgentStatus::Busy);
}

#[test]
fn status_unknown_agent_is_noop() {
    let (mut svc, _rx, store) = setup();
    svc.handle(&status_event("agent.ghost", "idle"), &store)
        .unwrap();
    assert_eq!(svc.agent_count(), 0);
}

#[test]
fn send_to_registered_agent_emits_message() {
    let (mut svc, mut rx, store) = setup();
    svc.handle(&register_event("agent.reviewer", &["review"]), &store)
        .unwrap();
    svc.handle(
        &send_event("agent.rust", "agent.reviewer", serde_json::json!({"text": "hi"})),
        &store,
    )
    .unwrap();

    let msg = rx.try_recv().unwrap();
    assert_eq!(msg.event_type, "mesh.message");
    assert_eq!(msg.target_actor_id.as_deref(), Some("agent.reviewer"));
    let p: serde_json::Value = serde_json::from_str(&msg.payload_json).unwrap();
    assert_eq!(p["from"], "agent.rust");
    assert_eq!(p["body"]["text"], "hi");
}

#[test]
fn send_to_unknown_agent_emits_failed() {
    let (mut svc, mut rx, store) = setup();
    svc.handle(
        &send_event("agent.rust", "agent.ghost", serde_json::json!({})),
        &store,
    )
    .unwrap();

    let msg = rx.try_recv().unwrap();
    assert_eq!(msg.event_type, "mesh.send.failed");
    assert_eq!(msg.target_actor_id.as_deref(), Some("agent.rust"));
    let p: serde_json::Value = serde_json::from_str(&msg.payload_json).unwrap();
    assert_eq!(p["reason"], "agent not found");
}

#[test]
fn broadcast_delivers_to_all_except_sender() {
    let (mut svc, mut rx, store) = setup();
    svc.handle(&register_event("agent.a", &["rust"]), &store).unwrap();
    svc.handle(&register_event("agent.b", &["rust"]), &store).unwrap();
    svc.handle(&register_event("agent.c", &["go"]), &store).unwrap();

    svc.handle(
        &broadcast_event("agent.a", serde_json::json!({"ping": true}), None),
        &store,
    )
    .unwrap();

    let mut targets: Vec<String> = Vec::new();
    while let Ok(msg) = rx.try_recv() {
        assert_eq!(msg.event_type, "mesh.message");
        targets.push(msg.target_actor_id.unwrap());
    }
    targets.sort();
    assert_eq!(targets, vec!["agent.b", "agent.c"]);
}

#[test]
fn broadcast_with_capability_filter() {
    let (mut svc, mut rx, store) = setup();
    svc.handle(&register_event("agent.a", &["rust"]), &store).unwrap();
    svc.handle(&register_event("agent.b", &["rust"]), &store).unwrap();
    svc.handle(&register_event("agent.c", &["go"]), &store).unwrap();

    svc.handle(
        &broadcast_event("agent.a", serde_json::json!({}), Some("rust")),
        &store,
    )
    .unwrap();

    let mut targets: Vec<String> = Vec::new();
    while let Ok(msg) = rx.try_recv() {
        targets.push(msg.target_actor_id.unwrap());
    }
    assert_eq!(targets, vec!["agent.b"]);
}

#[test]
fn broadcast_without_filter_excludes_sender() {
    let (mut svc, mut rx, store) = setup();
    svc.handle(&register_event("agent.only", &[]), &store).unwrap();

    svc.handle(
        &broadcast_event("agent.only", serde_json::json!({}), None),
        &store,
    )
    .unwrap();

    assert!(rx.try_recv().is_err());
}

#[test]
fn discover_returns_all_agents() {
    let (mut svc, mut rx, store) = setup();
    svc.handle(&register_event("agent.a", &["rust"]), &store).unwrap();
    svc.handle(&register_event("agent.b", &["go"]), &store).unwrap();

    svc.handle(&discover_event("agent.requester", None, None), &store)
        .unwrap();

    let resp = rx.try_recv().unwrap();
    assert_eq!(resp.event_type, "mesh.discover.response");
    assert_eq!(resp.target_actor_id.as_deref(), Some("agent.requester"));
    let p: serde_json::Value = serde_json::from_str(&resp.payload_json).unwrap();
    assert_eq!(p["agents"].as_array().unwrap().len(), 2);
}

#[test]
fn discover_filters_by_capability() {
    let (mut svc, mut rx, store) = setup();
    svc.handle(&register_event("agent.a", &["rust"]), &store).unwrap();
    svc.handle(&register_event("agent.b", &["go"]), &store).unwrap();

    svc.handle(&discover_event("agent.requester", Some("rust"), None), &store)
        .unwrap();

    let resp = rx.try_recv().unwrap();
    let p: serde_json::Value = serde_json::from_str(&resp.payload_json).unwrap();
    let agents = p["agents"].as_array().unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["agent_id"], "agent.a");
}

#[test]
fn discover_filters_by_status() {
    let (mut svc, mut rx, store) = setup();
    svc.handle(&register_event("agent.a", &["rust"]), &store).unwrap();
    svc.handle(&register_event("agent.b", &["rust"]), &store).unwrap();
    svc.handle(&status_event("agent.b", "busy"), &store).unwrap();

    svc.handle(&discover_event("agent.requester", None, Some("active")), &store)
        .unwrap();

    let resp = rx.try_recv().unwrap();
    let p: serde_json::Value = serde_json::from_str(&resp.payload_json).unwrap();
    let agents = p["agents"].as_array().unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["agent_id"], "agent.a");
}

#[test]
fn discover_filters_by_capability_and_status() {
    let (mut svc, mut rx, store) = setup();
    svc.handle(&register_event("agent.a", &["rust"]), &store).unwrap();
    svc.handle(&register_event("agent.b", &["rust"]), &store).unwrap();
    svc.handle(&status_event("agent.a", "idle"), &store).unwrap();

    svc.handle(
        &discover_event("agent.requester", Some("rust"), Some("idle")),
        &store,
    )
    .unwrap();

    let resp = rx.try_recv().unwrap();
    let p: serde_json::Value = serde_json::from_str(&resp.payload_json).unwrap();
    let agents = p["agents"].as_array().unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["agent_id"], "agent.a");
}

#[test]
fn service_name_is_mesh() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let svc = MeshService::new(tx);
    assert_eq!(svc.name(), "mesh");
}

#[test]
fn unknown_mesh_event_is_ignored() {
    let (mut svc, _rx, store) = setup();
    let event = EventEnvelope::new("mesh.unknown.thing", "x", &serde_json::json!({})).unwrap();
    svc.handle(&event, &store).unwrap();
    assert_eq!(svc.agent_count(), 0);
}

#[test]
fn registered_agent_survives_malformed_events() {
    let (mut svc, _rx, store) = setup();
    svc.handle(&register_event("agent.rust", &["rust"]), &store).unwrap();
    // Malformed send (missing "to" field) — mesh should handle gracefully
    let bad = EventEnvelope::new("mesh.send", "agent.rust", &serde_json::json!({})).unwrap();
    let _ = svc.handle(&bad, &store);
    // Agent stays registered
    assert_eq!(svc.agent_count(), 1);
    assert!(svc.get_agent("agent.rust").is_some());
}
