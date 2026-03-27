---
title: Tool Reference
description: Complete reference for every Ship MCP tool with parameter tables, grouped by domain.
section: reference
order: 2
---

# Tool Reference

Every Ship MCP tool with its parameters. Parameter names map directly to JSON fields in tool calls.

## Project

### open_project
Register the active project. Call once per session.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | string | yes | Absolute path to the project root |

## Notes and ADRs

### create_note
Create a project note -- cross-session records for decisions, blockers, or signals.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | yes | Note title |
| `content` | string | no | Markdown content |
| `branch` | string | no | Git branch to associate with this note |

### update_note
Replace the content of an existing note.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Note ID (nanoid returned by `create_note`) |
| `content` | string | yes | Full replacement markdown content |
| `scope` | string | no | `project` (default) or `user` |

### create_adr
Record an Architecture Decision Record for decisions with real alternatives.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | yes | Short, specific title (e.g. "Use D1 for cloud state") |
| `decision` | string | yes | Full decision text: context, alternatives, consequences |

## Workspaces

Each workspace corresponds to a git worktree (imperative/declarative) or a standing service branch.

### create_workspace
Create a new workspace and its git worktree.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Human-readable workspace name |
| `kind` | string | yes | `imperative`, `declarative`, or `service` |
| `branch` | string | no | Branch name (derived from name if omitted) |
| `base_branch` | string | no | Base branch for worktree (default: `main`) |
| `file_scope` | string | no | Paths this workspace may edit (e.g. `crates/`) |
| `preset_id` | string | no | Agent profile to activate in this workspace |

### activate_workspace
Mark a workspace active. `branch` (string, required), optional `agent_id`.

### complete_workspace
Mark workspace complete. `workspace_id` (required), `summary` (required, written to handoff.md), optional `prune_worktree` (bool, default true for imperative).

### list_workspaces
Optional `status` filter: `active`, `idle`, `archived`.

### list_stale_worktrees
Optional `idle_hours` (integer, default: 24).

### set_agent
Optional `id` (string) -- omit to clear active agent profile.

## Sessions

One active session per workspace. Sessions track agent activity.

### start_session
Begin a session at the start of each agent visit.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `branch` | string | no | Workspace branch (resolves from git if omitted) |
| `goal` | string | no | What this visit aims to accomplish |
| `agent_id` | string | no | Agent profile override |
| `provider_id` | string | no | Provider override (e.g. `claude`, `codex`) |

### end_session
End the current session with a summary.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `branch` | string | no | Workspace branch (resolves from git if omitted) |
| `summary` | string | no | What was accomplished, what changed |
| `files_changed` | integer | no | Count of files modified |
| `model` | string | no | Model ID used during the session |
| `gate_result` | string | no | Gate result: `pass`, `fail`, or null |

### log_progress
Record a progress checkpoint. Requires an active session.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `note` | string | yes | What you did, decided, or got blocked on |
| `branch` | string | no | Workspace branch (resolves from git if omitted) |

## Jobs

The job queue coordinates work across agents and machines.

### create_job
Create a new job in the queue.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `kind` | string | yes | `feature`, `fix`, `infra`, `test`, `review`, etc. |
| `description` | string | yes | What needs to be done |
| `branch` | string | no | Git branch for this job |
| `assigned_to` | string | no | Agent id or workspace |
| `requesting_workspace` | string | no | Workspace that requested this job |
| `priority` | integer | no | Higher runs first (default 0) |
| `blocked_by` | string | no | Job id that must complete first |
| `touched_files` | string[] | no | Files this job intends to touch |
| `file_scope` | string[] | no | Paths the agent may touch -- enforced by gate |
| `acceptance_criteria` | string[] | no | Checklist items for the gate |
| `capability_id` | string | no | Capability this job delivers |
| `symlink_name` | string | no | Human-readable worktree label |

### update_job
Update status, assignment, or priority.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Job id (or unique prefix) |
| `status` | string | no | `pending`, `running`, `complete`, `failed` |
| `assigned_to` | string | no | Reassign to a different agent |
| `priority` | integer | no | Update scheduling priority |
| `blocked_by` | string | no | Set or clear blocking job id |
| `touched_files` | string[] | no | Replace the touched files list |

### list_jobs
List jobs. Optional filters: `status` (`pending`/`running`/`complete`/`failed`), `branch`.

### append_job_log
Append a log message to a job.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `job_id` | string | yes | Job id |
| `message` | string | yes | Log message |
| `level` | string | no | `info`, `warn`, or `error` |

File ownership shorthand: `append_job_log(job_id, "touched: apps/web/src/foo.ts")` registers that file as owned by this job.

### claim_file
Claim exclusive file ownership. `job_id` (required), `path` (required, relative to project root). Atomic, first-wins.

### get_file_owner
Look up which job owns a file. `path` (string, required).

## Skills

### list_skills
List installed skills. Optional `query` (string) -- substring filter on id, name, or description.

## Targets and Capabilities

Targets are milestones or product surfaces. Capabilities are verifiable features belonging to a target.

### create_target

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `kind` | string | yes | `milestone` or `surface` |
| `title` | string | yes | Short title |
| `description` | string | no | Longer description |
| `goal` | string | no | One-line north star goal |
| `status` | string | no | `active`, `planned`, `complete`, `frozen` (default: `active`) |
| `phase` | string | no | Current phase label |
| `due_date` | string | no | ISO 8601 date (e.g. "2026-06-01") |
| `body_markdown` | string | no | Long-form markdown body |
| `file_scope` | string[] | no | File/directory paths owned by this target |

### list_targets
List all targets. Optional `kind` filter: `milestone` or `surface`.

### get_target
Get a target with its capability progress board. Takes `id` (string, required).

### update_target
Patch-style update. `id` (required). Optional: `title`, `description`, `goal`, `status`, `phase`, `due_date`, `body_markdown`, `file_scope` (string[]).

### create_capability
Add a capability to a target.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `target_id` | string | yes | Target this capability belongs to |
| `title` | string | yes | Capability title |
| `milestone_id` | string | no | Milestone this is required for |
| `phase` | string | no | Phase grouping (e.g. "bootstrap", "core") |
| `acceptance_criteria` | string[] | no | Checklist items that define "done" |
| `file_scope` | string[] | no | File/directory scope |
| `assigned_to` | string | no | Agent or workspace id |
| `priority` | integer | no | Lower numbers run first (default 0) |

### update_capability
Patch-style update. `id` (required). Optional: `title`, `status` (`aspirational`/`in_progress`/`actual`), `phase`, `priority`, `acceptance_criteria` (string[]), `file_scope` (string[]), `assigned_to`.

### mark_capability_actual
Mark a capability as delivered. Evidence is required -- never mark without proof.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Capability id |
| `evidence` | string | yes | Test name, commit hash, or observable behavior |

### list_capabilities
Filter by `target_id`, `milestone_id`, `status` (`aspirational`/`in_progress`/`actual`), or `phase`. All optional.

### delete_capability
Remove a capability. Takes `id` (string, required).

## Events

### list_events
Query the append-only event log.

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `since` | string | no | ISO 8601 or relative: `1h`, `24h`, `7d` |
| `actor` | string | no | Filter by actor (substring) |
| `entity` | string | no | `workspace`, `session`, `note`, `adr`, etc. |
| `action` | string | no | `create`, `update`, `delete`, `start`, `stop`, etc. |
| `limit` | integer | no | Max results (default 50, max 200) |
