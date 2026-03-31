//! Actor cursors — track per-actor delivery position for replay.

use anyhow::Result;
use std::path::Path;

use crate::db::{block_on, open_db_at};

/// Tracks the last event delivered to an actor, keyed by label.
#[derive(Debug, Clone)]
pub struct ActorCursor {
    /// The actor's stable label.
    pub actor_label: String,
    /// The ID of the last event delivered, or None if first spawn.
    pub last_event_id: Option<String>,
}

/// Ensure the cursor table exists in the kernel DB.
pub(crate) fn init_cursor_table(kernel_db_path: &Path) -> Result<()> {
    let mut conn = open_db_at(kernel_db_path)?;
    block_on(async {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS actor_cursors (
                actor_label  TEXT PRIMARY KEY NOT NULL,
                last_event_id TEXT NOT NULL
            )",
        )
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}
