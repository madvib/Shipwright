---
group: Runtime
order: 3
title: Event System
description: KernelRouter, event envelopes, actor stores, mailboxes, validators, and event types.
audience: internal
---

# Event System

The event system is the primary communication and audit mechanism. All state changes flow as `EventEnvelope` values through the `KernelRouter`, which routes them to subscribed actor mailboxes and persists kernel-scoped events.

Source: `crates/core/runtime/src/events/`.

## EventEnvelope

The universal event record. Fields:

| Field | Type | Purpose |
|-------|------|---------|
| `id` | String | Monotonic ULID (encodes insertion time) |
| `event_type` | String | Dot-namespaced type (e.g. `workspace.created`) |
| `entity_id` | String | The entity this event describes |
| `actor` | String | Who emitted it (defaults to `"ship"`) |
| `payload_json` | String | Serialized JSON payload |
| `version` | u32 | Schema version (currently 1) |
| `correlation_id` | Option | Links related events across operations |
| `causation_id` | Option | The event that caused this one |
| `workspace_id` | Option | Scoping context |
| `session_id` | Option | Scoping context |
| `actor_id` | Option | Kernel actor that emitted |
| `parent_actor_id` | Option | Parent in actor hierarchy |
| `target_actor_id` | Option | Directed delivery target |
| `elevated` | bool | Platform-scope (visible to sync) |
| `created_at` | DateTime | UTC timestamp |

Builder methods: `with_correlation`, `with_causation`, `with_context`, `with_actor_id`, `with_parent_actor_id`, `with_target`, `elevate`.

## KernelRouter

The central event router. One instance per daemon, shared across all connections via `Arc<Mutex<KernelRouter>>`.

Source: `crates/core/runtime/src/events/kernel_router.rs`.

### Directory layout

```
{base_dir}/
  kernel/
    events.db        # kernel-scope events (workspace.*, session.*, actor.*, etc.)
  actors/
    {actor_id}/
      events.db      # per-actor event store
```

### Actor lifecycle

- **`spawn_actor(id, config)`** — creates a per-actor SQLite DB, allocates a mailbox channel (capacity 256), registers namespace subscriptions. Returns `(ActorStore, Mailbox)`.
- **`route(event, ctx)`** — validates via registered `EventValidator`s, persists kernel events, delivers to all actors whose `subscribe_namespaces` match the event type prefix.
- **`stop_actor(id)`** — removes the mailbox sender (closing the channel), removes subscriptions, flushes WAL.
- **`snapshot(id)`** — reads the actor's DB bytes, event count, and last event ID. Emits `kernel.actor.snapshot`.
- **`suspend(id)`** — snapshot + stop. Emits `kernel.actor.suspended`.
- **`restore(snapshot)`** — writes DB bytes back, re-spawns the actor. Emits `kernel.actor.restored`.

### ActorConfig

```rust
struct ActorConfig {
    namespace: String,           // actor's own namespace
    write_namespaces: Vec<String>,    // prefixes the actor may write
    read_namespaces: Vec<String>,     // prefixes the actor may read
    subscribe_namespaces: Vec<String>, // prefixes that route to this actor's mailbox
}
```

### Kernel namespaces

Events with these prefixes are persisted to the kernel store: `workspace.`, `session.`, `actor.`, `gate.`, `config.`, `runtime.`, `sync.`.

## Mailbox

The receive end of a per-actor `mpsc::channel` (capacity 256). Created by `spawn_actor`. Provides `recv()` (async) and `try_recv()` (non-blocking).

## ActorStore

Per-actor SQLite database. Provides scoped event storage with write/read namespace enforcement. The DB is initialized with the same `events` table schema as the kernel store.

## Validators

Events pass through registered `EventValidator`s before routing. Built-in validators:

- **NamespaceValidator** — skills can only emit events in their own namespace (`{skill_id}.*`). System namespaces are blocked for non-trusted callers.
- **ReservedNamespaceValidator** — blocks non-trusted callers (MCP, SDK) from emitting system-namespace events.

Trusted callers: `CallerKind::Runtime` and `CallerKind::Cli`. Untrusted: `CallerKind::Mcp`, `CallerKind::Sdk`, `CallerKind::CloudSync`.

### Reserved namespaces

`actor.`, `config.`, `gate.`, `job.`, `mesh.`, `project.`, `runtime.`, `session.`, `skill.`, `studio.`, `sync.`, `workspace.`.

## Event types

All event types are defined as constants in `crates/core/runtime/src/events/types.rs`:

**Workspace**: `workspace.created`, `workspace.deleted`, `workspace.status_changed`, `workspace.activated`, `workspace.compiled`, `workspace.compile_failed`, `workspace.archived`, `workspace.agent_changed`.

**Session**: `session.started`, `session.progress`, `session.ended`, `session.recorded`.

**Actor**: `actor.created`, `actor.woke`, `actor.slept`, `actor.stopped`, `actor.crashed`.

**Gate**: `gate.passed`, `gate.failed`.

**Job**: `job.created`, `job.claimed`, `job.completed`, `job.failed`, `job.dispatched`, `job.update`.

**Skill**: `skill.started`, `skill.completed`, `skill.failed`.

**Other**: `config.changed`, `project.log`.

## SqliteEventStore (platform)

The platform-level event store at `~/.ship/platform.db`. Implements the `EventStore` trait:

- `append` — INSERT into the `events` table.
- `get` — lookup by ID, falls back to workspace DBs.
- `query` — filter by entity_id, event_type, workspace_id, session_id, correlation_id, actor_id, parent_actor_id, elevated. Merges results from platform DB and all workspace DBs, deduplicates by ID.
- `query_aggregate` — all events for a given entity_id.
- `query_correlation` — all events sharing a correlation_id.

The `events` table in the platform DB uses the same schema as the `event_log` table but with additional columns for the kernel's envelope fields (actor_id, parent_actor_id, elevated, target_actor_id).

## Unstable features

Behind the `unstable` feature flag:

- **Identity** (`events/identity.rs`) — actor identity and authentication.
- **Permissions** (`events/permissions.rs`) — emit permission enforcement.
- **Cursor** (`events/cursor.rs`) — per-actor read cursors for catch-up delivery.
- **Kernel security** (`events/kernel_security.rs`) — actor metadata and scoped delivery.
