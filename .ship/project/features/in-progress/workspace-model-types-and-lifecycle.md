+++
id = "ydmqCLwp"
title = "Workspace model — types and lifecycle"
created = "2026-03-02T17:11:20.834710028Z"
updated = "2026-03-07T21:41:04.069510+00:00"
release_id = "v0.1.0-alpha"
active_target_id = "v0.1.0-alpha"
branch = "feature/workspace-model-types-and-lifecycle"
tags = []
+++

## Why

Workspace types and lifecycle state make execution status explicit across feature, refactor, hotfix, and experiment branches. This allows Ship to coordinate branch intent, activation semantics, and UI orchestration without relying on implicit git-only state.

## Acceptance Criteria

- [x] WorkspaceType supports `feature | refactor | experiment | hotfix`
- [x] WorkspaceStatus supports lifecycle states used by runtime/UI
- [x] Transition validation enforces allowed state changes by workspace type
- [x] Core CLI flows (`create/switch/list/sync/archive`) operate on lifecycle state
- [x] Non-feature workspaces can run without a linked feature
- [ ] 1:1 feature-to-workspace enforcement finalized for feature workspaces
- [ ] Hook-driven lifecycle automation (checkout/merge -> status transitions) fully enforced
- [ ] Activation cache policy based on context hash finalized and documented

## Delivery Todos

- [x] Add WorkspaceType and WorkspaceStatus enums to runtime model
- [x] Implement transition validation and runtime transition API
- [x] Expose workspace lifecycle in CLI and Tauri surfaces
- [x] Integrate workspace status indicators in workspace command-center views
- [ ] Finalize feature workspace ownership constraints
- [ ] Finalize automated status transitions from git hook events
- [ ] Add lifecycle audit telemetry for transition analytics

## Current Behavior

Lifecycle modeling is active and used throughout runtime/CLI/UI. Remaining work focuses on stronger ownership constraints and automatic lifecycle transitions from git events.

## Notes

This feature remains in-progress because policy enforcement and automation hardening are not complete yet.