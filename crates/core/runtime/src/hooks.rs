use crate::{EventAction, EventEntity, append_event, log_action};
use anyhow::Result;
use std::path::Path;

/// Runtime hooks exposed for higher-level ops/service layers.
/// App layers can use these to keep transport code thin while the runtime
/// remains the central integration point for event/log sinks.
pub trait RuntimeHooks: Send + Sync {
    fn append_entity_event(
        &self,
        ship_dir: &Path,
        actor: &str,
        entity: EventEntity,
        action: EventAction,
        subject: &str,
        details: Option<String>,
    ) -> Result<()>;

    fn append_log(&self, ship_dir: &Path, action: &str, details: &str) -> Result<()>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultRuntimeHooks;

impl RuntimeHooks for DefaultRuntimeHooks {
    fn append_entity_event(
        &self,
        ship_dir: &Path,
        actor: &str,
        entity: EventEntity,
        action: EventAction,
        subject: &str,
        details: Option<String>,
    ) -> Result<()> {
        append_event(
            ship_dir,
            actor,
            entity,
            action,
            subject.to_string(),
            details,
        )?;
        Ok(())
    }

    fn append_log(&self, ship_dir: &Path, action: &str, details: &str) -> Result<()> {
        log_action(ship_dir, action, details)
    }
}
