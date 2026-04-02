# Handoff: tmux_session_name + PTY WebSocket endpoint

Branch: `v0.2.0`

## What was done

### Migration
`crates/core/runtime/migrations/0004_workspace_tmux.sql` — `ALTER TABLE workspace ADD COLUMN tmux_session_name TEXT;`

### DB layer (`crates/core/runtime/src/db/`)
- `types.rs` — `WorkspaceDbRow` extended to 15 elements (added `Option<String>` for `tmux_session_name`); `WorkspaceDbListRow` extended to 16 elements
- `workspace_state.rs` — all SELECT queries include `tmux_session_name`; new `set_workspace_tmux_session_db(branch, session_name)` updates the column; new `get_workspace_by_id_db(id)` queries `WHERE id = ? OR branch = ?` for lookup by workspace ID

### Workspace module (`crates/core/runtime/src/workspace/`)
- `types.rs` — `Workspace` struct gains `#[serde(skip_serializing_if = "Option::is_none")] pub tmux_session_name: Option<String>`
- `helpers.rs` — `new_workspace` initializes `tmux_session_name: None`
- `crud.rs` — all tuple destructuring updated; new `get_workspace_by_id(_ship_dir, id)` public function
- `lifecycle.rs` — new `set_workspace_tmux_session(ship_dir, branch, session_name)` public function
- `mod.rs` — re-exports both new functions

### Tests (`crates/core/runtime/src/workspace/tests_crud.rs`)
- `set_workspace_tmux_session_write_and_read_back` — write, read, and clear cycle
- `set_workspace_tmux_session_errors_for_missing_workspace` — failure path

### shipd PTY endpoint (`apps/shipd/src/`)
- `pty_handler.rs` — `workspace_pty` handler, `handle_pty_socket` bridge loop using `script(1)` for PTY allocation, `PtyConnectionGuard` drop guard
- `pty_handler_tests.rs` — integration tests: 404 for unknown workspace, 404 for no tmux session, 429 over limit
- `rest_api.rs` — `ApiState` gains `pty_connections: Arc<Mutex<HashMap<String, usize>>>`
- `lib.rs` — module declared, route `/workspaces/:id/pty` registered, field initialized
- `runtime_api.rs` — test helper updated

### Cargo.toml (`apps/shipd/Cargo.toml`)
- `axum` gains `ws` feature
- `tower` added as dev-dependency

## Route summary

```
GET /api/runtime/workspaces/:id/pty   — WS upgrade
  - :id is workspace id (or branch as fallback)
  - 404 if workspace not found
  - 404 if tmux_session_name is null or empty
  - 429 if >= 5 concurrent connections for workspace
  - Spawns: script -q -c "tmux attach-session -t <name>" /dev/null
  - Bridges: WS frames ↔ process stdin/stdout
  - Heartbeat: ping every 30s, close if no pong within 10s
  - On close: kills attach process (not the tmux session)
```

## Workspace response change

`GET /api/runtime/workspaces` now includes `tmux_session_name` in each workspace entry (omitted when null due to `skip_serializing_if`).

## Verification

```bash
cargo test -p runtime -- workspace
cargo test -p shipd
just build
```
