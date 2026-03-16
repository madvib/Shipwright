//! One-time migration: copy notes and ADRs from state_db (ship.db) into platform.db.
//!
//! Safe to run multiple times — uses INSERT OR IGNORE to skip already-migrated records.

use anyhow::Result;
use sqlx::Row;
use std::path::Path;

use crate::db::{block_on, open_db};
use crate::state_db::open_project_connection;

/// Summary of what was moved (or skipped) during a migration run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub notes_migrated: usize,
    pub adrs_migrated: usize,
    pub notes_skipped: usize,
    pub adrs_skipped: usize,
}

/// Copy notes and ADRs from the old `ship.db` (state_db) into the new `platform.db`.
///
/// Uses `INSERT OR IGNORE` so repeated invocations are safe — records already
/// present in `platform.db` are counted as skipped rather than overwritten.
pub fn migrate_notes_and_adrs(ship_dir: &Path) -> Result<MigrationReport> {
    // ── Read from old DB ──────────────────────────────────────────────────────
    let mut old_conn = open_project_connection(ship_dir)?;

    // Old note schema: id, title, content, tags_json, scope, created_at, updated_at
    let old_notes = block_on(async {
        sqlx::query(
            "SELECT id, title, content, tags_json, scope, created_at, updated_at FROM note",
        )
        .fetch_all(&mut old_conn)
        .await
    })?;

    // Old adr schema: id, title, status, date, context, decision, tags_json,
    //                 spec_id, supersedes_id, created_at, updated_at
    let old_adrs = block_on(async {
        sqlx::query(
            "SELECT id, title, status, date, context, decision, tags_json,
                    supersedes_id, created_at, updated_at
             FROM adr",
        )
        .fetch_all(&mut old_conn)
        .await
    })?;

    // ── Write into new DB ─────────────────────────────────────────────────────
    let mut new_conn = open_db(ship_dir)?;

    let mut notes_migrated = 0usize;
    let mut notes_skipped = 0usize;

    for row in &old_notes {
        let id: String = row.get(0);
        let title: String = row.get(1);
        let content: String = row.get(2);
        let tags_json: String = row.get(3);
        // Old DB uses `scope` (e.g. "project"); map to `branch` (nullable) in new DB.
        // Scope values like "project" are not valid branch names — store as NULL.
        let scope: String = row.get(4);
        let branch: Option<String> = if scope == "project" || scope.is_empty() {
            None
        } else {
            Some(scope)
        };
        let created_at: String = row.get(5);
        let updated_at: String = row.get(6);

        let result = block_on(async {
            sqlx::query(
                "INSERT OR IGNORE INTO note
                   (id, title, content, tags_json, branch, synced_at, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, NULL, ?, ?)",
            )
            .bind(&id)
            .bind(&title)
            .bind(&content)
            .bind(&tags_json)
            .bind(&branch)
            .bind(&created_at)
            .bind(&updated_at)
            .execute(&mut new_conn)
            .await
        })?;

        if result.rows_affected() > 0 {
            notes_migrated += 1;
        } else {
            notes_skipped += 1;
        }
    }

    let mut adrs_migrated = 0usize;
    let mut adrs_skipped = 0usize;

    for row in &old_adrs {
        let id: String = row.get(0);
        let title: String = row.get(1);
        let status: String = row.get(2);
        let date: String = row.get(3);
        let context: String = row.get(4);
        let decision: String = row.get(5);
        let tags_json: String = row.get(6);
        let supersedes_id: Option<String> = row.get(7);
        let created_at: String = row.get(8);
        let updated_at: String = row.get(9);

        let result = block_on(async {
            sqlx::query(
                "INSERT OR IGNORE INTO adr
                   (id, title, status, date, context, decision, tags_json,
                    supersedes_id, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(&title)
            .bind(&status)
            .bind(&date)
            .bind(&context)
            .bind(&decision)
            .bind(&tags_json)
            .bind(&supersedes_id)
            .bind(&created_at)
            .bind(&updated_at)
            .execute(&mut new_conn)
            .await
        })?;

        if result.rows_affected() > 0 {
            adrs_migrated += 1;
        } else {
            adrs_skipped += 1;
        }
    }

    Ok(MigrationReport {
        notes_migrated,
        adrs_migrated,
        notes_skipped,
        adrs_skipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::ensure_db;
    use crate::project::init_project;
    use crate::state_db::{block_on as state_block_on, open_project_connection};
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        // Ensure both DBs are initialised.
        crate::state_db::ensure_project_database(&ship_dir).unwrap();
        ensure_db(&ship_dir).unwrap();
        (tmp, ship_dir)
    }

    fn seed_old_note(ship_dir: &std::path::Path, id: &str, title: &str) {
        let mut conn = open_project_connection(ship_dir).unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        state_block_on(async {
            sqlx::query(
                "INSERT INTO note (id, title, content, tags_json, scope, created_at, updated_at)
                 VALUES (?, ?, '', '[]', 'project', ?, ?)",
            )
            .bind(id)
            .bind(title)
            .bind(&now)
            .bind(&now)
            .execute(&mut conn)
            .await
        })
        .unwrap();
    }

    fn seed_old_adr(ship_dir: &std::path::Path, id: &str, title: &str) {
        let mut conn = open_project_connection(ship_dir).unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        state_block_on(async {
            sqlx::query(
                "INSERT INTO adr (id, title, status, date, context, decision, tags_json,
                                  supersedes_id, created_at, updated_at)
                 VALUES (?, ?, 'proposed', ?, '', '', '[]', NULL, ?, ?)",
            )
            .bind(id)
            .bind(title)
            .bind(&now)
            .bind(&now)
            .bind(&now)
            .execute(&mut conn)
            .await
        })
        .unwrap();
    }

    #[test]
    fn test_migrate_notes_and_adrs_empty() {
        let (_tmp, ship_dir) = setup();
        let report = migrate_notes_and_adrs(&ship_dir).unwrap();
        assert_eq!(report.notes_migrated, 0);
        assert_eq!(report.adrs_migrated, 0);
        assert_eq!(report.notes_skipped, 0);
        assert_eq!(report.adrs_skipped, 0);
    }

    #[test]
    fn test_migrate_notes() {
        let (_tmp, ship_dir) = setup();
        seed_old_note(&ship_dir, "note-001", "First Note");
        seed_old_note(&ship_dir, "note-002", "Second Note");

        let report = migrate_notes_and_adrs(&ship_dir).unwrap();
        assert_eq!(report.notes_migrated, 2);
        assert_eq!(report.notes_skipped, 0);

        // Verify the notes landed in platform.db.
        let notes = crate::db::notes::list_notes(&ship_dir, None).unwrap();
        assert_eq!(notes.len(), 2);
        let titles: Vec<&str> = notes.iter().map(|n| n.title.as_str()).collect();
        assert!(titles.contains(&"First Note"));
        assert!(titles.contains(&"Second Note"));
    }

    #[test]
    fn test_migrate_adrs() {
        let (_tmp, ship_dir) = setup();
        seed_old_adr(&ship_dir, "adr-001", "Use SQLite");
        seed_old_adr(&ship_dir, "adr-002", "Use Rust");

        let report = migrate_notes_and_adrs(&ship_dir).unwrap();
        assert_eq!(report.adrs_migrated, 2);
        assert_eq!(report.adrs_skipped, 0);

        let adrs = crate::db::adrs::list_adrs(&ship_dir).unwrap();
        assert_eq!(adrs.len(), 2);
    }

    #[test]
    fn test_migrate_idempotent() {
        let (_tmp, ship_dir) = setup();
        seed_old_note(&ship_dir, "note-idem", "Idempotent Note");
        seed_old_adr(&ship_dir, "adr-idem", "Idempotent ADR");

        let first = migrate_notes_and_adrs(&ship_dir).unwrap();
        assert_eq!(first.notes_migrated, 1);
        assert_eq!(first.adrs_migrated, 1);

        let second = migrate_notes_and_adrs(&ship_dir).unwrap();
        assert_eq!(second.notes_migrated, 0);
        assert_eq!(second.notes_skipped, 1);
        assert_eq!(second.adrs_migrated, 0);
        assert_eq!(second.adrs_skipped, 1);
    }

    // ── Priority 3 gap tests ──────────────────────────────────────────────────

    /// Running migrate three times produces the same final DB state each time
    /// (total record count does not grow beyond the initial migration).
    #[test]
    fn test_migrate_twice_same_result() {
        let (_tmp, ship_dir) = setup();
        seed_old_note(&ship_dir, "n-dbl-1", "Double Note 1");
        seed_old_note(&ship_dir, "n-dbl-2", "Double Note 2");
        seed_old_adr(&ship_dir, "a-dbl-1", "Double ADR 1");

        let r1 = migrate_notes_and_adrs(&ship_dir).unwrap();
        assert_eq!(r1.notes_migrated, 2);
        assert_eq!(r1.adrs_migrated, 1);

        let r2 = migrate_notes_and_adrs(&ship_dir).unwrap();
        // Second run: nothing new migrated, all skipped.
        assert_eq!(r2.notes_migrated, 0);
        assert_eq!(r2.notes_skipped, 2);
        assert_eq!(r2.adrs_migrated, 0);
        assert_eq!(r2.adrs_skipped, 1);

        // DB contents are unchanged after the idempotent second run.
        let notes = crate::db::notes::list_notes(&ship_dir, None).unwrap();
        assert_eq!(notes.len(), 2);
        let adrs = crate::db::adrs::list_adrs(&ship_dir).unwrap();
        assert_eq!(adrs.len(), 1);
    }

    /// Notes with duplicate IDs (same id already in platform.db) are skipped via
    /// INSERT OR IGNORE, not overwritten. The existing title is preserved.
    #[test]
    fn test_duplicate_note_id_is_skipped_not_overwritten() {
        let (_tmp, ship_dir) = setup();
        seed_old_note(&ship_dir, "note-dup", "Original Title");

        // First migration lands the note.
        let r1 = migrate_notes_and_adrs(&ship_dir).unwrap();
        assert_eq!(r1.notes_migrated, 1);
        assert_eq!(r1.notes_skipped, 0);

        // Simulate the old DB being updated with a new title for the same id.
        // (In practice the old DB is immutable post-migration, but the INSERT OR
        // IGNORE contract should hold regardless.)
        {
            let mut conn = open_project_connection(&ship_dir).unwrap();
            state_block_on(async {
                sqlx::query("UPDATE note SET title = 'Mutated Title' WHERE id = 'note-dup'")
                    .execute(&mut conn)
                    .await
            })
            .unwrap();
        }

        // Second migration sees the record already in platform.db — skips it.
        let r2 = migrate_notes_and_adrs(&ship_dir).unwrap();
        assert_eq!(r2.notes_migrated, 0);
        assert_eq!(r2.notes_skipped, 1);

        // The title in platform.db is still the original, not the mutated one.
        let note = crate::db::notes::get_note(&ship_dir, "note-dup").unwrap().unwrap();
        assert_eq!(note.title, "Original Title");
    }
}
