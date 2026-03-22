---
name: ship-coordination
description: Use when logging progress or tracking work status in the Ship project. Covers ship MCP tools for session management and the CLI coordination protocol.
---

# Ship Coordination Protocol

This project runs 3 parallel work lanes + 1 orchestrator.
All lanes share a SQLite DB (`~/.ship/platform.db`) — global, not inside project `.ship/`.

## Lane → Orchestrator communication

Use the ship MCP tools at key milestones:

```
start_session    — when beginning a significant task
log_progress     — at each meaningful checkpoint
end_session      — when a task or subtask is complete
```

Notes and ADRs are human-facing documents. Do NOT use `create_note` for agent coordination, plans, or scratch work. Use `log_progress` or `.ship-session/` files instead.

## What to log

**start_session**: describe the task, branch, goal
**log_progress**: what was done, what's next, any discoveries that affect other lanes
**end_session**: what was completed, what was NOT done, what unblocks downstream

## Cross-lane signals

When your work unblocks another lane, use `log_progress` with a clear message:
```
log_progress "[UNBLOCKS web-import] /api/github/import contract stable — types: ProjectLibrary, errors: 404/422"
```

## Lane dependency map

```
cli-init ──────────────────────────► web-pr (PR needs working CLI)
server-auth ───────────────────────► web-auth
server-github (import endpoint) ───► web-import (swap mock for real)
server-github (PR endpoint) ────────► web-pr
```

## Commit discipline

- Push frequently (every completed subtask minimum)
- Commit messages: `feat:`, `fix:`, `test:`, `chore:` — imperative, concise
- No AI attribution in commit messages

## CLI coordination (when `ship log` lands)

Once the CLI lane implements `ship log`, prefer CLI over MCP for progress notes:
```bash
ship log "ship init: scaffolding complete, idempotent"
ship log "[UNBLOCKS web-import] /api/github/import returning ProjectLibrary shape"
```
