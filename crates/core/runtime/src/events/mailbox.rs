//! Per-actor mailbox — the receive end of an event message channel.
//!
//! Typically created by `KernelRouter::spawn_actor` for local routing, or
//! via `Mailbox::from_receiver` when bridging an external event source
//! (e.g. daemon SSE stream) into the relay pipeline.

use tokio::sync::mpsc;

use crate::events::envelope::EventEnvelope;

/// The receive end of a per-actor message channel.
pub struct Mailbox {
    rx: mpsc::Receiver<EventEnvelope>,
}

impl Mailbox {
    pub(crate) fn new(rx: mpsc::Receiver<EventEnvelope>) -> Self {
        Self { rx }
    }

    /// Construct a `Mailbox` from an external receiver.
    ///
    /// Used when the mailbox source is not a local `KernelRouter` — e.g. when
    /// bridging events from the daemon's SSE stream into the relay pipeline.
    pub fn from_receiver(rx: mpsc::Receiver<EventEnvelope>) -> Self {
        Self { rx }
    }

    /// Receive the next event. Returns `None` if all senders have been dropped.
    pub async fn recv(&mut self) -> Option<EventEnvelope> {
        self.rx.recv().await
    }

    /// Try to receive without blocking.
    pub fn try_recv(&mut self) -> Result<EventEnvelope, mpsc::error::TryRecvError> {
        self.rx.try_recv()
    }
}
