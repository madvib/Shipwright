//! Tests for db::events — typed event queries, time filtering.

use super::events::{
    list_all_events, list_events_since_time, list_recent_events, query_events_since,
};
use crate::db::ensure_db;
use crate::events::envelope::EventEnvelope;
use crate::events::store::{EventStore, SqliteEventStore};
use crate::events::types::event_types;
use crate::events::types::ProjectLog;
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
fn query_events_since_no_cursor_returns_all() {
    let (_tmp, _ship_dir) = setup();
    let store = SqliteEventStore::new().unwrap();
    store.append(&make_log_event("a")).unwrap();
    store.append(&make_log_event("b")).unwrap();
    let all = query_events_since(None, false).unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn query_events_since_cursor_returns_later_events() {
    let (_tmp, _ship_dir) = setup();
    let store = SqliteEventStore::new().unwrap();
    let ev1 = make_log_event("first");
    let cursor = ev1.id.clone();
    store.append(&ev1).unwrap();
    store.append(&make_log_event("second")).unwrap();
    store.append(&make_log_event("third")).unwrap();
    let after = query_events_since(Some(&cursor), false).unwrap();
    assert_eq!(after.len(), 2);
}

#[test]
fn query_events_since_latest_cursor_returns_empty() {
    let (_tmp, _ship_dir) = setup();
    let store = SqliteEventStore::new().unwrap();
    let ev = make_log_event("only");
    let cursor = ev.id.clone();
    store.append(&ev).unwrap();
    let after = query_events_since(Some(&cursor), false).unwrap();
    assert!(after.is_empty());
}

#[test]
fn query_events_since_elevated_only() {
    use crate::events::types::{WorkspaceActivated, event_types as et};
    let (_tmp, _ship_dir) = setup();
    let store = SqliteEventStore::new().unwrap();
    // Non-elevated event
    store.append(&make_log_event("noise")).unwrap();
    // Elevated event
    let elevated = EventEnvelope::new(
        et::WORKSPACE_ACTIVATED,
        "ws-1",
        &WorkspaceActivated { agent_id: None, providers: vec![] },
    )
    .unwrap()
    .elevate();
    store.append(&elevated).unwrap();
    let all = query_events_since(None, false).unwrap();
    assert_eq!(all.len(), 2);
    let elevated_only = query_events_since(None, true).unwrap();
    assert_eq!(elevated_only.len(), 1);
    assert_eq!(elevated_only[0].event_type, et::WORKSPACE_ACTIVATED);
}
