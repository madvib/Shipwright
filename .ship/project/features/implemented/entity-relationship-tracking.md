+++
id = "E5Tf2KNN"
title = "Entity Relationship Tracking"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T21:48:11.143096+00:00"
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

Ship's planning model depends on stable relationships between entities (feature/spec/release/ADR/workspace/session). Relationship integrity is required for trustworthy context compilation and auditability.

## Acceptance Criteria

- [x] Entity records include stable IDs and link fields across planning objects
- [x] Workspace and spec flows carry feature/release linkage when available
- [x] Runtime and module operations resolve entities by ID/reference robustly
- [x] UI surfaces linked IDs/metadata for planning navigation

## Delivery Todos

- [x] Normalize ID-driven relationships across feature/release/spec/ADR/workspace entities
- [x] Add reference resolution helpers in CRUD/service layers
- [x] Maintain linkage through migration/import/update paths

## Current Behavior

Core relationship tracking is working and used in day-to-day planning flows.

## Follow-ups

- Add first-class relationship integrity diagnostics and graph-oriented introspection APIs.