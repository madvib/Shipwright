+++
id = "DjBDefkA"
title = "Feature Planning and Documentation"
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

A feature is the primary unit of work in Shipwright — it links a branch to a release, carries acceptance criteria and delivery todos, and configures the agent environment for that work. Features are also the input to the feature catalog: a machine-readable index of what the product does. Every capability in Shipwright is a feature, including Shipwright's own.

## Acceptance Criteria

- [ ] Feature CRUD: create, list, get, update, start, done via CLI and MCP
- [ ] Status via directory: `planned/`, `in-progress/`, `implemented/`, `deprecated/`
- [ ] `release_id` and `spec_id` cross-references in frontmatter
- [ ] `[agent]` block in frontmatter: skills, mcp_servers, optional model override
- [ ] `ship feature new` creates with title → slug → short ID
- [ ] `ship feature start <file>` sets branch, moves to `in-progress/`
- [ ] `ship feature done <file>` moves to `implemented/`
- [ ] Feature catalog: `list_features` with status filter, readable by agents and UI
- [ ] UI: feature list with Kanban-style status view

## Delivery Todos

- [ ] `ship feature start/switch` — encapsulate branch creation (backlog issue filed)
- [ ] Feature catalog MCP tool (`get_feature_catalog`)
- [ ] CLAUDE.md generation pulls feature Why, AC, and Todos
- [ ] UI feature list and detail views
- [ ] `ship feature new` validates slug uniqueness

## Notes

The `[agent]` block is a partial `AgentConfig` override applied on top of the active mode. Fields: `skills`, `mcp_servers`. Model override is possible but discouraged in the template — let the mode set it. Feature filename is `{slug}.md` — no date prefix. ID (8-char nanoid) is in frontmatter only.
