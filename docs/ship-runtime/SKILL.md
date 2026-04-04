---
name: ship-runtime
stable-id: ship-runtime
description: Use when working on Ship's runtime internals — the daemon (shipd), supervisor, service mesh, PTY bridge, event system, and database layer.
tags: [ship, runtime, internals, daemon]
authors: [ship]
audience: internal
---

# Ship Runtime Internals

Ship's runtime layer owns all persistent state and inter-agent communication. The daemon (`shipd`) is an Axum HTTP server that hosts the KernelRouter, service mesh, supervisor, and PTY bridge. Domain logic lives in `crates/core/runtime/`; the daemon binary lives in `apps/shipd/`.

## Crate structure

**`crates/core/runtime/src/`** — state management library:
- `db/` — SQLx/SQLite database layer, migrations, KV store, schema definitions
- `events/` — event envelope, kernel router, actor stores, mailboxes, validators
- `services/` — headless service actors (mesh, human gateway, dispatch, sync)
- `workspace.rs` — workspace CRUD and lifecycle
- `session.rs` — session lifecycle (start, progress, end, record)

**`apps/shipd/src/`** — daemon binary:
- `server.rs` — MCP server (NetworkServer) with mesh tools
- `rest_api.rs` — REST endpoints for mesh operations
- `runtime_api.rs` — read-only REST API for Studio (workspaces, sessions, agents, SSE events)
- `supervisor/` — job dispatch, workspace start, terminal launcher
- `pty_handler.rs` — WebSocket-to-tmux PTY bridge
- `connections.rs` — per-connection lifecycle (event relay, cleanup guard, mesh spawner)

For detailed documentation on each subsystem, see `references/docs/`.
