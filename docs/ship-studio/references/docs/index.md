---
group: Studio
title: Studio
order: 1
---

# Studio

Studio is a web-based interface for managing agent configuration. It provides a visual layer over the same `.ship/` directory that the CLI operates on, connected through MCP tool calls over HTTP.

## Launching Studio

**Hosted version:** Visit [getship.dev](https://getship.dev). The hosted app connects to your local CLI over localhost.

**Prerequisites:** The CLI must be running an HTTP MCP server:

```bash
ship mcp serve --http
```

This starts the MCP server on port 51741 (the default). Studio connects to `http://localhost:51741` using HTTP POST with JSON-RPC.

{% aside type="tip" %}
HTTPS-to-localhost connections work because browsers exempt `localhost` from mixed-content blocking. The hosted Studio at getship.dev (HTTPS) can call `http://localhost:51741` without security errors.
{% /aside %}

## Connection lifecycle

Studio manages the MCP connection through a React hook (`useLocalMcp`) with four states:

| State | Meaning |
|-------|---------|
| `connected` | MCP server reachable, tools available, health check passed |
| `connecting` | Handshake and tool discovery in progress |
| `disconnected` | No active connection |
| `error` | Connection failed (server not running, wrong port, stale session) |

On connection, Studio performs a full MCP handshake: `initialize` to get server info, `listTools` to discover available tools, then a `get_project_info` call as a health check to verify the session is live.

If Studio has previously connected (tracked in localStorage), it auto-reconnects on page load after a 500ms delay.

### Reactive cache invalidation

Rather than polling, Studio listens for server-sent events (SSE) via a persistent notification listener. When the CLI pushes a `notifications/resources/list_changed` event, all MCP query caches are invalidated and components re-fetch. If the SSE stream drops, a fallback invalidation fires and the listener reconnects after 3 seconds.

### Stale session recovery

If the CLI restarts, existing sessions become invalid. When a tool call returns a 4xx error or an "initialize" error, Studio automatically reconnects by running the full handshake again. React Query retries the failed call on the next refetch cycle.

## Architecture

Studio communicates with the filesystem exclusively through MCP tool calls. It never reads or writes `.ship/` directly.

```
Browser  -->  HTTP POST (JSON-RPC)  -->  ship mcp serve --http  -->  .ship/ on disk
```

**Read path:** `pull_agents` returns all agents from both project and library sources with fully resolved skills, rules, and configuration. `list_project_skills` returns all skills. TanStack Query caches results with a 5-second stale time.

**Write path:** `push_bundle` sends a `TransferBundle` containing the full agent config, skill files, and rules. The CLI writes these to disk. Query invalidation triggers a re-pull to confirm the write.

**Skill writes:** `write_skill_file` saves individual files. `delete_skill_file` removes them. Both invalidate the query cache on success.

**Skill variables:** `get_skill_vars` reads merged variable values. `set_skill_var` writes a single variable.

## Data flow

All data in Studio derives from two sources:

1. **CLI pull data** -- Agent profiles, skills, rules, and config read from `.ship/` via MCP tools. This is the source of truth.
2. **Local drafts** -- Unsaved edits stored in IndexedDB. These overlay pull data in the UI and are cleared on successful push or explicit discard.

The CLI filesystem is always authoritative. IndexedDB is a cache and draft buffer, not a source of truth.

## Type safety

Transfer types (`PullResponse`, `TransferBundle`, `ListAgentsResponse`, `PullSkill`) are defined in Rust and auto-generated to TypeScript via Specta. They live in `@ship/ui`. Studio imports them directly -- no hand-written parallel type definitions.

All MCP calls go through typed TanStack Query hooks in `features/studio/mcp-queries.ts`.

## Studio pages

Studio has three main sections accessible from the navigation dock:

- **Agents** -- List, create, edit, and manage agent profiles. See [Agent Management](./agents.md).
- **Skills IDE** -- Three-panel code editor for skill files. See [Skills IDE](./skills-ide.md).
- **Settings** -- Configure CLI connection port, theme, and view version info.
