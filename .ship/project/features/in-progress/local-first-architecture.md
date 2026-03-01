+++
id = "WZJa9Cdj"
title = "Local-First Architecture"
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

Developer tools that require accounts and internet connections create friction, lock-in, and failure modes. Shipwright is a tool developers trust with their project structure — it must work offline, on private networks, and without ever sending data to a server. Local-first is not a feature; it's the architecture. Cloud is additive, not foundational.

## Acceptance Criteria

- [ ] `ship init` works with zero network access
- [ ] All core operations (CRUD, hook, CLAUDE.md generation) work fully offline
- [ ] No account required for any alpha feature
- [ ] Data never leaves the machine without explicit user action
- [ ] Works in air-gapped environments
- [ ] Cloud/sync is opt-in and additive — removing it doesn't break local operation

## Delivery Todos

- [ ] Audit all runtime paths for accidental network calls
- [ ] Confirm MCP server operates purely over stdio (no HTTP endpoints in alpha)
- [ ] Document data residency guarantees in ship.toml or README
- [ ] Ensure catalog (community skills / official servers) degrades gracefully offline (embedded static list)

## Notes

The free tier is a complete product, not a trial. Local-first is permanent, not a stepping stone to SaaS. Cloud v2 design: event log is the replication unit; SQLite is the read model; frontmatter files are the git export. All three are local by default; cloud sync pushes the event log selectively.
