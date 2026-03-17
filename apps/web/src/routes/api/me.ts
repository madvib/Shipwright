// GET /api/me — return { user, org } for the authenticated user

import { createFileRoute } from '@tanstack/react-router'
import { requireAuth, getDb } from '#/lib/cloud-auth'

export const Route = createFileRoute('/api/me')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const auth = await requireAuth(request)
        if (auth instanceof Response) return auth

        const db = getDb()
        if (!db) {
          return Response.json({ error: 'Database unavailable' }, { status: 503 })
        }

        const user = await db
          .prepare('SELECT id, name, email, image, createdAt FROM user WHERE id = ?')
          .bind(auth.sub)
          .first<{ id: string; name: string; email: string; image: string | null; createdAt: number }>()

        if (!user) {
          return Response.json({ error: 'User not found' }, { status: 404 })
        }

        const org = await db
          .prepare('SELECT id, name, slug, created_at FROM orgs WHERE id = ?')
          .bind(auth.org)
          .first<{ id: string; name: string; slug: string; created_at: number }>()

        return Response.json({ user, org })
      },
    },
  },
})
