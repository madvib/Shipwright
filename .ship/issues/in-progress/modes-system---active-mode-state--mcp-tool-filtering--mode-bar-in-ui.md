+++
id = "cb235cce-f674-47d7-9a69-9b8ae853fbea"
title = "Modes system — active mode state, MCP tool filtering, mode bar in UI"
created = "2026-02-24T04:10:17.862368474Z"
updated = "2026-02-24T04:10:17.862369274Z"
tags = []
links = []
+++

## What
Implement the mode system end-to-end: mode definitions in config, active mode persistence, MCP tool filtering by mode, and a mode bar in the Tauri UI.

## Active Mode State

Active mode is stored in `~/.ship/config.toml` as `active_mode = "execution"` per project (keyed by project path). The MCP server reads it on startup and refilters tools if it changes.

```rust
// logic/src/modes.rs
pub fn get_active_mode(project_dir: &Path) -> Result<String>
pub fn set_active_mode(project_dir: &Path, mode_id: &str) -> Result<()>
pub fn list_modes(project_dir: &Path) -> Result<Vec<ModeConfig>>
pub fn get_mode(project_dir: &Path, mode_id: &str) -> Result<Option<ModeConfig>>
```

## MCP Tool Filtering

In `mcp/src/lib.rs`, the `ShipServer::get_info()` and the tool router need to be aware of the active mode. When a mode has a `mcp_tools` list, only those tools are exposed via `tools/list`. When `mcp_tools` is empty, all tools are exposed (default/unfiltered).

```rust
// In ShipServer init
let active_mode = get_active_mode(&project_dir).unwrap_or_default();
let mode_config = get_mode(&project_dir, &active_mode).ok().flatten();
// Filter tool router based on mode_config.mcp_tools
```

This is capability-based security: an agent in "planning" mode literally cannot see execution tools.

## Tauri Commands

```rust
fn get_active_mode_cmd(state: State<AppState>) -> Result<String, String>
fn set_active_mode_cmd(mode_id: String, state: State<AppState>) -> Result<(), String>
fn list_modes_cmd(state: State<AppState>) -> Result<Vec<ModeConfig>, String>
```

## UI: Mode Bar

A top-level bar in the Tauri UI (above sidebar, always visible) showing:
- Current mode name + color indicator
- Dropdown or button group to switch modes
- Mode switch writes via `set_active_mode_cmd`

This is a primary navigation element, not a settings page. Switching modes is a frequent, intentional action.

## Acceptance
- Switching to "planning" mode in UI causes MCP tool list to filter on next Claude Code query
- Mode persists across app restarts
- Default mode is "execution" if no modes defined