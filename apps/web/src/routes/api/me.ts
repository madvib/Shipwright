// GET /api/me -- return { user } for the authenticated user

import { createFileRoute } from '@tanstack/react-router'
import { requireSession } from '#/lib/session-auth'
import { getAuthDb } from '#/lib/d1'

export const Route = createFileRoute('/api/me')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        const db = getAuthDb()
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

        return Response.json({ user })
      },
    },
  },
})
