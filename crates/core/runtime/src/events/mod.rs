pub mod actor_store;
pub mod artifact_events;
pub mod envelope;
pub mod filter;
pub mod kernel_router;
pub mod mailbox;
pub mod snapshot;
pub mod store;
pub mod types;
pub mod validator;

pub mod global_kernel_router;

#[cfg(feature = "unstable")]
pub mod identity;
#[cfg(feature = "unstable")]
pub mod permissions;
#[cfg(feature = "unstable")]
pub mod cursor;
#[cfg(feature = "unstable")]
pub mod kernel_security;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_actor;
#[cfg(test)]
mod kernel_router_tests;
#[cfg(test)]
mod cross_actor_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod cli_routing_tests;
#[cfg(test)]
mod identity_tests;
#[cfg(test)]
mod permission_tests;
#[cfg(test)]
mod scoped_delivery_tests;
#[cfg(test)]
mod directed_delivery_tests;
#[cfg(test)]
mod cursor_tests;
#[cfg(test)]
mod integration_tests;

pub use actor_store::ActorStore;
pub use envelope::EventEnvelope;
pub use filter::EventFilter;
pub use kernel_router::{ActorConfig, KernelRouter};
pub use mailbox::Mailbox;
pub use snapshot::ActorSnapshot;
pub use store::{EventStore, SqliteEventStore};
pub use validator::{CallerKind, EmitContext, EventValidator, ValidationError, RESERVED_NAMESPACES};
pub use global_kernel_router::{init_kernel_router, kernel_router};

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::Path;

pub fn read_events(_ship_dir: &Path) -> Result<Vec<EventEnvelope>> {
    crate::db::events::list_all_events()
}

pub fn list_events_since(
    _ship_dir: &Path,
    since: &DateTime<Utc>,
    limit: Option<usize>,
) -> Result<Vec<EventEnvelope>> {
    crate::db::events::list_events_since_time(since, limit)
}

pub fn read_recent_events(_ship_dir: &Path, limit: usize) -> Result<Vec<EventEnvelope>> {
    crate::db::events::list_recent_events(limit)
}

/// Query events with ID greater than the given cursor.
///
/// Used by the sync client to find events that haven't been pushed yet.
/// If `elevated_only` is true, only returns elevated events (platform-scope).
pub fn query_events_since(
    cursor: Option<&str>,
    elevated_only: bool,
) -> Result<Vec<EventEnvelope>> {
    crate::db::events::query_events_since(cursor, elevated_only)
}
