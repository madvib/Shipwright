import { createFileRoute } from '@tanstack/react-router'

const VERSION = '0.1.0'

type D1Database = {
  prepare: (query: string) => { run: () => Promise<unknown> }
}

async function checkDb(): Promise<'connected' | 'unavailable'> {
  // In the Cloudflare Workers runtime, D1 bindings are exposed as globals.
  // `(globalThis as Record<string, unknown>)['DB']` resolves the binding without
  // importing `cloudflare:workers`, which is not available in local Vite SSR dev.
  const db = (globalThis as Record<string, unknown>)['DB'] as D1Database | undefined
  if (!db) return 'unavailable'
  try {
    await db.prepare('SELECT 1').run()
    return 'connected'
  } catch {
    return 'unavailable'
  }
}

export const Route = createFileRoute('/api/health')({
  server: {
    handlers: {
      GET: async () => {
        const db = await checkDb()
        return Response.json({ ok: true, version: VERSION, db })
      },
    },
  },
})
