// PUT /api/workflows/:id    → { workflow: Workflow }
// DELETE /api/workflows/:id → { ok: true }

import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod/v4'
import { requireSession } from '#/lib/session-auth'
import { createRepositories } from '#/db/repositories'
import { getD1 } from '#/lib/d1'

const UpdateWorkflowInput = z.object({
  name: z.string().min(1).max(128).optional(),
  definition: z.record(z.unknown()).optional(),
})

export const Route = createFileRoute('/api/workflows/$id')({
  server: {
    handlers: {
      PUT: async ({ request, params }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        let body: unknown
        try { body = await request.json() } catch {
          return Response.json({ error: 'Invalid JSON body' }, { status: 400 })
        }

        const parsed = UpdateWorkflowInput.safeParse(body)
        if (!parsed.success) {
          const message = parsed.error.issues.map((i) => i.message).join('; ')
          return Response.json({ error: message }, { status: 400 })
        }

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRepositories(d1)
        const existing = await repos.getWorkflow(params.id, auth.org)
        if (!existing) return Response.json({ error: 'not found' }, { status: 404 })
        if (existing.orgId !== auth.org) return Response.json({ error: 'forbidden' }, { status: 403 })

        const workflow = await repos.upsertWorkflow({
          ...existing,
          name: parsed.data.name ?? existing.name,
          definition: parsed.data.definition !== undefined
            ? JSON.stringify(parsed.data.definition)
            : existing.definition,
          updatedAt: Date.now(),
        })
        return Response.json({ workflow })
      },

      DELETE: async ({ request, params }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRepositories(d1)
        const existing = await repos.getWorkflow(params.id, auth.org)
        if (!existing) return Response.json({ error: 'not found' }, { status: 404 })
        if (existing.orgId !== auth.org) return Response.json({ error: 'forbidden' }, { status: 403 })

        await repos.deleteWorkflow(params.id, auth.org)
        return Response.json({ ok: true })
      },
    },
  },
})
