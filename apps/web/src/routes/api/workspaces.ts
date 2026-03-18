// GET /api/workspaces — list workspaces for the authenticated user
// POST /api/workspaces — create a workspace record

import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod/v4'
import { requireSession } from '#/lib/session-auth'
import { getDb } from '#/lib/cloud-auth'

function nanoid(): string {
  const bytes = new Uint8Array(16)
  crypto.getRandomValues(bytes)
  return Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('')
}

const CreateWorkspaceInput = z.object({
  name: z.string().min(1, 'Workspace name is required').max(128),
  branch: z.string().max(256).default('main'),
})

export const Route = createFileRoute('/api/workspaces')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const auth = await requireSession(request)
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
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        let body: unknown
        try {
          body = await request.json()
        } catch {
          return Response.json({ error: 'Invalid JSON body' }, { status: 400 })
        }

        const parsed = CreateWorkspaceInput.safeParse(body)
        if (!parsed.success) {
          const message = parsed.error.issues.map((i) => i.message).join('; ')
          return Response.json({ error: message }, { status: 400 })
        }

        const { name, branch } = parsed.data

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

        return Response.json(
          { workspace: { id, org_id: auth.org, name, branch, status: 'idle', created_at: now } },
          { status: 201 },
        )
      },
    },
  },
})
