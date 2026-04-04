---
group: Architecture
order: 2
title: Runtime
description: What the runtime owns -- workspaces, sessions, events, jobs, file claims, targets, capabilities, skill vars, skill paths, and the platform database.
---

# Runtime

The runtime crate (`crates/core/runtime/`) owns all persistent state. It manages per-actor SQLite databases under `~/.ship/actors/` and the kernel store at `~/.ship/kernel/events.db`. Legacy state remains in `~/.ship/platform.db` during migration. No other layer touches the databases directly — all access goes through the KernelRouter and ActorStore.

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

The `db` module contains all platform.db table access. One file per domain:

| DB Module | Tables / Concerns |
|-----------|-------------------|
| `db::workspace_db` | Workspace records |
| `db::workspace_state` | Workspace upsert operations |
| `db::workspace_events` | Workspace event projections |
| `db::session` | Session records |
| `db::session_events` | Session event projections |
| `db::session_drain` | Session drain operations |
| `db::actor_events` | Actor lifecycle event persistence |
| `db::artifact_registry` | Artifact registry |
| `db::events` | Event stream persistence (platform.db) |
| `db::adrs` | Architecture decision records |
| `db::branch_context` | Branch docs and links |
| `db::kv` | General key-value store |
| `db::managed_state` | Managed state blobs |
| `db::schema` | Schema definitions |
| `db::types` | Shared DB type definitions |

The platform database path is `~/.ship/platform.db`. The `ensure_db()` function runs sqlx migrations idempotently, sets WAL journal mode and foreign keys, then opens a connection. Per-actor event stores live at `~/.ship/actors/{actor_id}/events.db` and are managed by the `KernelRouter` (see Actor Model below). The kernel's own event store lives at `~/.ship/kernel/events.db`.

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

## Actor Model

The runtime uses an actor model with per-actor isolation. Every entity on the runtime — agents, apps, services — is an actor with its own event store and mailbox. Actors communicate through kernel-routed messages, never shared storage.

### Actor types

| Type | Has UI | Has MCP | Examples |
|------|--------|---------|----------|
| Agent | no | yes (client) | Code agent, design agent |
| App | yes | yes (server) | Ship Studio |
| Service | no | no | Sync, auth, docs API |

### KernelRouter

The `KernelRouter` manages actor lifecycles and message routing. It replaces the old `EventRouter` and `global_router` singleton.

- `spawn_actor(id, config)` — creates actor directory, SQLite DB, mailbox. Returns `ActorStore` + `Mailbox`.
- `route(event, ctx)` — validates, persists to kernel store, delivers to subscribed mailboxes.
- `stop_actor(id)` — flushes WAL, drops mailbox, removes subscriptions.
- `snapshot(id)` — read-only copy of actor state for migration.
- `suspend(id)` — snapshot + stop.
- `restore(snapshot)` — rebuild actor from portable snapshot.

### ActorStore

A scoped event handle bound to one actor's SQLite DB. Enforces namespace boundaries.

- Writes reject event types outside the actor's `write_namespaces`.
- Reads reject filters targeting other namespaces.
- Each actor's DB lives at `~/.ship/actors/{actor_id}/events.db`.

Actors never construct their own store. The kernel creates it via `spawn_actor`.

### Mailbox

Per-actor mpsc channel. The kernel delivers messages based on namespace subscriptions. An actor subscribing to `["studio.", "agent."]` receives all events with those prefixes.

### Namespace enforcement

Every event type is namespaced (e.g. `studio.message.visual`, `agent.task.completed`). `RESERVED_NAMESPACES` in `validator.rs` is the single source of truth. Agents cannot emit reserved prefixes through MCP tools.

### Directory layout

```
~/.ship/
  kernel/
    events.db           # System lifecycle events only
  actors/
    studio/
      events.db         # Studio's domain events
    agent-{id}/
      events.db         # Agent's scoped events
```

### Event flow

```
Agent emits event → Agent's ActorStore (persisted)
                  → KernelRouter.route() (kernel store + routing)
                  → Studio's Mailbox (delivered if subscribed)
                  → Studio's EventRelay → SSE notification to UI
```

Cross-actor queries do not exist at the actor level. The kernel can join across stores for admin/debug purposes only.

## Events

Events are immutable, append-only records. Each event has: id (ULID), event_type (namespaced), entity_id, actor, payload_json, version, correlation_id, causation_id, workspace_id, session_id, actor_id, parent_actor_id, elevated flag, and created_at timestamp.

Skills declare artifact types in their SKILL.md frontmatter (`artifacts: [html, pdf, adr]`), not events. The platform infers applicable event types from artifact types. There is no `events.json` in skills.

Agents do not have read access to the event store. They receive events through their mailbox via SSE notifications. The `list_events` MCP tool has been removed.

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
