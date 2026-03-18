// GET /api/libraries     → { libraries: Library[] }
// POST /api/libraries    → { library: Library }

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

const CreateLibraryInput = z.object({
  name: z.string().min(1, 'Name is required').max(128),
  slug: z.string().max(128).optional(),
  data: z.record(z.unknown()).optional(),
})

export const Route = createFileRoute('/api/libraries')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRepositories(d1)
        const libraries = await repos.getLibraries(auth.org, auth.sub)
        return Response.json({ libraries })
      },

      POST: async ({ request }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        let body: unknown
        try { body = await request.json() } catch {
          return Response.json({ error: 'Invalid JSON body' }, { status: 400 })
        }

        const parsed = CreateLibraryInput.safeParse(body)
        if (!parsed.success) {
          const message = parsed.error.issues.map((i) => i.message).join('; ')
          return Response.json({ error: message }, { status: 400 })
        }

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRepositories(d1)
        const now = Date.now()
        const library = await repos.upsertLibrary({
          id: nanoid(),
          orgId: auth.org,
          userId: auth.sub,
          name: parsed.data.name,
          slug: parsed.data.slug ?? null,
          data: JSON.stringify(parsed.data.data ?? {}),
          createdAt: now,
          updatedAt: now,
        })
        return Response.json({ library }, { status: 201 })
      },
    },
  },
})
