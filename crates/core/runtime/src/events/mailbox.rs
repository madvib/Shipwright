//! Per-actor mailbox — the receive end of a kernel-managed message channel.
//!
//! Created exclusively by `KernelRouter::spawn_actor`. Events are delivered
//! to matching mailboxes by `KernelRouter::route`.

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

    /// Receive the next event. Returns `None` if all senders have been dropped.
    pub async fn recv(&mut self) -> Option<EventEnvelope> {
        self.rx.recv().await
    }

    /// Try to receive without blocking.
    pub fn try_recv(&mut self) -> Result<EventEnvelope, mpsc::error::TryRecvError> {
        self.rx.try_recv()
    }
}
