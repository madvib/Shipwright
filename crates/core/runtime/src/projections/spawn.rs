//! Projection startup utilities.
//!
//! `spawn_with_failure_counter` spawns an async projection that reads from a
//! mailbox channel, increments a failure counter on errors, and exits cleanly
//! when the channel closes.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use sqlx::SqliteConnection;
use tokio::task::JoinHandle;

use crate::events::mailbox::Mailbox;
use crate::projections::AsyncProjection;

/// Handle for a running async projection task.
pub struct ProjectionHandle {
    pub name: String,
    pub handle: JoinHandle<()>,
    pub failure_count: Arc<AtomicU64>,
}

/// Spawn a projection consuming from a `Mailbox` with a failure counter.
///
/// Increments `failure_count` on every `apply()` or `db_opener()` error.
pub fn spawn_with_failure_counter<P: AsyncProjection>(
    projection: Arc<P>,
    mut mailbox: Mailbox,
    db_opener: impl Fn() -> Result<SqliteConnection> + Send + 'static,
) -> ProjectionHandle {
    let name = projection.name().to_string();
    let failure_count = Arc::new(AtomicU64::new(0));
    let counter = failure_count.clone();

    let handle = tokio::spawn(async move {
        while let Some(event) = mailbox.recv().await {
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
    });

    ProjectionHandle { name, handle, failure_count }
}
