+++
id = "ydmqCLwp"
title = "Workspace model — types and lifecycle"
created = "2026-03-02T17:11:20.834710028Z"
updated = "2026-03-03T04:33:27.134103+00:00"
release_id = "v0.1.0-alpha"
adr_ids = []
tags = []
+++

## Why

The current feature/branch model is flat — every branch is treated identically. Workspace types (feature, refactor, experiment, hotfix) give meaningful structure to different kinds of work. Lifecycle states (planned -> active -> idle -> review -> merged -> archived) make branch status visible and actionable without reading git.

Workspace is a core runtime concern and should remain in runtime, not moved into a workflow module. Git integration remains modular: hooks and git-specific reconciliation belong in modules.

## Acceptance Criteria

- [x] Architecture direction locked: workspace lifecycle/state lives in runtime as a first-class concern
- [x] Git integration remains modular (hooks + checkout/worktree reconciliation in modules)
- [ ] WorkspaceType gates valid lifecycle transitions (experiment never reaches merged)
- [ ] Status transitions wired to git hooks (post-checkout sets active, post-merge sets merged)
- [ ] `ship workspace create/switch/list/sync/archive` commands working
- [ ] 1:1 Feature-to-Workspace relationship enforced
- [ ] Refactor/experiment/hotfix workspaces work without a linked feature
- [ ] Workspace activation recompiles agent context only when context_hash is stale
- [ ] Workspace suite UI uses register + detail layout with dense status indicators

## Delivery Todos

- [ ] WorkspaceType enum: feature | refactor | experiment | hotfix
- [ ] WorkspaceStatus enum: planned | active | idle | review | merged | archived
- [ ] Transition validation matrix per workspace type
- [ ] Runtime workspace service: list/get/create/activate/sync/archive
- [ ] Wire post-checkout and sync paths to workspace state transitions
- [ ] Add activation timing telemetry and cached compile path (<800ms target)
- [ ] Add workspace-first command palette switcher with activation action
- [ ] Show workspace status chips in Feature/Release hubs (quick activate only)

## Notes

Specs are first-class work units inside workspaces (closer to commit-sized execution slices). Issues are intentionally de-emphasized for now and should not drive page IA decisions.