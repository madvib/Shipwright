use crate::db::{block_on, ensure_db, open_db_at};
use crate::events::envelope::EventEnvelope;
use crate::events::filter::EventFilter;
use crate::events::store::{EventStore, SqliteEventStore};
use crate::events::types::*;
use crate::events::types::event_types;
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
fn test_round_trip() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let ev = EventEnvelope::new(
        event_types::SESSION_STARTED,
        "entity-a",
        &SessionStarted { goal: Some("build the thing".into()), ..Default::default() },
    )?
    .with_context(Some("ws-1"), Some("sess-1"));

    store.append(&ev)?;
    let got = store.get(&ev.id)?.expect("event not found");

    assert_eq!(got.id, ev.id);
    assert_eq!(got.event_type, ev.event_type);
    assert_eq!(got.entity_id, ev.entity_id);
    assert_eq!(got.actor, ev.actor);
    assert_eq!(got.payload_json, ev.payload_json);
    assert_eq!(got.version, ev.version);
    assert_eq!(got.workspace_id, ev.workspace_id);
    assert_eq!(got.session_id, ev.session_id);
    Ok(())
}

#[test]
fn test_ordering() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let entity = "ordering-entity";
    for _ in 0..100 {
        let ev = EventEnvelope::new(event_types::SESSION_PROGRESS, entity, &SessionProgress { message: "tick".into() })?;
        store.append(&ev)?;
    }
    let results = store.query_aggregate(entity)?;
    assert_eq!(results.len(), 100);
    let ids: Vec<&str> = results.iter().map(|e| e.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "events must be in ULID (time) order");
    Ok(())
}

#[test]
fn test_filter_by_entity_id() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let ev_a = EventEnvelope::new(event_types::SESSION_STARTED, "entity-a", &SessionStarted { goal: None, ..Default::default() })?;
    let ev_b = EventEnvelope::new(event_types::SESSION_STARTED, "entity-b", &SessionStarted { goal: None, ..Default::default() })?;
    store.append(&ev_a)?;
    store.append(&ev_b)?;

    let results = store.query_aggregate("entity-a")?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entity_id, "entity-a");
    Ok(())
}

#[test]
fn test_filter_by_event_type() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let entity = "filter-type-entity";
    store.append(&EventEnvelope::new(event_types::WORKSPACE_ACTIVATED, entity, &WorkspaceActivated { agent_id: None, providers: vec![] })?)?;
    store.append(&EventEnvelope::new(event_types::SESSION_STARTED, entity, &SessionStarted { goal: None, ..Default::default() })?)?;
    store.append(&EventEnvelope::new(event_types::WORKSPACE_COMPILED, entity, &WorkspaceCompiled { config_generation: 1, duration_ms: 50 })?)?;

    let ws = store.query(&EventFilter { entity_id: Some(entity.into()), event_type: Some(event_types::WORKSPACE_ACTIVATED.into()), ..Default::default() })?;
    assert_eq!(ws.len(), 1);
    assert_eq!(ws[0].event_type, event_types::WORKSPACE_ACTIVATED);

    let sess = store.query(&EventFilter { entity_id: Some(entity.into()), event_type: Some(event_types::SESSION_STARTED.into()), ..Default::default() })?;
    assert_eq!(sess.len(), 1);
    assert_eq!(sess[0].event_type, event_types::SESSION_STARTED);
    Ok(())
}

#[test]
fn test_filter_by_workspace_id() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let ev = EventEnvelope::new(event_types::SESSION_STARTED, "e1", &SessionStarted { goal: None, ..Default::default() })?
        .with_context(Some("ws-scoped"), None);
    store.append(&ev)?;
    store.append(&EventEnvelope::new(event_types::SESSION_STARTED, "e2", &SessionStarted { goal: None, ..Default::default() })?)?;

    let results = store.query(&EventFilter { workspace_id: Some("ws-scoped".into()), ..Default::default() })?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].workspace_id.as_deref(), Some("ws-scoped"));
    Ok(())
}

#[test]
fn test_causation_chain() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let ev_a = EventEnvelope::new(event_types::SESSION_STARTED, "cause-entity", &SessionStarted { goal: None, ..Default::default() })?;
    store.append(&ev_a)?;
    let ev_b = EventEnvelope::new(event_types::SESSION_ENDED, "cause-entity", &SessionEnded { summary: None, duration_secs: None, gate_result: None, ..Default::default() })?
        .with_causation(&ev_a.id);
    store.append(&ev_b)?;

    let got_b = store.get(&ev_b.id)?.unwrap();
    assert_eq!(got_b.causation_id.as_deref(), Some(ev_a.id.as_str()));
    Ok(())
}

#[test]
fn test_immutability() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let ev = EventEnvelope::new(event_types::SESSION_STARTED, "immut-entity", &SessionStarted { goal: None, ..Default::default() })?;
    store.append(&ev)?;

    let db_p = crate::db::db_path()?;
    let mut conn = open_db_at(&db_p)?;
    let result = block_on(async {
        sqlx::query("UPDATE events SET actor = 'hacked' WHERE id = ?")
            .bind(&ev.id)
            .execute(&mut conn)
            .await
    });
    assert!(result.is_err(), "immutability trigger must block UPDATE");
    Ok(())
}

#[test]
fn test_payload_round_trip() -> anyhow::Result<()> {
    let (_tmp, store) = setup();

    macro_rules! rt {
        ($etype:expr, $payload:expr) => {{
            let ev = EventEnvelope::new($etype, "rt-entity", &$payload)?;
            store.append(&ev)?;
            let got = store.get(&ev.id)?.expect("not found");
            assert_eq!(got.payload_json, ev.payload_json, "payload mismatch: {}", $etype);
        }};
    }

    rt!(event_types::WORKSPACE_ACTIVATED,   WorkspaceActivated { agent_id: Some("a1".into()), providers: vec!["claude".into()] });
    rt!(event_types::WORKSPACE_COMPILED,    WorkspaceCompiled { config_generation: 3, duration_ms: 200 });
    rt!(event_types::WORKSPACE_COMPILE_FAILED, WorkspaceCompileFailed { error: "bad syntax".into() });
    rt!(event_types::WORKSPACE_ARCHIVED,    WorkspaceArchived {});
    rt!(event_types::SESSION_STARTED,       SessionStarted { goal: Some("deploy".into()), ..Default::default() });
    rt!(event_types::SESSION_PROGRESS,      SessionProgress { message: "halfway".into() });
    rt!(event_types::SESSION_ENDED,         SessionEnded { summary: Some("done".into()), duration_secs: Some(120), gate_result: Some("pass".into()), ..Default::default() });
    rt!(event_types::ACTOR_CREATED,         ActorCreated { kind: "worker".into(), environment_type: "cli".into(), workspace_id: None, parent_actor_id: None, restart_count: 0 });
    rt!(event_types::ACTOR_WOKE,            ActorWoke {});
    rt!(event_types::ACTOR_SLEPT,           ActorSlept { idle_secs: 45 });
    rt!(event_types::ACTOR_STOPPED,         ActorStopped { reason: "shutdown".into() });
    rt!(event_types::ACTOR_CRASHED,         ActorCrashed { error: "OOM".into(), restart_count: 2 });

    Ok(())
}

#[test]
fn test_concurrent_appends() -> anyhow::Result<()> {
    use std::sync::Arc;

    let (_tmp, store) = setup();
    let store = Arc::new(store);
    let entity = "concurrent-entity";
    let mut handles = Vec::new();

    for _ in 0..10 {
        let store = Arc::clone(&store);
        let entity = entity.to_string();
        handles.push(std::thread::spawn(move || {
            let ev = EventEnvelope::new(
                event_types::SESSION_STARTED,
                &entity,
                &SessionStarted { goal: None, ..Default::default() },
            )
            .expect("create envelope");
            store.append(&ev).expect("append");
        }));
    }

    for h in handles {
        h.join().expect("thread panicked");
    }

    let results = store.query_aggregate(entity)?;
    assert_eq!(results.len(), 10, "all 10 appends must succeed");

    let ids: std::collections::HashSet<_> = results.iter().map(|e| &e.id).collect();
    assert_eq!(ids.len(), 10, "no duplicate event IDs");
    Ok(())
}

#[test]
fn test_empty_queries() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let results = store.query_aggregate("no-such-entity")?;
    assert!(results.is_empty());
    let results = store.query(&EventFilter { event_type: Some("nonexistent.type".into()), ..Default::default() })?;
    assert!(results.is_empty());
    Ok(())
}

#[test]
fn test_version_field() -> anyhow::Result<()> {
    let (_tmp, store) = setup();

    // Default version is 1
    let ev = EventEnvelope::new(event_types::SESSION_STARTED, "ver-entity", &SessionStarted { goal: None, ..Default::default() })?;
    assert_eq!(ev.version, 1);
    store.append(&ev)?;
    let got = store.get(&ev.id)?.unwrap();
    assert_eq!(got.version, 1);

    // Explicit version
    let mut ev2 = EventEnvelope::new(event_types::SESSION_ENDED, "ver-entity", &SessionEnded { summary: None, duration_secs: None, gate_result: None, ..Default::default() })?;
    ev2.version = 2;
    store.append(&ev2)?;
    let got2 = store.get(&ev2.id)?.unwrap();
    assert_eq!(got2.version, 2);
    Ok(())
}

#[test]
fn test_property_event_ordering() -> anyhow::Result<()> {
    let (_tmp, store) = setup();
    let entity = "prop-entity";
    let types = event_types::ALL;
    let n = 75;
    let mut expected_ids = Vec::with_capacity(n);

    for i in 0..n {
        let etype = types[i % types.len()];
        let ev = EventEnvelope::new(etype, entity, &SessionProgress { message: format!("step {i}") })?;
        expected_ids.push(ev.id.clone());
        store.append(&ev)?;
    }

    let results = store.query_aggregate(entity)?;

    // Count matches
    assert_eq!(results.len(), n, "all events must be stored");

    // ULID ordering is monotonic
    let ids: Vec<&str> = results.iter().map(|e| e.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "results must be in ULID order");

    // No events missing or duplicated
    let result_set: std::collections::HashSet<_> = results.iter().map(|e| &e.id).collect();
    assert_eq!(result_set.len(), n, "no duplicate events");
    for id in &expected_ids {
        assert!(result_set.contains(id), "event {id} is missing");
    }
    Ok(())
}
