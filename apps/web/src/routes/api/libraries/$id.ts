// PUT /api/libraries/:id    → { library: Library }
// DELETE /api/libraries/:id → { ok: true }

import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod/v4'
import { requireSession } from '#/lib/session-auth'
import { createRepositories } from '#/db/repositories'
import { getD1 } from '#/lib/d1'

const UpdateLibraryInput = z.object({
  name: z.string().min(1).max(128).optional(),
  data: z.record(z.unknown()).optional(),
})

export const Route = createFileRoute('/api/libraries/$id')({
  server: {
    handlers: {
      PUT: async ({ request, params }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        let body: unknown
        try { body = await request.json() } catch {
          return Response.json({ error: 'Invalid JSON body' }, { status: 400 })
        }

        const parsed = UpdateLibraryInput.safeParse(body)
        if (!parsed.success) {
          const message = parsed.error.issues.map((i) => i.message).join('; ')
          return Response.json({ error: message }, { status: 400 })
        }

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRepositories(d1)
        const existing = await repos.getLibrary(params.id, auth.org)
        if (!existing) return Response.json({ error: 'not found' }, { status: 404 })
        if (existing.orgId !== auth.org) return Response.json({ error: 'forbidden' }, { status: 403 })

        const library = await repos.upsertLibrary({
          ...existing,
          name: parsed.data.name ?? existing.name,
          data: parsed.data.data !== undefined ? JSON.stringify(parsed.data.data) : existing.data,
          updatedAt: Date.now(),
        })
        return Response.json({ library })
      },

      DELETE: async ({ request, params }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRepositories(d1)
        const existing = await repos.getLibrary(params.id, auth.org)
        if (!existing) return Response.json({ error: 'not found' }, { status: 404 })
        if (existing.orgId !== auth.org) return Response.json({ error: 'forbidden' }, { status: 403 })

        await repos.deleteLibrary(params.id, auth.org)
        return Response.json({ ok: true })
      },
    },
  },
})
