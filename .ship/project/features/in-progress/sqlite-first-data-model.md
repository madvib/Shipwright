+++
id = "beUJ4VtG"
title = "SQLite-first data model"
created = "2026-03-02T17:11:10.062517276Z"
updated = "2026-03-07T21:49:49.075937+00:00"
release_id = "v0.1.0-alpha"
active_target_id = "v0.1.0-alpha"
branch = "feature/sqlite-first-data-model"
tags = []

[agent]
mcp_servers = []
skills = []
+++

## Why

As Ship grows, markdown-only planning state becomes brittle. SQLite-first modeling makes relationships, analytics, sync, and UI behavior deterministic while preserving markdown exports where needed.

## Acceptance Criteria

- [x] Core planning entities (features/specs/releases/ADRs/notes) are persisted in SQLite-backed flows
- [x] Structured checklist state for features is stored and consumed from DB tables
- [x] Workspace/session lifecycle state is persisted in runtime DB
- [x] Markdown read views are derived/exported artifacts rather than sole source of truth
- [ ] Complete removal of legacy markdown-dependent write paths
- [ ] Finalize canonical storage policy per entity (DB text vs generated markdown)

## Delivery Todos

- [x] Migrate major planning/runtime entities to SQLite-backed operations
- [x] Add regression coverage for DB/file reconciliation bugs
- [x] Shift feature checklist rendering to DB-backed structured fields
- [ ] Eliminate remaining dual-write edge cases and legacy assumptions
- [ ] Document final canonical storage matrix for launch docs

## Current Behavior

Ship is mostly SQLite-first for runtime and planning operations. Remaining work is hardening and final cleanup of residual markdown-era assumptions.

## Notes

This feature stays in-progress until storage semantics are fully consistent across all surfaces.