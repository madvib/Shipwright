+++
id = "fahbueRy"
title = "Ship Command Line Interface"
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

The CLI is the primary interface for developers who live in the terminal. It must cover all alpha primitives (features, specs, releases, issues, ADRs, notes, skills), project management (`init`, `projects`), and agent tooling (`git`, `mode`, `mcp`, `config`). Every MCP tool should have a CLI equivalent — the two surfaces should be symmetric.

## Acceptance Criteria

- [ ] Binary name: `ship` (not `cli`)
- [ ] Commands: `init`, `feature`, `spec`, `release`, `issue`, `adr`, `note`, `skill`, `event`, `projects`, `git`, `config`, `mode`, `mcp`, `catalog`
- [ ] Each entity command has: `new`, `list`, `show`, `edit` subcommands
- [ ] Lifecycle commands: `feature start`, `feature done`, `spec start`, `spec done`
- [ ] `ship git sync` regenerates CLAUDE.md + .mcp.json for current branch
- [ ] `ship init` installs git hooks automatically
- [ ] `ship mode list` / `ship mode set <name>`
- [ ] `ship mcp list` / `ship mcp add` / `ship mcp remove`
- [ ] All commands work without a UI or MCP server running

## Delivery Todos

- [ ] `ship feature start/switch` — encapsulate branch creation (backlog issue filed)
- [ ] `ship mode` subcommand (list, set)
- [ ] `ship catalog` subcommand (list, search)
- [ ] `ship config` subcommand (get, set, list)
- [ ] `ship spec start/done` lifecycle commands (backlog issue filed)
- [ ] `ship workspace` subcommand (show, link)
- [ ] Verify `ship init` runs hook install

## Notes

CLI and MCP should be symmetric — every MCP tool has a CLI equivalent. The reverse is not required (some CLI operations like `ship init` don't need an MCP surface). Hidden commands (ghost, time, demo, migrate) still work but are not documented in `--help` for alpha.
