+++
title = "Platform"
owners = ["crates/core/", "crates/modules/", "crates/packages/"]
profile_hint = "rust-runtime"
+++

# Platform

Runtime primitives — sessions, workspaces, jobs, permissions, events. The stable layer everything else builds on.

## Actual
- [x] Job queue — create, update, list, log
- [x] Workspace lifecycle — create, activate, complete, handoff
- [x] Session lifecycle — start, log progress, end
- [x] Notes and ADRs
- [x] MCP server — ship:// protocol, all core tools

## Aspirational
- [ ] `touched_files` per job — file ownership, atomic claim, conflict detection
- [ ] `assigned_to` + `risk_level` on jobs — human inbox, approval tiers
- [ ] `target` field on jobs — links job to milestone capability
- [ ] Cloud job queue — Docs API / D1 backend for multi-device coordination
- [ ] Event log — append-only, queryable session records
- [ ] `~/.ship/` spec v1 — credentials, worktree path, device identity, global config
- [ ] Capability tracking — first-class capability records, delta computation
