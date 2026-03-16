---
name: drizzle-d1
description: Use when working with Drizzle ORM on Cloudflare D1. Covers schema definition, migrations via drizzle-kit, D1 adapter wiring, Better Auth integration, and Rivetkit actor state patterns.
---

# Drizzle ORM + Cloudflare D1

## Stack
- ORM: `drizzle-orm` with `drizzle-orm/d1` adapter
- Migration: `drizzle-kit` with `push` (dev) or `generate` + `wrangler d1 migrations apply` (prod)
- Auth schema: `@better-auth/drizzle-adapter`
- Actor state: Rivetkit's Drizzle driver

## Setup

```ts
// src/db/index.ts
import { drizzle } from 'drizzle-orm/d1'
import * as schema from './schema'

export function createDb(d1: D1Database) {
  return drizzle(d1, { schema })
}

export type Db = ReturnType<typeof createDb>
```

## Schema

```ts
// src/db/schema.ts
import { sqliteTable, text, integer } from 'drizzle-orm/sqlite-core'

export const workspaces = sqliteTable('workspace', {
  id: text('id').primaryKey(),
  name: text('name').notNull(),
  createdAt: integer('created_at', { mode: 'timestamp' }).notNull(),
})
```

## drizzle.config.ts

```ts
import { defineConfig } from 'drizzle-kit'

export default defineConfig({
  schema: './src/db/schema.ts',
  out: './migrations',
  dialect: 'sqlite',
  driver: 'd1-http',
  dbCredentials: {
    accountId: process.env.CLOUDFLARE_ACCOUNT_ID!,
    databaseId: process.env.CLOUDFLARE_D1_DATABASE_ID!,
    token: process.env.CLOUDFLARE_D1_TOKEN!,
  },
})
```

## Migrations

```bash
# Dev: push schema directly (no migration files)
npx drizzle-kit push

# Prod: generate SQL + apply via wrangler
npx drizzle-kit generate
wrangler d1 migrations apply <DB_NAME> --remote
```

## Better Auth Integration

Better Auth generates its own schema. Let it own its tables; use Drizzle for app-level tables.

```ts
// src/lib/auth.ts
import { betterAuth } from 'better-auth'
import { drizzleAdapter } from 'better-auth/adapters/drizzle'
import { createDb } from '../db'

export function createAuth(env: Env) {
  const db = createDb(env.DB)
  return betterAuth({
    database: drizzleAdapter(db, { provider: 'sqlite' }),
    // ...
  })
}
```

Run `npx @better-auth/cli generate` to get Better Auth's schema, then add it to `src/db/schema.ts`.

## Rivetkit Actor State (Drizzle driver)

Rivetkit supports Drizzle as a storage backend for actor state:

```ts
import { actor, setup } from 'rivetkit'
import { drizzle } from 'rivetkit/drivers/drizzle'

const myActor = actor({
  state: { count: 0 },
  // Rivetkit persists state via its own mechanism —
  // use createDb() inside actions for querying app tables
})
```

## Wrangler Bindings

```toml
# wrangler.toml
[[d1_databases]]
binding = "DB"
database_name = "ship-prod"
database_id = "<id>"
```

Access in Workers: `env.DB` typed as `D1Database`.

## Common Patterns

**Transactions:**
```ts
await db.transaction(async (tx) => {
  await tx.insert(workspaces).values({ id, name, createdAt: new Date() })
  await tx.insert(sessions).values({ workspaceId: id, ... })
})
```

**Typed queries:**
```ts
const ws = await db.query.workspaces.findFirst({
  where: (w, { eq }) => eq(w.id, workspaceId),
  with: { sessions: true },
})
```

## Rules
- Never use raw `env.DB.prepare()` — always go through Drizzle
- Schema lives in `src/db/schema.ts` — single source of truth
- Better Auth owns its tables; don't modify them manually
- Use `push` in dev (fast), `generate` + `apply` in prod (auditable)
