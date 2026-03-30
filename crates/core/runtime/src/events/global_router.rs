//! Global EventRouter singleton — initialized once at startup.

use std::sync::{Arc, OnceLock};

use crate::events::envelope::EventEnvelope;
use crate::events::filter::EventFilter;
use crate::events::router::EventRouter;
use crate::events::store::EventStore;
use crate::events::validator::{NamespaceValidator, ReservedNamespaceValidator};

static ROUTER: OnceLock<Arc<EventRouter>> = OnceLock::new();

/// Initialize the global EventRouter with an explicit store. Call once at
/// startup. Idempotent — returns the existing router if already initialized.
pub fn init_router(store: Arc<dyn EventStore>) -> Arc<EventRouter> {
    ROUTER
        .get_or_init(|| {
            let router = EventRouter::new(store, EventRouter::default_capacity())
                .with_validator(Box::new(NamespaceValidator))
                .with_validator(Box::new(ReservedNamespaceValidator));
            Arc::new(router)
        })
        .clone()
}

/// Get the global EventRouter. Auto-initializes with a dynamic store that
/// resolves the DB path on every call (safe across test temp dirs).
pub fn router() -> Arc<EventRouter> {
    ROUTER
        .get_or_init(|| {
            let store: Arc<dyn EventStore> = Arc::new(DynamicEventStore);
            let router = EventRouter::new(store, EventRouter::default_capacity())
                .with_validator(Box::new(NamespaceValidator))
                .with_validator(Box::new(ReservedNamespaceValidator));
            Arc::new(router)
        })
        .clone()
}

/// Event store that resolves the DB path on every call via `db_path()`.
///
/// Unlike `SqliteEventStore` (which caches the path at construction), this
/// works correctly across test threads that each set up their own temp dir
/// via `init_project`. Production callers should use `init_router` with a
/// concrete `SqliteEventStore` for optimal performance.
struct DynamicEventStore;

impl EventStore for DynamicEventStore {
    fn append(&self, event: &EventEnvelope) -> anyhow::Result<()> {
        crate::events::SqliteEventStore::new()?.append(event)
    }

    fn get(&self, id: &str) -> anyhow::Result<Option<EventEnvelope>> {
        crate::events::SqliteEventStore::new()?.get(id)
    }

    fn query(&self, filter: &EventFilter) -> anyhow::Result<Vec<EventEnvelope>> {
        crate::events::SqliteEventStore::new()?.query(filter)
    }

    fn query_aggregate(&self, entity_id: &str) -> anyhow::Result<Vec<EventEnvelope>> {
        crate::events::SqliteEventStore::new()?.query_aggregate(entity_id)
    }

    fn query_correlation(
        &self,
        correlation_id: &str,
    ) -> anyhow::Result<Vec<EventEnvelope>> {
        crate::events::SqliteEventStore::new()?.query_correlation(correlation_id)
    }
}

#[cfg(test)]
pub fn init_router_for_test(store: Arc<dyn EventStore>) -> Arc<EventRouter> {
    Arc::new(EventRouter::new(store, 64))
}
