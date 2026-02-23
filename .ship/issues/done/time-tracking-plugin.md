+++
title = "Time Tracking Plugin"
created = "2026-02-22T05:30:30.857814407Z"
updated = "2026-02-22T05:30:30.857815475Z"
tags = []
links = []
+++

Add a time tracking plugin so users can log time spent on issues directly from the UI.

## Implementation — 2026-02-22

**New crate:** `crates/plugins/time-tracker`
- `Plugin` trait impl with auto-stop hook when issue moves to `done`
- `start_timer`, `stop_timer`, `log_time`, `list_entries`, `generate_report`
- Storage: `.ship/plugins/time-tracker/active.json` + `entries.json`

**Changed files:**
- `crates/logic/src/plugin.rs` — new: Plugin trait + PluginRegistry
- `crates/logic/src/lib.rs` — export Plugin, PluginRegistry
- `crates/cli/src/lib.rs` — `ship time start|stop|status|log|list|report`
- `crates/mcp/src/lib.rs` — `time_start`, `time_stop`, `time_status`, `time_report` tools
- `Cargo.toml` — added `crates/plugins/time-tracker` to workspace