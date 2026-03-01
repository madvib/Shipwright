+++
id = "U7xtfX7R"
title = "Lightweight Note Capture"
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

Not every thought needs a spec or a feature. Developers need a frictionless place to drop a link, a rough idea, or a session summary without committing to structure. Notes are Shipwright's scratch space — freeform markdown, local-only by default, indexed but not enforced. They feed into context generation when relevant and get promoted to specs or features when they mature.

## Acceptance Criteria

- [ ] Note CRUD: create, list, get, update via CLI and MCP
- [ ] Notes live in `.ship/project/notes/` — gitignored by default
- [ ] No mandatory frontmatter beyond `title` (ID generated, `created`/`updated` stamped automatically)
- [ ] `ship note new "<title>"` opens editor or accepts stdin
- [ ] MCP: `list_notes`, `get_note`, `create_note`, `update_note`
- [ ] UI: note list with search; simple editor
- [ ] Notes can be scoped: `project` (default) or `user` (global, in `~/.ship/notes/`)

## Delivery Todos

- [ ] Confirm `note.rs` CRUD works correctly (already implemented — verify)
- [ ] `ship note` CLI subcommand (new, list, show, edit)
- [ ] User-scoped notes at `~/.ship/notes/` (deferred to V1 or implement now?)
- [ ] UI note list and editor
- [ ] Note search by title and content

## Notes

Notes are the lowest-friction primitive. No ID required in the filename — slug is sufficient. Notes are NOT yet in the UI (alpha blocker filed). The `create_note` MCP tool already exists. Filename: `{slug}.md`, no date prefix.
