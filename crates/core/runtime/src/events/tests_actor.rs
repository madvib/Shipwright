use crate::db::ensure_db;
use crate::events::envelope::EventEnvelope;
use crate::events::filter::EventFilter;
use crate::events::store::{EventStore, SqliteEventStore};
use crate::events::types::event_types;
use crate::events::types::{SessionEnded, SessionProgress, SessionStarted, WorkspaceActivated};
use crate::project::init_project;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, SqliteEventStore) {
    let tmp = tempdir().unwrap();
    let _ship = init_project(tmp.path().to_path_buf()).unwrap();
    ensure_db().unwrap();
    let store = SqliteEventStore::new().unwrap();
    (tmp, store)
}

#[test]
fn test_actor_fields_round_trip() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let ev = EventEnvelope::new(event_types::SESSION_STARTED, "ws-1", &SessionStarted { goal: None, ..Default::default() })?
        .with_actor_id("actor-ws-001")
        .with_parent_actor_id("actor-system-000")
        .elevate();

    store.append(&ev)?;
    let got = store.get(&ev.id)?.unwrap();

    assert_eq!(got.actor_id.as_deref(), Some("actor-ws-001"));
    assert_eq!(got.parent_actor_id.as_deref(), Some("actor-system-000"));
    assert!(got.elevated);
    Ok(())
}

#[test]
fn test_filter_by_actor_id() -> anyhow::Result<()> {
    let (_tmp, store) = setup();

    // actor A emits two events
    for _ in 0..2 {
        let ev = EventEnvelope::new(event_types::SESSION_PROGRESS, "entity-1", &SessionProgress { message: "tick".into() })?
            .with_actor_id("actor-a");
        store.append(&ev)?;
    }
    // actor B emits one event
    let ev_b = EventEnvelope::new(event_types::SESSION_PROGRESS, "entity-1", &SessionProgress { message: "tock".into() })?
        .with_actor_id("actor-b");
    store.append(&ev_b)?;

    let results = store.query(&EventFilter { actor_id: Some("actor-a".into()), ..Default::default() })?;
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|e| e.actor_id.as_deref() == Some("actor-a")));

    let results_b = store.query(&EventFilter { actor_id: Some("actor-b".into()), ..Default::default() })?;
    assert_eq!(results_b.len(), 1);
    Ok(())
}

#[test]
fn test_elevated_filter() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let entity = "elevation-entity";

    // session.progress — local only (not elevated)
    for _ in 0..3 {
        let ev = EventEnvelope::new(event_types::SESSION_PROGRESS, entity, &SessionProgress { message: "noise".into() })?;
        store.append(&ev)?;
    }
    // session.ended — elevated (bubbles to workspace)
    let ev_end = EventEnvelope::new(event_types::SESSION_ENDED, entity, &SessionEnded { summary: None, duration_secs: None, gate_result: None, ..Default::default() })?
        .elevate();
    store.append(&ev_end)?;
    // workspace.activated — elevated
    let ev_ws = EventEnvelope::new(event_types::WORKSPACE_ACTIVATED, entity, &WorkspaceActivated { agent_id: None, providers: vec![] })?
        .elevate();
    store.append(&ev_ws)?;

    let all = store.query_aggregate(entity)?;
    assert_eq!(all.len(), 5);

    let elevated = store.query(&EventFilter { entity_id: Some(entity.into()), elevated_only: true, ..Default::default() })?;
    assert_eq!(elevated.len(), 2);
    assert!(elevated.iter().all(|e| e.elevated));
    Ok(())
}

#[test]
fn supervisor_queries_elevated_child_events_by_parent_actor_id() -> anyhow::Result<()> {
    let (_tmp, store) = setup();

    // Parent actor event (elevated, actor_id="supervisor-1") — should NOT appear
    let parent_ev = EventEnvelope::new(event_types::ACTOR_CREATED, "supervisor-1", &crate::events::types::ActorCreated { kind: "supervisor".into(), environment_type: "local".into(), workspace_id: None, parent_actor_id: None, restart_count: 0 })?
        .with_actor_id("supervisor-1")
        .elevate();
    store.append(&parent_ev)?;

    // Child 1 — elevated, parent_actor_id="supervisor-1"
    let child1 = EventEnvelope::new(event_types::SESSION_STARTED, "child-entity-1", &SessionStarted { goal: None, ..Default::default() })?
        .with_actor_id("child-actor-1")
        .with_parent_actor_id("supervisor-1")
        .elevate();
    store.append(&child1)?;

    // Child 2 — elevated, parent_actor_id="supervisor-1"
    let child2 = EventEnvelope::new(event_types::SESSION_ENDED, "child-entity-2", &crate::events::types::SessionEnded { summary: None, duration_secs: None, gate_result: None, ..Default::default() })?
        .with_actor_id("child-actor-2")
        .with_parent_actor_id("supervisor-1")
        .elevate();
    store.append(&child2)?;

    // Child 3 — NOT elevated, parent_actor_id="supervisor-1" — must be filtered out
    let child3 = EventEnvelope::new(event_types::SESSION_PROGRESS, "child-entity-3", &SessionProgress { message: "noise".into() })?
        .with_actor_id("child-actor-3")
        .with_parent_actor_id("supervisor-1");
    store.append(&child3)?;

    let results = store.query(&EventFilter {
        parent_actor_id: Some("supervisor-1".into()),
        elevated_only: true,
        ..Default::default()
    })?;

    assert_eq!(results.len(), 2, "expected exactly 2 elevated child events, got {}", results.len());
    assert!(results.iter().all(|e| e.elevated), "all returned events must be elevated");
    assert!(results.iter().all(|e| e.parent_actor_id.as_deref() == Some("supervisor-1")));
    Ok(())
}

#[test]
fn test_hierarchy_traversal_via_parent_actor_id() -> anyhow::Result<()> {
    let (_tmp, store) = setup();

    // Workspace actor emits an event
    let ws_ev = EventEnvelope::new(event_types::WORKSPACE_ACTIVATED, "ws-entity", &WorkspaceActivated { agent_id: None, providers: vec![] })?
        .with_actor_id("actor-ws-1");
    store.append(&ws_ev)?;

    // Session actor emits under workspace
    let sess_ev = EventEnvelope::new(event_types::SESSION_STARTED, "sess-entity", &SessionStarted { goal: None, ..Default::default() })?
        .with_actor_id("actor-sess-1")
        .with_parent_actor_id("actor-ws-1");
    store.append(&sess_ev)?;

    // Query all events where parent is the workspace actor
    let children = store.query(&EventFilter {
        actor_id: Some("actor-sess-1".into()),
        ..Default::default()
    })?;
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].parent_actor_id.as_deref(), Some("actor-ws-1"));
    Ok(())
}
