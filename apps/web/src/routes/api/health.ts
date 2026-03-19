import { createFileRoute } from '@tanstack/react-router'
import { getD1 } from '#/lib/d1'

const VERSION = '0.1.0'

async function checkDb(): Promise<'connected' | 'unavailable'> {
  const db = getD1()
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
