---
name: tanstack-start
description: Use when building with TanStack Start — file-based routing, server functions, API routes, TanStack Query, Cloudflare Workers deployment. Covers route conventions, data loading, mutations, and SSR patterns.
---

# TanStack Start — Patterns & Conventions

## Stack
- Framework: TanStack Start (SSR + client, built on Vite + Vinxi)
- Routing: TanStack Router (file-based, type-safe)
- Data: TanStack Query (`@tanstack/react-query`)
- Deployment: Cloudflare Workers (`@cloudflare/vite-plugin`)

## Route file conventions (`apps/web/src/routes/`)

```
routes/
  __root.tsx           — root layout (providers, global nav)
  index.tsx            — / (landing)
  studio.tsx           — /studio
  api/
    auth/$.ts          — /api/auth/* catch-all
    github/
      import.ts        — /api/github/import
    me.ts              — /api/me
```

## Page route pattern

```typescript
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/studio')({
  component: StudioPage,
  // loader runs on server during SSR, or on client during navigation
  loader: async ({ context }) => {
    return { /* data */ }
  },
})

function StudioPage() {
  const data = Route.useLoaderData()
  return <div>{/* ... */}</div>
}
```

## API route pattern

```typescript
import { createAPIFileRoute } from '@tanstack/react-start/api'
import { json } from '@tanstack/react-start'

export const APIRoute = createAPIFileRoute('/api/github/import')({
  POST: async ({ request }) => {
    const body = await request.json() as { url: string }
    // ... handle
    return json({ modes: [], skills: [], rules: [], mcp_servers: [] })
  },
})
```

## Server function pattern (alternative to API routes)

```typescript
import { createServerFn } from '@tanstack/react-start'
import { z } from 'zod'

export const importRepo = createServerFn({ method: 'POST' })
  .validator(z.object({ url: z.string().url() }))
  .handler(async ({ data }) => {
    // runs on server only
    return fetchGitHubRepo(data.url)
  })

// Client usage:
const result = await importRepo({ data: { url } })
```

## TanStack Query patterns

```typescript
// In a component
import { useQuery, useMutation } from '@tanstack/react-query'

// Query
const { data, isPending, error } = useQuery({
  queryKey: ['github-import', url],
  queryFn: () => fetch('/api/github/import', {
    method: 'POST',
    body: JSON.stringify({ url }),
  }).then(r => r.json()),
  enabled: !!url,
})

// Mutation
const importMutation = useMutation({
  mutationFn: (url: string) =>
    fetch('/api/github/import', { method: 'POST', body: JSON.stringify({ url }) })
      .then(r => { if (!r.ok) throw new Error(r.statusText); return r.json() }),
  onSuccess: (library) => {
    // populate state
  },
})
```

## Cloudflare env bindings in route handlers

```typescript
import { getWebRequest } from '@tanstack/react-start/server'

export const APIRoute = createAPIFileRoute('/api/me')({
  GET: async ({ request }) => {
    // Access Cloudflare bindings via the request context
    // DB, KV, R2 are available via process.env or the CF env object
    const db = process.env.DB as D1Database
    // ...
  },
})
```

## Key constraints
- Routes in `apps/web/src/routes/` only — no manual router config
- API routes MUST use `createAPIFileRoute`, not `createFileRoute`
- `createServerFn` runs on server only — safe for secrets, DB calls
- Cloudflare Workers: no Node.js built-ins (`fs`, `path`, `crypto` → use `globalThis.crypto`)
- No dynamic `require()` — ESM only
- `@tanstack/react-start` imports are for client code; `@tanstack/react-start/server` for server
