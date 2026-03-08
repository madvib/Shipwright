+++
id = "yE7nvJYC"
title = "Agent Orchestration"
created = "2026-03-03T06:00:57.188148412+00:00"
updated = "2026-03-07T22:33:23.547531+00:00"
tags = []
+++

## Why

Ship needs orchestration primitives for coordinating multiple sessions/providers/workspaces beyond single-branch local execution.

## Acceptance Criteria

- [ ] Multi-session orchestration primitives exist in runtime with explicit coordination state
- [ ] Session coordination can detect conflicting file touch patterns across active sessions
- [ ] Orchestration controls are available in UI/CLI with clear operator actions
- [ ] Orchestration policy integrates with control-plane permissions

## Delivery Todos

- [ ] Define orchestration state model (session graph, ownership, conflicts)
- [ ] Add runtime coordination APIs and persistence model
- [ ] Add UI orchestration visibility and controls
- [ ] Add conflict detection and warning pipeline

## Current Scope

Planned post-alpha capability.

## Notes

This feature should build on workspace/session primitives, not bypass them.