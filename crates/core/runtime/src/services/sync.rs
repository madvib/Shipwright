//! Sync service — cursor-based push/pull of elevated events to Ship Cloud.
//!
//! Push cycle: fires every `push_interval_secs`. Reads elevated events from the
//! kernel store since the push cursor, ships them to cloud, advances the cursor.
//!
//! Pull cycle: fires every `pull_interval_secs` (approximated by counting push
//! ticks). Pulls remote events and records receipt in the service's own store.
//! Full kernel routing of pulled events is a follow-up (requires kernel access
//! from within the service task).

use std::time::Duration;

use anyhow::Result;

use crate::events::{ActorStore, EventEnvelope, query_events_since};
use crate::services::ServiceHandler;
use crate::sync::{SyncClient, get_cursor, set_cursor};

const PUSH_SCOPE_PREFIX: &str = "push:platform";
const PULL_SCOPE_PREFIX: &str = "pull:platform";

/// Configuration for the sync service, read from `ship.jsonc` `sync` block.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub endpoint: String,
    pub project_id: String,
    pub push_interval_secs: u64,
    pub pull_interval_secs: u64,
    /// Emit a push immediately once this many events have accumulated.
    pub push_threshold: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.getship.dev".to_string(),
            project_id: String::new(),
            push_interval_secs: 30,
            pull_interval_secs: 60,
            push_threshold: 50,
        }
    }
}

/// Headless sync service implementing [`ServiceHandler`].
///
/// Push and pull cycles are driven by the tick interval. Threshold-based push
/// is triggered from `handle` when elevated events accumulate in the mailbox.
pub struct SyncServiceHandler {
    config: SyncConfig,
    client: SyncClient,
    /// Push ticks elapsed since last pull cycle.
    ticks_since_pull: u32,
    /// Number of push ticks per pull tick.
    pub(crate) pull_every_n_ticks: u32,
    /// Count of elevated events seen since last push (for threshold trigger).
    elevated_since_push: usize,
}

impl SyncServiceHandler {
    pub fn new(config: SyncConfig) -> Self {
        let client = SyncClient::new(&config.endpoint);
        let pull_every_n_ticks = (config.pull_interval_secs
            / config.push_interval_secs.max(1))
        .max(1) as u32;
        Self {
            config,
            client,
            ticks_since_pull: 0,
            pull_every_n_ticks,
            elevated_since_push: 0,
        }
    }

    fn push_scope(&self) -> String {
        format!("{PUSH_SCOPE_PREFIX}:{}", self.config.project_id)
    }

    fn pull_scope(&self) -> String {
        format!("{PULL_SCOPE_PREFIX}:{}", self.config.project_id)
    }

    fn push_cycle(&mut self, store: &ActorStore) -> Result<()> {
        let scope = self.push_scope();
        let cursor = get_cursor(&scope)?;
        let events = query_events_since(cursor.as_deref(), true)?;
        if events.is_empty() {
            return Ok(());
        }
        let resp = self
            .client
            .push_platform_events(&self.config.project_id, &events)?;
        set_cursor(&scope, &resp.cursor)?;
        self.elevated_since_push = 0;
        let payload = serde_json::json!({
            "event_count": resp.accepted,
            "cursor": resp.cursor,
        });
        store.append(&EventEnvelope::new("sync.push.completed", "sync", &payload)?)?;
        Ok(())
    }

    fn pull_cycle(&mut self, store: &ActorStore) -> Result<()> {
        let scope = self.pull_scope();
        let cursor = get_cursor(&scope)?;
        let resp = self
            .client
            .pull_platform_events(&self.config.project_id, cursor.as_deref())?;
        if resp.events.is_empty() {
            return Ok(());
        }
        set_cursor(&scope, &resp.cursor)?;
        // TODO: route pulled events through KernelRouter so subscribed actors
        // receive them. Requires kernel routing access from within the service
        // task — deferred to Phase 2.
        let payload = serde_json::json!({
            "event_count": resp.events.len(),
            "cursor": resp.cursor,
        });
        store.append(&EventEnvelope::new("sync.pull.completed", "sync", &payload)?)?;
        Ok(())
    }
}

impl ServiceHandler for SyncServiceHandler {
    fn name(&self) -> &str {
        "sync"
    }

    fn handle(&mut self, event: &EventEnvelope, store: &ActorStore) -> Result<()> {
        if event.event_type == "sync.trigger.push" {
            return self.push_cycle(store);
        }
        // Count elevated events for threshold-based push.
        if event.elevated {
            self.elevated_since_push += 1;
            if self.elevated_since_push >= self.config.push_threshold {
                self.push_cycle(store)?;
            }
        }
        Ok(())
    }

    fn on_start(&mut self, store: &ActorStore) -> Result<()> {
        store.append(&EventEnvelope::new(
            "sync.started",
            "sync",
            &serde_json::json!({"project_id": &self.config.project_id}),
        )?)?;
        Ok(())
    }

    fn on_stop(&mut self, store: &ActorStore) -> Result<()> {
        store.append(&EventEnvelope::new(
            "sync.stopped",
            "sync",
            &serde_json::json!({}),
        )?)?;
        Ok(())
    }

    fn tick_interval(&self) -> Option<Duration> {
        Some(Duration::from_secs(self.config.push_interval_secs))
    }

    fn on_tick(&mut self, store: &ActorStore) -> Result<()> {
        if let Err(e) = self.push_cycle(store) {
            eprintln!("[sync] push_cycle error: {e}");
            let payload = serde_json::json!({"error": e.to_string()});
            let _ = store.append(&EventEnvelope::new("sync.push.failed", "sync", &payload)?);
        }

        self.ticks_since_pull += 1;
        if self.ticks_since_pull >= self.pull_every_n_ticks {
            self.ticks_since_pull = 0;
            if let Err(e) = self.pull_cycle(store) {
                eprintln!("[sync] pull_cycle error: {e}");
            }
        }

        Ok(())
    }
}
