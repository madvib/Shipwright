+++
id = "SjS4tQUW"
title = "Desktop User Interface"
created = "2026-02-28T15:56:07Z"
updated = "2026-02-28T15:56:07Z"
branch = ""
release_id = "v0.1.0-alpha"
spec_id = "ship-ui-alpha-gaps.md"
adr_ids = []
tags = []

[agent]
mcp_servers = []
skills = []
+++

## Why

While CLI and MCP are the primary surfaces for agents and power users, a visual interface dramatically lowers the bar for exploring project state, managing issues, and configuring agent context. The Tauri desktop app gives Shipwright a native home that works alongside the terminal — not instead of it. For alpha, the UI is a companion to the CLI, not a replacement.

## Acceptance Criteria

- [ ] Views: Issues (Kanban), Features list, Specs list, ADRs list, Notes list, Vision, Settings
- [ ] Issue Kanban: drag-to-move between backlog/in-progress/done columns
- [ ] Feature detail: full content editor, linked spec/release display
- [ ] Settings: project config, provider detection, git policy
- [ ] Workspace panel: current branch, active mode, resolved agent config
- [ ] Activity log derived from event stream
- [ ] All data via Tauri commands (Rust backend) — no direct file access from frontend
- [ ] Typed Specta bindings: `ShipEvent` enum for real-time updates

## Delivery Todos

- [ ] Fix project auto-detection (currently broken — hardcoded path issues)
- [ ] Implement Issues Kanban with drag-and-drop
- [ ] Features, Specs, ADRs views with correct status directory reading
- [ ] Notes list and editor (currently missing — alpha blocker)
- [ ] Vision view and editor
- [ ] Workspace panel component
- [ ] Settings panel: providers, config, git policy
- [ ] Fix type misalignment between Rust enums and TypeScript (use Specta throughout)

## Notes

The UI is a Tauri + React app. Rust backend exposes ~70 commands via Specta for type-safe bindings. Real-time updates use typed `ShipEvent` enum events emitted from Tauri commands. The UI does not build in WSL — use a Windows host or native Linux machine for UI development.
