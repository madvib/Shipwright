+++
id = "06f7da0d-f73b-49f5-b34a-bebf37c61f6e"
title = "Settings UI — agent configuration panel (providers, MCP registry, modes)"
created = "2026-02-24T04:11:01.079731080Z"
updated = "2026-02-24T04:11:01.079731780Z"
tags = []
links = []
+++

## What
A dedicated settings panel in the Tauri UI that gives users a GUI for configuring AI providers, the MCP server registry, and modes. This is the central control surface for all agent configuration.

## Sections

### AI Provider
- Dropdown: Claude / Codex / Gemini / Custom
- Binary path field (auto-detected, override if needed)
- Optional model override
- "Test" button — runs `<binary> -p "say hello"` and shows output or error

### MCP Servers
Table view of all registered MCP servers:
```
Name          Command                     Modes         Actions
─────────────────────────────────────────────────────────────────
Ship          /path/to/ship-mcp           All           [Edit] [Remove]
GitHub        npx @mcp/server-github      execution     [Edit] [Remove]
Filesystem    npx @mcp/server-filesystem  execution     [Edit] [Remove]
[+ Add Server]
```

Add/Edit form:
- ID, Name, Command, Args (space-separated), Env (key=value pairs), Modes (multi-select)

### Modes
List of defined modes with name, description, tool list, and context files.
- "Add mode" opens a form
- Edit mode's tool list (checklist of available ship MCP tools)
- Reorder modes (display order in mode bar)

### Export
- "Export to Claude" / "Export to Codex" / "Export to Gemini" / "Export All" buttons
- Shows which files will be written before confirming
- After export: lists files written and any warnings
- Last export timestamp per provider

## Tauri Commands Needed
All from the MCP registry, modes system, and config export issues — this panel is purely the UI layer on top of those.

## Acceptance
- User can add a new MCP server and have it appear in exported config without touching a file manually
- "Export to Claude" button updates ~/.claude.json and CLAUDE.md correctly
- Provider test button gives immediate feedback on whether the binary works