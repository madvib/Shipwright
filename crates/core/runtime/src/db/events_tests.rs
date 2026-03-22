//! Tests for db::events — context columns, entity/action types, migration.

use super::events::*;
use crate::db::ensure_db;
use crate::events::{EventAction, EventEntity};
use crate::project::init_project;
use chrono::Utc;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempdir().unwrap();
    let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
    ensure_db(&ship_dir).unwrap();
    (tmp, ship_dir)
}

#[test]
fn insert_and_read_event() {
    let (_tmp, ship_dir) = setup();
    let record = insert_event(
        &ship_dir,
        "ship",
        &EventEntity::Project,
        Some("my-project"),
        &EventAction::Init,
        Some("initialized"),
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(record.entity, EventEntity::Project);
    assert_eq!(record.subject, "my-project");
    assert_eq!(record.actor, "ship");
    assert!(record.workspace_id.is_none());

    let all = list_all_events(&ship_dir).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, record.id);
    assert_eq!(all[0].entity, EventEntity::Project);
    assert_eq!(all[0].actor, "ship");
}

#[test]
fn append_only_ordering() {
    let (_tmp, ship_dir) = setup();
    insert_event(
        &ship_dir,
        "ship",
        &EventEntity::Workspace,
        Some("feat/a"),
        &EventAction::Create,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    insert_event(
        &ship_dir,
        "agent",
        &EventEntity::Session,
        Some("sess-1"),
        &EventAction::Start,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    insert_event(
        &ship_dir,
        "logic",
        &EventEntity::Config,
        Some("ship.toml"),
        &EventAction::Update,
        None,
        None,
        None,
        None,
    )
    .unwrap();

    let all = list_all_events(&ship_dir).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].entity, EventEntity::Workspace);
    assert_eq!(all[1].entity, EventEntity::Session);
    assert_eq!(all[2].entity, EventEntity::Config);
}

#[test]
fn list_since_filters_by_time() {
    let (_tmp, ship_dir) = setup();
    insert_event(
        &ship_dir,
        "ship",
        &EventEntity::Project,
        Some("p1"),
        &EventAction::Log,
        None,
        None,
        None,
        None,
    )
    .unwrap();

    let future = Utc::now() + chrono::Duration::hours(1);
    let filtered = list_events_since_time(&ship_dir, &future, None).unwrap();
    assert!(filtered.is_empty());

    let past = Utc::now() - chrono::Duration::hours(1);
    let all = list_events_since_time(&ship_dir, &past, None).unwrap();
    assert_eq!(all.len(), 1);
}

#[test]
fn insert_with_context_ids_and_query_back() {
    let (_tmp, ship_dir) = setup();
    let rec = insert_event(
        &ship_dir,
        "agent",
        &EventEntity::Job,
        Some("job-1"),
        &EventAction::Start,
        Some("started work"),
        Some("ws-abc"),
        Some("sess-xyz"),
        Some("job-1"),
    )
    .unwrap();
    assert_eq!(rec.workspace_id.as_deref(), Some("ws-abc"));
    assert_eq!(rec.session_id.as_deref(), Some("sess-xyz"));
    assert_eq!(rec.job_id.as_deref(), Some("job-1"));

    // Query by job
    let by_job = list_events_by_job(&ship_dir, "job-1").unwrap();
    assert_eq!(by_job.len(), 1);
    assert_eq!(by_job[0].id, rec.id);

    // Query by session
    let by_sess = list_events_by_session(&ship_dir, "sess-xyz").unwrap();
    assert_eq!(by_sess.len(), 1);

    // Query by workspace
    let by_ws = list_events_by_workspace(&ship_dir, "ws-abc").unwrap();
    assert_eq!(by_ws.len(), 1);

    // Unmatched context returns empty
    let empty = list_events_by_job(&ship_dir, "no-such-job").unwrap();
    assert!(empty.is_empty());
}

#[test]
fn context_columns_round_trip_as_none() {
    let (_tmp, ship_dir) = setup();
    let rec = insert_event(
        &ship_dir,
        "ship",
        &EventEntity::Project,
        Some("p"),
        &EventAction::Init,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert!(rec.workspace_id.is_none());
    assert!(rec.session_id.is_none());
    assert!(rec.job_id.is_none());

    let all = list_all_events(&ship_dir).unwrap();
    assert!(all[0].workspace_id.is_none());
    assert!(all[0].session_id.is_none());
    assert!(all[0].job_id.is_none());
}

#[test]
fn new_entity_types_serialize_correctly() {
    let (_tmp, ship_dir) = setup();

    // Gate entity with Pass action
    let gate = insert_event(
        &ship_dir,
        "reviewer",
        &EventEntity::Gate,
        Some("gate-1"),
        &EventAction::Pass,
        Some("all checks green"),
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(gate.entity, EventEntity::Gate);
    assert_eq!(gate.action, EventAction::Pass);

    // Capability entity with Complete action
    let cap = insert_event(
        &ship_dir,
        "agent",
        &EventEntity::Capability,
        Some("cap-1"),
        &EventAction::Complete,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(cap.entity, EventEntity::Capability);
    assert_eq!(cap.action, EventAction::Complete);

    // Target entity
    let tgt = insert_event(
        &ship_dir,
        "ship",
        &EventEntity::Target,
        Some("v0.1"),
        &EventAction::Create,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(tgt.entity, EventEntity::Target);

    // Job entity with Claim action
    let job = insert_event(
        &ship_dir,
        "agent-1",
        &EventEntity::Job,
        Some("job-x"),
        &EventAction::Claim,
        None,
        None,
        None,
        Some("job-x"),
    )
    .unwrap();
    assert_eq!(job.entity, EventEntity::Job);
    assert_eq!(job.action, EventAction::Claim);

    // Dispatch action
    let disp = insert_event(
        &ship_dir,
        "ship",
        &EventEntity::Job,
        Some("job-y"),
        &EventAction::Dispatch,
        None,
        None,
        None,
        Some("job-y"),
    )
    .unwrap();
    assert_eq!(disp.action, EventAction::Dispatch);

    // Fail action
    let fail = insert_event(
        &ship_dir,
        "reviewer",
        &EventEntity::Gate,
        Some("gate-2"),
        &EventAction::Fail,
        Some("test failures"),
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(fail.action, EventAction::Fail);

    // All round-trip through DB
    let all = list_all_events(&ship_dir).unwrap();
    assert_eq!(all.len(), 6);
    assert_eq!(all[0].entity, EventEntity::Gate);
    assert_eq!(all[0].action, EventAction::Pass);
    assert_eq!(all[3].entity, EventEntity::Job);
    assert_eq!(all[3].action, EventAction::Claim);
}

#[test]
fn migrate_job_log_to_events_works() {
    let (_tmp, ship_dir) = setup();

    // Seed job_log entries directly
    crate::db::jobs::append_log(
        &ship_dir,
        "compiling module A",
        Some("j1"),
        Some("feat/x"),
        Some("agent-1"),
    )
    .unwrap();
    crate::db::jobs::append_log(
        &ship_dir,
        "tests passing",
        Some("j1"),
        Some("feat/x"),
        Some("agent-1"),
    )
    .unwrap();
    crate::db::jobs::append_log(&ship_dir, "branch note", None, Some("main"), None).unwrap();

    // Migrate
    let migrated = migrate_job_log_to_events(&ship_dir).unwrap();
    assert_eq!(migrated, 3);

    // Verify events exist
    let all = list_all_events(&ship_dir).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].entity, EventEntity::Job);
    assert_eq!(all[0].action, EventAction::Log);
    assert_eq!(all[0].details.as_deref(), Some("compiling module A"));
    assert_eq!(all[0].job_id.as_deref(), Some("j1"));
    assert_eq!(all[0].actor, "agent-1");

    // Third entry had no actor, should default to "ship"
    assert_eq!(all[2].actor, "ship");
    assert!(all[2].job_id.is_none());

    // Migration is additive (running again adds duplicates -- caller should
    // run once). This tests that the function itself succeeds twice.
    let migrated2 = migrate_job_log_to_events(&ship_dir).unwrap();
    assert_eq!(migrated2, 3);
}

// ── Gate outcome tests ────────────────────────────────────────────────────

#[test]
fn record_gate_pass_creates_event_and_completes_job() {
    let (_tmp, ship_dir) = setup();
    let job = crate::db::jobs::create_job(
        &ship_dir,
        "gate-test",
        None,
        None,
        None,
        None,
        0,
        None,
        vec![],
        vec![],
    )
    .unwrap();
    crate::db::jobs::update_job_status(&ship_dir, &job.id, "running").unwrap();

    let rec = record_gate_outcome(&ship_dir, &job.id, true, "all tests green").unwrap();
    assert_eq!(rec.entity, EventEntity::Gate);
    assert_eq!(rec.action, EventAction::Pass);
    assert_eq!(rec.subject, job.id);
    assert_eq!(rec.details.as_deref(), Some("all tests green"));
    assert_eq!(rec.job_id.as_deref(), Some(job.id.as_str()));

    // Job should now be "complete"
    let updated = crate::db::jobs::get_job(&ship_dir, &job.id)
        .unwrap()
        .unwrap();
    assert_eq!(updated.status, "complete");
}

#[test]
fn record_gate_fail_creates_event_leaves_job_running() {
    let (_tmp, ship_dir) = setup();
    let job = crate::db::jobs::create_job(
        &ship_dir,
        "gate-test",
        None,
        None,
        None,
        None,
        0,
        None,
        vec![],
        vec![],
    )
    .unwrap();
    crate::db::jobs::update_job_status(&ship_dir, &job.id, "running").unwrap();

    let rec = record_gate_outcome(&ship_dir, &job.id, false, "3 tests failed").unwrap();
    assert_eq!(rec.entity, EventEntity::Gate);
    assert_eq!(rec.action, EventAction::Fail);
    assert_eq!(rec.details.as_deref(), Some("3 tests failed"));

    // Job should still be "running"
    let updated = crate::db::jobs::get_job(&ship_dir, &job.id)
        .unwrap()
        .unwrap();
    assert_eq!(updated.status, "running");
}

#[test]
fn list_gate_outcomes_filters_by_job() {
    let (_tmp, ship_dir) = setup();
    let job_a = crate::db::jobs::create_job(
        &ship_dir,
        "gate-a",
        None,
        None,
        None,
        None,
        0,
        None,
        vec![],
        vec![],
    )
    .unwrap();
    let job_b = crate::db::jobs::create_job(
        &ship_dir,
        "gate-b",
        None,
        None,
        None,
        None,
        0,
        None,
        vec![],
        vec![],
    )
    .unwrap();
    crate::db::jobs::update_job_status(&ship_dir, &job_a.id, "running").unwrap();
    crate::db::jobs::update_job_status(&ship_dir, &job_b.id, "running").unwrap();

    // Record outcomes for both jobs
    record_gate_outcome(&ship_dir, &job_a.id, false, "lint errors").unwrap();
    record_gate_outcome(&ship_dir, &job_a.id, true, "all clean").unwrap();
    record_gate_outcome(&ship_dir, &job_b.id, false, "build broken").unwrap();

    // Also insert an unrelated event to prove filtering works
    insert_event(
        &ship_dir,
        "ship",
        &EventEntity::Job,
        Some(&job_a.id),
        &EventAction::Log,
        Some("noise"),
        None,
        None,
        Some(&job_a.id),
    )
    .unwrap();

    let outcomes_a = list_gate_outcomes(&ship_dir, &job_a.id).unwrap();
    assert_eq!(outcomes_a.len(), 2);
    assert_eq!(outcomes_a[0].action, EventAction::Fail);
    assert_eq!(outcomes_a[1].action, EventAction::Pass);

    let outcomes_b = list_gate_outcomes(&ship_dir, &job_b.id).unwrap();
    assert_eq!(outcomes_b.len(), 1);
    assert_eq!(outcomes_b[0].action, EventAction::Fail);

    // Non-existent job returns empty
    let outcomes_none = list_gate_outcomes(&ship_dir, "no-such-job").unwrap();
    assert!(outcomes_none.is_empty());
}
