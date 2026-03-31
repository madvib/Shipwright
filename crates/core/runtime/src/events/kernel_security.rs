//! Security extensions for KernelRouter — identity, permissions, scoped delivery.
//!
//! Adds `spawn_actor_with_permissions`, emit enforcement, scoped/directed
//! delivery, and cursor-based replay to the kernel router.

use anyhow::{Result, anyhow};
use tokio::sync::mpsc;

use crate::events::actor_store::{ActorStore, init_actor_db};
use crate::events::envelope::EventEnvelope;
use crate::events::identity::ActorIdentity;
use crate::events::kernel_router::{KernelRouter, MAILBOX_CAPACITY};
use crate::events::mailbox::Mailbox;
use crate::events::permissions::{ActorPermissions, DeliveryScope, PermittedSubscription};
use crate::events::validator::{CallerKind, EmitContext};

/// Per-actor metadata for permission-based actors.
pub(crate) struct ActorMeta {
    pub permissions: ActorPermissions,
    pub workspace_id: Option<String>,
    pub subscriptions: Vec<PermittedSubscription>,
}

impl KernelRouter {
    /// Spawn an actor with identity and permissions.
    pub fn spawn_actor_with_permissions(
        &mut self,
        identity: ActorIdentity,
        permissions: ActorPermissions,
    ) -> Result<(ActorStore, Mailbox)> {
        self.spawn_actor_with_permissions_inner(identity, permissions, None)
    }

    /// Spawn an actor with identity, permissions, and workspace binding.
    pub fn spawn_actor_with_permissions_in_workspace(
        &mut self,
        identity: ActorIdentity,
        permissions: ActorPermissions,
        workspace_id: &str,
    ) -> Result<(ActorStore, Mailbox)> {
        self.spawn_actor_with_permissions_inner(
            identity,
            permissions,
            Some(workspace_id.to_string()),
        )
    }

    fn spawn_actor_with_permissions_inner(
        &mut self,
        identity: ActorIdentity,
        permissions: ActorPermissions,
        workspace_id: Option<String>,
    ) -> Result<(ActorStore, Mailbox)> {
        permissions.validate()?;

        let label = &identity.label;
        if self.mailboxes.contains_key(label) {
            return Err(anyhow!("actor '{}' already exists", label));
        }

        let cursor_pos = self.actor_cursors.get(label).copied();

        let db_path = self.actor_db_path(label);
        init_actor_db(&db_path)?;

        let (tx, rx) = mpsc::channel(MAILBOX_CAPACITY);
        self.mailboxes.insert(label.to_string(), tx.clone());

        let subs = permissions.subscribe.clone();
        for sub in &subs {
            self.subscriptions
                .entry(sub.namespace.clone())
                .or_default()
                .push(label.to_string());
        }

        self.actor_meta.insert(
            label.to_string(),
            ActorMeta {
                permissions: permissions.clone(),
                workspace_id,
                subscriptions: subs,
            },
        );

        // Replay events since cursor, sorted by ULID for guaranteed ordering.
        if let Some(pos) = cursor_pos {
            let log = self.event_log.lock().unwrap();
            let mut replay: Vec<_> = log[pos..]
                .iter()
                .filter(|ev| self.should_deliver(ev, label))
                .cloned()
                .collect();
            replay.sort_by(|a, b| a.id.cmp(&b.id));
            for ev in &replay {
                let _ = tx.try_send(ev.clone());
            }
        }

        let store = ActorStore::new(
            label,
            db_path,
            permissions.emit.clone(),
            subs_to_namespaces(&permissions.subscribe),
        );
        Ok((store, Mailbox::new(rx)))
    }

    /// Check if an event matches an actor's subscription scope.
    /// Returns None if the actor has no permission metadata (legacy spawn_actor).
    pub(crate) fn event_matches_actor_scope(
        &self,
        event: &EventEnvelope,
        actor_label: &str,
    ) -> Option<bool> {
        let meta = self.actor_meta.get(actor_label)?;
        for sub in &meta.subscriptions {
            if !event.event_type.starts_with(sub.namespace.as_str()) {
                continue;
            }
            match sub.scope {
                DeliveryScope::Global => return Some(true),
                DeliveryScope::Workspace => {
                    if event.workspace_id.as_deref() == meta.workspace_id.as_deref() {
                        return Some(true);
                    }
                }
                DeliveryScope::Directed => {
                    if event.target_actor_id.as_deref() == Some(actor_label) {
                        return Some(true);
                    }
                }
                DeliveryScope::Elevated => {
                    if event.elevated {
                        return Some(true);
                    }
                }
            }
        }
        Some(false)
    }

    /// Enforce emit permissions. CLI callers bypass.
    pub(crate) fn enforce_emit_permissions(
        &self,
        event: &EventEnvelope,
        ctx: &EmitContext,
    ) -> Result<()> {
        if ctx.caller_kind == CallerKind::Cli {
            return Ok(());
        }
        if ctx.caller_kind == CallerKind::Runtime && event.actor_id.is_none() {
            return Ok(());
        }

        if let Some(ref actor_id) = event.actor_id {
            if let Some(meta) = self.actor_meta.get(actor_id) {
                if !meta.permissions.can_emit(&event.event_type) {
                    return Err(anyhow!(
                        "actor '{}' not permitted to emit '{}'",
                        actor_id,
                        event.event_type
                    ));
                }
            }
        }

        Ok(())
    }

    /// Should this actor receive this event?
    pub(crate) fn should_deliver(&self, event: &EventEnvelope, actor_id: &str) -> bool {
        match self.event_matches_actor_scope(event, actor_id) {
            Some(matches) => matches,
            None => true, // Legacy actor — namespace match already done by caller
        }
    }

    /// Deliver event to matching subscribers respecting scope and directed delivery.
    pub(crate) async fn deliver_with_scope(&self, event: &EventEnvelope) {
        if let Some(ref target) = event.target_actor_id {
            if let Some(tx) = self.mailboxes.get(target) {
                if self.should_deliver(event, target) {
                    let _ = tx.send(event.clone()).await;
                }
            }
            return;
        }

        for (ns, actor_ids) in &self.subscriptions {
            if event.event_type.starts_with(ns.as_str()) {
                for actor_id in actor_ids {
                    if self.should_deliver(event, actor_id) {
                        if let Some(tx) = self.mailboxes.get(actor_id) {
                            let _ = tx.send(event.clone()).await;
                        }
                    }
                }
            }
        }
    }
}

fn subs_to_namespaces(subs: &[PermittedSubscription]) -> Vec<String> {
    subs.iter().map(|s| s.namespace.clone()).collect()
}
