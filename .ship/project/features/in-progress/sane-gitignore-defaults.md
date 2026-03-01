+++
id = "hSkxpvxR"
title = "Sane Gitignore Defaults"
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

Shipwright creates files that should never be committed (local issues, notes, agent configs, SQLite DBs) and files that should always be committed (features, specs, ADRs, releases). Getting this wrong leads to developers accidentally committing private notes or losing issue state on clone. The `.ship/.gitignore` must be correct by default — users shouldn't have to think about it.

## Acceptance Criteria

- [ ] `.ship/.gitignore` written by `ship init` with correct defaults
- [ ] Gitignored by default: `workflow/issues/`, `project/notes/`, `agents/`, `events.ndjson`, `ship.db`, `*.db`
- [ ] Committed by default: `project/features/`, `project/releases/`, `project/adrs/`, `workflow/specs/`, `ship.toml`, `project/VISION.md`
- [ ] `ship git-config get/set` allows per-category override (already implemented via MCP)
- [ ] `ship init` does not clobber existing `.gitignore` entries
- [ ] Root `.gitignore` updated to ignore generated files (`CLAUDE.md`, `.mcp.json`, `SHIPWRIGHT.md`)

## Delivery Todos

- [ ] Verify `ship init` writes `.ship/.gitignore` correctly
- [ ] Confirm `CLAUDE.md`, `.mcp.json`, `SHIPWRIGHT.md` are in root `.gitignore`
- [ ] Test: clone a repo, run `ship init`, verify correct files are tracked/ignored
- [ ] `ship git-config` CLI surface (currently MCP-only)

## Notes

`CLAUDE.md` and `.mcp.json` are generated per-branch by the post-checkout hook. They must be gitignored — committing them would create merge conflicts and expose branch-specific config in the main branch. The pre-commit hook should warn if these are staged.
