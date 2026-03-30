use async_trait::async_trait;
use runtime::events::EventEnvelope;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::warn;

/// Abstraction over the MCP peer to enable testing.
/// Sends `ship/event` custom notifications with the full event payload.
#[async_trait]
pub trait EventSink: Send + Sync + 'static {
    async fn send_event(&self, event: &EventEnvelope);
}

/// A connected agent peer with its allowed event types.
pub struct PeerHandle {
    pub id: String,
    pub actor_id: String,
    pub sink: Box<dyn EventSink>,
    /// Event types this agent is allowed to receive, derived from
    /// the agent's active skills' declared `in` events.
    /// Empty = system peer, receives all (e.g. internal projections).
    pub allowed_events: HashSet<String>,
}

/// Routes events from a workspace broadcast channel to connected agent peers.
///
/// Filtering happens HERE, not at the receiver. Each peer only receives
/// events that match its agent's active skill declarations. This enforces
/// least privilege — an agent without `visual-brainstorm` never sees
/// `visual-brainstorm.annotation_created`.
///
/// Events are sent as `ship/event` custom MCP notifications with the full
/// EventEnvelope payload, not as `notify_resource_updated` nudges.
pub struct EventRelay {
    peers: Arc<RwLock<Vec<PeerHandle>>>,
}

impl EventRelay {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn peers(&self) -> Arc<RwLock<Vec<PeerHandle>>> {
        self.peers.clone()
    }

    pub async fn add_peer(&self, handle: PeerHandle) {
        self.peers.write().await.push(handle);
    }

    pub async fn remove_peer(&self, peer_id: &str) {
        self.peers.write().await.retain(|p| p.id != peer_id);
    }

    pub fn spawn(self, mut rx: broadcast::Receiver<EventEnvelope>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(env) => self.handle_event(&env).await,
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("event relay lagged, skipped {n} events");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        })
    }

    async fn handle_event(&self, event: &EventEnvelope) {
        let peers = self.peers.read().await;
        for peer in peers.iter() {
            if self.peer_should_receive(peer, event) {
                peer.sink.send_event(event).await;
            }
        }
    }

    /// Filter at the relay, not at the receiver.
    ///
    /// System events (workspace.*, session.*, actor.*, config.*, gate.*)
    /// are delivered to all peers — they're runtime lifecycle, not skill-scoped.
    ///
    /// Skill-namespaced events (e.g. visual-brainstorm.annotation_created)
    /// are only delivered if the peer's allowed_events contains that type.
    ///
    /// If allowed_events is empty, the peer is a system peer (receives all).
    fn peer_should_receive(&self, peer: &PeerHandle, event: &EventEnvelope) -> bool {
        // System peers receive everything
        if peer.allowed_events.is_empty() {
            return true;
        }

        // System namespace events go to all peers
        if is_system_event(&event.event_type) {
            return true;
        }

        // Skill-namespaced events: check if peer's skills declared this as `in`
        peer.allowed_events.contains(&event.event_type)
    }
}

/// System namespaces that all agents receive regardless of skill set.
const SYSTEM_PREFIXES: &[&str] = &[
    "workspace.", "session.", "actor.", "config.",
    "gate.", "runtime.", "sync.", "project.",
];

fn is_system_event(event_type: &str) -> bool {
    SYSTEM_PREFIXES.iter().any(|p| event_type.starts_with(p))
}

#[cfg(test)]
mod tests {
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

    async fn run_relay(relay: EventRelay, events: Vec<EventEnvelope>) {
        let (tx, rx) = broadcast::channel(16);
        let handle = relay.spawn(rx);
        for e in events {
            tx.send(e).unwrap();
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
    async fn relay_handles_lagged() {
        let (tx, rx) = broadcast::channel(2);
        let (sink, log) = MockSink::new();
        let relay = EventRelay::new();
        for _ in 0..4 {
            let _ = tx.send(envelope("session.started"));
        }
        relay.add_peer(system_peer("p", sink)).await;
        let handle = relay.spawn(rx);
        tx.send(envelope("session.started")).unwrap();
        drop(tx);
        handle.await.unwrap();
        assert!(!log.lock().await.is_empty());
    }

    #[tokio::test]
    async fn relay_stops_on_channel_close() {
        let (tx, rx) = broadcast::channel::<EventEnvelope>(16);
        let handle = EventRelay::new().spawn(rx);
        drop(tx);
        tokio::time::timeout(std::time::Duration::from_secs(1), handle)
            .await
            .expect("relay should stop")
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
        assert!(!is_system_event("visual-brainstorm.annotation_created"));
        assert!(!is_system_event("mcp-setup.server_ready"));
    }
}
