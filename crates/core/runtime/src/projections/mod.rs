//! Projection system — derived state from the event log.
//!
//! A [`Projection`] declares which event types it handles and how to apply them
//! to a read model (SQL table). The [`EventBus`] dispatches events to registered
//! projections after every append.
//!
//! Projections are the ONLY mutable state in the system. They can be truncated
//! and rebuilt from the event log at any time via [`rebuild`].

mod registry;
mod workspace;

#[cfg(test)]
mod tests;

pub use registry::{EventBus, Projection};
pub use workspace::WorkspaceProjection;
