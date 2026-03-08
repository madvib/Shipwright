+++
id = "U7xtfX7R"
title = "Lightweight Note Capture"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T21:46:36.304677+00:00"
release_id = "v0.1.0-alpha"
active_target_id = "v0.1.0-alpha"
spec_id = ""
branch = "feature/lightweight-note-capture"
tags = []

[agent]
model = "claude"
max_cost_per_session = 10.0
mcp_servers = []
skills = []
+++

## Why

Notes provide low-friction capture for ideas, findings, and session context that does not yet belong in a feature/spec/ADR. They protect momentum without forcing premature structure.

## Acceptance Criteria

- [x] Note CRUD is available in CLI/MCP/backend surfaces
- [x] Project-scoped notes are persisted and visible in desktop UI
- [x] User-scoped (global) note support exists in backend APIs
- [x] Note scope is explicit (`project` vs `user`) and enforced by operations
- [ ] Add higher-quality search and filtering across note content
- [ ] Add note-to-spec/feature promotion flow

## Delivery Todos

- [x] Implement scoped note persistence and operations
- [x] Wire notes into desktop planning views
- [x] Add tests for project/user note scope behavior
- [ ] Add note promotion and structured conversion helpers
- [ ] Improve content search ergonomics in UI

## Current Behavior

Notes are functional and persisted in DB-backed state. Remaining work is around discovery/promotion workflows to move valuable notes into structured planning entities.

## Notes

Notes should stay fast and lightweight; promotion into specs/features should be optional but easy.