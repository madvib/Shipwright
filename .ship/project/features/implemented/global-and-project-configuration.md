+++
id = "pUccKNoA"
title = "Global and Project Configuration"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T22:33:20.559999+00:00"
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

Agent control-plane behavior depends on clean separation between project-shared config and user-local preferences.

## Acceptance Criteria

- [x] Project scope config and user scope config are both supported
- [x] Scope-aware config read/write flows are available across surfaces
- [x] Provider declarations and mode defaults are consumed from canonical config paths
- [x] Runtime config APIs provide deterministic effective config resolution

## Delivery Todos

- [x] Keep project/user config scope explicit in runtime config APIs
- [x] Ensure CLI/UI/MCP surfaces use the same config read/write backend
- [x] Normalize provider/mode settings before export/activation

## Current Behavior

Global/project config separation is operational and used by agent export, workspace activation, and planning surfaces.

## Follow-ups

- Improve config UX for scoped overrides and conflict explanation.