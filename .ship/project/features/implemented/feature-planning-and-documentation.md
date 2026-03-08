+++
id = "DjBDefkA"
title = "Feature Planning and Documentation"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T21:48:09.502989+00:00"
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

Features are Ship's primary planning unit: each feature captures intent, execution readiness, release linkage, and documentation state for a capability slice.

## Acceptance Criteria

- [x] Feature CRUD/start/done flows are available in CLI and backend surfaces
- [x] Feature metadata and checklist structure are persisted in SQLite
- [x] Feature body content and status are surfaced in desktop planning views
- [x] Feature documentation records are tracked with status/revision metadata
- [x] Features can link to release/spec/workspace context

## Delivery Todos

- [x] Migrate feature persistence and lifecycle operations to SQLite-backed module
- [x] Ensure metadata updates preserve markdown body content
- [x] Add regression tests for content preservation on update/move
- [x] Add docs status/revision support in feature detail UX

## Current Behavior

Feature planning is first-class across runtime, CLI, and UI.

## Follow-ups

- Add stronger completion guardrails directly in feature detail interactions.