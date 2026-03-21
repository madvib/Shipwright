// GET /api/profiles/:id    → { profile: Profile }
// PUT /api/profiles/:id    → { profile: Profile }
// DELETE /api/profiles/:id → { ok: true }

import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod/v4'
import { requireSession } from '#/lib/session-auth'
import { createRepositories } from '#/db/repositories'
import { getD1 } from '#/lib/d1'

const UpdateProfileInput = z.object({
  name: z.string().min(1).max(128).optional(),
  content: z.string().min(1).optional(),
  provider: z.string().max(64).nullable().optional(),
})

export const Route = createFileRoute('/api/profiles/$id')({
  server: {
    handlers: {
      GET: async ({ request, params }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRepositories(d1)
        const profile = await repos.getProfile(params.id, auth.org)
        if (!profile) return Response.json({ error: 'Profile not found' }, { status: 404 })
        if (profile.orgId !== auth.org) return Response.json({ error: 'forbidden' }, { status: 403 })

        return Response.json({ profile })
      },

      PUT: async ({ request, params }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        let body: unknown
        try { body = await request.json() } catch {
          return Response.json({ error: 'Invalid JSON body' }, { status: 400 })
        }

        const parsed = UpdateProfileInput.safeParse(body)
        if (!parsed.success) {
          const message = parsed.error.issues.map((i) => i.message).join('; ')
          return Response.json({ error: message }, { status: 400 })
        }

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRepositories(d1)
        const existing = await repos.getProfile(params.id, auth.org)
        if (!existing) return Response.json({ error: 'not found' }, { status: 404 })
        if (existing.orgId !== auth.org) return Response.json({ error: 'forbidden' }, { status: 403 })

        const profile = await repos.upsertProfile({
          ...existing,
          name: parsed.data.name ?? existing.name,
          content: parsed.data.content ?? existing.content,
          provider: parsed.data.provider !== undefined ? (parsed.data.provider ?? null) : existing.provider,
          updatedAt: Date.now(),
        })
        return Response.json({ profile })
      },

      DELETE: async ({ request, params }) => {
        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRepositories(d1)
        const existing = await repos.getProfile(params.id, auth.org)
        if (!existing) return Response.json({ error: 'not found' }, { status: 404 })
        if (existing.orgId !== auth.org) return Response.json({ error: 'forbidden' }, { status: 403 })

        await repos.deleteProfile(params.id, auth.org)
        return Response.json({ ok: true })
      },
    },
  },
})
