# Handoff: Live MCP Sync — Studio <-> CLI

## What Changed

Built full 2-way sync between Studio (browser SPA) and the local CLI via MCP over localhost HTTP. Zero cloud infrastructure. Browser connects directly to `ship mcp serve --http` on the user's machine.

## Key Decisions

- **No cloud storage** — no D1, KV, Durable Objects, or URL encoding. Direct localhost MCP connection.
- **SSE fix** — `tower-http::CorsLayer` broke chunked SSE streaming. Replaced with manual CORS middleware. Disabled `sse_retry` (priming event) and `sse_keep_alive` so SSE body closes cleanly after response — browser fetch can't reliably read multi-chunk SSE streams that never close.
- **Default port 51741** — high ephemeral range, not IANA registered.
- **Shared context** — `LocalMcpProvider` wraps the Studio layout so both PublishPanel and agent list pages can access MCP connection state.

## New Files

| File | Purpose |
|------|---------|
| `apps/web/src/lib/mcp-client.ts` | Browser MCP client — pure fetch, JSON-RPC 2.0, SSE parsing |
| `apps/web/src/lib/__tests__/mcp-client.test.ts` | 9 tests, all passing |
| `apps/web/src/features/studio/useLocalMcp.ts` | React hook — connection lifecycle, config persistence, localAgentIds tracking |
| `apps/web/src/features/studio/McpConnectionSection.tsx` | UI — connect/disconnect, push to CLI, import from CLI |
| `apps/web/src/features/studio/LocalMcpContext.tsx` | React context provider for shared MCP state |
| `apps/mcp/src/tools/studio.rs` | MCP tools: `pull_agents`, `list_local_agents` |
| `apps/mcp/src/tools/studio_push.rs` | MCP tool: `push_bundle` with security scanning |

## Modified Files

| File | Change |
|------|--------|
| `apps/mcp/src/http.rs` | Manual CORS middleware replacing CorsLayer, disabled sse_retry/sse_keep_alive |
| `apps/mcp/Cargo.toml` | Removed `tower-http` dependency |
| `apps/mcp/src/server/mod.rs` | Wired `push_bundle`, `pull_agents`, `list_local_agents` tools |
| `apps/mcp/src/server/tool_gate.rs` | Added new tools to PLATFORM_TOOLS |
| `apps/mcp/src/tools/mod.rs` | Added `studio`, `studio_push` modules |
| `apps/mcp/src/requests/project.rs` | Added `PushBundleRequest` |
| `apps/web/src/features/studio/PublishPanel.tsx` | Uses LocalMcpContext, renders McpConnectionSection |
| `apps/web/src/routes/studio.tsx` | Wraps layout with `LocalMcpProvider` |
| `apps/web/src/routes/studio/agents/index.tsx` | Green "Local" badge on agents synced to CLI |

## MCP Tools Added

| Tool | Direction | What |
|------|-----------|------|
| `pull_agents` | CLI -> Studio | Reads all `.ship/agents/*.jsonc`, resolves skills from SKILL.md, rules from rules/*.md, returns ResolvedAgentProfile JSON |
| `push_bundle` | Studio -> CLI | Receives TransferBundle JSON, security-scans it, writes to `.ship/agents/` and `.ship/skills/` |
| `list_local_agents` | Status | Returns agent IDs from `.ship/agents/` for sync badges |

## What Works

- Connect from Studio to local `ship mcp serve --http --port 51741`
- Push active agent to CLI's `.ship/`
- Import all local agents (with resolved skills, rules, MCP refs) into Studio
- Green "Local" badge on agent cards when agent exists in `.ship/`
- Auto-refresh badges after push/pull

## What's Next

1. **Skills on push** — `McpConnectionSection` sends `skills: {}` on push. Need to populate with actual skill content from `activeAgent.skills` so push includes inline skills.
2. **`ship sync` CLI alias** — shorthand for `ship mcp serve --http --port 51741`.
3. **Auto-open project** — MCP server should auto-detect the project dir on connect instead of requiring `open_project` call.
4. **Diff-aware sync** — show which agents differ between Studio and CLI before bulk import.
5. **Permissions roundtrip** — `pull_agents` returns permissions JSON but `push_bundle` doesn't write them yet.
