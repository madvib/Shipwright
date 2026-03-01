+++
id = "9dSBptkS"
title = "Work Units and Specifications"
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

Non-trivial features need a written contract before implementation begins — scope, goals, non-goals, approach, open questions. Without a spec, agents and developers improvise and diverge. Shipwright treats specs as first-class documents linked to features, surfaced in agent context, and versioned in git. They are the handoff between planning and execution.

## Acceptance Criteria

- [ ] Spec CRUD: create, list, get, update via CLI and MCP
- [ ] Status via directory: `draft/`, `active/`, `archived/`
- [ ] `feature_id` and `release_id` cross-references in spec frontmatter
- [ ] `ship spec new` creates a spec and optionally links it to a feature
- [ ] `ship spec start` moves spec to `active/` (same pattern as feature start)
- [ ] Spec content injected into CLAUDE.md for the linked branch
- [ ] UI: spec list view with status filter; inline editor

## Delivery Todos

- [ ] Confirm `spec.rs` CRUD handles all status directories
- [ ] `ship spec start` / `ship spec done` lifecycle commands (backlog issue filed)
- [ ] Link spec to feature at create time (`--feature <id>`)
- [ ] Wire spec content into CLAUDE.md generation for linked features
- [ ] MCP: `list_specs`, `get_spec`, `create_spec`, `update_spec`
- [ ] UI spec views

## Notes

Spec lifecycle mirrors feature lifecycle: draft → active → archived. A spec is "active" when the implementation branch exists. "Archived" when the feature ships. The spec is the contract; the feature is the delivery vehicle. One feature, one spec — enforced by convention, not schema.
