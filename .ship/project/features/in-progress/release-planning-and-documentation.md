+++
id = "2uemcUp4"
title = "Release Planning and Documentation"
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

Releases give coherence to a collection of features. Without a release container, features float unanchored and there's no clear answer to "what's in the next version." Shipwright treats releases as first-class documents — versioned by semver, linked to features and ADRs, and moving through a clear lifecycle from planned to shipped.

## Acceptance Criteria

- [ ] Release CRUD: create, list, get, update via CLI and MCP
- [ ] Version format enforced: `v{major}.{minor}.{patch}[-{pre-release}]` (semver with `v` prefix)
- [ ] Release ID = version string (`v0.1.0-alpha`) — not a short nanoid
- [ ] Filename: `v0.1.0-alpha.md` (version string is the filename)
- [ ] Status: `planned | active | shipped | archived` (directory-based or frontmatter field?)
- [ ] `feature_ids` and `adr_ids` cross-references in release frontmatter
- [ ] `ship release new v0.2.0` validates version format
- [ ] `ship release list` shows status, version, feature count
- [ ] UI: release detail showing linked features and ADRs

## Delivery Todos

- [ ] Implement semver validation in `create_release`
- [ ] Confirm release status handling (directory vs frontmatter field — align with ADR)
- [ ] `ship release` CLI subcommand (new, list, show)
- [ ] MCP: `list_releases`, `get_release`, `create_release`, `update_release`
- [ ] UI release detail view with linked features

## Notes

Release ID is the version string — not a nanoid. This is the intentional exception to the short ID convention because release version IS the identity and IS immutable. Renaming a release (e.g., v0.1.0-alpha → v0.1.0) is a legitimate operation but requires updating all `release_id` cross-references. A `ship release rename` command should handle this atomically.
