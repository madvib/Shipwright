//! Async projection — spawns projections as tokio tasks reading from broadcast.

use std::sync::Arc;

use anyhow::Result;
use sqlx::SqliteConnection;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::events::EventEnvelope;

/// Async counterpart to [`super::Projection`]. Same contract — declares event
/// types and applies them to a read model — but designed to run as a long-lived
/// tokio task consuming from a broadcast channel.
pub trait AsyncProjection: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn event_types(&self) -> &[&str];
    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()>;
    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()>;
}

/// Spawn a projection as a tokio task that reads from a broadcast receiver.
///
/// - Filters events by `event_types()` before calling `apply()`.
/// - Opens a fresh DB connection per event via `db_opener`.
/// - Logs errors without panicking.
/// - Handles `Lagged(n)` by logging and continuing.
/// - Returns when the channel is closed.
pub fn spawn_projection<P: AsyncProjection>(
    projection: Arc<P>,
    mut rx: broadcast::Receiver<EventEnvelope>,
    db_opener: impl Fn() -> Result<SqliteConnection> + Send + 'static,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if !projection
                        .event_types()
                        .contains(&event.event_type.as_str())
                    {
                        continue;
                    }
                    match db_opener() {
                        Ok(mut conn) => {
                            if let Err(e) = projection.apply(&event, &mut conn) {
                                eprintln!(
                                    "[async-projection:{}] apply error for {}: {}",
                                    projection.name(),
                                    event.event_type,
                                    e,
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[async-projection:{}] db_opener error: {}",
                                projection.name(),
                                e,
                            );
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!(
                        "[async-projection:{}] lagged by {} events, continuing",
                        projection.name(),
                        n,
                    );
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::Duration;

    // ── Mock projection ──────────────────────────────────────────────────

    struct MockProjection {
        name: &'static str,
        types: &'static [&'static str],
        applied: Arc<Mutex<Vec<String>>>,
    }

    impl AsyncProjection for MockProjection {
        fn name(&self) -> &str {
            self.name
        }

        fn event_types(&self) -> &[&str] {
            self.types
        }

        fn apply(&self, event: &EventEnvelope, _conn: &mut SqliteConnection) -> Result<()> {
            self.applied.lock().unwrap().push(event.event_type.clone());
            Ok(())
        }

        fn truncate(&self, _conn: &mut SqliteConnection) -> Result<()> {
            Ok(())
        }
    }

    fn make_event(event_type: &str) -> EventEnvelope {
        EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
    }

    fn test_db_opener() -> Result<SqliteConnection> {
        use sqlx::Connection;
        crate::db::block_on(async {
            SqliteConnection::connect("sqlite::memory:").await
        })
        .map_err(|e| anyhow::anyhow!("{e}"))
    }

    // ── Tests ────────────────────────────────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn projection_receives_matching_events() {
        let applied = Arc::new(Mutex::new(Vec::new()));
        let proj = Arc::new(MockProjection {
            name: "test",
            types: &["workspace.created"],
            applied: applied.clone(),
        });
        let (tx, rx) = broadcast::channel(16);
        let _handle = spawn_projection(proj, rx, test_db_opener);

        tx.send(make_event("workspace.created")).unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(applied.lock().unwrap().len(), 1);
        assert_eq!(applied.lock().unwrap()[0], "workspace.created");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn projection_filters_non_matching() {
        let applied = Arc::new(Mutex::new(Vec::new()));
        let proj = Arc::new(MockProjection {
            name: "test",
            types: &["workspace.created"],
            applied: applied.clone(),
        });
        let (tx, rx) = broadcast::channel(16);
        let _handle = spawn_projection(proj, rx, test_db_opener);

        tx.send(make_event("session.started")).unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(applied.lock().unwrap().is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn projection_handles_lagged() {
        let applied = Arc::new(Mutex::new(Vec::new()));
        let proj = Arc::new(MockProjection {
            name: "test",
            types: &["workspace.created"],
            applied: applied.clone(),
        });

        // Small channel — overflow triggers Lagged
        let (tx, rx) = broadcast::channel(2);

        // Fill past capacity before spawning receiver task
        for _ in 0..5 {
            tx.send(make_event("workspace.created")).unwrap();
        }

        let _handle = spawn_projection(proj, rx, test_db_opener);
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Projection should recover from lag and process the remaining events
        let count = applied.lock().unwrap().len();
        assert!(count >= 1, "projection should process events after lag, got {count}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn projection_stops_on_channel_close() {
        let applied = Arc::new(Mutex::new(Vec::new()));
        let proj = Arc::new(MockProjection {
            name: "test",
            types: &["workspace.created"],
            applied: applied.clone(),
        });
        let (tx, rx) = broadcast::channel(16);
        let handle = spawn_projection(proj, rx, test_db_opener);

        drop(tx);

        // Task should complete within a reasonable time
        let result = tokio::time::timeout(Duration::from_secs(2), handle).await;
        assert!(result.is_ok(), "projection task should stop when channel closes");
    }
}
