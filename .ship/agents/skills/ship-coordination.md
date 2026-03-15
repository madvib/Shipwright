---
name: ship-coordination
description: Use when logging progress, leaving cross-lane notes, or tracking work status in the Ship project. Covers ship MCP tools for session/note management and the CLI coordination protocol.
---

# Ship Coordination Protocol

This project runs 3 parallel work lanes + 1 orchestrator.
All lanes share a SQLite DB (`.ship/state/`) synced via Syncthing across machines.

## Lane → Orchestrator communication

Use the ship MCP tools at key milestones:

```
start_session    — when beginning a significant task
log_progress     — at each meaningful checkpoint
end_session      — when a task or subtask is complete
create_note      — for cross-lane signals or blockers
```

## What to log

**start_session**: describe the task, branch, goal
**log_progress**: what was done, what's next, any discoveries that affect other lanes
**end_session**: what was completed, what was NOT done, what unblocks downstream

## Cross-lane signals

When your work unblocks another lane, `create_note` with:
- Title: `[UNBLOCKS web-import] /api/github/import contract stable`
- Content: the exact contract (types, errors, URL) the downstream agent needs

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
