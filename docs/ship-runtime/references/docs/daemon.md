---
group: Runtime
order: 2
title: Daemon (shipd)
description: Supervisor, PTY bridge, terminal launcher, and service actor lifecycle.
audience: internal
---

# Daemon (shipd)

## Supervisor

The supervisor handles workspace lifecycle and job dispatch. Source: `apps/shipd/src/supervisor/`.

### Workspace start (`POST /api/supervisor/workspaces/{id}/start`)

Accepts `{ agent_id, base_branch?, open_terminal? }`. Steps:

1. Look up workspace by ID.
2. Create git worktree at `{SHIP_WORKTREE_DIR}/{branch}` (idempotent).
3. Run `ship use {agent_id}` in the worktree to compile agent config.
4. Create tmux session (idempotent).
5. Send agent command into the tmux window.
6. Register agent on mesh via `mesh.register` event.
7. Write `worktree_path` and `tmux_session_name` to workspace record.
8. Optionally launch a terminal tab via `terminal_launcher`.

The worktree directory defaults to `~/.ship/worktrees/` and can be overridden with `SHIP_WORKTREE_DIR`.

### Job dispatch (`service.job-dispatch`)

A kernel actor subscribed to `job.*` events. Source: `apps/shipd/src/supervisor/job_dispatch.rs`.

**`job.created`** — dispatches a job:
1. Create git worktree at `{worktrees_dir}/{slug}`.
2. Copy spec file to `.ship-session/job-spec.md` in the worktree.
3. Run `ship use {agent}`.
4. Create tmux session `job-{slug}`.
5. Spawn agent CLI with `SHIP_MESH_ID={slug}` in the tmux session.
6. Launch terminal (respects `SHIP_DEFAULT_TERMINAL`).
7. Emit `job.dispatched` event.

**DAG dependencies** — if `depends_on` is non-empty, the job is deferred into a pending map. When a dependency completes (`job.completed` or `job.merged`), unblocked jobs are dispatched.

**`job.update`** — forwarded to the agent's mailbox via `mesh.send`.

**`job.completed` / `job.merged`** — cleanup: kill tmux session, remove worktree, delete branch. Then dispatch any unblocked pending jobs.

### Terminal launcher

Detects the host terminal and opens a tab attached to a tmux session. Source: `apps/shipd/src/supervisor/terminal_launcher.rs`.

Detection order (first match wins):

1. `SHIP_DEFAULT_TERMINAL` env var (`wt`, `tmux`, `manual`).
2. `wt.exe` in PATH — Windows Terminal via WSL.
3. `$TMUX` set — opens a new tmux window.
4. `$TERM_PROGRAM` is `iTerm.app` — AppleScript.
5. macOS — AppleScript with Terminal.app.
6. `$DISPLAY` or `$WAYLAND_DISPLAY` with `$TERMINAL` — XDG terminal.
7. Fallback: `manual` (prints attach command).

## PTY bridge

WebSocket endpoint at `GET /api/runtime/workspaces/{id}/pty`. Source: `apps/shipd/src/pty_handler.rs`.

Bridges a WebSocket connection to `tmux attach-session -t {session_name}` via `script(1)` (which allocates a PTY). Binary frames flow bidirectionally between the WebSocket and the tmux process stdin/stdout.

**Connection limit**: 5 concurrent attachments per workspace (`MAX_PTY_CONNECTIONS`). Enforced via a counter in `ApiState.pty_connections`. A `PtyConnectionGuard` decrements the counter on drop.

**Keepalive**: sends WebSocket ping every 30 seconds; closes if no pong within 10 seconds.

The bridge kills the `tmux attach` process on disconnect but does not kill the tmux session itself.

## Service actor lifecycle

Services implement the `ServiceHandler` trait (source: `crates/core/runtime/src/services/mod.rs`):

```rust
trait ServiceHandler: Send + 'static {
    fn name(&self) -> &str;
    fn handle(&mut self, event: &EventEnvelope, store: &ActorStore) -> Result<()>;
    fn on_start(&mut self, store: &ActorStore) -> Result<()> { Ok(()) }
    fn on_stop(&mut self, store: &ActorStore) -> Result<()> { Ok(()) }
    fn tick_interval(&self) -> Option<Duration> { None }
    fn on_tick(&mut self, store: &ActorStore) -> Result<()> { Ok(()) }
}
```

`spawn_service` creates an actor via `KernelRouter::spawn_actor`, then spawns `run_service` as a tokio task. The event loop calls `handle` for each mailbox event and `on_tick` on the configured interval. When the mailbox closes (actor stopped), `on_stop` runs and the task exits.

## Connection lifecycle

Each MCP HTTP session creates a `NetworkServer` with a shared `KernelRouter`. Source: `apps/shipd/src/connections.rs`.

- **ConnectionGuard** — held via `Arc`. On drop (last clone gone = session ended), emits `mesh.deregister` and removes the actor from the kernel.
- **EventRelay** — consumes an actor's mailbox, forwarding events to MCP peers (via `McpEventSink`) and a polling `Inbox` buffer (capped at 256 messages).
- **Inbox** — `VecDeque` behind `RwLock`, drained by the `mesh_inbox` MCP tool.
- **MeshService spawner** — `spawn_mesh_service` is called once (via `OnceLock`) to register the mesh service actor.
