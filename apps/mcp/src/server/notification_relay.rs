use runtime::events::{EventEnvelope, Mailbox};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::push::PushAdapter;

/// A connected agent peer with its allowed event types.
pub struct PeerHandle {
    pub id: String,
    pub actor_id: String,
    pub adapter: Box<dyn PushAdapter>,
    /// Event types this agent is allowed to receive, derived from
    /// the agent's active skills' declared `in` events.
    /// Empty = system peer, receives all (e.g. internal projections).
    pub allowed_events: HashSet<String>,
}

/// Routes events from an actor's `Mailbox` to connected agent peers.
///
/// Filtering happens HERE, not at the receiver. Each peer only receives
/// events that match its agent's active skill declarations. This enforces
/// least privilege — an agent without `visual-brainstorm` never sees
/// `visual-brainstorm.annotation_created`.
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

    /// Spawn the relay loop consuming from a `Mailbox`.
    ///
    /// Returns `None` when the mailbox closes (all senders dropped).
    pub fn spawn(self, mut mailbox: Mailbox) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(env) = mailbox.recv().await {
                self.handle_event(&env).await;
            }
        })
    }

    async fn handle_event(&self, event: &EventEnvelope) {
        let peers = self.peers.read().await;
        for peer in peers.iter() {
            if self.peer_should_receive(peer, event) {
                peer.adapter.push_event(event).await;
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
/// `studio.` is included so Studio→agent events (visual messages, canvas
/// annotations) reach connected peers without skill declarations.
const SYSTEM_PREFIXES: &[&str] = &[
    "workspace.", "session.", "actor.", "config.",
    "gate.", "runtime.", "sync.", "project.",
    "studio.", "mesh.",
];

pub(crate) fn is_system_event(event_type: &str) -> bool {
    SYSTEM_PREFIXES.iter().any(|p| event_type.starts_with(p))
}

#[cfg(test)]
#[path = "notification_relay_tests.rs"]
mod tests;
