---
group: Runtime
order: 4
title: Service Mesh & Database Layer
description: Agent discovery, messaging, broadcast, and the SQLite database layer.
audience: internal
---

# Service Mesh

The mesh is a thin validator and address resolver for agent-to-agent communication. It does not route events directly — it creates `EventEnvelope` values with `target_actor_id` set and pushes them to an outbox channel. The caller drains the outbox through the kernel.

Source: `crates/core/runtime/src/services/mesh.rs`.

## MeshService

Implements `ServiceHandler`. Runs as `service.mesh` actor, subscribed to `mesh.*` events. Handles:

| Event type | Behavior |
|------------|----------|
| `mesh.register` | Add agent to registry with capabilities and `Active` status |
| `mesh.deregister` | Remove agent from registry |
| `mesh.send` | Validate target exists, emit `mesh.message` with `target_actor_id` set. If target not found, emit `mesh.send.failed` back to sender |
| `mesh.broadcast` | Emit `mesh.message` to all agents (except sender), optionally filtered by capability |
| `mesh.discover.request` | Return `mesh.discover.response` with matching agents (filterable by capability and status) |
| `mesh.status` | Update agent status (`active`, `busy`, `idle`) |

## MeshEntry

Each registered agent has:

```rust
struct MeshEntry {
    agent_id: String,
    label: String,
    capabilities: Vec<String>,
    registered_at: DateTime<Utc>,
    status: AgentStatus,  // Active | Busy | Idle
}
```

## SharedMeshRegistry

`Arc<RwLock<HashMap<String, MeshEntry>>>` — the mesh writes, REST API reads. Synchronized via `sync_shared_registry()` after every mutation. Uses `try_write` to avoid blocking the service event loop; falls back to an async write task if contended.

## Mesh spawning

`connections::spawn_mesh_service` (called once via `OnceLock`):

1. Creates an unbounded outbox channel.
2. Spawns `MeshService` as a kernel actor with config: namespace `service.mesh`, subscribe to `mesh.*`.
3. Drains the outbox in a background task, routing each event through the kernel for directed delivery to agent mailboxes.

## Connection cleanup

When an MCP session ends, `ConnectionGuard::drop` emits `mesh.deregister` and calls `kernel.stop_actor`. This removes the agent from both the kernel and the mesh registry.

---

# Database Layer

Single SQLite database at `~/.ship/platform.db`. Never inside a project directory. Source: `crates/core/runtime/src/db/`.

## Stack

- **SQLx** with compile-time checked queries.
- **SQLite** with WAL journal mode and foreign keys enabled.
- **Migrations** in `crates/core/runtime/migrations/`, managed by `sqlx::migrate!`.

## Initialization

`db::ensure_db()`:

1. Create parent directories.
2. Open connection with `create_if_missing(true)`.
3. Set `PRAGMA journal_mode = WAL` and `PRAGMA foreign_keys = ON`.
4. Run sqlx migrations (tracks applied migrations in `_sqlx_migrations`).

Idempotent — safe to call multiple times.

## Connection helpers

- `db_path()` — returns `~/.ship/platform.db`.
- `open_db()` — calls `ensure_db()`, then connects.
- `open_db_at(path)` — open a specific DB file (no migration run).
- `block_on(future)` — run a sqlx future synchronously. Uses `block_in_place` if a tokio runtime exists, otherwise creates a single-threaded runtime.

## Schema

Two layers, one database:

### Platform tables (portable runtime)

| Table | Purpose |
|-------|---------|
| `kv_state` | Namespaced key-value store (`namespace + key` primary key) |
| `workspace` | Branch-keyed unit of work |
| `workspace_session` | Time-bounded work interval within a workspace |
| `workspace_session_record` | Immutable end-of-session snapshot |
| `branch_context` | Branch-to-entity links |
| `event_log` | Append-only audit trail |
| `agent_artifact_registry` | Content-addressed compiled artifact store |
| `managed_mcp_state` | Ship-managed MCP server tracking per provider |

### Workflow tables (opinionated planning)

| Table | Purpose |
|-------|---------|
| `target` | Named goals (milestones and surfaces) |
| `capability` | Concrete requirements under targets |
| `jobs` | Agent work queue |
| `file_claim` | Batch-atomic file-path claims |
| `note` | Human-facing scratchpad |
| `adr` | Architecture decision records |

## KV store

Generic namespaced key-value store. Source: `crates/core/runtime/src/db/kv.rs`.

- `kv::set(namespace, key, value)` — upsert a JSON value.
- `kv::get(namespace, key)` — read, returns `Option<Value>`.
- `kv::delete(namespace, key)` — remove.
- `kv::list_keys(namespace)` — list all keys in a namespace, sorted.

Namespaces are isolated — same key in different namespaces are independent entries.

## DB modules

| Module | Purpose |
|--------|---------|
| `events.rs` | Platform event queries (list_all, list_recent, list_since, query_since) |
| `actor_events.rs` | Actor-scoped event persistence |
| `session.rs` | Session CRUD and queries |
| `session_events.rs` | Session lifecycle event handling |
| `session_drain.rs` | Session drain/cleanup |
| `workspace_db.rs` | Workspace CRUD |
| `workspace_events.rs` | Workspace event projection |
| `workspace_state.rs` | Workspace runtime state (tmux session, worktree) |
| `branch_context.rs` | Branch-to-entity linking |
| `adrs.rs` | ADR CRUD |
| `artifact_registry.rs` | Content-addressed artifact store |
| `managed_state.rs` | MCP server state tracking |
| `kv.rs` | Key-value store |

## Test isolation

Tests get automatic isolation via `get_global_dir()`'s test-binary detection, which returns a per-thread temporary directory instead of `~/.ship/`.
