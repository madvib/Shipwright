---
title: "Runtime"
description: "What the runtime owns -- workspaces, sessions, events, jobs, file claims, targets, capabilities, skill vars, skill paths, and the platform database."
sidebar:
  label: "Runtime"
  order: 2
---
The runtime crate (`crates/core/runtime/`) owns all persistent state. It manages `~/.ship/platform.db` via SQLite and sqlx. No other layer touches the database.

## Module Map

The runtime's `lib.rs` declares these top-level modules:

| Module | Owns |
|--------|------|
| `workspace` | Workspace CRUD, session lifecycle, worktree management, status transitions |
| `events` | Append-only event log, event queries by time/actor/entity/action |
| `db` | Platform database schema, migrations, connections, all table modules |
| `config` | Project config (`ship.jsonc`), agents, MCP servers, hooks, statuses, gitignore |
| `agents` | Agent config, export/import, permissions, rules, skills |
| `skill_vars` | Skill variable state (get, set, list, reset) |
| `skill_paths` | Skill file path resolution |
| `catalog` | Skill/rule catalog listing and search |
| `project` | Project init, directory resolution, worktree-aware path lookup |
| `hooks` | Runtime hook trait and default implementation |
| `plugin` | Plugin trait, plugin registry |
| `log` | Project action log (file-based) |
| `security` | Security checks |
| `registry` | Registry operations |
| `fs_util` | Filesystem utilities |

## Database Layer

The `db` module contains all SQLite table access. One file per domain:

| DB Module | Tables / Concerns |
|-----------|-------------------|
| `db::workspace` | Workspace records |
| `db::workspace_state` | Workspace upsert operations |
| `db::session` | Session records |
| `db::jobs` | Job queue (CRUD, status transitions, logs) |
| `db::file_claims` | File ownership claims (atomic, first-wins) |
| `db::targets` | Targets and capabilities |
| `db::events` | Event stream persistence |
| `db::notes` | Project notes |
| `db::adrs` | Architecture decision records |
| `db::agents` | Agent config and artifact registry |
| `db::branch` | Branch metadata |
| `db::branch_context` | Branch docs and links |
| `db::kv` | General key-value store |
| `db::managed_state` | Managed state blobs |
| `db::schema` | Schema definitions |
| `db::types` | Shared DB type definitions |

The database path is always `~/.ship/platform.db`. The `ensure_db()` function runs sqlx migrations idempotently, sets WAL journal mode and foreign keys, then opens a connection.

{% aside type="tip" %}
Tests get automatic isolation. The `get_global_dir()` function detects test binaries and returns a per-thread temp directory instead of `~/.ship/`.
{% /aside %}

## Workspaces

A workspace is a unit of parallel work keyed by git branch. Three kinds:

- **imperative** -- creates a git worktree, pruned on completion
- **declarative** -- creates a git worktree, kept on completion
- **service** -- no worktree, for standing processes

Workspace operations: `create_workspace`, `activate_workspace`, `delete_workspace`, `list_workspaces`, `get_workspace`, `upsert_workspace`, `sync_workspace`, `repair_workspace`, `transition_workspace_status`, `set_workspace_active_agent`.

Workspaces resolve `.ship/` paths through the main checkout even when running in a worktree. The `get_project_dir` function follows `.git` file pointers to find the main repo's `.ship/` directory.

## Sessions

A session is one agent visit to a workspace. Lifecycle: start, work (log progress), end with summary. One active session per workspace.

Session records include: start/end timestamps, model used, progress notes, summary, files changed count, and gate result (pass/fail for gated sessions).

Key functions: `start_workspace_session`, `end_workspace_session`, `record_workspace_session_progress`, `get_active_workspace_session`, `list_workspace_sessions`, `get_workspace_session_record`.

## Events

Every state change emits an event to an append-only log. Events are never updated or deleted.

Each event has: id, timestamp, actor, entity type, action, subject, and optional details. Entity types: `Workspace`, `Session`, `Note`, `Project`, and others. Actions: `Create`, `Update`, `Delete`, `Start`, `Stop`, `Log`.

Functions: `append_event`, `append_event_with_context`, `read_events`, `read_recent_events`, `list_events_since`, `record_gate_outcome`, `list_gate_outcomes`.

## Jobs

Jobs coordinate work across agents. Status lifecycle: `pending` -> `running` -> `complete` or `failed`. Jobs can be blocked by other jobs via `blocked_by`.

Each job carries: kind, description, assignment, branch, file scope, touched files, acceptance criteria, capability link, priority, and a log.

## File Claims

File ownership prevents concurrent modification. `claim_files` is atomic and first-wins. `check_conflicts` validates before claiming. `list_claims` shows current ownership. `release_claims` frees files when a job completes.

## Targets and Capabilities

Targets are named goals. Milestones are time-bounded (v0.1). Surfaces are evergreen domains (Compiler, Studio). Targets carry `body_markdown` for strategy and context.

Capabilities are verifiable slices of a target. Status: `aspirational` -> `in_progress` -> `actual`. Evidence required to mark actual. Each has acceptance criteria, phase, file scope, priority, and assignment.

## Skill Vars

Runtime functions for skill variable state management:

- `get_skill_vars(ship_dir, skill_id)` -- merged variable state (defaults + user + project)
- `set_skill_var(ship_dir, skill_id, key, value)` -- set a variable value
- `list_skill_vars(ship_dir)` -- all skills with configurable variables
- `reset_skill_vars(ship_dir, skill_id)` -- clear all overrides for a skill

Variable values are stored in the platform DB's KV store with keys like `skill_vars:{id}`, `skill_vars.local:{ctx}:{id}`, and `skill_vars.project:{ctx}:{id}`.

## Skill Paths

`read_skill_paths(ship_dir)` resolves all skill file paths for the active project, walking the skills directory tree.

## How Transport Uses the Runtime

The CLI maps clap flags to runtime function calls and formats results for terminal output. The MCP server wraps runtime calls as tool handlers, adding project directory resolution and resource notification. Both layers are thin -- validation, state management, and business logic live in the runtime.
