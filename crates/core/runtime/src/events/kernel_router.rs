//! KernelRouter — per-actor mailboxes and isolated event stores.
//!
//! Additive: `EventRouter` / `global_router` are unchanged and continue to
//! function. `KernelRouter` is the foundation for actor isolation; MCP
//! migration happens in Phase 3.
//!
//! Directory layout managed by this module:
//! ```text
//! {base_dir}/
//!   kernel/
//!     events.db        # system lifecycle events only
//!   actors/
//!     {actor_id}/
//!       events.db      # actor-scoped events
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use tokio::sync::mpsc;

use crate::events::actor_store::{ActorStore, init_actor_db, raw_append};
use crate::events::envelope::EventEnvelope;
use crate::events::mailbox::Mailbox;
use crate::events::validator::{EmitContext, EventValidator};

const MAILBOX_CAPACITY: usize = 256;

/// Namespace prefixes owned by the kernel. Only these are persisted to the
/// kernel store by `route`.
const KERNEL_NAMESPACES: &[&str] = &[
    "workspace.",
    "session.",
    "actor.",
    "gate.",
    "config.",
    "runtime.",
    "sync.",
];

fn is_kernel_event(event_type: &str) -> bool {
    KERNEL_NAMESPACES
        .iter()
        .any(|ns| event_type.starts_with(ns))
}

/// Configuration provided to `KernelRouter::spawn_actor`.
pub struct ActorConfig {
    /// Logical namespace, e.g. `"studio"` or `"agent.abc123"`.
    pub namespace: String,
    /// Event type prefixes this actor may emit (e.g. `["studio."]`).
    pub write_namespaces: Vec<String>,
    /// Event type prefixes this actor may query (e.g. `["studio.", "session."]`).
    pub read_namespaces: Vec<String>,
    /// Event type prefixes routed into this actor's mailbox.
    pub subscribe_namespaces: Vec<String>,
}

/// Kernel-managed event router with per-actor isolation.
///
/// Manages actor lifecycles, scoped event stores, and per-actor mailboxes.
/// Coexists with the existing `EventRouter` during the Phase 1→3 transition.
pub struct KernelRouter {
    /// Root Ship directory (e.g. `~/.ship`).
    base_dir: PathBuf,
    /// Path to the kernel's own event store.
    kernel_store_path: PathBuf,
    /// Send ends of per-actor mailboxes, keyed by actor_id.
    mailboxes: HashMap<String, mpsc::Sender<EventEnvelope>>,
    /// Namespace prefix → subscribed actor_ids.
    subscriptions: HashMap<String, Vec<String>>,
    /// Ingress validators run by `route`.
    validators: Vec<Box<dyn EventValidator>>,
}

impl KernelRouter {
    /// Create a `KernelRouter` rooted at `base_dir`.
    ///
    /// Initialises `{base_dir}/kernel/events.db` on first call.
    pub fn new(base_dir: PathBuf) -> Result<Self> {
        let kernel_store_path = base_dir.join("kernel").join("events.db");
        init_actor_db(&kernel_store_path)?;
        Ok(Self {
            base_dir,
            kernel_store_path,
            mailboxes: HashMap::new(),
            subscriptions: HashMap::new(),
            validators: Vec::new(),
        })
    }

    /// Builder: add a validator to the ingress pipeline.
    pub fn with_validator(mut self, v: Box<dyn EventValidator>) -> Self {
        self.validators.push(v);
        self
    }

    /// Spawn a new actor: create its directory, initialise its DB, issue a mailbox.
    ///
    /// Returns the scoped `ActorStore` (write handle) and a `Mailbox` (receive handle).
    /// Errors if an actor with the same `actor_id` is already live.
    pub fn spawn_actor(
        &mut self,
        actor_id: &str,
        config: ActorConfig,
    ) -> Result<(ActorStore, Mailbox)> {
        if self.mailboxes.contains_key(actor_id) {
            return Err(anyhow!("actor '{}' already exists", actor_id));
        }

        let db_path = self
            .base_dir
            .join("actors")
            .join(actor_id)
            .join("events.db");
        init_actor_db(&db_path)?;

        let (tx, rx) = mpsc::channel(MAILBOX_CAPACITY);
        self.mailboxes.insert(actor_id.to_string(), tx);

        for ns in &config.subscribe_namespaces {
            self.subscriptions
                .entry(ns.clone())
                .or_default()
                .push(actor_id.to_string());
        }

        let store = ActorStore::new(
            actor_id,
            db_path,
            config.write_namespaces,
            config.read_namespaces,
        );
        Ok((store, Mailbox::new(rx)))
    }

    /// Route an event: validate → persist system events to kernel store → deliver to mailboxes.
    pub async fn route(&self, event: EventEnvelope, ctx: &EmitContext) -> Result<()> {
        for v in &self.validators {
            v.validate(&event, ctx).map_err(|e| anyhow!("{e}"))?;
        }

        if is_kernel_event(&event.event_type) {
            raw_append(&self.kernel_store_path, &event)?;
        }

        for (ns, actor_ids) in &self.subscriptions {
            if event.event_type.starts_with(ns.as_str()) {
                for actor_id in actor_ids {
                    if let Some(tx) = self.mailboxes.get(actor_id) {
                        let _ = tx.send(event.clone()).await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Tear down an actor: flush its event store, drop its mailbox, and remove
    /// its subscriptions.
    ///
    /// Errors if the actor does not exist.
    pub fn stop_actor(&mut self, actor_id: &str) -> Result<()> {
        if self.mailboxes.remove(actor_id).is_none() {
            return Err(anyhow!("actor '{}' not found", actor_id));
        }
        for actor_ids in self.subscriptions.values_mut() {
            actor_ids.retain(|id| id != actor_id);
        }
        // Flush WAL to ensure all writes are checkpointed before teardown.
        let db_path = self.actor_db_path(actor_id);
        if db_path.exists() {
            flush_wal(&db_path)?;
        }
        Ok(())
    }

    /// Number of currently live actors.
    pub fn actor_count(&self) -> usize {
        self.mailboxes.len()
    }

    /// Path to the kernel's own event store.
    pub fn kernel_store_path(&self) -> &Path {
        &self.kernel_store_path
    }

    /// Resolved DB path for an actor.
    fn actor_db_path(&self, actor_id: &str) -> PathBuf {
        self.base_dir
            .join("actors")
            .join(actor_id)
            .join("events.db")
    }
}

/// Checkpoint WAL to main database file.
fn flush_wal(db_path: &Path) -> Result<()> {
    use crate::db::open_db_at;
    let mut conn = open_db_at(db_path)?;
    crate::db::block_on(async {
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}
