//! Tests for db::events — typed event queries, gate outcomes, time filtering.

use super::events::{
    list_all_events, list_events_since_time, list_gate_outcomes, list_recent_events,
    record_gate_outcome,
};
use crate::db::ensure_db;
use crate::events::envelope::EventEnvelope;
use crate::events::store::{EventStore, SqliteEventStore};
use crate::events::types::event_types;
use crate::events::types::{GateFailed, GatePassed, ProjectLog};
use crate::project::init_project;
use chrono::Utc;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    ensure_db().unwrap();
    (tmp, ship_dir)
}

fn make_log_event(action: &str) -> EventEnvelope {
    EventEnvelope::new(
        event_types::PROJECT_LOG,
        "project",
        &ProjectLog {
            action: action.to_string(),
            details: String::new(),
        },
    )
    .unwrap()
}

#[test]
fn list_all_events_starts_empty() {
    let (_tmp, _ship_dir) = setup();
    let all = list_all_events().unwrap();
    assert!(all.is_empty());
}

#[test]
fn append_and_list_all() {
    let (_tmp, _ship_dir) = setup();
    let store = SqliteEventStore::new().unwrap();
    store.append(&make_log_event("first")).unwrap();
    store.append(&make_log_event("second")).unwrap();

    let all = list_all_events().unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].event_type, event_types::PROJECT_LOG);
    assert_eq!(all[1].event_type, event_types::PROJECT_LOG);
}

#[test]
fn list_since_filters_by_time() {
    let (_tmp, _ship_dir) = setup();
    let store = SqliteEventStore::new().unwrap();
    store.append(&make_log_event("before")).unwrap();

    let future = Utc::now() + chrono::Duration::hours(1);
    let filtered = list_events_since_time(&future, None).unwrap();
    assert!(filtered.is_empty());

    let past = Utc::now() - chrono::Duration::hours(1);
    let all = list_events_since_time(&past, None).unwrap();
    assert_eq!(all.len(), 1);
}

#[test]
fn list_since_with_limit() {
    let (_tmp, _ship_dir) = setup();
    let store = SqliteEventStore::new().unwrap();
    let past = Utc::now() - chrono::Duration::hours(1);
    for i in 0..5 {
        store.append(&make_log_event(&format!("event-{}", i))).unwrap();
    }
    let limited = list_events_since_time(&past, Some(3)).unwrap();
    assert_eq!(limited.len(), 3);
}

#[test]
fn list_recent_returns_in_asc_order() {
    let (_tmp, _ship_dir) = setup();
    let store = SqliteEventStore::new().unwrap();
    let mut inserted_ids = Vec::new();
    for i in 0..5 {
        let ev = make_log_event(&format!("event-{}", i));
        inserted_ids.push(ev.id.clone());
        store.append(&ev).unwrap();
    }
    // Ask for last 3, returned in ascending (oldest-first) order
    let recent = list_recent_events(3).unwrap();
    assert_eq!(recent.len(), 3);
    // The returned slice must be sorted ascending by id
    let ids: Vec<&str> = recent.iter().map(|e| e.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "list_recent_events must return events in ASC order");
}

#[test]
fn record_gate_pass_creates_event_and_completes_job() {
    let (_tmp, _ship_dir) = setup();
    let job =
        crate::db::jobs::create_job("gate-test", None, None, None, None, 0, None, vec![], vec![])
            .unwrap();
    crate::db::jobs::update_job_status(&job.id, "running").unwrap();

    let env = record_gate_outcome(&job.id, true, "all tests green").unwrap();
    assert_eq!(env.event_type, event_types::GATE_PASSED);
    assert_eq!(env.entity_id, job.id);

    let payload: GatePassed = serde_json::from_str(&env.payload_json).unwrap();
    assert_eq!(payload.evidence, "all tests green");

    let updated = crate::db::jobs::get_job(&job.id).unwrap().unwrap();
    assert_eq!(updated.status, "complete");
}

#[test]
fn record_gate_fail_creates_event_leaves_job_running() {
    let (_tmp, _ship_dir) = setup();
    let job =
        crate::db::jobs::create_job("gate-test", None, None, None, None, 0, None, vec![], vec![])
            .unwrap();
    crate::db::jobs::update_job_status(&job.id, "running").unwrap();

    let env = record_gate_outcome(&job.id, false, "3 tests failed").unwrap();
    assert_eq!(env.event_type, event_types::GATE_FAILED);

    let payload: GateFailed = serde_json::from_str(&env.payload_json).unwrap();
    assert_eq!(payload.evidence, "3 tests failed");

    let updated = crate::db::jobs::get_job(&job.id).unwrap().unwrap();
    assert_eq!(updated.status, "running");
}

#[test]
fn list_gate_outcomes_filters_by_job() {
    let (_tmp, _ship_dir) = setup();
    let job_a =
        crate::db::jobs::create_job("gate-a", None, None, None, None, 0, None, vec![], vec![])
            .unwrap();
    let job_b =
        crate::db::jobs::create_job("gate-b", None, None, None, None, 0, None, vec![], vec![])
            .unwrap();
    crate::db::jobs::update_job_status(&job_a.id, "running").unwrap();
    crate::db::jobs::update_job_status(&job_b.id, "running").unwrap();

    record_gate_outcome(&job_a.id, false, "lint errors").unwrap();
    record_gate_outcome(&job_a.id, true, "all clean").unwrap();
    record_gate_outcome(&job_b.id, false, "build broken").unwrap();

    // Non-gate event in the store (no job_id set — must not appear in gate outcomes)
    let store = SqliteEventStore::new().unwrap();
    store.append(&make_log_event("noise")).unwrap();

    let outcomes_a = list_gate_outcomes(&job_a.id).unwrap();
    assert_eq!(outcomes_a.len(), 2);
    assert_eq!(outcomes_a[0].event_type, event_types::GATE_FAILED);
    assert_eq!(outcomes_a[1].event_type, event_types::GATE_PASSED);

    let outcomes_b = list_gate_outcomes(&job_b.id).unwrap();
    assert_eq!(outcomes_b.len(), 1);
    assert_eq!(outcomes_b[0].event_type, event_types::GATE_FAILED);

    let outcomes_none = list_gate_outcomes("no-such-job").unwrap();
    assert!(outcomes_none.is_empty());
}
