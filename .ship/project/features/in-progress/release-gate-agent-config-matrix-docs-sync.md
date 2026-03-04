+++
id = "ATeRtCPJ"
title = "Release Gate: Agent Config Matrix + Docs Sync"
created = "2026-03-04T03:01:06.866179+00:00"
updated = "2026-03-04T07:05:24+0000"
tags = []
+++

## Progress

- [x] Added runtime matrix coverage for MCP server filtering precedence:
  - active mode filter + feature filter intersection
  - feature model override while preserving project providers when feature providers are empty
- [x] Added docs-sync e2e coverage proving rule markdown updates propagate to regenerated `CLAUDE.md`.
- [x] Full `cargo test -p e2e` green with docs-sync test included.
- [x] Added CLI/export boundary e2e coverage for mode hook/permission propagation:
  - `ship config export --target claude` writes mode `permissions` + `hooks` into `~/.claude/settings.json`
  - non-Claude target exports do not mutate Claude settings

## Next

- [ ] Add docs-sync assertions for skill/prompt updates across repeated regeneration cycles.
