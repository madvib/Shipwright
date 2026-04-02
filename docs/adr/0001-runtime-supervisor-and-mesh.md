# ADR-0001: Runtime Supervisor and Service Mesh Architecture

**Status:** Accepted  
**Date:** 2026-04-02  
**Context:** v0.2.0 runtime consolidation

---

## Context

As Ship grows, we have accumulated multiple overlapping tracking surfaces:

- `dispatch.sh` — shell script that creates worktrees, names tmux sessions, spawns agents
- `runtime` DB — workspace records (branch, status, worktree_path)  
- `shipd` in-memory — mesh registrations, PTY connection counts
- No single place knows the full picture of a running agent

This fragmentation means: a restart loses mesh state, a crash loses PTY connections, and understanding what's running requires reading four separate systems. This does not scale to a product, let alone an ecosystem.

The long-term vision is for Ship to function as an agent operating system — process management is the foundation everything else depends on.

---

## Decision

### 1. `shipd` is the process supervisor

`shipd` owns the full lifecycle of a `WorkspaceSession`:

```
WorkspaceSession
├── workspace_id      FK → workspaces table
├── branch            git branch name
├── worktree_path     filesystem path
├── tmux_session      tmux session name (if terminal-based)
├── pid               direct process PID (if process-based)
├── mesh_agent_id     registration handle on the mesh
├── pty_connections   bounded list of active WS connections
└── health            last_heartbeat, restart_policy
```

A single call to `POST /api/supervisor/workspaces/:id/start` creates the worktree, spawns the agent (via tmux or direct process), registers on the mesh, and writes the full record. There is no second place to look.

### 2. `dispatch.sh` is a pontoon bridge — not a destination

The shell script makes things possible today but is not reliable or monitorable. Its logic moves into `shipd` incrementally:

| Script step | Future owner |
|-------------|-------------|
| `git worktree add` | `shipd` supervisor via `WorkspaceSession::spawn()` |
| `tmux new-session -s <id> -c <path>` | `shipd` supervisor |
| Write workspace record | `shipd` supervisor (already writes; now also tmux name) |
| Register on mesh | `shipd` supervisor on spawn |
| `ship use <agent>` | Compile step called by supervisor before spawn |

`dispatch.sh` becomes a thin CLI wrapper: `ship dispatch <agent> --workspace <branch>` calls the supervisor API and optionally opens a terminal. The script is not removed immediately — it bridges the gap until the supervisor endpoint exists.

### 3. Domain boundaries within `shipd`

`shipd` must not become a monolith. It is structured as composable internal services:

```
shipd
├── supervisor/     WorkspaceSession lifecycle (spawn, monitor, stop, restart)
├── mesh/          Agent registration, capability routing, message delivery
├── runtime_api/   Read surface for studio: workspaces, sessions, agents, events SSE
├── pty/           WebSocket PTY proxy (thin: calls tmux attach, bridges IO)
└── webhook/       External ingress (Telegram, etc.) → mesh events
```

Each module has a narrow interface. The supervisor reads/writes DB. The mesh is in-memory with DB persistence on change. The PTY handler is stateless beyond connection counting. Shared state passes through `ApiState`, not global singletons.

### 4. The mesh is the integration bus

The mesh is not internal tooling — it is the extensibility surface. Any process that can make HTTP calls can participate:

```
Agent (Claude Code) ──mesh_register──▶ shipd mesh
Slack adapter       ──mesh_register──▶ shipd mesh  ←── community
Smart home bridge   ──mesh_register──▶ shipd mesh  ←── community
Custom app          ──mesh_register──▶ shipd mesh  ←── community
```

A mesh participant declares capabilities and subscribes to event namespaces. The supervisor routes messages by capability match. Ship does not need to know about Slack; a Slack adapter just registers `capabilities: ["notify", "receive_message"]`.

This is the path toward community adapters: if you can describe what you send and receive, you can plug into any Ship runtime.

### 5. ADRs live in `docs/adr/`

Architecture decision records are committed to the repository at `docs/adr/NNNN-title.md`. They are not ephemeral. Supersession is explicit: a new ADR references the one it replaces and sets the old one's status to "Superseded by ADR-XXXX."

---

## Consequences

**Immediate (v0.2.0):**
- `tmux_session_name` added to workspace record (done)
- PTY WS endpoint on shipd (done)
- `ship daemon start/stop/status` CLI (done)
- These are correct steps; the supervisor spawn endpoint is next

**Near-term:**
- `POST /api/supervisor/workspaces/:id/start` — replaces dispatch.sh spawn logic
- `WorkspaceSession` as unified record (extend workspaces table or new table)
- Health monitor loop in supervisor — detect dead processes, emit `agent.exited` event
- `dispatch.sh` reduced to: call supervisor API + open terminal

**Long-term:**
- `MeshAdapter` trait published as a crate — enables community adapters
- Supervisor restart policy (crash → restart N times, then alert via mesh)
- Process model generalizes: tmux, direct process, Docker container, remote SSH — all `WorkspaceSession` variants
- Ship runtime as embeddable library for custom deployments

---

## Alternatives Considered

**Keep dispatch.sh as primary:** Rejected. Scripts are not monitorable, not restartable, and cannot enforce invariants. They are appropriate for bootstrapping, not for a product runtime.

**Split supervisor into a separate daemon:** Rejected for now. `shipd` is already the network-facing process; adding a supervisor co-process introduces IPC complexity with no benefit at this scale. Revisit if `shipd` grows past ~5k lines.

**Use an existing process supervisor (systemd, supervisor, PM2):** Rejected. We need Ship-aware supervision — knowing that a dead process was serving workspace `rust-lane` and should emit `agent.exited` on the mesh. Generic supervisors don't have this context.
