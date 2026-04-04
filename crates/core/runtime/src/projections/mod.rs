//! Projection system — derived state from the event log.
//!
//! A [`Projection`] declares which event types it handles and how to apply them
//! to a read model (SQL table). The [`EventBus`] dispatches events to registered
//! projections after every append.
//!
//! Projections are the ONLY mutable state in the system. They can be truncated
//! and rebuilt from the event log at any time via [`rebuild`].

pub mod async_projection;
pub mod job;
pub mod spawn;
mod actor;
mod registry;
mod session;
mod workspace;
mod workspace_handlers;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_actor;
#[cfg(test)]
mod tests_async;
#[cfg(test)]
mod tests_session;

pub use actor::ActorProjection;
pub use async_projection::{AsyncProjection, spawn_projection};
pub use job::{JobRecord, JobStatus, load_jobs, project as project_jobs};
pub use registry::{EventBus, Projection};
pub use session::SessionProjection;
pub use spawn::{ProjectionHandle, spawn_with_failure_counter};
pub use workspace::WorkspaceProjection;
