---
name: Studio MCP Bridge
description: Architecture and patterns for the MCP bridge between Studio (web app) and the local CLI. Transfer types, sync protocol, TanStack Query layer.
---

# Studio MCP Bridge

Studio communicates with the local CLI via MCP over HTTP. The CLI runs `ship mcp serve --http --port 51741` and Studio connects from the browser.

## Architecture

```
Studio (browser)  →  HTTP POST  →  ship mcp serve (localhost:51741)  →  .ship/ filesystem
```

- **Studio** is a TanStack Start app on Cloudflare Workers (or local dev)
- **CLI** runs an MCP server with JSON-RPC over HTTP
- **HTTPS → localhost**: The staging/prod site (HTTPS) calls `http://localhost`. This works because localhost is exempt from mixed-content blocking in browsers.

## Transfer Types

All transfer types are defined in Rust (`crates/core/compiler/src/types/transfer.rs`) with `#[derive(specta::Type)]`. TypeScript types are auto-generated to `packages/ui/src/generated.ts` via `cargo run -p xtask -- gen-types`. **Never hand-write transfer types in TypeScript.**

### Push: Studio → CLI

```typescript
// TransferBundle — sent via push_bundle MCP tool
{
  agent: AgentBundle,        // Full agent config (every schema field)
  skills: Record<string, SkillBundle>,  // Skill ID → file content
  rules: Record<string, string>,        // Rule filename → content
  dependencies: Record<string, string>,
}
```

`AgentBundle` carries every field from `schemas/agent.schema.json`:
- Identity: id, name, description, version, providers
- Model: model, available_models, agent_limits
- Refs: skill_refs, rule_refs, rules_inline, mcp_servers
- Structured: permissions, plugins, provider_settings, hooks, env

The CLI writes this to `.ship/agents/<id>.jsonc` in the correct nested schema structure, plus skill files to `.ship/skills/` and rule files to `.ship/rules/`.

### Pull: CLI → Studio

```typescript
// PullResponse — returned by pull_agents MCP tool
{
  agents: PullAgent[]  // All agents from project .ship/ + library ~/.ship/
}
```

`PullAgent` returns every schema field plus resolved content:
- `profile`: id, name, description, version, providers
- `skills`: resolved Skill[] with content from SKILL.md files
- `rules`: resolved Rule[] with content from .md files
- `rules_inline`: inline rules text
- `mcp_servers`: server names (command/url not yet resolved)
- `permissions`, `model`, `env`, `available_models`, `agent_limits`, `plugins`, `provider_settings`: passed through as-is
- `hooks`: array of hook configs
- `source`: "project" or "library" (project .ship/ shadows library ~/.ship/)

### List

```typescript
// ListAgentsResponse — returned by list_local_agents MCP tool
{ agents: string[] }  // Agent IDs from both project and library
```

## MCP Tools (Studio-relevant)

| Tool | Direction | Purpose |
|------|-----------|---------|
| `pull_agents` | CLI → Studio | Return all agents with resolved skills/rules/config |
| `list_local_agents` | CLI → Studio | Return agent IDs (lightweight, for badges) |
| `push_bundle` | Studio → CLI | Write agent config + skills + rules to .ship/ |
| `open_project` | Studio → CLI | Set active project directory |

## TanStack Query Layer

All MCP calls go through TanStack Query hooks in `apps/web/src/features/studio/mcp-queries.ts`.

```typescript
// Read: auto-refetch when connected
useLocalAgentIds()  // list_local_agents, 10s refetch, enabled when connected
usePullAgents()     // pull_agents, auto-refetch when connected

// Write: mutation with cache invalidation
usePushBundle()     // push_bundle, invalidates agent queries on success
```

Query keys in `apps/web/src/lib/query-keys.ts`:
```typescript
mcpKeys = {
  all: ['mcp'],
  agents: () => ['mcp', 'agents'],
  agentList: () => ['mcp', 'agents', 'list'],
  pull: () => ['mcp', 'agents', 'pull'],
}
```

## Connection Management

`useLocalMcp()` hook (`apps/web/src/features/studio/useLocalMcp.ts`) manages the MCP connection:
- Status: disconnected → connecting → connected | error
- Port configurable (default 51741)
- `callTool(name, args)` sends JSON-RPC to the MCP server
- `localAgentIds` set kept warm via `list_local_agents` polling
- Wrapped in `LocalMcpContext` for app-wide access

## Sync Protocol

1. **CLI → Studio (automatic):** `usePullAgents()` refetches on interval. Studio always shows what's on disk.
2. **Studio → CLI (confirmed):** User edits create drafts. Push requires explicit confirmation. `usePushBundle()` writes to disk, then query invalidation re-pulls to confirm.
3. **Project vs Library:** `pull_agents` scans both `<project>/.ship/agents/` and `~/.ship/agents/`. Project agents shadow library agents with the same ID. `PullAgent.source` tags each as "project" or "library".

## Key Files

| File | Purpose |
|------|---------|
| `crates/core/compiler/src/types/transfer.rs` | Rust transfer type definitions (source of truth) |
| `packages/ui/src/generated.ts` | Auto-generated TypeScript types |
| `apps/mcp/src/tools/studio.rs` | pull_agents, list_local_agents implementations |
| `apps/mcp/src/tools/studio_push.rs` | push_bundle implementation |
| `apps/web/src/features/studio/mcp-queries.ts` | TanStack Query hooks |
| `apps/web/src/features/studio/useLocalMcp.ts` | MCP connection hook |
| `apps/web/src/features/studio/LocalMcpContext.tsx` | React context wrapper |
| `apps/web/src/features/studio/CliStatusPopover.tsx` | Push/pull UI |
| `schemas/agent.schema.json` | The definitive agent schema |

## Rules

- Never hand-write types that exist in `@ship/ui`. Import them.
- Never construct JSON manually for MCP calls. Use the typed hooks.
- Never store agent data in localStorage as source of truth. CLI `.ship/` is the source. localStorage is a cache.
- Every field in the schema must survive a push→pull round trip. Test it.
