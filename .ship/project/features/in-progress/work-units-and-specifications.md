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

Specs and issues are sibling execution primitives under a feature — not a hierarchy. A spec is targeted context for a unit of work (the contract: scope, goals, approach). An issue is a discrete task (the execution: what to do, who does it). Both live inside the feature hub. Neither is a child of the other.

## Acceptance Criteria

- [ ] Spec CRUD fully working via CLI, MCP, and UI
- [ ] Specs displayed as siblings of issues inside the feature hub (not a separate top-level page)
- [ ] Spec status: draft → active → archived (directory-based, same pattern as features)
- [ ] `ship spec start` / `ship spec done` lifecycle commands
- [ ] Active spec narrows agent context injection (only that spec's content in CLAUDE.md)
- [ ] Issues can reference a spec (soft link, not required)
- [ ] UI: spec list with status chips inside feature hub Specs tab

## Delivery Todos

- [ ] Confirm `spec.rs` CRUD handles draft/active/archived directories
- [ ] `ship spec start` / `ship spec done` commands
- [ ] Link spec to feature at create time
- [ ] Active spec scoping in CLAUDE.md generation
- [ ] Issue → spec soft link (optional `spec_id` on issue)
- [ ] Feature hub Specs tab (part of feature-hub-ui feature)
- [ ] Remove specs from top-level nav — surface only inside feature context

## Current State

Backend: `spec.rs` CRUD fully implemented — `create_spec`, `list_specs`, `get_spec`, `update_spec`. Directory-based status (draft/active/archived) working. MCP tools exposed: `list_specs`, `get_spec`, `create_spec`, `update_spec`. `ship spec` CLI subcommand exists with basic operations.

Not yet done: `ship spec start` / `ship spec done` lifecycle commands. Active spec scoping in CLAUDE.md (currently the whole feature spec is injected, not narrowed). Feature hub UI Specs tab (blocked on feature-hub-ui). Specs still appear in top-level nav — should surface only inside feature context.

## Notes

One feature can have multiple specs — each representing a distinct unit of implementation work. Activating a spec narrows the agent's context to just that spec's symbols and files, reducing token cost on large features.
