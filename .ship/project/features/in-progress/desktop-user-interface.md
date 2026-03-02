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

## Current State

Backend: ~70 Tauri commands exposed via Specta. ShipEvent typed enum for real-time updates. Vision, Notes, Rules, Workspace, Skills, Permissions, Modes all have backend commands.

UI routes that exist and work: Issues (Kanban), Features list, Specs list, Releases list, ADRs list, Agents panel (partial), Settings, Activity log, Overview.

UI routes missing (alpha blockers): `notes.tsx`, `vision.tsx`, `rules.tsx`.

Agents panel: MCP Servers tab works. Skills tab is read-only — CRUD not wired. Modes tab and Permissions tab not implemented.

Workspace panel: not yet in sidebar.

Build constraint: does not build in WSL. Use Windows host or native Linux for UI work.

## Notes

Tauri + React + TanStack Router + React Query + shadcn/ui. Specta auto-generates `src/bindings.ts` — never edit manually. Full implementation guide for missing views in `.ship/workflow/specs/draft/ship-ui-alpha-gaps.md`.
