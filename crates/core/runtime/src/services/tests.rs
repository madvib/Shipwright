//! Tests for the service actor infrastructure and sync service.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use tempfile::tempdir;
use tokio::sync::mpsc;

use crate::events::actor_store::init_actor_db;
use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::{ActorStore, EventEnvelope, Mailbox};
use crate::services::{ServiceHandle, ServiceHandler, run_service};
use crate::services::sync::{SyncConfig, SyncServiceHandler};

// ── Fixtures ──────────────────────────────────────────────────────────────────

fn ev(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
}

fn sync_config() -> ActorConfig {
    ActorConfig {
        namespace: "sync".into(),
        write_namespaces: vec!["sync.".into()],
        read_namespaces: vec!["sync.".into()],
        subscribe_namespaces: vec!["sync.".into(), "workspace.".into()],
    }
}

fn setup_router() -> (tempfile::TempDir, KernelRouter) {
    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(tmp.path().join(".ship")).unwrap();
    (tmp, router)
}

/// A minimal service handler that records lifecycle calls.
struct RecordingHandler {
    name: String,
    calls: Arc<Mutex<Vec<String>>>,
}

impl RecordingHandler {
    fn new(name: &str) -> (Self, Arc<Mutex<Vec<String>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let handler = Self {
            name: name.to_string(),
            calls: calls.clone(),
        };
        (handler, calls)
    }
}

impl ServiceHandler for RecordingHandler {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle(&mut self, event: &EventEnvelope, _store: &ActorStore) -> Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("handle:{}", event.event_type));
        Ok(())
    }

    fn on_start(&mut self, _store: &ActorStore) -> Result<()> {
        self.calls.lock().unwrap().push("on_start".into());
        Ok(())
    }

    fn on_stop(&mut self, _store: &ActorStore) -> Result<()> {
        self.calls.lock().unwrap().push("on_stop".into());
        Ok(())
    }
}

// ── Service lifecycle ─────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn service_on_start_and_on_stop_called() {
    let (_tmp, mut router) = setup_router();
    let (store, mailbox) = router.spawn_actor("svc", sync_config()).unwrap();

    let (handler, calls) = RecordingHandler::new("test");
    let task = tokio::spawn(run_service(Box::new(handler), store, mailbox));

    // Dropping the mailbox sender (via stop_actor) closes the channel.
    router.stop_actor("svc").unwrap();
    task.await.unwrap();

    let calls = calls.lock().unwrap();
    assert_eq!(calls[0], "on_start");
    assert_eq!(calls[calls.len() - 1], "on_stop");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn service_handle_called_for_each_mailbox_event() {
    let (tx, rx) = mpsc::channel(10);
    let mailbox = Mailbox::new_for_test(rx);

    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("events.db");
    init_actor_db(&db_path).unwrap();
    let store = ActorStore::new("test-svc", db_path, vec!["sync.".into()], vec![]);

    let (handler, calls) = RecordingHandler::new("test");
    let task = tokio::spawn(run_service(Box::new(handler), store, mailbox));

    tx.send(ev("sync.trigger.push")).await.unwrap();
    tx.send(ev("sync.pull.completed")).await.unwrap();
    drop(tx);
    task.await.unwrap();

    let calls = calls.lock().unwrap();
    let handle_calls: Vec<_> = calls.iter().filter(|c| c.starts_with("handle:")).collect();
    assert_eq!(handle_calls.len(), 2);
    assert_eq!(handle_calls[0], "handle:sync.trigger.push");
    assert_eq!(handle_calls[1], "handle:sync.pull.completed");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn service_exits_cleanly_when_mailbox_closes() {
    let (tx, rx) = mpsc::channel(4);
    let mailbox = Mailbox::new_for_test(rx);

    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("events.db");
    init_actor_db(&db_path).unwrap();
    let store = ActorStore::new("test-svc", db_path, vec!["sync.".into()], vec![]);

    let (handler, _calls) = RecordingHandler::new("test");
    let task = tokio::spawn(run_service(Box::new(handler), store, mailbox));

    drop(tx); // close channel immediately
    // Task must complete without hanging.
    tokio::time::timeout(Duration::from_secs(2), task)
        .await
        .expect("service did not exit within timeout")
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_service_wires_actor_and_task() {
    let (_tmp, mut router) = setup_router();

    let (handler, calls) = RecordingHandler::new("test");
    let ServiceHandle { name, handle } =
        router.spawn_service("sync", sync_config(), Box::new(handler)).unwrap();

    assert_eq!(name, "test");
    assert_eq!(router.actor_count(), 1);

    router.stop_actor("sync").unwrap();
    handle.await.unwrap();

    let calls = calls.lock().unwrap();
    assert!(calls.contains(&"on_start".to_string()));
    assert!(calls.contains(&"on_stop".to_string()));
}

// ── SyncConfig ────────────────────────────────────────────────────────────────

#[test]
fn sync_config_defaults() {
    let cfg = SyncConfig::default();
    assert_eq!(cfg.push_interval_secs, 30);
    assert_eq!(cfg.pull_interval_secs, 60);
    assert_eq!(cfg.push_threshold, 50);
    assert_eq!(cfg.endpoint, "https://api.getship.dev");
}

#[test]
fn sync_handler_tick_interval_from_config() {
    let cfg = SyncConfig {
        push_interval_secs: 45,
        ..SyncConfig::default()
    };
    let handler = SyncServiceHandler::new(cfg);
    assert_eq!(
        handler.tick_interval(),
        Some(Duration::from_secs(45))
    );
}

#[test]
fn sync_handler_pull_every_n_ticks_divides_intervals() {
    // pull=60, push=30 → every 2 ticks
    let h = SyncServiceHandler::new(SyncConfig {
        push_interval_secs: 30,
        pull_interval_secs: 60,
        ..SyncConfig::default()
    });
    assert_eq!(h.pull_every_n_ticks, 2);

    // pull=90, push=30 → every 3 ticks
    let h = SyncServiceHandler::new(SyncConfig {
        push_interval_secs: 30,
        pull_interval_secs: 90,
        ..SyncConfig::default()
    });
    assert_eq!(h.pull_every_n_ticks, 3);

    // pull < push → clamped to 1
    let h = SyncServiceHandler::new(SyncConfig {
        push_interval_secs: 60,
        pull_interval_secs: 10,
        ..SyncConfig::default()
    });
    assert_eq!(h.pull_every_n_ticks, 1);
}
