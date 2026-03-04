+++
id = "H6tnM9P2"
title = "CLI startup import — idempotent incremental sync"
status = "done"
created = "2026-03-03T06:11:57.220594614+00:00"
updated = "2026-03-03T20:48:00.000000000+00:00"
author = ""
tags = []
+++

## Overview

`ensure_imported()` in `crates/cli/src/lib.rs` runs on every CLI invocation, unconditionally
re-importing all entity files from markdown into SQLite. This causes O(n) filesystem I/O on
every command and noisy output. Three distinct bugs compound to create the problem.

Now that SQLite is the canonical store (refactor complete 2026-03-03), this migration bridge
should run at most once per install, not on every command.

---

## Bug A — Feature import has no skip check

**File:** `crates/modules/project/src/feature/migration.rs`

`import_features_from_files` calls `upsert_feature_db` unconditionally for every `.md` file
it finds. It never checks whether the feature is already in SQLite. Result: `count` is always
equal to the total number of feature files, so it always logs "Imported N features" even when
nothing changed.

Compare to `crates/modules/project/src/adr/migration.rs` line 85, which correctly skips
existing records:

```rust
// ADR does this right:
if super::db::get_adr_db(ship_dir, &adr.metadata.id)?.is_some() {
    continue;
}
```

Feature migration has no equivalent guard.

**Fix:** Add an existence check in `import_features_from_files` before calling
`upsert_feature_db`. If a feature with the same ID already exists in SQLite, skip it.
Also check release migration — likely the same issue.

```rust
// After parsing feature from markdown:
if let Ok(feature) = Feature::from_markdown(&content) {
    // Skip if already in SQLite
    if get_feature_db(ship_dir, &feature.metadata.id)?.is_some() {
        continue;
    }
    upsert_feature_db(ship_dir, &feature, &status)?;
    count += 1;
}
```

**Effort:** ~30 minutes. High impact — immediately silences the log noise.

---

## Bug B — `ensure_imported` called multiple times per command

**File:** `crates/cli/src/lib.rs`

`get_project_dir_cli()` is a path resolver with a side effect baked in:

```rust
fn get_project_dir_cli() -> Result<PathBuf> {
    let project_dir = get_project_dir(None)?;
    ensure_imported(&project_dir)?;   // side effect in a resolver
    Ok(project_dir)
}
```

Many commands call `get_project_dir_cli()` multiple times in the same execution path.
For example, `skill get` with `Effective` scope calls it twice (lines 826 and 833).
Write commands call it once to get the dir, then again after the write to read back.
Each call triggers a full `ensure_imported` scan.

**Fix:** Remove `ensure_imported` from `get_project_dir_cli()`. It is a pure resolver —
it should return a path and nothing else. Call `ensure_imported` exactly once at the top
of the CLI entry point, before command dispatch, guarded by Bug C's migration flag.

```rust
// In the CLI run() or main dispatch:
if let Ok(dir) = get_project_dir(None) {
    ensure_imported_once(&dir).ok();   // guarded by migration_complete flag
}

// get_project_dir_cli() becomes pure:
fn get_project_dir_cli() -> Result<PathBuf> {
    get_project_dir(None)
}
```

**Effort:** ~30 minutes.

---

## Bug C — `ensure_imported` runs on every invocation even after migration is complete

**File:** `crates/cli/src/lib.rs`, `crates/modules/project/src/*/migration.rs`

Even with Bug A and B fixed, `ensure_imported` still scans the filesystem on every command
to check for new files. This is O(n) I/O for a condition that is almost never true once
the initial migration has run.

SQLite is now canonical. Markdown files are generated exports, not the source of truth.
New entities are created via CLI/MCP → they go straight to SQLite. There is no ongoing
workflow that creates markdown files independently of the runtime. The migration bridge
served its purpose and should not run repeatedly.

**Fix:** Add a `migration_meta` table to SQLite tracking per-entity-type migration state.

```sql
CREATE TABLE IF NOT EXISTS migration_meta (
    entity_type TEXT PRIMARY KEY,
    migrated_at TEXT NOT NULL,
    file_count   INTEGER NOT NULL DEFAULT 0
);
```

On startup, check this table. If all entity types have a `migrated_at` entry, skip
`ensure_imported` entirely. On first run (table empty or entity type missing), run the
import for that type, then insert the row.

`ship migrate` CLI command (already hidden, currently exists) clears the table and
re-runs all imports. Add `--force` flag for explicit re-migration.

```rust
fn ensure_imported_once(project_dir: &Path) -> Result<()> {
    let types = ["adr", "feature", "release", "note_project", "note_user"];
    for entity_type in &types {
        if migration_complete(project_dir, entity_type)? {
            continue;
        }
        let count = match *entity_type {
            "adr"           => import_adrs_from_files(project_dir)?,
            "feature"       => import_features_from_files(project_dir)?,
            "release"       => import_releases_from_files(project_dir)?,
            "note_project"  => import_notes_from_files(NoteScope::Project, Some(project_dir))?,
            "note_user"     => import_notes_from_files(NoteScope::User, None)?,
            _               => 0,
        };
        mark_migration_complete(project_dir, entity_type, count)?;
        if count > 0 {
            println!("[ship] Migrated {} {}s from files to SQLite", count, entity_type);
        }
    }
    Ok(())
}
```

**Effort:** ~2 hours including SQLite migration script.

---

## Release migration — same bug as feature

**File:** `crates/modules/project/src/release/migration.rs`

Audit `import_releases_from_files` for the same unconditional upsert pattern as features.
Apply the same existence-check fix.

---

## Implementation Order

1. **Bug A** — add skip check to feature (and release) migration. Ship immediately.
2. **Bug C** — add `migration_meta` table, gate `ensure_imported` behind it.
3. **Bug B** — remove side effect from `get_project_dir_cli()`, call once at entry.

Fix in this order so each step is independently testable.

---

## Acceptance Criteria

- `ship feature list` on a fully-migrated project prints no import log lines
- `ship feature create "X"` logs import output at most once (first run only)
- `get_project_dir_cli()` contains no calls to `ensure_imported` or any I/O side effect
- `migration_meta` table exists in SQLite after first CLI invocation
- `ship migrate` clears `migration_meta` and re-runs all imports
- `ship migrate --force` works even if already migrated
- All existing CLI tests pass
- Feature and release migration functions return 0 when all entities already in SQLite

---

## Files Touched

```
crates/cli/src/lib.rs
crates/modules/project/src/feature/migration.rs
crates/modules/project/src/release/migration.rs
crates/runtime/src/state_db.rs          (migration_meta table + helpers)
```
