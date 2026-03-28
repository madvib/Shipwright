//! Projection trait and event bus.

use anyhow::Result;
use sqlx::SqliteConnection;

use crate::events::EventEnvelope;

// Fallback if tracing is not available — errors are still visible via test output.
#[allow(unused_macros)]
macro_rules! log_error {
    ($($arg:tt)*) => { eprintln!($($arg)*) }
}

/// A projection maintains a derived read model from events.
///
/// Projections are registered with the [`EventBus`]. On each event append,
/// the bus calls [`apply`] on every projection whose [`event_types`] match.
/// The same [`apply`] path is used during replay — one code path for live
/// and rebuild.
pub trait Projection: Send + Sync {
    /// Human-readable name for logging and selective rebuild.
    fn name(&self) -> &str;

    /// Event types this projection handles.
    fn event_types(&self) -> &[&str];

    /// Apply a single event to the read model.
    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()>;

    /// Drop all derived state so replay can rebuild from scratch.
    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()>;
}

/// Dispatches events to registered projections.
pub struct EventBus {
    projections: Vec<Box<dyn Projection>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            projections: Vec::new(),
        }
    }

    pub fn register(&mut self, projection: Box<dyn Projection>) {
        self.projections.push(projection);
    }

    /// Dispatch an event to all matching projections.
    ///
    /// Called synchronously after every event append in 0.2.0.
    /// Handler errors are logged but do not fail the caller —
    /// the event is already persisted.
    pub fn dispatch(&self, event: &EventEnvelope, conn: &mut SqliteConnection) {
        for proj in &self.projections {
            if proj.event_types().contains(&event.event_type.as_str()) {
                if let Err(e) = proj.apply(event, conn) {
                    eprintln!(
                        "[projection:{}] handler failed for {}: {}",
                        proj.name(),
                        event.event_type,
                        e,
                    );
                }
            }
        }
    }

    /// Replay all events through registered projections.
    ///
    /// Truncates each projection first, then applies events in order.
    /// This is the correctness guarantee: projections are always rebuildable.
    pub fn rebuild(&self, events: &[EventEnvelope], conn: &mut SqliteConnection) -> Result<RebuildReport> {
        let mut report = RebuildReport {
            events_replayed: 0,
            projections_rebuilt: Vec::new(),
        };

        for proj in &self.projections {
            proj.truncate(conn)?;
            report.projections_rebuilt.push(proj.name().to_string());
        }

        for event in events {
            self.dispatch(event, conn);
            report.events_replayed += 1;
        }

        Ok(report)
    }

    /// List registered projection names.
    pub fn projection_names(&self) -> Vec<&str> {
        self.projections.iter().map(|p| p.name()).collect()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RebuildReport {
    pub events_replayed: usize,
    pub projections_rebuilt: Vec<String>,
}
