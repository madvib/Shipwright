---
group: Workflow
title: Sessions
order: 3
---

# Sessions

A session is a heartbeat within a workspace. It represents one continuous unit of agent work -- from the moment an agent starts a task to the moment it finishes or is interrupted.

## Session fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique session ID (nanoid). |
| `workspace_id` | string | ID of the parent workspace. |
| `workspace_branch` | string | Branch name of the parent workspace. |
| `status` | `active` or `ended` | Current session state. |
| `started_at` | datetime | When the session began. |
| `ended_at` | datetime (optional) | When the session ended. |
| `agent_id` | string (optional) | Agent profile active during this session. |
| `primary_provider` | string (optional) | AI provider used (e.g. "claude", "gemini"). |
| `goal` | string (optional) | What the session set out to accomplish. |
| `summary` | string (optional) | What was accomplished (set on end). |
| `stale_context` | bool | Whether the workspace config changed after this session started. |
| `config_generation_at_start` | i64 (optional) | Config generation counter at session start, used for staleness detection. |
| `session_record_id` | string (optional) | ID of the immutable record created on end. |

## Starting a session

```
start_session({
  branch: "feat/new-ui",       // optional, defaults to current git branch
  goal: "implement sidebar",   // optional
  agent_id: "web-lane",        // optional, overrides workspace agent
  provider_id: "claude"        // optional, must be in the agent's allowed providers
})
```

`start_session` performs these steps:

1. Resolves the branch. If omitted, uses the current git branch.
2. Auto-creates the workspace record if it does not exist.
3. Activates the workspace if it is not already active.
4. Sets the agent if `agent_id` is provided.
5. Checks for an existing active session. If one exists, returns it (attach semantics).
6. Resolves allowed providers from the agent config and validates the requested provider.
7. Compiles the workspace context (agent config into provider-specific output).
8. Emits a `session.started` event and creates the session record.
9. Persists a session artifact snapshot.

Only one active session per workspace. The attach-on-existing behavior prevents duplicate sessions when an agent restarts or reconnects.

## Logging progress

```
log_progress({
  note: "Completed sidebar layout, starting event handlers",
  branch: "feat/new-ui"    // optional
})
```

Progress notes are appended to the session as `session.progress` events. They serve two purposes:

- **Observability** -- Other agents (like a commander) can monitor what specialists are doing.
- **Persistence** -- Notes are written to `notes.ndjson` in the session artifacts directory.

`log_progress` requires an active session. If no session exists on the branch, it returns an error asking you to call `start_session` first.

Notes cannot be empty. The runtime trims whitespace and rejects blank strings.

## Ending a session

```
end_session({
  branch: "feat/new-ui",
  summary: "Implemented sidebar with navigation and event handlers",
  model: "claude-opus-4-20250514",
  files_changed: 5,
  gate_result: "pass"     // optional, result of any quality gate
})
```

`end_session` performs these steps:

1. Finds the active session on the workspace.
2. Sets the session status to `ended` and records the end timestamp.
3. Computes duration from `started_at` to `ended_at`.
4. Emits a `session.ended` event with summary, duration, and gate result.
5. Creates an immutable `WorkspaceSessionRecord` with the session's metadata.
6. Emits a `session.recorded` event.
7. Persists the final session artifact snapshot and appends to the timeline.
8. Runs any configured post-session hooks (hooks with trigger `session_end` or `stop`).

## Session records

When a session ends, the runtime creates a `WorkspaceSessionRecord` -- an immutable snapshot of the session's outcome. Records include:

| Field | Description |
|-------|-------------|
| `summary` | What was accomplished. |
| `duration_secs` | How long the session lasted. |
| `provider` | AI provider used. |
| `model` | Model ID used. |
| `agent_id` | Agent profile that was active. |
| `files_changed` | Number of files modified. |
| `gate_result` | Quality gate outcome (pass/fail/skip). |

Records are write-once. They provide the historical trail for what each session produced.

## Staleness detection

Sessions track the workspace's config generation counter at start time. If the workspace config is recompiled while a session is active (e.g., because the agent profile changed), the session is marked `stale_context: true`. This signals that the agent may be operating with outdated instructions.

## Events

Sessions emit four event types:

| Event | When | Payload |
|-------|------|---------|
| `session.started` | Session begins | goal, workspace_id, agent_id, provider, config_generation |
| `session.progress` | Agent logs progress | message |
| `session.ended` | Session closes | summary, duration_secs, gate_result, updated_workspace_ids |
| `session.recorded` | Immutable record created | record_id, summary, duration, provider, model, files_changed |

Events are append-only. They are never updated or deleted.

## Session artifacts

The runtime persists session artifacts to `~/.ship/projects/<slug>/sessions/<session-id>/`:

- **`session.json`** -- JSON snapshot of the session, updated at start, attach, and end.
- **`timeline.ndjson`** -- Append-only log of phase transitions with timestamps.
- **`notes.ndjson`** -- Append-only log of progress notes with timestamps.

These files exist for debugging and observability. They are not the source of truth -- the event store is.

## Post-session hooks

Hooks configured with trigger `session_end` or `stop` run after a session ends. They receive `SHIP_SESSION_ID` and `SHIP_SESSION_BRANCH` as environment variables. Hook failures are logged to stderr but do not fail the session end operation.

## Querying sessions

```
get_session({ branch: "feat/new-ui" })          // active session for a branch
list_sessions({ branch: "feat/new-ui", limit: 10 })  // recent sessions
list_sessions({ limit: 20 })                    // all recent sessions across workspaces
```

`list_sessions` returns up to 100 sessions (default 20), ordered by recency. Results include staleness annotations and session record IDs where available.
