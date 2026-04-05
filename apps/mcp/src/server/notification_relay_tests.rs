use super::*;
use crate::push::PushAdapter;
use tokio::sync::Mutex;

type Log = Arc<Mutex<Vec<EventEnvelope>>>;

struct MockAdapter {
    events: Log,
}

impl MockAdapter {
    fn new() -> (Self, Log) {
        let log: Log = Arc::new(Mutex::new(Vec::new()));
        (Self { events: log.clone() }, log)
    }
}

#[async_trait::async_trait]
impl PushAdapter for MockAdapter {
    async fn push_event(&self, event: &EventEnvelope) {
        self.events.lock().await.push(event.clone());
    }

    fn adapter_name(&self) -> &'static str {
        "mock"
    }
}

fn envelope(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "e-1", &serde_json::json!({}))
        .unwrap()
        .with_context(Some("feature/auth"), Some("sess-1"))
}

fn system_peer(id: &str, adapter: MockAdapter) -> PeerHandle {
    PeerHandle {
        id: id.into(),
        actor_id: format!("actor-{id}"),
        adapter: Box::new(adapter),
        allowed_events: HashSet::new(),
    }
}

fn skill_peer(id: &str, adapter: MockAdapter, events: Vec<&str>) -> PeerHandle {
    PeerHandle {
        id: id.into(),
        actor_id: format!("actor-{id}"),
        adapter: Box::new(adapter),
        allowed_events: events.into_iter().map(String::from).collect(),
    }
}

/// Run a relay with a one-shot mailbox: send `events`, close the sender,
/// wait for the relay task to finish.
async fn run_relay(relay: EventRelay, events: Vec<EventEnvelope>) {
    let (tx, rx) = tokio::sync::mpsc::channel(64);
    let mailbox = runtime::events::Mailbox::from_receiver(rx);
    let handle = relay.spawn(mailbox);
    for e in events {
        tx.send(e).await.unwrap();
    }
    drop(tx);
    handle.await.unwrap();
}

// ── Delivery tests ───────────────────────────────────────────

#[tokio::test]
async fn system_peer_receives_all_events() {
    let (adapter, log) = MockAdapter::new();
    let relay = EventRelay::new();
    relay.add_peer(system_peer("p1", adapter)).await;
    run_relay(
        relay,
        vec![
            envelope("session.started"),
            envelope("visual-brainstorm.annotation_created"),
        ],
    )
    .await;
    assert_eq!(log.lock().await.len(), 2);
}

#[tokio::test]
async fn skill_peer_receives_matching_skill_events() {
    let (adapter, log) = MockAdapter::new();
    let relay = EventRelay::new();
    relay
        .add_peer(skill_peer(
            "p1",
            adapter,
            vec!["visual-brainstorm.annotation_created"],
        ))
        .await;
    run_relay(
        relay,
        vec![envelope("visual-brainstorm.annotation_created")],
    )
    .await;
    assert_eq!(log.lock().await.len(), 1);
}

#[tokio::test]
async fn skill_peer_blocked_from_unmatched_skill_events() {
    let (adapter, log) = MockAdapter::new();
    let relay = EventRelay::new();
    relay
        .add_peer(skill_peer("p1", adapter, vec!["mcp-setup.server_ready"]))
        .await;
    run_relay(
        relay,
        vec![envelope("visual-brainstorm.annotation_created")],
    )
    .await;
    assert!(log.lock().await.is_empty());
}

#[tokio::test]
async fn skill_peer_still_receives_system_events() {
    let (adapter, log) = MockAdapter::new();
    let relay = EventRelay::new();
    relay
        .add_peer(skill_peer("p1", adapter, vec!["mcp-setup.server_ready"]))
        .await;
    run_relay(relay, vec![envelope("session.started")]).await;
    assert_eq!(log.lock().await.len(), 1);
}

#[tokio::test]
async fn multiple_peers_filtered_independently() {
    let (a, la) = MockAdapter::new();
    let (b, lb) = MockAdapter::new();
    let relay = EventRelay::new();
    relay
        .add_peer(skill_peer(
            "a",
            a,
            vec!["visual-brainstorm.annotation_created"],
        ))
        .await;
    relay
        .add_peer(skill_peer("b", b, vec!["mcp-setup.server_ready"]))
        .await;
    run_relay(
        relay,
        vec![envelope("visual-brainstorm.annotation_created")],
    )
    .await;
    assert_eq!(la.lock().await.len(), 1);
    assert!(lb.lock().await.is_empty());
}

#[tokio::test]
async fn event_payload_delivered_intact() {
    let (adapter, log) = MockAdapter::new();
    let relay = EventRelay::new();
    relay.add_peer(system_peer("p1", adapter)).await;
    let evt = envelope("session.started");
    let evt_id = evt.id.clone();
    run_relay(relay, vec![evt]).await;
    let received = log.lock().await;
    assert_eq!(received[0].id, evt_id);
    assert_eq!(received[0].event_type, "session.started");
}

// ── Peer lifecycle tests ─────────────────────────────────────

#[tokio::test]
async fn remove_peer_stops_delivery() {
    let (a, la) = MockAdapter::new();
    let (b, lb) = MockAdapter::new();
    let relay = EventRelay::new();
    relay.add_peer(system_peer("a", a)).await;
    relay.add_peer(system_peer("b", b)).await;
    relay.remove_peer("a").await;
    run_relay(relay, vec![envelope("session.started")]).await;
    assert!(la.lock().await.is_empty());
    assert_eq!(lb.lock().await.len(), 1);
}

// ── Channel lifecycle tests ──────────────────────────────────

#[tokio::test]
async fn relay_stops_on_mailbox_close() {
    use tokio::sync::mpsc;
    let (tx, rx) = mpsc::channel::<EventEnvelope>(16);
    let mailbox = runtime::events::Mailbox::from_receiver(rx);
    let handle = EventRelay::new().spawn(mailbox);
    drop(tx);
    tokio::time::timeout(std::time::Duration::from_secs(1), handle)
        .await
        .expect("relay should stop when mailbox closes")
        .unwrap();
}

// ── System event classification ──────────────────────────────

#[tokio::test]
async fn system_event_detection() {
    assert!(is_system_event("workspace.created"));
    assert!(is_system_event("session.started"));
    assert!(is_system_event("actor.woke"));
    assert!(is_system_event("config.changed"));
    assert!(is_system_event("gate.passed"));
    assert!(is_system_event("studio.message.visual"));
    assert!(is_system_event("studio.canvas.annotation"));
    assert!(!is_system_event("visual-brainstorm.annotation_created"));
    assert!(!is_system_event("mcp-setup.server_ready"));
}

#[tokio::test]
async fn studio_events_delivered_to_skill_peers_without_declaration() {
    let (adapter, log) = MockAdapter::new();
    let relay = EventRelay::new();
    relay
        .add_peer(skill_peer("p1", adapter, vec!["some-skill.event"]))
        .await;
    run_relay(
        relay,
        vec![envelope("studio.message.visual")],
    )
    .await;
    assert_eq!(log.lock().await.len(), 1);
}
