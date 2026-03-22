import { createFileRoute } from '@tanstack/react-router'
import { getAuthDb, getRegistryDb } from '#/lib/d1'

const VERSION = '0.1.0'

async function checkDb(db: D1Database | null): Promise<'connected' | 'unavailable'> {
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
        const [authDb, registryDb] = await Promise.all([
          checkDb(getAuthDb()),
          checkDb(getRegistryDb()),
        ])
        return Response.json({ ok: true, version: VERSION, authDb, registryDb })
      },
    },
  },
})
