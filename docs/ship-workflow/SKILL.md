---
name: ship-workflow
stable-id: ship-workflow
description: Use when working with Ship's workflow primitives — workspaces, sessions, multi-agent coordination, job dispatch, and the session lifecycle.
tags: [ship, workflow, sessions, workspaces]
authors: [ship]
---

# Ship Workflow

Use this skill when working with Ship's workflow primitives. Workspaces and sessions form the runtime state machine that tracks what agents are doing, where they are doing it, and what happened.

See `references/docs/` for the full reference:

- **Overview** -- How workspaces, sessions, and multi-agent coordination fit together.
- **Workspaces** -- Branch-based identity, creation, activation, worktrees, and file scoping.
- **Sessions** -- The session lifecycle, progress logging, events, and drain.

## Workspaces

A workspace is a branch-based identity for a unit of work. Every workspace is keyed by its git branch name and tracks:

- **Status** -- `active` or `archived`. Only active workspaces accept sessions.
- **Active agent** -- The agent profile compiled into this workspace's provider configs.
- **Worktree** -- Whether the workspace has a dedicated git worktree and its filesystem path.

Workspaces are created implicitly when a session starts on a branch, or explicitly via `create_workspace`. Activation compiles the agent's config into provider-specific output files.

## Sessions

A session is a heartbeat within a workspace. It represents one continuous unit of agent work. The lifecycle is:

1. **Start** -- `start_session` creates a session record, auto-creates the workspace if needed, compiles agent config, and resolves providers.
2. **Work** -- The agent performs its task, logging progress with `log_progress`.
3. **End** -- `end_session` closes the session, computes duration, creates an immutable session record, persists artifacts, and runs post-session hooks.

Only one active session per workspace at a time. Starting a session on a workspace that already has one returns the existing session (attach semantics).

## Multi-Agent Coordination

Ship supports multiple agents working in parallel through git worktrees:

- The **commander** operates on the main branch and dispatches work by creating workspaces with dedicated worktrees.
- **Specialist agents** each get their own worktree, branch, workspace, and session. They work in isolation with scoped file access.
- `create_workspace` sets up a git worktree, writes a `workspace.jsonc` config, and optionally assigns a preset and file scope.
- Each workspace tracks which agent is active and which providers are compiled.

## MCP Tools

| Tool | Purpose |
|------|---------|
| `start_session` | Start or attach to a session. Params: `branch`, `goal`, `agent_id`, `provider_id`. |
| `log_progress` | Append a progress note to the active session. Params: `note`, `branch`. |
| `end_session` | End the active session with summary and metadata. Params: `branch`, `summary`, `model`, `files_changed`, `gate_result`. |
| `get_session` | Get the active session for a workspace. |
| `list_sessions` | List sessions, optionally filtered by branch. Params: `branch`, `limit`. |
| `create_workspace` | Create a workspace with a git worktree. Params: `name`, `branch`, `base_branch`, `preset_id`, `file_scope`. |
| `activate_workspace` | Activate a workspace and compile its agent config. Params: `branch`, `agent_id`. |
| `list_workspaces` | List all workspaces, optionally filtered by status. |
| `complete_workspace` | Archive a workspace after work is done. |

## Events

All state changes emit append-only events. Session events include `session.started`, `session.progress`, `session.ended`, and `session.recorded`. These are never updated or deleted.

## Session Artifacts

Sessions persist artifacts to `~/.ship/projects/<slug>/sessions/<session-id>/`:

- `session.json` -- Current session snapshot.
- `timeline.ndjson` -- Append-only log of phase transitions (start, attach, end).
- `notes.ndjson` -- Append-only log of progress notes.
