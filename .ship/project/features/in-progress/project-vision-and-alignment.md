+++
id = "PapezM4g"
title = "Project Vision and Alignment"
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

Every project needs a north star — a document that answers "what is this and why does it exist" without ambiguity. The vision is what agents read at the start of every session to understand the project's purpose, constraints, and principles. It's also what a new team member reads first. Shipwright makes vision a first-class singleton: one file, always findable, always current.

## Acceptance Criteria

- [ ] Vision lives at `.ship/project/VISION.md` — singleton, no frontmatter required
- [ ] `ship vision edit` opens the vision in the configured editor
- [ ] `get_vision` / `update_vision` MCP tools work correctly
- [ ] Vision content injected into every CLAUDE.md generation (brief excerpt or full)
- [ ] Vision template has no frontmatter — pure markdown prose
- [ ] UI: vision view and inline editor

## Delivery Todos

- [ ] Confirm `vision.rs` `get_vision` / `update_vision` point to `VISION.md` (uppercase, correct path)
- [ ] Remove stale `workflow/specs/vision.md` duplicate (it's identical to VISION.md)
- [ ] `ship vision` CLI command (show, edit)
- [ ] Wire vision into CLAUDE.md generation header
- [ ] UI vision panel

## Notes

Vision has no frontmatter. The title is the `# H1`. The `updated` field (if needed) should come from `git log`, not frontmatter. The template is intentionally minimal — what the project does (problem), who it's for (users), what success looks like (outcomes), what it won't do (non-goals). The Shipwright vision document itself (`project/VISION.md`) is the reference implementation.
