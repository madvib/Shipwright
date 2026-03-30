//! Projection startup — spawn all three projections as async consumers.
//!
//! Call `spawn_all_projections` once at runtime init. The returned handles
//! keep the tasks alive and expose failure counters for monitoring.
//!
//! Each task subscribes to the platform broadcast channel and applies
//! matching events to platform.db. If a projection's apply() fails, the
//! error is logged and a failure counter is incremented — the event stays
//! persisted regardless.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use sqlx::SqliteConnection;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::events::EventEnvelope;
use crate::events::router::EventRouter;
use crate::projections::{ActorProjection, AsyncProjection, SessionProjection, WorkspaceProjection};

/// Handle for a running async projection task.
pub struct ProjectionHandle {
    pub name: String,
    pub handle: JoinHandle<()>,
    pub failure_count: Arc<AtomicU64>,
}

/// Spawn a projection with a failure counter.
///
/// Unlike the basic `spawn_projection`, this version increments `failure_count`
/// on every apply() or db_opener() error, enabling monitoring at runtime.
pub fn spawn_with_failure_counter<P: AsyncProjection>(
    projection: Arc<P>,
    mut rx: broadcast::Receiver<EventEnvelope>,
    db_opener: impl Fn() -> Result<SqliteConnection> + Send + 'static,
) -> ProjectionHandle {
    let name = projection.name().to_string();
    let failure_count = Arc::new(AtomicU64::new(0));
    let counter = failure_count.clone();

    let handle = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if !projection.event_types().contains(&event.event_type.as_str()) {
                        continue;
                    }
                    match db_opener() {
                        Ok(mut conn) => {
                            if let Err(e) = projection.apply(&event, &mut conn) {
                                counter.fetch_add(1, Ordering::Relaxed);
                                eprintln!(
                                    "[projection:{}] apply error for {}: {}",
                                    projection.name(),
                                    event.event_type,
                                    e,
                                );
                            }
                        }
                        Err(e) => {
                            counter.fetch_add(1, Ordering::Relaxed);
                            eprintln!(
                                "[projection:{}] db_opener error: {}",
                                projection.name(),
                                e,
                            );
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!(
                        "[projection:{}] lagged by {} events, continuing",
                        projection.name(),
                        n,
                    );
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    ProjectionHandle { name, handle, failure_count }
}

/// Spawn all three projections as async consumers of the platform broadcast.
///
/// Returns a handle per projection. Store the handles for the lifetime of
/// the runtime — dropping them cancels the tasks.
///
/// Projections write to platform.db. `open_db()` resolves the path at each
/// event; safe for production where the path is stable.
pub fn spawn_all_projections(router: &EventRouter) -> Vec<ProjectionHandle> {
    vec![
        spawn_with_failure_counter(
            Arc::new(WorkspaceProjection::new()),
            router.subscribe_platform(),
            crate::db::open_db,
        ),
        spawn_with_failure_counter(
            Arc::new(SessionProjection::new()),
            router.subscribe_platform(),
            crate::db::open_db,
        ),
        spawn_with_failure_counter(
            Arc::new(ActorProjection::new()),
            router.subscribe_platform(),
            crate::db::open_db,
        ),
    ]
}
