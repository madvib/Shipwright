+++
id = "KLrvRWB8"
title = "Scoped Workspaces"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T21:40:41.298868+00:00"
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

Workspaces are the runtime execution boundary for Ship. They bind a branch to planning context and agent configuration so the same branch can be resumed, inspected, and operated without redoing setup every session.

## Acceptance Criteria

- [x] Workspace records persist in SQLite with branch identity, type, status, and timestamps
- [x] Workspace records can link to feature/spec/release IDs
- [x] `ship workspace create/list/switch/sync/archive` flows are available in CLI
- [x] Workspace mode override and active mode inheritance are supported
- [x] Worktree path support is available for branch-scoped execution environments
- [x] Workspace state is visible in the desktop command-center view

## Delivery Todos

- [x] Persist workspace model in runtime state DB
- [x] Add CLI workspace surface for create/list/switch/sync/archive
- [x] Add Tauri/UI bindings for workspace state and activation
- [x] Add workspace mode override plumbing end-to-end
- [x] Add worktree-aware workspace creation/open behavior

## Current Behavior

Ship treats workspace as the long-lived execution context for a branch. Sessions are short-lived runs inside that workspace. Activation and sync use workspace state as the source of truth for provider config generation.

## Follow-ups

- Improve lifecycle analytics and audit visualization in UI
- Add stronger cross-workspace conflict signaling for overlapping file edits