+++
id = "ydmqCLwp"
title = "Workspace model — types and lifecycle"
status = "planned"
created = "2026-03-02T17:11:20.834710028Z"
updated = "2026-03-02T17:11:20.834710028Z"
release_id = "v0.1.0-alpha"
adr_ids = []
tags = []

[agent]
mcp_servers = []
skills = []
+++

## Why

The current feature/branch model is flat — every branch is treated identically. Workspace types (feature, refactor, experiment, hotfix) give meaningful structure to different kinds of work. Lifecycle states (planned → active → idle → review → merged → archived) make branch status visible and actionable without reading git.

## Acceptance Criteria

- [ ] WorkspaceType gates valid lifecycle transitions (experiment never reaches merged)
- [ ] Status transitions wired to git hooks (post-checkout sets active, post-merge sets merged)
- [ ] ship workspace create/switch/list/sync/archive CLI commands working
- [ ] 1:1 Feature-to-Workspace relationship enforced
- [ ] Refactor/experiment/hotfix workspaces work without a linked feature
- [ ] Workspace activation recompiles agent context if context_hash is stale

## Delivery Todos

- [ ] WorkspaceType enum: feature | refactor | experiment | hotfix
- [ ] WorkspaceStatus enum: planned | active | idle | review | merged | archived
- [ ] Transition validation (allowed_transitions per type)
- [ ] Wire post-checkout hook → set workspace active
- [ ] Wire post-merge hook → set workspace merged
- [ ] ship workspace CLI subcommand
- [ ] Context hash invalidation on skill/rule/MCP change
- [ ] UI: workspace status indicators (◉ active, ○ idle, ⚑ needs attention, ✓ ready)

## Notes

Refactor and experiment types are the answer to "I want to open a workspace but I'm not adding a feature." They get full agent context compilation, just no release linkage.
