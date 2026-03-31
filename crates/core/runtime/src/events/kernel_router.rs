//! KernelRouter — per-actor mailboxes and isolated event stores.
//!
//! This is the primary event routing infrastructure. Each actor gets its own
//! SQLite event store and mailbox. The kernel routes events between actors
//! based on namespace subscriptions.
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
use std::sync::Mutex;

use anyhow::{Result, anyhow};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::events::actor_store::{ActorStore, init_actor_db, raw_append};
use crate::events::envelope::EventEnvelope;
use crate::events::mailbox::Mailbox;
use crate::events::snapshot::{ActorSnapshot, event_stats};
use crate::events::validator::{EmitContext, EventValidator};

#[cfg(feature = "unstable")]
use crate::events::kernel_security::ActorMeta;

pub(crate) const MAILBOX_CAPACITY: usize = 256;

/// Namespace prefixes owned by the kernel.
const KERNEL_NAMESPACES: &[&str] = &[
    "workspace.", "session.", "actor.", "gate.", "config.", "runtime.", "sync.",
];

fn is_kernel_event(event_type: &str) -> bool {
    KERNEL_NAMESPACES
        .iter()
        .any(|ns| event_type.starts_with(ns))
}

/// Configuration provided to `KernelRouter::spawn_actor`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorConfig {
    pub namespace: String,
    pub write_namespaces: Vec<String>,
    pub read_namespaces: Vec<String>,
    pub subscribe_namespaces: Vec<String>,
}

/// Kernel-managed event router with per-actor isolation.
pub struct KernelRouter {
    pub(crate) base_dir: PathBuf,
    pub(crate) kernel_store_path: PathBuf,
    pub(crate) mailboxes: HashMap<String, mpsc::Sender<EventEnvelope>>,
    pub(crate) actor_configs: HashMap<String, ActorConfig>,
    pub(crate) subscriptions: HashMap<String, Vec<String>>,
    validators: Vec<Box<dyn EventValidator>>,
    #[cfg(feature = "unstable")]
    pub(crate) actor_meta: HashMap<String, ActorMeta>,
    #[cfg(feature = "unstable")]
    pub(crate) event_log: Mutex<Vec<EventEnvelope>>,
    #[cfg(feature = "unstable")]
    pub(crate) actor_cursors: HashMap<String, usize>,
}

impl KernelRouter {
    pub fn new(base_dir: PathBuf) -> Result<Self> {
        let kernel_store_path = base_dir.join("kernel").join("events.db");
        init_actor_db(&kernel_store_path)?;
        #[cfg(feature = "unstable")]
        crate::events::cursor::init_cursor_table(&kernel_store_path)?;

        Ok(Self {
            base_dir,
            kernel_store_path,
            mailboxes: HashMap::new(),
            actor_configs: HashMap::new(),
            subscriptions: HashMap::new(),
            validators: Vec::new(),
            #[cfg(feature = "unstable")]
            actor_meta: HashMap::new(),
            #[cfg(feature = "unstable")]
            event_log: Mutex::new(Vec::new()),
            #[cfg(feature = "unstable")]
            actor_cursors: HashMap::new(),
        })
    }

    pub fn with_validator(mut self, v: Box<dyn EventValidator>) -> Self {
        self.validators.push(v);
        self
    }

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
        self.actor_configs
            .insert(actor_id.to_string(), config.clone());
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

    pub async fn route(&self, event: EventEnvelope, ctx: &EmitContext) -> Result<()> {
        for v in &self.validators {
            v.validate(&event, ctx).map_err(|e| anyhow!("{e}"))?;
        }

        #[cfg(feature = "unstable")]
        self.enforce_emit_permissions(&event, ctx)?;

        if is_kernel_event(&event.event_type) {
            raw_append(&self.kernel_store_path, &event)?;
        }

        #[cfg(feature = "unstable")]
        {
            self.event_log.lock().unwrap().push(event.clone());
            self.deliver_with_scope(&event).await;
            return Ok(());
        }

        #[cfg(not(feature = "unstable"))]
        {
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
    }

    pub fn stop_actor(&mut self, actor_id: &str) -> Result<()> {
        if self.mailboxes.remove(actor_id).is_none() {
            return Err(anyhow!("actor '{}' not found", actor_id));
        }
        self.actor_configs.remove(actor_id);

        #[cfg(feature = "unstable")]
        {
            let log_len = self.event_log.lock().unwrap().len();
            self.actor_cursors.insert(actor_id.to_string(), log_len);
            self.actor_meta.remove(actor_id);
        }

        for actor_ids in self.subscriptions.values_mut() {
            actor_ids.retain(|id| id != actor_id);
        }
        let db_path = self.actor_db_path(actor_id);
        if db_path.exists() {
            flush_wal(&db_path)?;
        }
        Ok(())
    }

    pub fn snapshot(&self, actor_id: &str) -> Result<ActorSnapshot> {
        if !self.mailboxes.contains_key(actor_id) {
            return Err(anyhow!("actor '{}' not found", actor_id));
        }
        let config = self
            .actor_configs
            .get(actor_id)
            .ok_or_else(|| anyhow!("actor config missing for '{}'", actor_id))?
            .clone();
        let db_path = self.actor_db_path(actor_id);
        flush_wal(&db_path)?;
        let db_bytes = std::fs::read(&db_path)?;
        let (event_count, last_event_id) = event_stats(&db_path)?;
        let snap = ActorSnapshot {
            actor_id: actor_id.to_string(),
            namespace: config.namespace.clone(),
            config,
            db_bytes,
            created_at: Utc::now(),
            event_count,
            last_event_id,
        };
        let payload = serde_json::json!({
            "actor_id": actor_id,
            "event_count": snap.event_count,
            "last_event_id": snap.last_event_id,
            "size_bytes": snap.db_bytes.len(),
        });
        raw_append(
            &self.kernel_store_path,
            &EventEnvelope::new("kernel.actor.snapshot", actor_id, &payload)?,
        )?;
        Ok(snap)
    }

    pub fn suspend(&mut self, actor_id: &str) -> Result<ActorSnapshot> {
        let snap = self.snapshot(actor_id)?;
        self.stop_actor(actor_id)?;
        let payload = serde_json::json!({
            "actor_id": actor_id,
            "reason": "suspend_requested",
        });
        raw_append(
            &self.kernel_store_path,
            &EventEnvelope::new("kernel.actor.suspended", actor_id, &payload)?,
        )?;
        Ok(snap)
    }

    pub fn restore(&mut self, snapshot: ActorSnapshot) -> Result<(ActorStore, Mailbox)> {
        if self.mailboxes.contains_key(&snapshot.actor_id) {
            return Err(anyhow!("actor '{}' already exists", snapshot.actor_id));
        }
        let db_path = self.actor_db_path(&snapshot.actor_id);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&db_path, &snapshot.db_bytes)?;
        let event_count = snapshot.event_count;
        let actor_id = snapshot.actor_id.clone();
        let result = self.spawn_actor(&actor_id, snapshot.config)?;
        let payload = serde_json::json!({
            "actor_id": &actor_id,
            "event_count": event_count,
        });
        raw_append(
            &self.kernel_store_path,
            &EventEnvelope::new("kernel.actor.restored", &actor_id, &payload)?,
        )?;
        Ok(result)
    }

    pub fn actor_count(&self) -> usize {
        self.mailboxes.len()
    }

    pub fn kernel_store_path(&self) -> &Path {
        &self.kernel_store_path
    }

    pub(crate) fn actor_db_path(&self, actor_id: &str) -> PathBuf {
        self.base_dir
            .join("actors")
            .join(actor_id)
            .join("events.db")
    }
}

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
