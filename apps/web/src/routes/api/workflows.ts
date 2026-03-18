// GET /api/workflows     → { workflows: Workflow[] }
// POST /api/workflows    → { workflow: Workflow }

import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod/v4'
import { requireSession } from '#/lib/session-auth'
import { createRepositories } from '#/db/repositories'

function nanoid(): string {
  const bytes = new Uint8Array(16)
  crypto.getRandomValues(bytes)
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
}

function getD1(): D1Database | null {
  return (globalThis as Record<string, unknown>)['DB'] as D1Database | undefined ?? null
}

const CreateWorkflowInput = z.object({
  name: z.string().min(1, 'Name is required').max(128),
  definition: z.record(z.unknown()).optional(),
})

export const Route = createFileRoute('/api/workflows')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRepositories(d1)
        const workflows = await repos.getWorkflows(auth.org, auth.sub)
        return Response.json({ workflows })
      },

      POST: async ({ request }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        let body: unknown
        try { body = await request.json() } catch {
          return Response.json({ error: 'Invalid JSON body' }, { status: 400 })
        }

        const parsed = CreateWorkflowInput.safeParse(body)
        if (!parsed.success) {
          const message = parsed.error.issues.map((i) => i.message).join('; ')
          return Response.json({ error: message }, { status: 400 })
        }

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRepositories(d1)
        const now = Date.now()
        const workflow = await repos.upsertWorkflow({
          id: nanoid(),
          orgId: auth.org,
          userId: auth.sub,
          name: parsed.data.name,
          definition: JSON.stringify(parsed.data.definition ?? {}),
          createdAt: now,
          updatedAt: now,
        })
        return Response.json({ workflow }, { status: 201 })
      },
    },
  },
})
