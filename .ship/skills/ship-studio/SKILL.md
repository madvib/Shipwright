---
name: ship-studio
stable-id: ship-studio
description: Use when working with Ship Studio — the web-based IDE for managing agents, editing skills, configuring settings, and previewing compiler output.
tags: [ship, studio, web]
authors: [ship]
---

# Ship Studio

Studio is the web-based IDE for Ship agent configuration. It runs at getship.dev (hosted) or locally via `pnpm dev` in `apps/web/`. It communicates with the local CLI through an MCP bridge over HTTP.

## Architecture

```
Studio (browser)  -->  HTTP POST  -->  ship mcp serve --http (localhost:51741)  -->  .ship/ filesystem
```

Studio is a TanStack Start app. The CLI runs `ship mcp serve --http --port 51741` and Studio connects from the browser. HTTPS-to-localhost works because browsers exempt localhost from mixed-content blocking.

## What You Can Do

- **Agents** -- List all agents from project and library sources. Create, edit, and delete agents. Add/remove skills, MCP servers, rules. Set permission presets and per-tool permissions. Configure providers, models, environment variables, and hooks. Drafts are stored in IndexedDB until explicitly pushed to the CLI.

- **Skills IDE** -- Three-panel editor: file explorer, code editor with syntax highlighting, and detail panel (Variables, Info, Used By tabs). Edit SKILL.md and all skill files. Create new skills and add files (vars.json, reference docs, scripts, templates). Save via Cmd+S writes through MCP `write_skill_file`. Offline support with IndexedDB draft persistence.

- **Settings** -- Configure CLI connection (port, connect/disconnect). Toggle light/dark theme. View version and links.

## MCP Tools Used

| Tool | Purpose |
|------|---------|
| `pull_agents` | Fetch all agents with resolved skills, rules, and config |
| `push_bundle` | Write agent config, skills, and rules to `.ship/` |
| `list_local_agents` | Fetch agent IDs for sync badges |
| `write_skill_file` | Save a single skill file to disk |

## Key Patterns

- All transfer types come from Rust via Specta. TypeScript types are auto-generated to `@ship/ui`. Never hand-write transfer types.
- `useLocalMcp()` manages the MCP connection lifecycle and exposes `callTool()`.
- `LocalMcpContext` wraps the app so any component can call MCP tools.
- TanStack Query hooks in `mcp-queries.ts` handle all MCP reads (auto-refetch) and writes (mutation with cache invalidation).
- Agent drafts use `useSyncExternalStore` with IndexedDB persistence. Skills IDE drafts use a separate IndexedDB store with debounced saves.
- The skills IDE tracks open tabs, expanded folders, and active file in localStorage for session continuity.
