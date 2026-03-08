+++
id = "WZJa9Cdj"
title = "Local-First Architecture"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T21:49:47.787492+00:00"
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

Local-first is the baseline operating model for Ship: planning, execution, and agent context should work without cloud dependencies.

## Acceptance Criteria

- [x] Core workflows run without network access (`init`, planning CRUD, workspace/session operations)
- [x] No account is required for v0.1.0-alpha local usage
- [x] Project/runtime state remains local unless user explicitly enables sync/export
- [x] MCP and CLI surfaces operate against local state by default

## Delivery Todos

- [x] Keep runtime dependencies local-only for alpha core paths
- [x] Validate local initialization + planning flows in offline/dev scenarios
- [x] Ensure cloud-oriented features remain optional extensions

## Current Behavior

Ship is operational as a local-first desktop/CLI/MCP tool. Planned sync/cloud capabilities are additive and not required for the core loop.

## Follow-ups

- Add explicit offline-mode diagnostics in `doctor` and UI health views.