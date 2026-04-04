---
name: ship-coordination
stable-id: ship-coordination
audience: internal
description: Use when logging progress or tracking work status in the Ship project. Covers ship MCP tools for session and workspace management.
---

# Ship Coordination Protocol

Agents coordinate through MCP tools backed by a per-project SQLite database. The orchestrator (Commander) monitors state and routes work.

## Session lifecycle

Use the ship MCP tools at key milestones:

```
start_session    — when beginning a significant task (requires active workspace)
log_progress     — at each meaningful checkpoint within a session
end_session      — when a task or subtask is complete (include summary)
get_session      — check if a session is active on the current branch
list_sessions    — review recent sessions (filterable by branch)
```

## Workspace tools

```
activate_workspace  — activate a workspace for the current branch
create_workspace    — create a new workspace (sets up git worktree)
list_workspaces     — list all workspaces (filterable by status)
complete_workspace  — finalize a workspace with a handoff summary
set_agent           — assign an agent to the active workspace
```

## What to log

**start_session**: the task, branch, and goal.
**log_progress**: what was done, what is next, any discoveries that affect other workspaces.
**end_session**: what was completed, what was not done, what unblocks downstream.

Notes and ADRs are human-facing documents. Do not use `create_note` for agent coordination, plans, or scratch work. Use `log_progress` or `.ship-session/` files instead.

## Cross-workspace signals

When your work unblocks another workspace, use `log_progress` with a clear message:
```
log_progress "[UNBLOCKS web-import] /api/github/import contract stable — types: ProjectLibrary, errors: 404/422"
```

## Commit discipline

- Push frequently (every completed subtask minimum).
- Commit messages: `feat:`, `fix:`, `test:`, `chore:` — imperative, concise.
- No AI attribution in commit messages.
