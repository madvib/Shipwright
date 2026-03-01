+++
id = "WV9uuDCJ"
title = "Architecture Decision Records"
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

Architecture decisions that affect the whole project are expensive to revisit and easy to forget. ADRs give every decision a permanent home — context, rationale, and consequences captured at the moment the decision was made. Shipwright makes ADRs a first-class primitive so they're written, linked to specs, and surfaced to agents automatically rather than buried in Slack threads or commit messages.

## Acceptance Criteria

- [ ] ADR CRUD: create, list, get, update via CLI and MCP tools
- [ ] Status managed via directory: `proposed/`, `accepted/`, `rejected/`, `superseded/`, `deprecated/`
- [ ] `spec_id` cross-reference links ADR to the spec that motivated it
- [ ] `supersedes_id` links replacement ADRs
- [ ] ADRs surfaced in CLAUDE.md generation for linked features
- [ ] `ship adr list` shows status, title, date
- [ ] UI: ADR list with status filter and detail view

## Delivery Todos

- [ ] Confirm `adr.rs` CRUD handles all status directories
- [ ] `ship adr new` CLI with optional `--spec` flag
- [ ] MCP: `list_adrs`, `get_adr`, `create_adr`, `generate_adr` (AI-assisted from problem statement)
- [ ] Inject relevant ADR content into CLAUDE.md for linked branches
- [ ] UI ADR list and detail views

## Notes

ADR filename: `{slug}.md` — no date prefix, date is in `date` frontmatter field. Status is the directory, not a frontmatter field. The `generate_adr` MCP tool uses AI to draft Context/Decision/Consequences from a problem statement prompt.
