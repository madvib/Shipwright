# Session Handoff — Studio v0.1.0 Phase 1 + Architecture Plan

## Branch: `v0.1.0`

## What was done this session

### Phase 1: Delete Lies (COMPLETE)
Removed all hardcoded/fake/demo data from Studio UI. 341/341 tests passing, TypeScript clean.

**Deleted files:**
- `features/agents/sections/SettingsSection.tsx` (dead component)
- `features/agents/sections/McpToolPanel.tsx` (depended on fake registry)
- `features/registry/mock-data.ts` (12 fake packages)

**Cleaned files:**
- `SectionShell.tsx` — removed OrangeDot export and showOrangeDot prop
- `SettingsLayout.tsx` — removed OrangeDot export
- `types.ts` — removed MCP_TOOL_REGISTRY, GITHUB_TOOLS, McpToolConfig
- `SkillsPreviewPanel.tsx` — replaced MOCK_AGENTS with real useAgentStore queries
- `SkillsFileExplorer.tsx` — replaced INSTALLED_SKILLS with empty state
- `registry-cards.ts` — removed MOCK_CARDS, REGISTRY_CARDS now empty array
- `RegistryCardGrid.tsx` — replaced OrangeDots with disabled "Coming soon" buttons
- `ProviderSettingsSection.tsx` — rebuilt as JSON editor per provider
- `McpSection.tsx` — rebuilt without fake tool list
- `SettingsForm.tsx` — removed hardcoded CLAUDE_MODELS, added dontAsk to DEFAULT_MODES
- All provider lists updated to 5 providers: claude, gemini, codex, cursor, opencode
- `ProviderLogo.tsx` + `ModeHeader.tsx` — added opencode support

**Test fixes:** schema-validation, schema-hints, mcp-client (8 expectations updated)

### State Management Audit (COMPLETE)
Every store is localStorage-only. Server sync is stubbed (agent-api.ts no-ops, useLibrarySync returns 'idle'). Provider settings and MCP tool permissions are ephemeral component state — data loss bugs.

### Product Direction Decided
- v0.1.0: CLI-first, KPI = CLI installs. MCP bridge = primary sync.
- Better Auth + GitHub app: keep code, don't surface in UI.
- localStorage = "draft state", push to CLI = real persistence.
- Two local layers: `~/.ship` (global library cache, single DB) and `<project>/.ship` (project declarations + exports).
- Studio surfaces library vs project as a **filter** in agents/skills pages.
- v0.1.X: add auth + GitHub publishing.
- v0.2.0: paid tier, agent cloud, workflows.

---

## BLOCKER: HTTPS → localhost bridge test

Must validate `fetch('http://localhost:51741/mcp')` works from `https://staging.getship.dev`. Browsers special-case localhost as potentially trustworthy (Chrome, Firefox) but needs live testing before further investment.

### Deploy to staging

```bash
cd apps/web
pnpm build
pnpm wrangler d1 migrations apply ship-auth-staging --remote --env staging
pnpm wrangler d1 migrations apply ship-registry-staging --remote --env staging
pnpm wrangler deploy --env staging
```

Set secrets (one-time):
```bash
pnpm wrangler secret put BETTER_AUTH_SECRET --env staging
pnpm wrangler secret put BETTER_AUTH_URL --env staging   # https://staging.getship.dev
pnpm wrangler secret put GITHUB_CLIENT_ID --env staging
pnpm wrangler secret put GITHUB_CLIENT_SECRET --env staging
```

DNS: ensure `staging.getship.dev` CNAME exists in Cloudflare zone.

**TODO:** Add `"deploy:staging": "pnpm build && wrangler deploy --env staging"` to `apps/web/package.json`.

---

## Execution Plan (after bridge test passes)

Full plan: `.ship-session/studio-v010-execution-plan.md`

| Phase | Work | Depends on |
|-------|------|------------|
| 1 | Add TransferBundle + pull response types to compiler crate with `#[derive(specta::Type)]` | — |
| 2 | Regen specta types + rebuild WASM | Phase 1 |
| 3 | Create useMcpQuery/useMcpMutation hooks (TanStack Query wrapping MCP calls) | Phase 2 |
| 4 | Fix ephemeral state: provider settings + tool permissions persisted on agent profile | — |
| 5 | Strip auth/GitHub from Studio UI surfaces | — |
| 6 | Replace SyncStatus with honest local/CLI status | — |
| 7 | Library/project filter in agents + skills pages | Phase 3 |
| 8 | Agent editor schema alignment (hooks section, permissions default_mode, SECTION_DEFS reorder) | — |

Phases 4-6 can run parallel with 1-3.

### MCP bridge type requirement
TransferBundle, AgentBundle, SkillBundle, PullResponse, ListAgentsResponse must be specta-generated from the compiler crate → flow to @ship/ui. Currently hand-written in `apps/mcp/src/tools/studio_push.rs` with no TypeScript equivalents. Web app constructs bundles inline in CliStatusPopover.tsx with zero type safety.

### TanStack Query requirement
MCP calls currently use raw `callTool()` → `JSON.parse(raw) as SomeType`. Must wrap with TanStack Query for loading states, caching, invalidation, retry. QueryClient is already set up (`integrations/tanstack-query/root-provider.tsx`) but only used for Registry features.

---

## Key files

| File | What |
|------|------|
| `.ship-session/studio-v010-execution-plan.md` | Full execution plan |
| `apps/web/src/features/agents/useAgentStore.ts` | Agent persistence (localStorage, server sync stubbed) |
| `apps/web/src/features/compiler/useLibrary.ts` | Library persistence (localStorage only) |
| `apps/web/src/features/compiler/useLibrarySync.ts` | Dead stub, returns 'idle' |
| `apps/web/src/features/agents/agent-api.ts` | Server API — all no-ops |
| `apps/web/src/features/studio/useLocalMcp.ts` | MCP connection hook |
| `apps/web/src/lib/mcp-client.ts` | Low-level MCP JSON-RPC client |
| `apps/web/src/features/studio/CliStatusPopover.tsx` | Push/pull UI + untyped bundle construction |
| `apps/mcp/src/tools/studio_push.rs` | Hand-written TransferBundle (needs specta migration) |
| `apps/mcp/src/tools/studio.rs` | pull_agents, list_local_agents |
| `apps/web/wrangler.jsonc` | Cloudflare Workers config with staging env |

## Uncommitted changes

All Phase 1 changes on branch `v0.1.0`. Ready to commit — 341 tests pass, TypeScript clean.

Changed: 17 files. Deleted: 3 files. New: 2 plan files in .ship-session/.
