//! Async event router — validates, persists, and broadcasts events.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{broadcast, RwLock};

use crate::events::EventEnvelope;
use crate::events::store::EventStore;
use crate::events::validator::{EmitContext, EventValidator};

const DEFAULT_CAPACITY: usize = 256;

/// Central event router. Replaces the synchronous `OnceLock<EventBus>` statics
/// with async broadcast channels scoped to platform and per-workspace.
pub struct EventRouter {
    /// Platform-scope bus — always active, receives elevated events.
    platform_tx: broadcast::Sender<EventEnvelope>,

    /// Per-workspace buses — lazy, created on first subscriber.
    workspace_txs: RwLock<HashMap<String, broadcast::Sender<EventEnvelope>>>,

    /// Append-only event persistence.
    store: Arc<dyn EventStore>,

    /// Ingress validators — all must pass before persistence.
    validators: Vec<Box<dyn EventValidator>>,

    /// Broadcast channel capacity.
    channel_capacity: usize,
}

impl EventRouter {
    pub fn new(store: Arc<dyn EventStore>, channel_capacity: usize) -> Self {
        let (platform_tx, _) = broadcast::channel(channel_capacity);
        Self {
            platform_tx,
            workspace_txs: RwLock::new(HashMap::new()),
            store,
            validators: Vec::new(),
            channel_capacity,
        }
    }

    /// Builder: add a validator to the ingress pipeline.
    pub fn with_validator(mut self, validator: Box<dyn EventValidator>) -> Self {
        self.validators.push(validator);
        self
    }

    /// Validate → persist → broadcast. Rejects if any validator fails.
    pub async fn emit(&self, event: EventEnvelope, ctx: &EmitContext) -> Result<()> {
        // 1. Run all validators
        for v in &self.validators {
            v.validate(&event, ctx)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }

        // 2. Persist (source of truth)
        self.store.append(&event)?;

        // 3. Broadcast to platform bus if elevated
        if event.elevated {
            let _ = self.platform_tx.send(event.clone());
        }

        // 4. Broadcast to workspace bus if scoped
        if let Some(ref ws_id) = event.workspace_id {
            let txs = self.workspace_txs.read().await;
            if let Some(tx) = txs.get(ws_id) {
                let _ = tx.send(event);
            }
        }

        Ok(())
    }

    /// Subscribe to the platform-scope broadcast.
    pub fn subscribe_platform(&self) -> broadcast::Receiver<EventEnvelope> {
        self.platform_tx.subscribe()
    }

    /// Subscribe to a workspace-scoped broadcast. Creates the channel lazily.
    pub async fn subscribe_workspace(
        &self,
        ws_id: &str,
    ) -> broadcast::Receiver<EventEnvelope> {
        let mut txs = self.workspace_txs.write().await;
        let tx = txs.entry(ws_id.to_string()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(self.channel_capacity);
            tx
        });
        tx.subscribe()
    }

    /// Drop workspace channels with zero active receivers.
    pub async fn reap_idle_channels(&self) {
        let mut txs = self.workspace_txs.write().await;
        txs.retain(|_, tx| tx.receiver_count() > 0);
    }

    /// Number of active workspace channels (for testing).
    pub async fn workspace_channel_count(&self) -> usize {
        self.workspace_txs.read().await.len()
    }

    /// Default channel capacity.
    pub fn default_capacity() -> usize {
        DEFAULT_CAPACITY
    }
}
