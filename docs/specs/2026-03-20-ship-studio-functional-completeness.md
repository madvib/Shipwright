# Ship Studio v0.1.0 — Functional Completeness Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close every functional gap between the current UI-shell state and a launchable product where all types are derived from Rust crates or D1 schema, state flows through React Query + server, and every user action is wired to real backend operations.

**Architecture:** Three layers — (1) Rust compiler types generated via Specta → `@ship/ui`, (2) D1 schema types inferred via Drizzle, (3) React Query mutations/queries as the sole data-fetching layer. localStorage becomes a cache/offline fallback, not the source of truth. The `fetchApi` utility and `query-keys.ts` factory already exist and are well-designed — we extend them, not replace them.

**Tech Stack:** Rust + Specta (type gen), Drizzle ORM + D1 (persistence), TanStack Query (state), TanStack Start (server functions), Vitest (tests), WASM compiler (@ship/compiler)

---

## Scope

This plan covers **4 independent workstreams** that can execute in parallel (separate worktrees). Each produces working, testable software on its own.

| # | Workstream | Owner Agent | Dependencies |
|---|-----------|-------------|--------------|
| 1 | Type Foundation | rust-compiler + web-lane | None — do first |
| 2 | State Management Migration | react-architect | Workstream 1 (types) |
| 3 | Distribution Pipeline | web-lane | Workstream 2 |
| 4 | Registry Completion | web-lane + cloudflare | Workstream 2 |

### Not In Scope (Deferred)

These capabilities from `studio-v0.1-capabilities.json` are intentionally deferred to separate plans:

- **Onboarding flow** — anonymous URL import, GitHub repo picker, download without account
- **Studio editor enhancements** — permissions UI completion, dependency management, compile error surface
- **Publishing wizard** — module identity editor, exports editor, full publish flow
- **Unofficial pipeline** — batch import, seed list, content hash dedup, claim flow
- **Windsurf scrub** — remove all Windsurf references

These depend on the foundation this plan establishes (types, state, server CRUD).

### Already Complete (Verified)

Server-backed CRUD routes for libraries, profiles, and workflows already exist with full GET/POST/PUT/DELETE:
- `apps/web/src/routes/api/libraries.ts` + `libraries/$id.ts`
- `apps/web/src/routes/api/profiles.ts` + `profiles/$id.ts`
- `apps/web/src/routes/api/workflows.ts` + `workflows/$id.ts`

Install tracking at `apps/web/src/routes/api/registry/$path.install.ts` is also fully implemented with rate limiting and atomic D1 increment. The `/api/github/create-pr` route exists.

---

## Codebase Conventions

These patterns are established in the codebase. All new code must follow them:

- **Auth:** Use `requireSession(request)` from `#/lib/session-auth` (not `requireAuth`). Returns `SessionUser | Response` — check `if (auth instanceof Response) return auth`.
- **Responses:** Use `Response.json({ ... })` (standard Web API). No `json<T>()` generic helper.
- **Route handlers:** Access via `Route.options.server!.handlers` in tests. Import `Route` from the route file.
- **File routing:** Directory-based (`libraries/$id.ts`), not dot-based (`libraries.$id.ts`).
- **Query keys:** Import from `#/lib/query-keys`. Consolidate duplicates (e.g., `registryKeys` in `useRegistry.ts` duplicates `query-keys.ts`).
- **Test mocking:** `vi.mock('cloudflare:workers', ...)`, `vi.mock('#/lib/d1', ...)`, `vi.mock('#/lib/session-auth', ...)`.

---

## Workstream 1: Type Foundation

**Problem:** 19 Rust runtime types exist but aren't generated to TypeScript. `AgentProfile` name collision between `@ship/ui` (Rust-derived) and `features/agents/types.ts` (local). The local `Profile` in `useProfiles.ts` also collides with the D1 schema `Profile`. API routes return untyped rows.

**Files:**
- Modify: `crates/xtask/src/main.rs` — register runtime types (requires adding `runtime` crate dependency to xtask)
- Modify: `crates/xtask/Cargo.toml` — add `runtime` dependency
- Modify: `packages/ui/src/types.ts` — re-export new types
- Regenerate: `packages/ui/src/generated.ts` — via `cargo xtask gen-types`
- Modify: `apps/web/src/features/compiler/types.ts` — re-export new types
- Modify: `apps/web/src/features/agents/types.ts` — rename `AgentProfile` → `AgentDetailState`
- Modify: `apps/web/src/features/studio/useProfiles.ts` — rename `Profile` → `StudioProfile`
- Create: `apps/web/src/lib/api-types.ts` — typed API response wrappers derived from D1 schema
- Test: `apps/web/src/lib/api-types.test.ts`

### Task 1.1: Register Missing Runtime Types in Specta

- [ ] **Step 1: Check xtask Cargo.toml for runtime dependency**

Read `crates/xtask/Cargo.toml`. The existing xtask only imports the `compiler` crate. To register runtime types, add `runtime` as a dependency:

```toml
[dependencies]
compiler = { path = "../core/compiler", features = ["specta"] }
runtime = { path = "../core/runtime", features = ["specta"] }
```

If the runtime crate doesn't have a `specta` feature flag, add one to `crates/core/runtime/Cargo.toml`:

```toml
[features]
specta = ["dep:specta"]
```

- [ ] **Step 2: Add specta derives to runtime types**

Check which types in `crates/core/runtime/src/workspace.rs` and `crates/core/runtime/src/events.rs` have `#[derive(specta::Type)]` behind a feature gate. If any are missing, add:

```rust
#[cfg_attr(feature = "specta", derive(specta::Type))]
```

Only add to API-facing types. Skip internal DB types (`WorkspaceSessionDb`, `CapabilityDb`).

- [ ] **Step 3: Register runtime types in xtask**

```rust
// crates/xtask/src/main.rs — add after existing compiler type registrations

// runtime/workspace.rs
c.register::<runtime::ShipWorkspaceKind>();
c.register::<runtime::WorkspaceStatus>();
c.register::<runtime::WorkspaceSessionStatus>();

// runtime/events.rs
c.register::<runtime::EventEntity>();
c.register::<runtime::EventAction>();
```

- [ ] **Step 4: Regenerate types**

Run: `cargo xtask gen-types`
Expected: `packages/ui/src/generated.ts` updated with new type exports.

- [ ] **Step 5: Re-export from packages/ui/types.ts**

Add the new runtime types to the re-export list in `packages/ui/src/types.ts`:

```typescript
export type {
  // ... existing exports ...
  ShipWorkspaceKind,
  WorkspaceStatus,
  WorkspaceSessionStatus,
  EventEntity,
  EventAction,
} from './generated'
```

- [ ] **Step 6: Verify compilation**

Run: `cd apps/web && npx tsc --noEmit`
Expected: No type errors.

- [ ] **Step 7: Commit**

```bash
git add crates/xtask/ crates/core/runtime/ packages/ui/src/generated.ts packages/ui/src/types.ts
git commit -m "feat: generate runtime types (workspace, session, event) to TypeScript"
```

### Task 1.2: Fix AgentProfile Name Collision

- [ ] **Step 1: Rename local AgentProfile → AgentDetailState**

In `apps/web/src/features/agents/types.ts`:
- Rename `AgentProfile` → `AgentDetailState`
- Update `DEMO_AGENT` type annotation: `export const DEMO_AGENT: AgentDetailState = { ... }`
- Keep imports from `@ship/ui` unambiguous

- [ ] **Step 2: Update all imports**

Run: `grep -r 'AgentProfile' apps/web/src/ --include='*.ts' --include='*.tsx'`

Update every import of `AgentProfile` from `features/agents/types` to `AgentDetailState`. Expected files:
- `features/agents/useAgentDetail.ts`
- `routes/studio/agents.$id.tsx`
- Any component importing from `features/agents/types`

- [ ] **Step 3: Verify no remaining collision**

Run: `cd apps/web && npx tsc --noEmit`
Run: `cd apps/web && npm test`
Expected: All pass.

- [ ] **Step 4: Commit**

```bash
git add apps/web/src/features/agents/
git commit -m "fix: rename local AgentProfile to AgentDetailState, resolve @ship/ui collision"
```

### Task 1.3: Fix Profile Name Collision

The `Profile` interface in `apps/web/src/features/studio/useProfiles.ts` (line 9) collides with the D1 schema `Profile` type from `#/db/schema`. When React Query hooks fetch `Profile` records from the server, the naming will be ambiguous.

- [ ] **Step 1: Rename local Profile → StudioProfile**

In `apps/web/src/features/studio/useProfiles.ts`:
- Rename `Profile` → `StudioProfile`
- Rename `makeProfile` → `makeStudioProfile`
- Update `profileToLibrary(profile: StudioProfile)`

- [ ] **Step 2: Update all imports**

Run: `grep -r "from.*useProfiles" apps/web/src/ --include='*.ts' --include='*.tsx'`

Update every import of `Profile` from `useProfiles` to `StudioProfile`.

- [ ] **Step 3: Verify**

Run: `cd apps/web && npm test`
Expected: All pass.

- [ ] **Step 4: Commit**

```bash
git add apps/web/src/features/studio/ apps/web/src/routes/
git commit -m "fix: rename studio Profile to StudioProfile, resolve D1 schema collision"
```

### Task 1.4: Create Typed API Response Layer

- [ ] **Step 1: Write the test**

Create `apps/web/src/lib/api-types.test.ts`:

```typescript
import { describe, it, expect } from 'vitest'
import type { LibraryResponse, LibrariesResponse, ProfilesResponse, WorkflowsResponse } from './api-types'
import type { Library, Profile, Workflow } from '#/db/schema'

describe('api-types', () => {
  it('LibraryResponse wraps schema Library type', () => {
    const lib = {} as Library
    const resp: LibraryResponse = { library: lib }
    expect(resp).toBeDefined()
  })

  it('LibrariesResponse wraps schema Library[] type', () => {
    const resp: LibrariesResponse = { libraries: [] }
    expect(resp.libraries).toEqual([])
  })

  it('ProfilesResponse wraps schema Profile[] type', () => {
    const resp: ProfilesResponse = { profiles: [] }
    expect(resp.profiles).toEqual([])
  })

  it('WorkflowsResponse wraps schema Workflow[] type', () => {
    const resp: WorkflowsResponse = { workflows: [] }
    expect(resp.workflows).toEqual([])
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd apps/web && npx vitest run src/lib/api-types.test.ts`
Expected: FAIL — module not found.

- [ ] **Step 3: Create api-types.ts**

Create `apps/web/src/lib/api-types.ts`:

```typescript
import type { Library, Profile, Workflow } from '#/db/schema'
import type { RegistrySearchResponse, PackageDetailResponse } from '#/features/registry/types'

// ── Library ─────────────────────────────────────────────────────────────────
export interface LibraryResponse { library: Library }
export interface LibrariesResponse { libraries: Library[] }

// ── Profiles ────────────────────────────────────────────────────────────────
export interface ProfileResponse { profile: Profile }
export interface ProfilesResponse { profiles: Profile[] }

// ── Workflows ───────────────────────────────────────────────────────────────
export interface WorkflowResponse { workflow: Workflow }
export interface WorkflowsResponse { workflows: Workflow[] }

// ── Registry (re-export for convenience) ────────────────────────────────────
export type { RegistrySearchResponse, PackageDetailResponse }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd apps/web && npx vitest run src/lib/api-types.test.ts`
Expected: PASS.

- [ ] **Step 5: Update API routes to use typed returns**

For each API route (`libraries.ts`, `libraries/$id.ts`, `profiles.ts`, `profiles/$id.ts`, `workflows.ts`, `workflows/$id.ts`), import the response type and add a type assertion comment. Example for libraries:

```typescript
import type { LibrariesResponse } from '#/lib/api-types'

// In GET handler — type-check the response shape:
const result: LibrariesResponse = { libraries }
return Response.json(result)
```

This uses standard `Response.json()` (codebase convention) while ensuring the response shape is compile-time checked.

- [ ] **Step 6: Run full test suite**

Run: `cd apps/web && npm test`
Expected: All 155+ tests pass.

- [ ] **Step 7: Commit**

```bash
git add apps/web/src/lib/api-types.ts apps/web/src/lib/api-types.test.ts apps/web/src/routes/api/
git commit -m "feat: add typed API response layer derived from D1 schema"
```

---

## Workstream 2: State Management Migration

**Problem:** 3 useQuery hooks, 0 useMutation calls. All core state (profiles, library, agent configs) lives in localStorage. Query key factory exists but is unused. No cache invalidation.

**Architecture:** Introduce a `hooks/` directory with React Query-backed hooks. Each hook:
1. Uses `useQuery` to fetch server state (with `fetchApi`)
2. Uses `useMutation` + `queryClient.invalidateQueries` for writes
3. Falls back to localStorage when unauthenticated (offline-first)
4. Uses the existing `query-keys.ts` factory

**Files:**
- Create: `apps/web/src/hooks/useServerLibrary.ts` — React Query wrapper for library CRUD
- Create: `apps/web/src/hooks/useServerProfiles.ts` — React Query wrapper for profile CRUD
- Create: `apps/web/src/hooks/useServerWorkflows.ts` — React Query wrapper for workflow CRUD
- Modify: `apps/web/src/features/studio/useProfiles.ts` — delegate to useServerProfiles when authenticated
- Modify: `apps/web/src/features/compiler/useLibrary.ts` — delegate to useServerLibrary when authenticated
- Modify: `apps/web/src/features/compiler/useLibrarySync.ts` — replace ad-hoc sync with React Query mutation
- Modify: `apps/web/src/features/registry/useRegistry.ts` — consolidate duplicate `registryKeys`, use `fetchApi`
- Test: `apps/web/src/hooks/__tests__/useServerLibrary.test.ts`
- Test: `apps/web/src/hooks/__tests__/useServerProfiles.test.ts`

### Task 2.1: Consolidate registryKeys Duplication

Before creating new hooks, fix the existing duplication.

- [ ] **Step 1: Remove local registryKeys from useRegistry.ts**

In `apps/web/src/features/registry/useRegistry.ts`, remove the local `registryKeys` definition (lines 8-13) and import from `query-keys.ts` instead:

```typescript
import { registryKeys } from '#/lib/query-keys'
```

- [ ] **Step 2: Verify tests pass**

Run: `cd apps/web && npm test`

- [ ] **Step 3: Commit**

```bash
git add apps/web/src/features/registry/useRegistry.ts
git commit -m "fix: consolidate registryKeys — single source in query-keys.ts"
```

### Task 2.2: Server Library Hook

- [ ] **Step 1: Write the failing test**

Create `apps/web/src/hooks/__tests__/useServerLibrary.test.ts`:

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import React from 'react'

vi.mock('#/lib/api-errors', () => ({
  fetchApi: vi.fn(),
}))

import { fetchApi } from '#/lib/api-errors'
import { useServerLibrary } from '../useServerLibrary'

function wrapper({ children }: { children: React.ReactNode }) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return React.createElement(QueryClientProvider, { client: qc }, children)
}

describe('useServerLibrary', () => {
  beforeEach(() => { vi.clearAllMocks() })

  it('fetches libraries on mount', async () => {
    const mockLibraries = [{ id: 'lib-1', name: 'test', data: '{}' }]
    vi.mocked(fetchApi).mockResolvedValueOnce({ libraries: mockLibraries })

    const { result } = renderHook(() => useServerLibrary(), { wrapper })

    await waitFor(() => expect(result.current.isSuccess).toBe(true))
    expect(result.current.data).toEqual(mockLibraries)
    expect(fetchApi).toHaveBeenCalledWith(
      '/api/libraries',
      expect.objectContaining({ credentials: 'include' }),
    )
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd apps/web && npx vitest run src/hooks/__tests__/useServerLibrary.test.ts`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement useServerLibrary**

Create `apps/web/src/hooks/useServerLibrary.ts`:

```typescript
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchApi } from '#/lib/api-errors'
import { libraryKeys } from '#/lib/query-keys'
import type { LibrariesResponse, LibraryResponse } from '#/lib/api-types'

export function useServerLibrary() {
  return useQuery({
    queryKey: libraryKeys.list(),
    queryFn: () =>
      fetchApi<LibrariesResponse>('/api/libraries', { credentials: 'include' }),
    select: (data) => data.libraries,
    staleTime: 60_000,
  })
}

export function useServerLibraryDetail(id: string) {
  return useQuery({
    queryKey: libraryKeys.detail(id),
    queryFn: () =>
      fetchApi<LibraryResponse>(`/api/libraries/${id}`, { credentials: 'include' }),
    select: (data) => data.library,
    enabled: !!id,
  })
}

export function useSaveLibrary() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ id, name, data }: { id: string; name: string; data: unknown }) =>
      fetchApi<LibraryResponse>(`/api/libraries/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ name, data }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: libraryKeys.all() })
    },
  })
}

export function useCreateLibrary() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ name, data }: { name: string; data: unknown }) =>
      fetchApi<LibraryResponse>('/api/libraries', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ name, data }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: libraryKeys.all() })
    },
  })
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd apps/web && npx vitest run src/hooks/__tests__/useServerLibrary.test.ts`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add apps/web/src/hooks/
git commit -m "feat: add React Query hooks for server-backed library CRUD"
```

### Task 2.3: Server Profiles Hook

Same pattern as 2.2 but for profiles. Uses `profileKeys` from `query-keys.ts`.

- [ ] **Step 1: Write the failing test**

Create `apps/web/src/hooks/__tests__/useServerProfiles.test.ts` — tests `useServerProfiles()` fetches from `/api/profiles`, `useSaveProfile()` invalidates `profileKeys.all()`.

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement useServerProfiles**

Create `apps/web/src/hooks/useServerProfiles.ts`:
- `useServerProfiles()` → `useQuery` with `profileKeys.list()`
- `useServerProfile(id)` → `useQuery` with `profileKeys.detail(id)`
- `useSaveProfile()` → `useMutation` that PUTs to `/api/profiles/:id`, invalidates `profileKeys.all()`
- `useCreateProfile()` → `useMutation` that POSTs to `/api/profiles`, invalidates `profileKeys.all()`
- `useDeleteProfile()` → `useMutation` that DELETEs `/api/profiles/:id`, invalidates `profileKeys.all()`

- [ ] **Step 4: Run test to verify it passes**

- [ ] **Step 5: Commit**

```bash
git add apps/web/src/hooks/
git commit -m "feat: add React Query hooks for server-backed profile CRUD"
```

### Task 2.4: Integrate Server Hooks into useLibrary

Replace the ad-hoc `useLibrarySync` pattern with proper React Query integration.

- [ ] **Step 1: Modify useLibrary to accept optional server data**

In `apps/web/src/features/compiler/useLibrary.ts`:
- Add an `initialData` parameter from server query
- When authenticated: use server data as initial state, debounced `useSaveLibrary` mutation for writes
- When unauthenticated: localStorage-only (current behavior)
- Remove manual `StorageEvent` dispatch — React Query cache is the reactivity layer

- [ ] **Step 2: Simplify useLibrarySync**

Replace the 148-line `useLibrarySync.ts` with a thin wrapper:

```typescript
import { useAuth } from '#/lib/components/protected-route'
import { useServerLibrary, useSaveLibrary } from '#/hooks/useServerLibrary'

export type SyncStatus = 'idle' | 'saving' | 'saved' | 'error'

export function useLibrarySync() {
  const { isAuthenticated } = useAuth()
  const { data: serverLibraries, isLoading } = useServerLibrary()
  const { mutate: save, isPending, isError } = useSaveLibrary()

  const syncStatus: SyncStatus = isError
    ? 'error'
    : isPending
      ? 'saving'
      : isLoading
        ? 'idle'
        : 'saved'

  return { syncStatus, serverLibraries, save, isAuthenticated }
}
```

- [ ] **Step 3: Run full test suite**

Run: `cd apps/web && npm test`
Expected: All tests pass. If existing tests mock `useLibrarySync`, update the mocks.

- [ ] **Step 4: Commit**

```bash
git add apps/web/src/features/compiler/useLibrary.ts apps/web/src/features/compiler/useLibrarySync.ts
git commit -m "refactor: replace ad-hoc library sync with React Query mutations"
```

### Task 2.5: Integrate Server Hooks into useProfiles

- [ ] **Step 1: Modify useProfiles to sync with server**

In `apps/web/src/features/studio/useProfiles.ts`:
- When authenticated: initial load from `useServerProfiles()`, mutations via `useSaveProfile()`
- When unauthenticated: localStorage-only (current behavior)
- Keep `profileToLibrary()` and auto-compile logic unchanged
- Note: The `StudioProfile` type (renamed in Task 1.3) is the client-side editing model. Server persistence stores the compiled TOML content as the D1 `Profile.content` field.

- [ ] **Step 2: Run full test suite**

Run: `cd apps/web && npm test`
Expected: All pass.

- [ ] **Step 3: Commit**

```bash
git add apps/web/src/features/studio/useProfiles.ts
git commit -m "feat: sync studio profiles with server when authenticated"
```

---

## Workstream 3: Distribution Pipeline

**Problem:** "Push to repo", "Publish to registry", and "Download files" buttons in PublishPanel.tsx are stubs. No actual distribution logic.

**Important:** The `CompileResult` type (aliased from `CompileOutput`) does NOT have a flat `files[]` array. Its fields are: `context_content`, `mcp_servers`, `mcp_config_path`, `skill_files`, `rule_files`, `agent_files`, `claude_settings_patch`, `codex_config_patch`, `gemini_settings_patch`, `gemini_policy_patch`, `cursor_hooks_patch`, `cursor_cli_permissions`, `cursor_environment_json`, `plugins_manifest`. The download hook must reconstruct file paths from these fields.

**Files:**
- Modify: `apps/web/src/features/studio/PublishPanel.tsx` — wire buttons to real actions
- Create: `apps/web/src/hooks/useDownloadFiles.ts` — client-side file download from compile output
- Create: `apps/web/src/hooks/usePublishToRegistry.ts` — mutation for registry publish
- Create: `apps/web/src/hooks/usePushToRepo.ts` — mutation for GitHub PR creation
- Test: `apps/web/src/hooks/__tests__/useDownloadFiles.test.ts`

### Task 3.1: Download Files Action

- [ ] **Step 1: Write the failing test**

Create `apps/web/src/hooks/__tests__/useDownloadFiles.test.ts`:

```typescript
import { describe, it, expect } from 'vitest'
import { collectFiles } from '../useDownloadFiles'
import type { CompileResult } from '#/features/compiler/types'

describe('collectFiles', () => {
  it('extracts context file from compile output', () => {
    const result: Partial<CompileResult> = {
      context_content: '# Claude config',
      mcp_servers: null,
      mcp_config_path: '.mcp.json',
      skill_files: {},
      rule_files: {},
      agent_files: {},
      claude_settings_patch: null,
      plugins_manifest: { install: [], scope: 'project' },
    }
    const files = collectFiles('claude', result as CompileResult)
    expect(files).toContainEqual({ path: 'CLAUDE.md', content: '# Claude config' })
  })

  it('extracts skill files', () => {
    const result: Partial<CompileResult> = {
      context_content: null,
      mcp_servers: null,
      mcp_config_path: null,
      skill_files: { '.claude/skills/foo/SKILL.md': '---\nname: foo\n---\nContent' },
      rule_files: {},
      agent_files: {},
      claude_settings_patch: null,
      plugins_manifest: { install: [], scope: 'project' },
    }
    const files = collectFiles('claude', result as CompileResult)
    expect(files).toContainEqual({
      path: '.claude/skills/foo/SKILL.md',
      content: '---\nname: foo\n---\nContent',
    })
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd apps/web && npx vitest run src/hooks/__tests__/useDownloadFiles.test.ts`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement useDownloadFiles**

Create `apps/web/src/hooks/useDownloadFiles.ts`:

```typescript
import { PROVIDERS, type CompileResult } from '#/features/compiler/types'

interface OutputFile {
  path: string
  content: string
}

/** Map provider ID to context file name. */
const CONTEXT_FILE: Record<string, string> = {
  claude: 'CLAUDE.md',
  gemini: 'GEMINI.md',
  codex: 'AGENTS.md',
  cursor: 'AGENTS.md',
}

/** Extract all writable files from a CompileResult. */
export function collectFiles(providerId: string, result: CompileResult): OutputFile[] {
  const files: OutputFile[] = []

  // Context file (CLAUDE.md, GEMINI.md, etc.)
  if (result.context_content) {
    const name = CONTEXT_FILE[providerId] ?? `${providerId.toUpperCase()}.md`
    files.push({ path: name, content: result.context_content })
  }

  // MCP config
  if (result.mcp_config_path && result.mcp_servers) {
    files.push({
      path: result.mcp_config_path,
      content: JSON.stringify(result.mcp_servers, null, 2),
    })
  }

  // Skill files
  for (const [path, content] of Object.entries(result.skill_files ?? {})) {
    if (content) files.push({ path, content })
  }

  // Rule files
  for (const [path, content] of Object.entries(result.rule_files ?? {})) {
    if (content) files.push({ path, content })
  }

  // Agent files
  for (const [path, content] of Object.entries(result.agent_files ?? {})) {
    if (content) files.push({ path, content })
  }

  // Claude settings patch
  if (result.claude_settings_patch) {
    files.push({
      path: '.claude/settings.json',
      content: JSON.stringify(result.claude_settings_patch, null, 2),
    })
  }

  // Gemini settings patch
  if (result.gemini_settings_patch) {
    files.push({
      path: '.gemini/settings.json',
      content: JSON.stringify(result.gemini_settings_patch, null, 2),
    })
  }

  // Codex config patch
  if (result.codex_config_patch) {
    files.push({ path: '.codex/config.toml', content: result.codex_config_patch })
  }

  return files
}

/** Trigger browser download for each file. */
function downloadFile(path: string, content: string) {
  const blob = new Blob([content], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = path.replace(/\//g, '_') // flatten paths for download
  a.click()
  URL.revokeObjectURL(url)
}

export function useDownloadFiles() {
  const download = (output: Record<string, CompileResult>) => {
    for (const [providerId, result] of Object.entries(output)) {
      const files = collectFiles(providerId, result)
      for (const file of files) {
        downloadFile(file.path, file.content)
      }
    }
  }

  return { download, collectFiles }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd apps/web && npx vitest run src/hooks/__tests__/useDownloadFiles.test.ts`
Expected: PASS.

- [ ] **Step 5: Wire into PublishPanel**

In `PublishPanel.tsx`, replace the stub "Download files" button onClick:

```typescript
import { useDownloadFiles } from '#/hooks/useDownloadFiles'

// Inside component:
const { download } = useDownloadFiles()

// onClick handler:
if (compileState.status === 'ok') {
  download(compileState.output)
}
```

- [ ] **Step 6: Commit**

```bash
git add apps/web/src/hooks/useDownloadFiles.ts apps/web/src/hooks/__tests__/useDownloadFiles.test.ts apps/web/src/features/studio/PublishPanel.tsx
git commit -m "feat: wire download files action in PublishPanel"
```

### Task 3.2: Publish to Registry Action

- [ ] **Step 1: Write the failing test**

Test that `usePublishToRegistry` calls `POST /api/registry/publish` with `{ repo_url, tag }`.

- [ ] **Step 2: Implement usePublishToRegistry**

Create `apps/web/src/hooks/usePublishToRegistry.ts`:

```typescript
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchApi } from '#/lib/api-errors'
import { registryKeys } from '#/lib/query-keys'

interface PublishInput {
  repo_url: string
  tag?: string
}

interface PublishResult {
  package_id: string
  version: string
  skills_indexed: number
}

export function usePublishToRegistry() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (input: PublishInput) =>
      fetchApi<PublishResult>('/api/registry/publish', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify(input),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: registryKeys.all() })
    },
  })
}
```

- [ ] **Step 3: Wire into PublishPanel**

- [ ] **Step 4: Commit**

```bash
git add apps/web/src/hooks/usePublishToRegistry.ts apps/web/src/features/studio/PublishPanel.tsx
git commit -m "feat: wire publish to registry action in PublishPanel"
```

### Task 3.3: Push to Repo (Create PR) Action

The `/api/github/create-pr` route already exists.

- [ ] **Step 1: Implement usePushToRepo**

Create `apps/web/src/hooks/usePushToRepo.ts`:

```typescript
import { useMutation } from '@tanstack/react-query'
import { fetchApi } from '#/lib/api-errors'

interface PushToRepoInput {
  repoUrl: string
  branch: string
  files: Array<{ path: string; content: string }>
  commitMessage: string
}

export function usePushToRepo() {
  return useMutation({
    mutationFn: (input: PushToRepoInput) =>
      fetchApi<{ pr_url: string }>('/api/github/create-pr', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify(input),
      }),
  })
}
```

- [ ] **Step 2: Wire into PublishPanel**

Use `collectFiles()` from `useDownloadFiles` to transform `CompileResult` into the `files` array the PR endpoint expects.

- [ ] **Step 3: Commit**

```bash
git add apps/web/src/hooks/usePushToRepo.ts apps/web/src/features/studio/PublishPanel.tsx
git commit -m "feat: wire push-to-repo (create PR) action in PublishPanel"
```

---

## Workstream 4: Registry Completion

**Problem:** Registry hooks fall back to mock data silently. No `/studio/registry` route. Browse UI doesn't exist.

**Already done:** Install tracking endpoint is fully implemented with rate limiting and atomic D1 increment.

**Files:**
- Modify: `apps/web/src/features/registry/useRegistry.ts` — remove mock fallback, use `fetchApi` + proper error states
- Delete: `apps/web/src/features/registry/mock-data.ts` — remove mock data
- Create: `apps/web/src/routes/studio/registry.tsx` — browse page
- Create: `apps/web/src/routes/studio/registry/$path.tsx` — detail page
- Modify: `apps/web/src/features/studio/StudioDock.tsx` — update registry link
- Create: `apps/web/src/hooks/useInstallPackage.ts` — install tracking mutation

### Task 4.1: Remove Mock Fallback from Registry Hooks

- [ ] **Step 1: Update useRegistrySearch to use fetchApi**

In `apps/web/src/features/registry/useRegistry.ts`:

```typescript
import { fetchApi } from '#/lib/api-errors'
import { registryKeys } from '#/lib/query-keys'

export function useRegistrySearch(query: string, scope: ScopeFilter, page: number) {
  return useQuery({
    queryKey: registryKeys.search(query, scope, page),
    queryFn: async (): Promise<RegistrySearchResponse> => {
      const params = new URLSearchParams()
      if (query) params.set('q', query)
      if (scope !== 'all') params.set('scope', scope)
      params.set('page', String(page))
      params.set('limit', String(ITEMS_PER_PAGE))

      return fetchApi<RegistrySearchResponse>(`/api/registry/search?${params}`)
    },
    staleTime: 30_000,
    placeholderData: (prev) => prev,
  })
}
```

Remove the `try/catch` with mock fallback. Let React Query handle error states. Components display empty state or error UI — not fake data.

- [ ] **Step 2: Update usePackageDetail similarly**

Remove mock fallback. Use `fetchApi`. Let the query error surface to the component.

- [ ] **Step 3: Delete mock-data.ts**

Remove `apps/web/src/features/registry/mock-data.ts` entirely. Also remove the import from `useRegistry.ts`.

- [ ] **Step 4: Run tests, fix any that relied on mock data**

Run: `cd apps/web && npm test`

- [ ] **Step 5: Commit**

```bash
git add apps/web/src/features/registry/
git commit -m "fix: remove mock data fallback from registry hooks, use real API with error states"
```

### Task 4.2: Install Tracking Client Hook

- [ ] **Step 1: Create useInstallPackage hook**

Create `apps/web/src/hooks/useInstallPackage.ts`:

```typescript
import { useMutation } from '@tanstack/react-query'
import { fetchApi } from '#/lib/api-errors'

export function useTrackInstall() {
  return useMutation({
    mutationFn: (packagePath: string) =>
      fetchApi<{ installs: number }>(
        `/api/registry/packages/${encodeURIComponent(packagePath)}/install`,
        { method: 'POST' },
      ),
  })
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/web/src/hooks/useInstallPackage.ts
git commit -m "feat: add install tracking client hook"
```

### Task 4.3: Studio Registry Route

- [ ] **Step 1: Create the browse page**

Create `apps/web/src/routes/studio/registry.tsx`:

```typescript
import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'
import { useRegistrySearch } from '#/features/registry/useRegistry'
import type { ScopeFilter } from '#/features/registry/types'
import { SCOPE_FILTERS, SCOPE_COLORS } from '#/features/registry/types'

export const Route = createFileRoute('/studio/registry')({
  component: RegistryBrowse,
})

function RegistryBrowse() {
  const [query, setQuery] = useState('')
  const [scope, setScope] = useState<ScopeFilter>('all')
  const [page, setPage] = useState(1)
  const { data, isLoading, isError, error } = useRegistrySearch(query, scope, page)

  if (isError) {
    return (
      <div className="p-8 text-center text-muted-foreground">
        <p>Unable to load registry.</p>
        <p className="text-sm">{error?.message}</p>
      </div>
    )
  }

  // Render: search bar, scope filter tabs, package card grid, pagination
  // Use existing RegistryCard components from features/registry/ if available
}
```

- [ ] **Step 2: Create the detail page**

Create `apps/web/src/routes/studio/registry/$path.tsx` using `usePackageDetail`.

- [ ] **Step 3: Update StudioDock**

In `apps/web/src/features/studio/StudioDock.tsx`, update the registry link to point to `/studio/registry`.

- [ ] **Step 4: Run tests, update StudioDock test if it checks link targets**

Run: `cd apps/web && npm test`

- [ ] **Step 5: Commit**

```bash
git add apps/web/src/routes/studio/registry* apps/web/src/features/studio/StudioDock.tsx
git commit -m "feat: add studio registry browse and detail pages"
```

---

## Verification Checklist

After all workstreams complete, verify:

- [ ] `cargo xtask gen-types --check` passes (types up to date)
- [ ] `cd apps/web && npx tsc --noEmit` passes (no type errors)
- [ ] `cd apps/web && npm test` passes (all tests green)
- [ ] No `AgentProfile` import ambiguity — `grep -r 'AgentProfile' apps/web/src/features/agents/` returns zero hits
- [ ] No `Profile` collision — `useProfiles.ts` uses `StudioProfile`, D1 schema uses `Profile`
- [ ] No mock data in registry hooks — `grep -r 'MOCK_PACKAGES' apps/web/src/` returns zero hits outside test files
- [ ] Single `registryKeys` definition — only in `query-keys.ts`
- [ ] Every `useMutation` invalidates the correct query keys
- [ ] Every API route returns a typed response via `Response.json()`
- [ ] Auth uses `requireSession` (not `requireAuth`) throughout
- [ ] PublishPanel buttons all trigger real actions (download, publish, push)
- [ ] `/studio/registry` route renders and fetches from real API
