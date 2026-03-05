+++
id = "ATeRtCPJ"
title = "Release Gate: Agent Config Matrix + Docs Sync"
created = "2026-03-04T03:01:06.866179+00:00"
updated = "2026-03-04T19:12:59+0000"
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
- [x] Added docs-sync regeneration coverage for skill and prompt updates:
  - `CLAUDE.md` reflects updated skill content after repeated post-checkout regeneration
  - `GEMINI.md` reflects updated mode prompt content after repeated export regeneration
- [x] Expanded multi-provider docs-sync parity for Codex/agents outputs:
  - `AGENTS.md` prompt output updates correctly after repeated Codex export regeneration
  - Codex provider context + `.agents/skills/*/SKILL.md` update and clear stale skill content after repeated post-checkout regeneration
- [x] Added runtime hardening coverage for skill lifecycle edge cases:
  - feature skill filters ignore missing IDs while preserving valid IDs
  - Codex skill export prunes stale Ship-managed skill directories after skill deletion
  - Codex skill export preserves unmanaged/manual skill directories

## Next

- [x] Add stale-skill deletion parity checks (skill removal, not just skill update) across Claude/Gemini/Codex outputs.
- [x] Release-gate hardening scope complete.
