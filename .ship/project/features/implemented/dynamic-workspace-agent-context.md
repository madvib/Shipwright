+++
id = "eZKy7Tym"
title = "Dynamic Workspace Agent Context"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T21:40:53.766335+00:00"
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

Workspace activation must compile and apply the correct agent context every time branch context changes. This is the bridge between planning state and real provider configuration used by live sessions.

## Acceptance Criteria

- [x] Workspace activation compiles provider-specific context from runtime state
- [x] `workspace sync` reconciles branch/worktree state and refreshes context metadata
- [x] `workspace repair` can detect and repair context/config drift
- [x] Session start can target a primary provider and persist session metadata
- [x] UI surfaces activation errors and provider/session state
- [x] Worktree-aware activation writes config to the correct workspace path

## Delivery Todos

- [x] Implement provider matrix + activation pipeline in runtime
- [x] Add sync/repair runtime APIs and CLI/Tauri surfaces
- [x] Persist context hash / compiled-at metadata for workspace state
- [x] Wire workspace session start/end with provider selection
- [x] Expose runtime diagnostics to UI instead of silent failures

## Current Behavior

Activating a workspace compiles agent config for the workspace branch and updates provider exports. Session start uses the active workspace context and selected provider. Sync/repair paths are available when config drifts or branch context changes.

## Follow-ups

- Add provider-native restart/resume hooks when compiled context changes
- Add stricter policy gates per provider before session launch