+++
id = "5JTT5zEB"
title = "Structured Frontmatter Schemas"
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

Frontmatter in plain text files has no runtime type enforcement — a user who hand-edits a feature file and writes `status = "done"` instead of the correct directory-based status, or uses `release = "..."` instead of `release_id`, creates silent bugs. Shipwright defines canonical schemas in Rust structs (via serde) and validates at every write boundary. The preferred surfaces (CLI/MCP/UI) are the guardrails; the schema is the contract.

## Acceptance Criteria

- [ ] All entity types have a canonical Rust struct with serde validation
- [ ] CLI and MCP tools validate input against the schema before writing
- [ ] Unknown frontmatter fields are preserved on read (not silently dropped)
- [ ] Backward-compat: serde `alias` for renamed fields (e.g. `release` → `release_id`)
- [ ] ID format: 8-char nanoid (alphanumeric, unambiguous chars)
- [ ] Release IDs: semver string (`v{major}.{minor}.{patch}[-{pre}]`), validated at create
- [ ] Template files reflect the canonical schema (no phantom fields)

## Delivery Todos

- [ ] Audit all entity structs for phantom/unused fields (e.g. `version` on Feature)
- [ ] Add semver validation to `create_release` / `ship release new`
- [ ] Verify serde aliases cover all renamed fields
- [ ] Update FEATURE.md, SPEC.md, ADR.md templates to match canonical schema
- [ ] Document the schema in a spec or ADR for user reference

## Notes

The four preferred surfaces (CLI, MCP, UI, file) all enforce schema at write time. The file surface is the weakest — we can only validate on read, not prevent bad writes. This is acceptable: the schema is the safety net for the other three; file editing is acknowledged as the least-supported path.
