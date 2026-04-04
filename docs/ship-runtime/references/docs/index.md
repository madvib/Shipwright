---
group: Runtime
order: 1
title: Runtime Overview
description: Ship daemon architecture, startup sequence, and API surface.
audience: internal
---

# Runtime Overview

The Ship daemon (`shipd`) is a long-running Axum HTTP server that provides the event kernel, service mesh, supervisor, and PTY bridge. It listens on `127.0.0.1:9315` by default and serves both MCP (Streamable HTTP) and REST endpoints.

## Startup sequence

`shipd::run_network(host, port)` performs these steps in order:

1. **Initialize KernelRouter** — creates the kernel event store at `{global_dir}/kernel/events.db`.
2. **Spawn MeshService** — registers a `service.mesh` actor subscribed to `mesh.*` events. Returns a `SharedMeshRegistry` for REST read access.
3. **Subscribe workspace events** — spawns `service.workspace-sync` actor to project `workspace.*` events into the workspace database table.
4. **Subscribe job events** — spawns `service.job-dispatch` actor to handle `job.*` lifecycle (create, update, complete, merge).
5. **Spawn human gateway** — if `SHIP_TELEGRAM_TOKEN` and `SHIP_TELEGRAM_CHAT_ID` are set, spawns a `service.human-gateway` actor.
6. **Write PID and port files** — `{global_dir}/network.pid` and `{global_dir}/network.port`.
7. **Build Axum router** — mounts MCP service at `/mcp`, REST APIs under `/api`.
8. **Listen** — binds TCP and serves with graceful shutdown on SIGINT/SIGTERM.

On shutdown, PID and port files are removed.

## API surface

### MCP endpoint (`/mcp`)

Streamable HTTP via the `rmcp` library. Each HTTP session gets its own `NetworkServer` instance sharing a single `KernelRouter`. MCP tools: `mesh_register`, `mesh_send`, `mesh_broadcast`, `mesh_discover`, `mesh_inbox`, `mesh_status`.

### REST mesh API (`/api/mesh/*`)

Bypasses MCP for relay clients (e.g. `ship-mcp`). Endpoints:

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api/mesh/register` | Register agent, spawn kernel actor |
| POST | `/api/mesh/send` | Directed message to another agent |
| POST | `/api/mesh/broadcast` | Broadcast to all (or filtered) agents |
| GET | `/api/mesh/discover` | List registered agents |
| POST | `/api/mesh/status` | Update agent status |
| GET | `/api/mesh/events/{agent_id}` | SSE stream of agent's mailbox |

### Runtime API (`/api/runtime/*`)

Read-only endpoints for Studio:

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/runtime/workspaces` | List all workspaces |
| GET | `/api/runtime/sessions` | List sessions (optional `?workspace_id=`) |
| GET | `/api/runtime/agents` | List mesh-registered agents |
| GET | `/api/runtime/events` | SSE stream of all kernel events |
| GET | `/api/runtime/workspaces/{id}/pty` | WebSocket PTY bridge |

### Supervisor API (`/api/supervisor/*`)

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/api/supervisor/workspaces/{id}/start` | Create worktree, configure agent, spawn tmux |

### Project API (`/api/*`)

Session files, git status/diff/log, agent/skill listing, workspace activation/deletion, and event emission.

## CORS

Allowed origins: `https://getship.dev`, `localhost:*`, `127.0.0.1:*`. Other browser origins are rejected. Non-browser clients (no `Origin` header) pass through.

## State sharing

All API routes share `ApiState`:

- `kernel: Arc<Mutex<KernelRouter>>` — the single event router
- `mesh_registry: SharedMeshRegistry` — read-optimized agent registry
- `agent_mailboxes: Arc<Mutex<HashMap<String, Mailbox>>>` — stashed for SSE endpoints
- `pty_connections: Arc<Mutex<HashMap<String, usize>>>` — per-workspace connection counts
