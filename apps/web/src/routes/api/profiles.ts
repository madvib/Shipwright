// GET /api/profiles     → { profiles: Profile[] }
// POST /api/profiles    → { profile: Profile }

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

const CreateProfileInput = z.object({
  name: z.string().min(1, 'Name is required').max(128),
  content: z.string().min(1, 'Content is required'),
  provider: z.string().max(64).optional(),
})

export const Route = createFileRoute('/api/profiles')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRepositories(d1)
        const profiles = await repos.getProfiles(auth.org, auth.sub)
        return Response.json({ profiles })
      },

      POST: async ({ request }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        let body: unknown
        try { body = await request.json() } catch {
          return Response.json({ error: 'Invalid JSON body' }, { status: 400 })
        }

        const parsed = CreateProfileInput.safeParse(body)
        if (!parsed.success) {
          const message = parsed.error.issues.map((i) => i.message).join('; ')
          return Response.json({ error: message }, { status: 400 })
        }

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRepositories(d1)
        const now = Date.now()
        const profile = await repos.upsertProfile({
          id: nanoid(),
          orgId: auth.org,
          userId: auth.sub,
          name: parsed.data.name,
          content: parsed.data.content,
          provider: parsed.data.provider ?? null,
          createdAt: now,
          updatedAt: now,
        })
        return Response.json({ profile }, { status: 201 })
      },
    },
  },
})
