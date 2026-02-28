+++
id = "708bc1e7-301b-45c0-b12d-e1a2e19d7a6b"
title = "Alpha Dogfood — End-to-End Workflow Validation"
status = "in-progress"
release_id = "v0.1.0-alpha.md"
branch = "feature/alpha-dogfood"
created = "2026-02-27T01:07:43.559571144Z"
updated = "2026-02-27T01:07:43.559571144Z"
adr_ids = []
tags = ["alpha", "dogfood", "workflow"]

[agent]
skills = [{id = "task-policy"}]
mcp_servers = [{id = "ship"}]
+++

## Why

Validate the full alpha workflow end-to-end using Shipwright itself as the test project. Every gap found becomes a real issue. Every friction point informs the UX.

## Acceptance Criteria

- [ ] `ship init` installs git hooks automatically
- [ ] CLI surface covers all alpha primitives (feature, spec, release, note, skill)
- [ ] MCP server management surfaced in CLI
- [ ] Branch checkout triggers CLAUDE.md + .mcp.json generation
- [ ] Issue workflow (backlog → in-progress → done) is clean and fast via MCP tools
- [ ] `ship git sync` regenerates context on demand

## Delivery Todos

- [ ] File issues for every gap found during dogfood
- [ ] Fix install-hooks on init
- [ ] Add MCP server management to CLI
- [ ] Validate git hook → CLAUDE.md end-to-end
- [ ] Run e2e test suite and fix failures
