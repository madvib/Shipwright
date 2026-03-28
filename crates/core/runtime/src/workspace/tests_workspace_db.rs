//! Contract tests for per-workspace SQLite databases.
//!
//! These tests define the behavior the implementation must satisfy.
//! They reference `workspace_db_path` and `open_workspace_db` which do not
//! exist yet — intentional compile errors confirm the implementation is pending.

#[cfg(test)]
mod tests {
    use crate::db::{block_on, db_path, ensure_db, open_db_at};
    use crate::workspace::*;
    use anyhow::Result;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        // Use get_global_dir() as ship_dir so workspace DB paths are consistent
        // with the routing in insert_actor_created / insert_session_with_started_event
        // (which also derive ship_dir from get_global_dir()).
        let ship_dir = crate::project::get_global_dir().unwrap();
        let base = ship_dir.parent().unwrap().to_path_buf();
        crate::project::init_project(base).unwrap();
        ensure_db().unwrap();
        (tmp, ship_dir)
    }

    // ── test 1: path is deterministic ─────────────────────────────────────────

    #[test]
    fn workspace_db_path_is_deterministic() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let ws_id = "ws-abc123";

        let path1 = workspace_db_path(&ship_dir, ws_id);
        let path2 = workspace_db_path(&ship_dir, ws_id);

        assert_eq!(path1, path2, "workspace_db_path must be deterministic");
        let s = path1.to_string_lossy();
        assert!(s.contains(ws_id), "path must contain workspace_id, got: {s}");
        assert!(path1.to_string_lossy().ends_with(".db"), "path must end with .db, got: {s}");
        Ok(())
    }

    // ── test 2: path differs from platform.db ─────────────────────────────────

    #[test]
    fn workspace_db_is_separate_from_platform_db() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        let ws_path = workspace_db_path(&ship_dir, "ws-abc123");
        let platform_path = db_path()?;

        assert_ne!(ws_path, platform_path, "workspace db path must differ from platform.db");
        let filename = ws_path.file_name().unwrap().to_string_lossy();
        assert_ne!(
            filename, "platform.db",
            "workspace db filename must not be 'platform.db', got: {filename}"
        );
        Ok(())
    }

    // ── test 3: DB file is created on first open ───────────────────────────────

    #[test]
    fn workspace_db_created_on_first_open() -> Result<()> {
        let (_tmp, ship_dir) = setup();
        let ws_id = "ws-new-never-seen";

        let ws_path = workspace_db_path(&ship_dir, ws_id);
        assert!(!ws_path.exists(), "workspace db must not exist before open, path: {ws_path:?}");

        let _ = open_workspace_db(&ship_dir, ws_id)?;

        assert!(ws_path.exists(), "workspace db must exist after open, path: {ws_path:?}");
        assert_ne!(
            ws_path,
            db_path()?,
            "workspace db must not be platform.db"
        );
        Ok(())
    }

    // ── test 4: workspace DB has events table ─────────────────────────────────

    #[test]
    fn workspace_db_has_events_table() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        let mut conn = open_workspace_db(&ship_dir, "ws-events-table")?;
        let rows: Vec<String> = block_on(async {
            sqlx::query_scalar(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='events'",
            )
            .fetch_all(&mut conn)
            .await
        })?;

        assert_eq!(rows.len(), 1, "workspace db must have exactly one 'events' table");
        assert_eq!(rows[0], "events");
        Ok(())
    }

    // ── test 5: events table has required columns ──────────────────────────────

    #[test]
    fn workspace_db_events_table_has_required_columns() -> Result<()> {
        let (_tmp, ship_dir) = setup();

        let mut conn = open_workspace_db(&ship_dir, "ws-columns")?;
        let columns: Vec<String> = block_on(async {
            sqlx::query_scalar("SELECT name FROM pragma_table_info('events')")
                .fetch_all(&mut conn)
                .await
        })?;

        for required in &[
            "id",
            "event_type",
            "entity_id",
            "actor",
            "payload_json",
            "actor_id",
            "parent_actor_id",
            "workspace_id",
            "elevated",
            "created_at",
        ] {
            assert!(
                columns.iter().any(|c| c == required),
                "events table missing required column '{required}', found: {columns:?}"
            );
        }
        Ok(())
    }
}
