---
name: better-auth
description: Use when implementing authentication with Better Auth — D1 adapter, GitHub OAuth, session handling, TanStack Start integration, Cloudflare Workers runtime. Covers schema setup, route wiring, middleware patterns, and client-side auth state.
---

# Better Auth on Cloudflare Workers + TanStack Start

## Stack
- Runtime: Cloudflare Workers (via TanStack Start server functions)
- Database: D1 (SQLite at edge) via Better Auth's built-in D1 adapter
- Auth: Better Auth v1+ with GitHub OAuth social provider
- Client: `better-auth/client` with `inferAdditionalFields`

## Server setup (`apps/web/src/lib/auth.ts`)

```typescript
import { betterAuth } from 'better-auth'
import { d1 } from 'better-auth/adapters/d1'
import { github } from 'better-auth/providers/github'
import { tanstackStartCookies } from 'better-auth/tanstack-start'

export const auth = betterAuth({
  database: d1(process.env.DB as unknown as D1Database),
  socialProviders: {
    github: {
      clientId: process.env.GITHUB_CLIENT_ID!,
      clientSecret: process.env.GITHUB_CLIENT_SECRET!,
    },
  },
  plugins: [tanstackStartCookies()],
})
```

## D1 Schema (run via wrangler d1 execute)

```sql
CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  email TEXT UNIQUE NOT NULL,
  name TEXT,
  github_id TEXT UNIQUE,
  github_username TEXT,
  avatar_url TEXT,
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token TEXT UNIQUE NOT NULL,
  expires_at INTEGER NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS accounts (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  provider TEXT NOT NULL,
  provider_account_id TEXT NOT NULL,
  access_token TEXT,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  UNIQUE(provider, provider_account_id)
);
```

## Catch-all auth route (`apps/web/src/routes/api/auth/$.ts`)

```typescript
import { createAPIFileRoute } from '@tanstack/react-start/api'
import { auth } from '~/lib/auth'

export const APIRoute = createAPIFileRoute('/api/auth/$')({
  GET: ({ request }) => auth.handler(request),
  POST: ({ request }) => auth.handler(request),
})
```

## Auth middleware helper

```typescript
// apps/web/src/lib/auth-middleware.ts
import { auth } from './auth'

export async function requireAuth(request: Request) {
  const session = await auth.api.getSession({ headers: request.headers })
  if (!session) {
    throw new Response(JSON.stringify({ error: 'Unauthorized' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    })
  }
  return session.user
}
```

## Client setup (`apps/web/src/lib/auth-client.ts`)

```typescript
import { createAuthClient } from 'better-auth/react'

export const authClient = createAuthClient({
  baseURL: typeof window !== 'undefined' ? window.location.origin : '',
})

export const { signIn, signOut, useSession } = authClient
```

## React usage

```typescript
// Sign in
authClient.signIn.social({ provider: 'github' })

// Sign out
authClient.signOut()

// Reactive session (hook)
const { data: session, isPending } = authClient.useSession()
const user = session?.user  // { id, name, email, image }
```

## Wrangler bindings (wrangler.jsonc)

```jsonc
{
  "d1_databases": [
    { "binding": "DB", "database_name": "ship", "database_id": "<your-id>" }
  ],
  "vars": {
    "GITHUB_CLIENT_ID": "",
    "GITHUB_CLIENT_SECRET": "",
    "GITHUB_TOKEN": "",
    "BETTER_AUTH_SECRET": ""
  }
}
```

## Key constraints
- D1 is SQLite — no JSON operators, no array columns. Keep schema flat.
- All secrets via `process.env` / wrangler vars. Never hardcode.
- `tanstackStartCookies()` plugin is required for session cookies to work with TanStack Start's SSR.
- GitHub OAuth callback URL must be registered: `https://<your-domain>/api/auth/callback/github`
- For local dev: `http://localhost:3000/api/auth/callback/github`
