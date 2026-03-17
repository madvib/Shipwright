// GET /api/jobs — list jobs from D1 cloud job queue for user's org

import { createFileRoute } from '@tanstack/react-router'
import { requireAuth, getDb } from '#/lib/cloud-auth'

export const Route = createFileRoute('/api/jobs')({
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
          .prepare(
            'SELECT id, type, status, workspace_id, payload, created_at, updated_at FROM cloud_jobs WHERE org_id = ? ORDER BY created_at DESC',
          )
          .bind(auth.org)
          .all<{
            id: string
            type: string
            status: string
            workspace_id: string | null
            payload: string | null
            created_at: number
            updated_at: number
          }>()

        return Response.json({ jobs: results })
      },
    },
  },
})
