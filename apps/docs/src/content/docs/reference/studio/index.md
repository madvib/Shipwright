---
title: "Ship Studio"
sidebar:
  label: "Ship Studio"
  order: 1
---
Ship Studio is a web-based IDE for managing agent configuration. It provides a visual interface over the same `.ship/` directory that the CLI operates on, connected through an MCP bridge.

## Running Studio

**Hosted:** Visit [getship.dev](https://getship.dev). The hosted version connects to your local CLI via localhost.

**Local development:**

```bash
cd apps/web
pnpm dev
```

This starts the TanStack Start dev server. Studio runs in the browser and connects to the CLI on the same machine.

## Connecting to the CLI

Studio requires a running CLI MCP server to read and write agent configuration. Start it with:

```bash
ship mcp serve --http --port 51741
```

Port 51741 is the default. You can change it in Studio's Settings page. The connection uses HTTP POST with JSON-RPC to the localhost MCP server.

Once the CLI is running, Studio auto-connects if it has previously connected (tracked via localStorage). Otherwise, go to Settings and click Connect.

### Connection States

| State | Meaning |
|-------|---------|
| Connected | MCP server reachable, tools available |
| Connecting | Handshake in progress |
| Disconnected | No active connection |
| Error | Connection attempt failed (server not running, wrong port, etc.) |

When disconnected, Studio still works in offline mode. Edits are stored locally in IndexedDB and sync when the connection is restored.

## How the MCP Bridge Works

Studio communicates with the local filesystem exclusively through MCP tool calls. It never reads or writes `.ship/` directly.

```
Browser  -->  HTTP POST (JSON-RPC)  -->  ship mcp serve  -->  .ship/ on disk
```

**Read path:** `pull_agents` returns all agents from both project (`.ship/agents/`) and library (`~/.ship/agents/`) with fully resolved skills, rules, and configuration. TanStack Query auto-refetches every 10 seconds while connected.

**Write path:** When a user pushes agent changes, `push_bundle` sends a `TransferBundle` containing the full agent config, skill files, and rules. The CLI writes these to the correct locations on disk. Query invalidation triggers a re-pull to confirm the write.

**Skill writes:** The Skills IDE uses `write_skill_file` for individual file saves, which is more granular than `push_bundle`.

HTTPS-to-localhost connections work because browsers exempt `localhost` from mixed-content blocking. The hosted Studio at getship.dev (HTTPS) can call `http://localhost:51741` without security errors.

## Studio Pages

Studio has three main sections, accessible from the navigation dock:

### Agents

List, create, edit, and delete agent profiles. Each agent card shows skill count, MCP server count, rule count, permission preset, and source (project vs library). A "Modified" badge appears when unsaved drafts exist. See the [Agent Management](./agents.md) guide.

### Skills IDE

A three-panel code editor for skill files. Browse the full skill directory tree, edit files with syntax highlighting, and inspect skill metadata in the detail panel. See the [Skills IDE](./skills-ide.md) guide.

### Settings

Configure the CLI connection port, toggle between light and dark themes, and view version information. The CLI command to start the MCP server is displayed with a copy button.

## Data Flow

All data in Studio derives from two sources:

1. **CLI pull data** -- Agent profiles, skills, rules, and configuration read from `.ship/` via MCP tools. This is the source of truth.
2. **Local drafts** -- Unsaved edits stored in IndexedDB. These overlay the pull data in the UI and are discarded on push or explicit discard.

Studio never stores agent data in localStorage as the source of truth. The CLI filesystem is always authoritative. IndexedDB is a cache and draft buffer.

## Type Safety

All transfer types (`PullResponse`, `TransferBundle`, `ListAgentsResponse`) are defined in Rust and auto-generated to TypeScript via Specta. They live in `@ship/ui`. Studio imports them directly and never hand-writes parallel type definitions.

API calls go through typed TanStack Query hooks in `apps/web/src/features/studio/mcp-queries.ts`. No raw `fetch` calls.
