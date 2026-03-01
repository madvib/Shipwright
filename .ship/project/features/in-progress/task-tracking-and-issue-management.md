+++
id = "Va8CWEb4"
title = "Task Tracking and Issue Management"
created = "2026-02-28T15:56:07Z"
updated = "2026-02-28T15:56:07Z"
branch = ""
release_id = "v0.1.0-alpha"
spec_id = ""
adr_ids = []
tags = []

[agent]
model = "claude"
max_cost_per_session = 10.0
mcp_servers = []
skills = []
+++

## Why

Execution-level work needs to be tracked somewhere other than the developer's head or a separate tool. Issues are Shipwright's lightweight task primitive — scoped to a feature or spec, local-only by default, fast to create and move. They are not project management state; they are execution scratch. The Kanban flow (backlog → in-progress → done) is the full lifecycle.

## Acceptance Criteria

- [ ] Issue CRUD: create, list, get, update, move via CLI and MCP
- [ ] Status via directory: `backlog/`, `in-progress/`, `done/`
- [ ] `spec_id` and `feature_id` cross-references
- [ ] `priority` field: `critical | high | medium | low`
- [ ] Issues are gitignored by default (local-only)
- [ ] `ship issue new "<title>"` — fast creation with AI-generated description option
- [ ] `ship issue move <file> <status>` moves issue between directories
- [ ] MCP: full CRUD including `generate_issue_description`, `search_issues`
- [ ] UI: Kanban board with columns matching status directories

## Delivery Todos

- [ ] Confirm `issue.rs` handles all status directories
- [ ] `ship issue` CLI (new, list, show, move, done)
- [ ] `generate_issue_description` MCP tool (AI drafts description from title)
- [ ] Issue search by title/content
- [ ] UI Kanban board (drag-to-move between columns)
- [ ] Confirm issues are gitignored via `.ship/.gitignore`

## Notes

Issues are execution scratch — keep them cheap to create. The `generate_issue_description` MCP tool is the killer feature here: give it a title and get a structured description with context, steps, and acceptance criteria. Issues stay local by default; promote to git only for durable records (rare). No due dates, no assignees in alpha — those are complexity traps.
