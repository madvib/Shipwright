use async_trait::async_trait;
use runtime::events::EventEnvelope;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::warn;

/// Abstraction over the MCP peer to enable testing.
#[async_trait]
pub trait NotificationSink: Send + Sync + 'static {
    async fn notify_resource_updated(&self, uri: String);
}

/// URI template with variable substitution from event envelope.
#[derive(Debug, Clone)]
pub enum UriTemplate {
    /// Static URI — always the same.
    Static(String),
    /// Dynamic URI — substitutes `{workspace_id}`, `{session_id}`,
    /// `{entity_id}`, `{event_type}` from the envelope.
    Dynamic(String),
}

impl UriTemplate {
    fn resolve(&self, env: &EventEnvelope) -> String {
        match self {
            Self::Static(uri) => uri.clone(),
            Self::Dynamic(tpl) => {
                let mut out = tpl.clone();
                out = out.replace("{workspace_id}", env.workspace_id.as_deref().unwrap_or(""));
                out = out.replace("{session_id}", env.session_id.as_deref().unwrap_or(""));
                out = out.replace("{entity_id}", &env.entity_id);
                out = out.replace("{event_type}", &env.event_type);
                out
            }
        }
    }
}

/// Handle to a connected peer.
pub struct PeerHandle {
    pub id: String,
    pub sink: Box<dyn NotificationSink>,
    /// Only notify for events matching these URIs. Empty = notify all.
    pub subscribed_uris: HashSet<String>,
}

/// Maps event types to MCP resource URIs for notification.
pub struct McpNotificationRelay {
    event_uri_map: HashMap<String, UriTemplate>,
    wildcard: Option<UriTemplate>,
    peers: Arc<RwLock<Vec<PeerHandle>>>,
}

impl McpNotificationRelay {
    pub fn new() -> Self {
        Self {
            event_uri_map: HashMap::new(),
            wildcard: None,
            peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_mapping(mut self, event_type: &str, template: UriTemplate) -> Self {
        self.event_uri_map.insert(event_type.to_string(), template);
        self
    }

    pub fn with_wildcard_mapping(mut self, _pattern: &str, template: UriTemplate) -> Self {
        self.wildcard = Some(template);
        self
    }

    pub fn with_default_mappings(self) -> Self {
        self.with_mapping(
            "session.started",
            UriTemplate::Dynamic("ship://session/{workspace_id}".into()),
        )
        .with_mapping(
            "session.ended",
            UriTemplate::Dynamic("ship://session/{workspace_id}".into()),
        )
        .with_mapping(
            "session.progress",
            UriTemplate::Dynamic("ship://session/{workspace_id}/progress".into()),
        )
        .with_wildcard_mapping(
            "*.*",
            UriTemplate::Dynamic("ship://workspace/{workspace_id}/events".into()),
        )
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
                        warn!("notification relay lagged, skipped {n} events");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        })
    }

    async fn handle_event(&self, env: &EventEnvelope) {
        let template = self
            .event_uri_map
            .get(&env.event_type)
            .or(self.wildcard.as_ref());
        let Some(template) = template else { return };
        let uri = template.resolve(env);

        let peers = self.peers.read().await;
        for peer in peers.iter() {
            if peer.subscribed_uris.is_empty() || peer.subscribed_uris.contains(&uri) {
                peer.sink.notify_resource_updated(uri.clone()).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    type Log = Arc<Mutex<Vec<String>>>;

    struct MockSink { notifications: Log }

    impl MockSink {
        fn new() -> (Self, Log) {
            let n: Log = Arc::new(Mutex::new(Vec::new()));
            (Self { notifications: n.clone() }, n)
        }
    }

    #[async_trait]
    impl NotificationSink for MockSink {
        async fn notify_resource_updated(&self, uri: String) {
            self.notifications.lock().await.push(uri);
        }
    }

    fn envelope(event_type: &str) -> EventEnvelope {
        EventEnvelope::new(event_type, "e-1", &serde_json::json!({}))
            .unwrap()
            .with_context(Some("feature/auth"), Some("sess-1"))
    }

    fn peer(id: &str, sink: MockSink, uris: HashSet<String>) -> PeerHandle {
        PeerHandle { id: id.into(), sink: Box::new(sink), subscribed_uris: uris }
    }

    async fn run_relay(
        relay: McpNotificationRelay,
        events: Vec<EventEnvelope>,
    ) {
        let (tx, rx) = broadcast::channel(16);
        let handle = relay.spawn(rx);
        for e in events { tx.send(e).unwrap(); }
        drop(tx);
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_relay_notifies_on_matching_event() {
        let (sink, log) = MockSink::new();
        let relay = McpNotificationRelay::new()
            .with_mapping("session.started", UriTemplate::Static("ship://s".into()));
        relay.add_peer(peer("p1", sink, HashSet::new())).await;
        run_relay(relay, vec![envelope("session.started")]).await;
        assert_eq!(log.lock().await.as_slice(), &["ship://s"]);
    }

    #[tokio::test]
    async fn test_relay_ignores_non_matching_event() {
        let (sink, log) = MockSink::new();
        let relay = McpNotificationRelay::new()
            .with_mapping("session.started", UriTemplate::Static("ship://s".into()));
        relay.add_peer(peer("p1", sink, HashSet::new())).await;
        run_relay(relay, vec![envelope("actor.created")]).await;
        assert!(log.lock().await.is_empty());
    }

    #[tokio::test]
    async fn test_relay_notifies_multiple_peers() {
        let relay = McpNotificationRelay::new()
            .with_mapping("session.started", UriTemplate::Static("ship://s".into()));
        let mut logs = Vec::new();
        for i in 0..3 {
            let (sink, log) = MockSink::new();
            relay.add_peer(peer(&format!("p{i}"), sink, HashSet::new())).await;
            logs.push(log);
        }
        run_relay(relay, vec![envelope("session.started")]).await;
        for log in &logs { assert_eq!(log.lock().await.len(), 1); }
    }

    #[tokio::test]
    async fn test_relay_respects_peer_uri_subscription() {
        let (sa, la) = MockSink::new();
        let (sb, lb) = MockSink::new();
        let relay = McpNotificationRelay::new()
            .with_mapping("session.started", UriTemplate::Static("ship://s".into()));
        relay.add_peer(peer("a", sa, HashSet::from(["ship://s".into()]))).await;
        relay.add_peer(peer("b", sb, HashSet::from(["ship://other".into()]))).await;
        run_relay(relay, vec![envelope("session.started")]).await;
        assert_eq!(la.lock().await.len(), 1);
        assert!(lb.lock().await.is_empty());
    }

    #[tokio::test]
    async fn test_uri_template_substitution() {
        let tpl = UriTemplate::Dynamic("ship://session/{workspace_id}/progress".into());
        assert_eq!(tpl.resolve(&envelope("session.progress")), "ship://session/feature/auth/progress");
    }

    #[tokio::test]
    async fn test_relay_handles_peer_disconnect() {
        let (sa, la) = MockSink::new();
        let (sb, lb) = MockSink::new();
        let relay = McpNotificationRelay::new()
            .with_mapping("session.started", UriTemplate::Static("ship://s".into()));
        relay.add_peer(peer("a", sa, HashSet::new())).await;
        relay.add_peer(peer("b", sb, HashSet::new())).await;
        relay.remove_peer("a").await;
        run_relay(relay, vec![envelope("session.started")]).await;
        assert!(la.lock().await.is_empty());
        assert_eq!(lb.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn test_relay_handles_lagged() {
        let (tx, rx) = broadcast::channel(2);
        let (sink, log) = MockSink::new();
        let relay = McpNotificationRelay::new()
            .with_mapping("x.y", UriTemplate::Static("ship://x".into()));
        for _ in 0..4 { let _ = tx.send(envelope("x.y")); }
        relay.add_peer(peer("p", sink, HashSet::new())).await;
        let handle = relay.spawn(rx);
        tx.send(envelope("x.y")).unwrap();
        drop(tx);
        handle.await.unwrap();
        assert!(!log.lock().await.is_empty());
    }

    #[tokio::test]
    async fn test_relay_stops_on_channel_close() {
        let (tx, rx) = broadcast::channel::<EventEnvelope>(16);
        let handle = McpNotificationRelay::new().spawn(rx);
        drop(tx);
        tokio::time::timeout(std::time::Duration::from_secs(1), handle)
            .await.expect("relay should stop").unwrap();
    }

    #[tokio::test]
    async fn test_wildcard_mapping() {
        let (sink, log) = MockSink::new();
        let relay = McpNotificationRelay::new().with_wildcard_mapping(
            "*.*", UriTemplate::Dynamic("ship://workspace/{workspace_id}/events".into()),
        );
        relay.add_peer(peer("p", sink, HashSet::new())).await;
        run_relay(relay, vec![envelope("skill.executed")]).await;
        assert_eq!(log.lock().await.as_slice(), &["ship://workspace/feature/auth/events"]);
    }

    #[tokio::test]
    async fn test_static_uri_template() {
        let tpl = UriTemplate::Static("ship://static/resource".into());
        assert_eq!(tpl.resolve(&envelope("any.event")), "ship://static/resource");
    }
}
