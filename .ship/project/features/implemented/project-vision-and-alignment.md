+++
id = "PapezM4g"
title = "Project Vision and Alignment"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T21:48:07.835499+00:00"
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

Project vision is the canonical intent layer for a Ship project. It gives humans and agents a stable statement of purpose, scope, and constraints so execution decisions stay aligned over time.

## Acceptance Criteria

- [x] Vision content is persisted and retrievable through Ship runtime APIs
- [x] UI can read and update project vision content
- [x] Vision is available for context compilation and workspace-facing UX
- [x] Vision exists as a single canonical project artifact (no parallel duplicates)

## Delivery Todos

- [x] Wire vision read/update commands through Tauri backend
- [x] Ensure vision is surfaced in project planning views
- [x] Keep vision as a singleton project artifact

## Current Behavior

Vision is treated as a first-class planning artifact in Ship and is accessible in UI/backend flows.

## Follow-ups

- Add dedicated `ship vision` CLI surface for direct edit/show workflows.