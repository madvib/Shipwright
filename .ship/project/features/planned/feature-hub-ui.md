+++
id = "ENZ3iX6C"
title = "Feature hub UI"
status = "planned"
created = "2026-03-02T17:11:33.592910150Z"
updated = "2026-03-02T17:11:33.592910150Z"
release_id = "v0.1.0-alpha"
adr_ids = []
tags = []

[agent]
mcp_servers = []
skills = []
+++

## Why

A feature is not a markdown file — it's a container for a unit of work. The current flat markdown display buries the relationships between specs, issues, ADRs, and agent config. The hub view makes a feature feel like a workspace: everything you need to understand and execute the work in one place.

## Acceptance Criteria

- [ ] Feature detail has tabs: Overview, Specs, Issues, ADRs, Agent Config
- [ ] Overview shows Why, Acceptance Criteria (interactive), Delivery Todos (interactive, with progress %)
- [ ] Specs tab shows linked specs with status chips — not a global list
- [ ] Issues tab shows issues for this feature only — kanban within feature context
- [ ] ADRs tab shows linked decisions
- [ ] Agent Config tab shows active skills, MCP servers, permissions for this workspace
- [ ] Checking a todo/criterion writes back to SQLite immediately (no save button)
- [ ] Issues and specs are siblings — neither is a child of the other

## Delivery Todos

- [ ] Feature detail tabbed layout
- [ ] Interactive FeatureTodo checklist with completion % badge
- [ ] Interactive FeatureAcceptanceCriteria checklist
- [ ] Specs panel: linked specs with status, click to open spec detail
- [ ] Issues panel: kanban (backlog/in-progress/blocked/done) scoped to this feature
- [ ] ADRs panel: linked decisions with status chips
- [ ] Agent config panel: skills list, MCP servers, active workspace type
- [ ] Remove issues from top-level nav (or demote to secondary)
- [ ] Feature list view shows todo completion % per feature

## Notes

Issues leave the global kanban. That page either goes away or becomes a lightweight cross-feature view for power users. The primary issues experience is inside the feature.
