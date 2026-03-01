+++
id = "sfCQ4U3x"
title = "Embedded Project and Global Database"
created = "2026-02-28T15:56:07Z"
updated = "2026-02-28T15:56:07Z"
branch = ""
release_id = "v0.1.0-alpha"
spec_id = ""
adr_ids = []
tags = []

[agent]
model = "claude"
max_cost_per_session = 10.0
mcp_servers = []
skills = []
+++

## Why

Shipwright needs two tiers of runtime state: project-scoped (workspace sessions, branch cache, UI prefs) and global (project registry, auth, model cache). File-based storage for this churn would be noisy and fragile. SQLite provides transactional, local, fast storage with no external dependencies — consistent with the local-first architecture.

## Acceptance Criteria

- [ ] Project DB: `.ship/ship.db` — active mode, workspace sessions, branch cache, UI preferences
- [ ] Global DB: `~/.ship/shipwright.db` — project registry, entitlements, model cache
- [ ] `state_db.rs` handles all SQLite reads/writes; no raw SQL outside this module
- [ ] Schema migrations run automatically on startup via embedded migration files
- [ ] Project DB is gitignored; global DB is user-local
- [ ] `get_workspace` / `upsert_workspace` work correctly across worktrees

## Delivery Todos

- [ ] Verify migration system handles schema evolution cleanly
- [ ] Confirm global DB path resolves correctly on all platforms
- [ ] Worktree isolation: each worktree reads the same project DB (shared) or gets its own (TBD)
- [ ] Expose project registry read/write through MCP (`list_projects`, `open_project`)

## Notes

Per the file-vs-SQLite ADR: if a human would read, diff, or commit it — it's a file. Everything else is SQLite. Workspace, sessions, branch cache, UI state = SQLite. Vision, features, specs, ADRs = files. No exceptions.
