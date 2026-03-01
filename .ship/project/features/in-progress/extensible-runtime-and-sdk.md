+++
id = "A5nhFQVq"
title = "Extensible Runtime and SDK"
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

Shipwright's value grows with the document types and workflows it understands. A monolithic design means every new primitive requires touching core code. A module system lets first-party and eventually third-party modules register document types, MCP tools, CLI commands, and UI contributions — making Shipwright extensible without forking. First-party modules are compiled in for alpha; the module trait is the extension point for later.

## Acceptance Criteria

- [ ] `crates/modules/` structure: `project/`, `workflow/`, `agents/`, `git/` sub-crates
- [ ] Each module compiles into the runtime and registers its document types, CRUD functions, and CLI commands
- [ ] `crates/runtime` is the pure foundation — types, I/O, state — with no module-specific logic
- [ ] Module trait defined (even if not yet dynamically dispatched)
- [ ] Git module wired: hook install, post-checkout handler, CLAUDE.md + .mcp.json generation
- [ ] No plugin loading at runtime for alpha (compile-time only)

## Delivery Todos

- [ ] Finalize `crates/modules/` scaffold (backlog issue: `scaffold-crates-modules`)
- [ ] Define module trait in `crates/runtime` or a shared crate
- [ ] Migrate git hook logic fully into `crates/modules/git`
- [ ] Document the module contribution model for V1 third-party extension
- [ ] Verify all modules compile cleanly after `crate-restructure` ADR

## Notes

Premium modules (GitHub Sync, Agent Runner, Team Sync) are compiled in and entitlement-gated — not runtime loaded. This is deliberate: single binary, instant unlock on license purchase, no installation complexity. Third-party dynamic modules are a V2+ concern once the module trait has scar tissue from real use.
