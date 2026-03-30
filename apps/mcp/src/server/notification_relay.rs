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
#[path = "notification_relay_tests.rs"]
mod tests;
