//! Platform service actors — headless actors that provide infrastructure APIs.
//!
//! A service actor runs continuously, consuming events from its mailbox and
//! optionally waking on a timer. Spawned by the kernel at startup; communicates
//! with other actors via kernel-routed events.
//!
//! # Implementing a service
//!
//! ```rust,ignore
//! struct MySvc;
//! impl ServiceHandler for MySvc {
//!     fn name(&self) -> &str { "my-svc" }
//!     fn handle(&mut self, event: &EventEnvelope, _store: &ActorStore) -> Result<()> { Ok(()) }
//! }
//! ```
//!
//! Register with the kernel:
//!
//! ```rust,ignore
//! kernel.spawn_service("my-svc", config, Box::new(MySvc))?;
//! ```

use std::time::Duration;

use anyhow::Result;
use tokio::task::JoinHandle;

use crate::events::{ActorStore, EventEnvelope, Mailbox};

pub mod sync;
#[cfg(test)]
mod tests;

/// Trait implemented by headless service actors.
///
/// The service runner calls `on_start` once, then drives the event loop —
/// calling `handle` for each mailbox event and `on_tick` on the configured
/// interval. When the mailbox closes, `on_stop` is called and the task exits.
pub trait ServiceHandler: Send + 'static {
    /// Stable identifier used in log output.
    fn name(&self) -> &str;

    /// Process one event delivered to the service's mailbox.
    fn handle(&mut self, event: &EventEnvelope, store: &ActorStore) -> Result<()>;

    /// Called once before the event loop starts. Errors abort the service.
    fn on_start(&mut self, _store: &ActorStore) -> Result<()> {
        Ok(())
    }

    /// Called once after the mailbox closes.
    fn on_stop(&mut self, _store: &ActorStore) -> Result<()> {
        Ok(())
    }

    /// If `Some`, `on_tick` fires on this interval alongside mailbox events.
    fn tick_interval(&self) -> Option<Duration> {
        None
    }

    /// Called on each timer tick. Only invoked when `tick_interval` returns `Some`.
    fn on_tick(&mut self, _store: &ActorStore) -> Result<()> {
        Ok(())
    }
}

/// Handle for a running service actor task.
pub struct ServiceHandle {
    pub name: String,
    pub handle: JoinHandle<()>,
}

/// Run the service event loop.
///
/// Drives `handler` until the mailbox closes. Uses `tokio::select!` when a
/// tick interval is configured; otherwise blocks only on the mailbox.
pub async fn run_service(
    mut handler: Box<dyn ServiceHandler>,
    store: ActorStore,
    mut mailbox: Mailbox,
) {
    if let Err(e) = handler.on_start(&store) {
        eprintln!("[service:{}] on_start failed: {e}", handler.name());
        return;
    }

    if let Some(dur) = handler.tick_interval() {
        let mut ticker = tokio::time::interval(dur);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        // Consume the immediate first tick so on_tick isn't called at t=0.
        ticker.tick().await;

        loop {
            tokio::select! {
                event = mailbox.recv() => {
                    match event {
                        Some(e) => {
                            if let Err(err) = handler.handle(&e, &store) {
                                eprintln!("[service:{}] handle error: {err}", handler.name());
                            }
                        }
                        None => break,
                    }
                }
                _ = ticker.tick() => {
                    if let Err(e) = handler.on_tick(&store) {
                        eprintln!("[service:{}] tick error: {e}", handler.name());
                    }
                }
            }
        }
    } else {
        while let Some(event) = mailbox.recv().await {
            if let Err(e) = handler.handle(&event, &store) {
                eprintln!("[service:{}] handle error: {e}", handler.name());
            }
        }
    }

    if let Err(e) = handler.on_stop(&store) {
        eprintln!("[service:{}] on_stop failed: {e}", handler.name());
    }
}
