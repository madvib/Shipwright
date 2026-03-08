+++
id = "FqLNtSDx"
title = "Unified Agent Configuration Standard"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T22:33:19.736754+00:00"
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

Ship needs one canonical agent configuration model so modes, workspace context, provider exports, and feature overrides all compose predictably.

## Acceptance Criteria

- [x] Canonical `AgentConfig` schema is used across runtime config surfaces
- [x] Mode configuration resolves into the same model used for provider exports
- [x] Feature agent overrides are merged with deterministic precedence
- [x] Resolved config generation is available for workspace/session operations
- [x] Config write paths validate schema shape and normalize provider IDs

## Delivery Todos

- [x] Consolidate config resolution logic in runtime agent config module
- [x] Wire mode + feature override merge behavior into export/activation flows
- [x] Add tests for provider/mode/filter merge edge cases
- [x] Remove stale duplicated config definitions in legacy paths

## Current Behavior

Unified config resolution is active and powers workspace activation, provider export, and mode switching.

## Follow-ups

- Expand policy-level validation messages for invalid mode/provider combinations.