+++
id = "1de4e078-a047-4c3d-8981-c69694d9621b"
title = "Agent config UI: Library + Composer (MCP servers, prompts, hooks, modes)"
created = "2026-02-24T14:55:20.329123415Z"
updated = "2026-02-24T14:55:20.329178579Z"
tags = []
links = []
+++

## Overview
Build the full agent configuration UI in the Settings > Agents tab. Ship manages a library of composable building blocks that get synced to Claude/Codex/Gemini on mode activation.

## UI Structure
Three-panel layout:
- **Library** (left): MCP Servers, Prompts, Hooks, Permissions — reusable building blocks
- **Modes** (center): Cards showing composition of each mode, active mode highlighted, one-click activate
- **Agents** (right): Which agents are configured, sync status, last-sync timestamp, Import button

## Features Required
- MCP server CRUD: id, name, command, args, env, type (stdio/sse/http), url (for sse/http), disabled toggle, scope badge
- Prompt CRUD: create/edit named prompts (markdown editor), assign to modes
- Hook CRUD: trigger dropdown (PreToolUse/PostToolUse/Stop/Notification/PreCompact), matcher field, shell command
- Permission sets: allow/deny tool pattern lists per mode
- Mode composer: pick servers + prompt + hooks + permissions + target agents (claude/codex/gemini checkboxes)
- Activate mode button → calls setActiveModeCmd → triggers auto-sync
- Import button → calls importFromAgentCmd → populates Library from ~/.claude.json etc.
- Sync status indicator per agent (last sync time, success/error)
- "Browse MCP Apps" placeholder (post-alpha)

## Backend Dependencies
Blocked on backend issue: agent-config-enrichment (MCP type/url/disabled, Prompt entity, Hook entity, rich sync, auto-sync on mode switch)

## Notes
- Library items are reusable across modes
- Mode activation is one-click and immediate (backend writes to agent configs in &lt;5ms)
- All Tauri commands will be typed via specta bindings
