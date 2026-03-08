+++
id = "9dSBptkS"
title = "Work Units and Specifications"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T21:46:14.226272+00:00"
release_id = "v0.1.0-alpha"
active_target_id = "v0.1.0-alpha"
spec_id = ""
branch = "feature/work-units-and-specifications"
tags = []

[agent]
model = "claude"
max_cost_per_session = 10.0
mcp_servers = []
skills = []
+++

## Why

Specs are execution-grade work units linked to active workspaces and features. They provide scoped implementation context that sits between feature intent and session-level execution.

## Acceptance Criteria

- [x] Spec CRUD is available across CLI, MCP, and desktop backend
- [x] Spec records are persisted in SQLite with workspace/feature/release linkage
- [x] Spec creation can inherit context from the active workspace
- [x] Spec list/detail UI is available
- [ ] Add explicit spec lifecycle commands (`start/done`) where needed
- [ ] Tighten spec visibility so it is centered in feature/workspace flows (not stray top-level UX)
- [ ] Add stronger session-to-spec foldback automation

## Delivery Todos

- [x] Implement and test workspace-aware spec creation behavior
- [x] Migrate spec entity to SQLite-backed persistence
- [x] Expose spec operations through desktop backend bindings
- [ ] Complete spec lifecycle command ergonomics and UI affordances
- [ ] Enforce execution loop: spec updates after workspace session end

## Current Behavior

Specs are working as a DB-backed entity with workspace-linked context inheritance. Remaining work is lifecycle ergonomics and making spec usage more central to the core workspace loop.

## Notes

Specs are long-lived enough to preserve decision history but small enough to map to concrete implementation passes.