+++
id = "5b736e36-eab9-4219-a1e5-92b5f403ebad"
title = "Manual E2E Debug Script — Worktree"
created = "2026-02-27T23:10:36.370161036Z"
updated = "2026-02-27T23:10:36.370161036Z"
tags = []
+++

## Purpose

Quick manual smoke test for the worktree CLAUDE.md auto-resolve flow.
Useful when debugging `ship git sync` behavior from a worktree without `SHIP_DIR`.

## Usage

```bash
SHIP=/path/to/target/debug/ship bash .ship/project/notes/scripts/worktree-debug.sh
# or default (uses dev build):
bash .ship/project/notes/scripts/worktree-debug.sh
```

## What it tests

- `ship init` creates full `.ship/` structure
- `ship feature create` with a branch sets frontmatter correctly
- `git worktree add` creates a worktree
- `ship git sync` from the worktree root (no `SHIP_DIR`) finds `.ship/` via walk-up
- CLAUDE.md is written to the **worktree root**, not the main repo root

## Expected output

```
[ship] loaded feature 'Auth Flow' for: claude
Exit code: 0
-rw-r--r-- ... CLAUDE.md       ← exists in worktree root
```
