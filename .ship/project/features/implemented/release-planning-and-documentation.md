+++
id = "2uemcUp4"
title = "Release Planning and Documentation"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T21:48:08.709381+00:00"
release_id = "v0.1.0-alpha"
active_target_id = "v0.1.0-alpha"
spec_id = ""
branch = ""
tags = []

[agent]
model = "claude"
max_cost_per_session = 10.0
mcp_servers = []
skills = []
+++

## Why

Releases are planning containers that communicate what is shipping, what remains, and how feature work maps to a concrete milestone.

## Acceptance Criteria

- [x] Release CRUD is available across CLI, MCP, and UI backend surfaces
- [x] Release identity is version-based and stable
- [x] Release status and metadata are persisted in SQLite
- [x] Features can link to release targets via release IDs
- [x] Release hub and release detail views are available in UI

## Delivery Todos

- [x] Migrate release persistence to SQLite-backed module APIs
- [x] Keep release markdown export/import flow aligned with DB state
- [x] Wire release list/detail/update operations in desktop backend

## Current Behavior

Release planning is operational and integrated with features.

## Follow-ups

- Finalize target-based UX (release as one target type) without adding user-facing complexity.