use super::*;
use tokio::sync::Mutex;

type Log = Arc<Mutex<Vec<EventEnvelope>>>;

struct MockSink {
    events: Log,
}

impl MockSink {
    fn new() -> (Self, Log) {
        let log: Log = Arc::new(Mutex::new(Vec::new()));
        (Self { events: log.clone() }, log)
    }
}

#[async_trait]
impl EventSink for MockSink {
    async fn send_event(&self, event: &EventEnvelope) {
        self.events.lock().await.push(event.clone());
    }
}

fn envelope(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "e-1", &serde_json::json!({}))
        .unwrap()
        .with_context(Some("feature/auth"), Some("sess-1"))
}

fn system_peer(id: &str, sink: MockSink) -> PeerHandle {
    PeerHandle {
        id: id.into(),
        actor_id: format!("actor-{id}"),
        sink: Box::new(sink),
        allowed_events: HashSet::new(),
    }
}

fn skill_peer(id: &str, sink: MockSink, events: Vec<&str>) -> PeerHandle {
    PeerHandle {
        id: id.into(),
        actor_id: format!("actor-{id}"),
        sink: Box::new(sink),
        allowed_events: events.into_iter().map(String::from).collect(),
    }
}

/// Run a relay with a one-shot mailbox: send `events`, close the sender,
/// wait for the relay task to finish.
async fn run_relay(relay: EventRelay, events: Vec<EventEnvelope>) {
    let (tx, rx) = tokio::sync::mpsc::channel(64);
    let mailbox = runtime::events::Mailbox::new_for_test(rx);
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
    let (sink, log) = MockSink::new();
    let relay = EventRelay::new();
    relay.add_peer(system_peer("p1", sink)).await;
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
    let (sink, log) = MockSink::new();
    let relay = EventRelay::new();
    relay
        .add_peer(skill_peer(
            "p1",
            sink,
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
    let (sink, log) = MockSink::new();
    let relay = EventRelay::new();
    relay
        .add_peer(skill_peer("p1", sink, vec!["mcp-setup.server_ready"]))
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
    let (sink, log) = MockSink::new();
    let relay = EventRelay::new();
    relay
        .add_peer(skill_peer("p1", sink, vec!["mcp-setup.server_ready"]))
        .await;
    run_relay(relay, vec![envelope("session.started")]).await;
    assert_eq!(log.lock().await.len(), 1);
}

#[tokio::test]
async fn multiple_peers_filtered_independently() {
    let (sa, la) = MockSink::new();
    let (sb, lb) = MockSink::new();
    let relay = EventRelay::new();
    relay
        .add_peer(skill_peer(
            "a",
            sa,
            vec!["visual-brainstorm.annotation_created"],
        ))
        .await;
    relay
        .add_peer(skill_peer("b", sb, vec!["mcp-setup.server_ready"]))
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
    let (sink, log) = MockSink::new();
    let relay = EventRelay::new();
    relay.add_peer(system_peer("p1", sink)).await;
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
    let (sa, la) = MockSink::new();
    let (sb, lb) = MockSink::new();
    let relay = EventRelay::new();
    relay.add_peer(system_peer("a", sa)).await;
    relay.add_peer(system_peer("b", sb)).await;
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
    let mailbox = runtime::events::Mailbox::new_for_test(rx);
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
    let (sink, log) = MockSink::new();
    let relay = EventRelay::new();
    relay
        .add_peer(skill_peer("p1", sink, vec!["some-skill.event"]))
        .await;
    run_relay(
        relay,
        vec![envelope("studio.message.visual")],
    )
    .await;
    assert_eq!(log.lock().await.len(), 1);
}
