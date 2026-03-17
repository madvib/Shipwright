// GET /api/workspaces — list workspaces for user's org
// POST /api/workspaces — create a workspace record in D1

import { createFileRoute } from '@tanstack/react-router'
import { requireAuth, getDb } from '#/lib/cloud-auth'

function nanoid(): string {
  const bytes = new Uint8Array(16)
  crypto.getRandomValues(bytes)
  return Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('')
}

export const Route = createFileRoute('/api/workspaces')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const auth = await requireAuth(request)
        if (auth instanceof Response) return auth

        const db = getDb()
        if (!db) {
          return Response.json({ error: 'Database unavailable' }, { status: 503 })
        }

        const { results } = await db
          .prepare('SELECT id, name, branch, status, created_at FROM workspaces WHERE org_id = ? ORDER BY created_at DESC')
          .bind(auth.org)
          .all<{ id: string; name: string; branch: string; status: string; created_at: number }>()

        return Response.json({ workspaces: results })
      },

      POST: async ({ request }) => {
        const auth = await requireAuth(request)
        if (auth instanceof Response) return auth

        let body: unknown
        try {
          body = await request.json()
        } catch {
          return Response.json({ error: 'Invalid JSON body' }, { status: 400 })
        }

        if (
          !body ||
          typeof body !== 'object' ||
          typeof (body as Record<string, unknown>)['name'] !== 'string'
        ) {
          return Response.json({ error: 'Missing name field' }, { status: 400 })
        }

        const { name, branch = 'main' } = body as { name: string; branch?: string }

        if (typeof branch !== 'string') {
          return Response.json({ error: 'branch must be a string' }, { status: 400 })
        }

        const db = getDb()
        if (!db) {
          return Response.json({ error: 'Database unavailable' }, { status: 503 })
        }

        const id = nanoid()
        const now = Date.now()

        await db
          .prepare('INSERT INTO workspaces (id, org_id, name, branch, status, created_at) VALUES (?, ?, ?, ?, ?, ?)')
          .bind(id, auth.org, name, branch, 'idle', now)
          .run()

        return Response.json({ workspace: { id, org_id: auth.org, name, branch, status: 'idle', created_at: now } }, { status: 201 })
      },
    },
  },
})
