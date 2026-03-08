+++
id = "WV9uuDCJ"
title = "Architecture Decision Records"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T21:48:10.301557+00:00"
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

ADRs capture durable architecture decisions with rationale, tradeoffs, and replacement history so teams and agents can reason from explicit decisions instead of tribal knowledge.

## Acceptance Criteria

- [x] ADR CRUD and status transitions are available in CLI/MCP/backend surfaces
- [x] ADR records persist in SQLite with status, date, and relationship fields
- [x] ADRs support links to motivating specs and superseded decisions
- [x] ADR list/detail workflows are available in the desktop planning area

## Delivery Todos

- [x] Migrate ADR persistence and status handling to SQLite-backed operations
- [x] Keep ADR markdown import/export compatibility for project artifacts
- [x] Wire ADR operations through desktop backend commands

## Current Behavior

ADR capability is operational and integrated.

## Follow-ups

- Expand decision support UX for option comparison and review workflows.