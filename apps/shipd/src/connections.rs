//! Per-connection lifecycle: event relay, cleanup guard, and mesh service spawner.

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use rmcp::model::CustomNotification;
use rmcp::{Peer, RoleServer, model::ServerNotification};
use runtime::events::{EventEnvelope, Mailbox};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

// ---- Inbox ----

/// Shared inbox buffer for polling-based message retrieval.
/// The EventRelay writes here; `mesh_inbox` drains it.
pub struct Inbox {
    buf: RwLock<VecDeque<EventEnvelope>>,
}

impl Inbox {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { buf: RwLock::new(VecDeque::new()) })
    }

    pub async fn push(&self, event: EventEnvelope) {
        let mut buf = self.buf.write().await;
        buf.push_back(event);
        // Cap at 256 to prevent unbounded growth
        while buf.len() > 256 {
            buf.pop_front();
        }
    }

    /// Drain all pending messages.
    pub async fn drain(&self) -> Vec<EventEnvelope> {
        self.buf.write().await.drain(..).collect()
    }
}

// ---- EventSink ----

#[async_trait]
pub trait EventSink: Send + Sync + 'static {
    async fn send_event(&self, event: &EventEnvelope);
}

pub struct McpEventSink {
    peer: Peer<RoleServer>,
}

impl McpEventSink {
    pub fn new(peer: Peer<RoleServer>) -> Self {
        Self { peer }
    }
}

#[async_trait]
impl EventSink for McpEventSink {
    async fn send_event(&self, event: &EventEnvelope) {
        let params = match serde_json::to_value(event) {
            Ok(v) => v,
            Err(e) => {
                warn!("failed to serialize event for MCP notification: {e}");
                return;
            }
        };
        let notification = CustomNotification::new("ship/event", Some(params));
        let server_notif = ServerNotification::CustomNotification(notification);
        if let Err(e) = self.peer.send_notification(server_notif).await {
            warn!("failed to send ship/event notification: {e}");
        }
    }
}

// ---- EventRelay ----

pub struct PeerHandle {
    pub sink: Box<dyn EventSink>,
}

/// Routes events from an actor mailbox to the connected MCP peer and inbox buffer.
pub struct EventRelay {
    peers: Arc<RwLock<Vec<PeerHandle>>>,
    inbox: Option<Arc<Inbox>>,
}

impl EventRelay {
    pub fn new() -> Self {
        Self { peers: Arc::new(RwLock::new(Vec::new())), inbox: None }
    }

    /// Attach an inbox so messages are buffered for polling via `mesh_inbox`.
    pub fn with_inbox(mut self, inbox: Arc<Inbox>) -> Self {
        self.inbox = Some(inbox);
        self
    }

    pub async fn add_peer(&self, handle: PeerHandle) {
        self.peers.write().await.push(handle);
    }

    /// Consume the mailbox in a background task, forwarding all events to peers.
    /// The task exits when the mailbox closes (actor stopped or sender dropped).
    pub fn spawn(self, mut mailbox: Mailbox) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(env) = mailbox.recv().await {
                // Buffer for polling
                if let Some(ref inbox) = self.inbox {
                    inbox.push(env.clone()).await;
                }
                // Push to SSE peers (best-effort)
                let peers = self.peers.read().await;
                for peer in peers.iter() {
                    peer.sink.send_event(&env).await;
                }
            }
        })
    }
}

// ---- ConnectionGuard ----

/// Cleanup guard held (via Arc) by every clone of a NetworkServer connection.
///
/// When the last clone is dropped (i.e. the HTTP session ends), Drop emits
/// `mesh.deregister` and removes the actor from the kernel router.
pub struct ConnectionGuard {
    pub actor_id: std::sync::Mutex<Option<String>>,
    pub relay_handle: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
    pub kernel: Arc<tokio::sync::Mutex<runtime::events::KernelRouter>>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        // Abort relay task
        if let Ok(mut h) = self.relay_handle.lock() {
            if let Some(handle) = h.take() {
                handle.abort();
            }
        }
        // Emit deregister + stop actor (requires tokio runtime)
        let id = match self.actor_id.lock().ok().and_then(|mut g| g.take()) {
            Some(id) => id,
            None => return,
        };
        let kernel = self.kernel.clone();
        let id_clone = id.clone();
        let Ok(rt) = tokio::runtime::Handle::try_current() else { return };
        rt.spawn(async move {
            if let Ok(env) = EventEnvelope::new(
                "mesh.deregister",
                &id,
                &serde_json::json!({ "agent_id": &id }),
            )
            .map(|e| e.with_actor_id(&id))
            {
                let ctx = runtime::events::EmitContext {
                    caller_kind: runtime::events::CallerKind::Mcp,
                    skill_id: None,
                    workspace_id: None,
                    session_id: None,
                };
                let _ = kernel.lock().await.route(env, &ctx).await;
            }
            let _ = kernel.lock().await.stop_actor(&id_clone);
        });
    }
}

// ---- MeshService spawner ----

static MESH_SPAWNED: std::sync::OnceLock<()> = std::sync::OnceLock::new();

/// Spawn the MeshService into the shared kernel once for the daemon lifetime.
/// Returns the shared registry for the REST API to read.
/// Idempotent — safe to call multiple times (returns new empty registry on re-call).
pub async fn spawn_mesh_service(
    kernel: &Arc<tokio::sync::Mutex<runtime::events::KernelRouter>>,
) -> Result<runtime::services::mesh::SharedMeshRegistry> {
    let registry: runtime::services::mesh::SharedMeshRegistry =
        Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));

    if MESH_SPAWNED.set(()).is_err() {
        return Ok(registry);
    }

    let (outbox_tx, mut outbox_rx) =
        tokio::sync::mpsc::unbounded_channel::<EventEnvelope>();

    let mesh_config = runtime::events::ActorConfig {
        namespace: "service.mesh".to_string(),
        write_namespaces: vec!["mesh.".to_string()],
        read_namespaces: vec!["mesh.".to_string()],
        subscribe_namespaces: vec!["mesh.".to_string()],
    };
    let handler: Box<dyn runtime::services::ServiceHandler> = Box::new(
        runtime::services::mesh::MeshService::with_shared_registry(
            outbox_tx,
            registry.clone(),
        ),
    );

    runtime::services::spawn_service(
        &mut *kernel.lock().await,
        "service.mesh",
        mesh_config,
        handler,
    )
    .map_err(|e| anyhow!("failed to spawn MeshService: {e}"))?;

    // Drain outbox → kernel (directed delivery to agent mailboxes)
    let kr = kernel.clone();
    tokio::spawn(async move {
        let ctx = runtime::events::EmitContext {
            caller_kind: runtime::events::CallerKind::Mcp,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        };
        while let Some(event) = outbox_rx.recv().await {
            if let Err(e) = kr.lock().await.route(event, &ctx).await {
                tracing::warn!("mesh outbox routing error: {e}");
            }
        }
    });

    Ok(registry)
}
